//! Whether FastCtx is currently connected to the host, as the control terminal reports it.
//!
//! Apply writes three things: the stable binary, the `mcp_servers` entry, and the marker block in
//! the host's `AGENTS.md`. Product updates may refresh one byte-frozen legacy block, but that
//! deliberately does not complete a new Apply. This module keeps the main menu aligned with the
//! more detailed Status diagnosis without hiding that remaining action.

use std::fs;

use crate::control::agents;
use crate::control::paths::ControlPaths;
use crate::control::settings::AppliedRecord;

/// How the connection to the host compares to the running build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkState {
    /// Apply has never recorded a connection.
    Absent,
    /// The managed guidance block matches what this build writes.
    Current,
    /// Current bytes were found, but the receipt does not record the current Apply contract.
    ApplyRequired,
    /// The exact known-bad 0.2.2/0.2.3 guidance block remains on disk.
    KnownLegacy,
    /// An Apply receipt exists, but the managed block or file is absent.
    Missing,
    /// The marker pair is valid, but the managed bytes were changed or use another shell shape.
    Drifted,
    /// The file is not UTF-8 or its managed markers are structurally damaged.
    Malformed,
    /// The AGENTS file exists but could not be read.
    Unreadable,
}

impl LinkState {
    /// Whether the visible connection still needs an explicit Apply before a Codex restart.
    pub(crate) const fn requires_apply(self) -> bool {
        !matches!(self, Self::Absent | Self::Current)
    }
}

/// Classifies the recorded connection against the guidance this build would write.
///
/// The block is compared with the receipt's shell state rather than the currently saved one so
/// this agrees with the `AGENTS guidance` doctor check; a pending extension-tool change is a
/// separate matter that the config page already explains.
pub fn link_state(paths: &ControlPaths, applied: Option<&AppliedRecord>) -> LinkState {
    let Some(record) = applied else {
        return LinkState::Absent;
    };
    match fs::read(&paths.codex_agents) {
        Ok(bytes) => match agents::classify_managed_section(&bytes, record.fastshell_enabled) {
            agents::ManagedSectionState::Current
                if record.agents_contract_id.as_deref()
                    == Some(agents::MANAGED_SECTION_CONTRACT_ID) =>
            {
                LinkState::Current
            }
            agents::ManagedSectionState::Current => LinkState::ApplyRequired,
            agents::ManagedSectionState::KnownLegacy => LinkState::KnownLegacy,
            agents::ManagedSectionState::Missing => LinkState::Missing,
            agents::ManagedSectionState::Drifted => LinkState::Drifted,
            agents::ManagedSectionState::Malformed(_) => LinkState::Malformed,
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => LinkState::Missing,
        Err(_) => LinkState::Unreadable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::settings::{ManagedFileRecord, Tier, ToolBudgets};

    fn paths_with_agents(home: &std::path::Path, body: Option<&str>) -> ControlPaths {
        let paths = ControlPaths::for_home(home);
        fs::create_dir_all(&paths.codex_dir).unwrap();
        if let Some(body) = body {
            fs::write(&paths.codex_agents, body).unwrap();
        }
        paths
    }

    fn receipt(fastshell_enabled: bool) -> AppliedRecord {
        let managed = |path: &str| ManagedFileRecord {
            path: path.to_string(),
            original_existed: true,
            applied_sha256: "managed-hash".to_string(),
        };
        AppliedRecord {
            applied_at_utc: "2026-08-04T00:00:00Z".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            command: "fastctx".to_string(),
            tier: Tier::Standard,
            tool_output_token_limit: 30_000,
            tool_timeout_sec: Some(300),
            previous_token_limit_present: false,
            previous_token_limit: None,
            fastctx_token_budget: 27_000,
            tool_budgets: ToolBudgets::default(),
            fastshell_enabled,
            fastedit_enabled: false,
            codex_dir_created: false,
            codex_config: managed("config.toml"),
            codex_agents: managed("AGENTS.md"),
            agents_contract_id: Some(agents::MANAGED_SECTION_CONTRACT_ID.to_string()),
            codex_agents_inserted_separator: None,
            binary_sha256: "binary-hash".to_string(),
        }
    }

    #[test]
    fn a_missing_receipt_reads_as_absent_without_touching_the_host_file() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths_with_agents(home.path(), None);
        assert_eq!(link_state(&paths, None), LinkState::Absent);
    }

    #[test]
    fn a_block_from_this_build_reads_as_current() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths_with_agents(home.path(), Some(&agents::section(false)));
        assert_eq!(
            link_state(&paths, Some(&receipt(false))),
            LinkState::Current
        );
    }

    #[test]
    fn an_automatically_refreshed_block_still_requires_an_explicit_apply() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths_with_agents(home.path(), Some(&agents::section(false)));
        let mut old_receipt = receipt(false);
        old_receipt.agents_contract_id = None;
        assert_eq!(
            link_state(&paths, Some(&old_receipt)),
            LinkState::ApplyRequired
        );
    }

    #[test]
    fn every_owned_noncurrent_shape_keeps_its_diagnostic_state() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths_with_agents(home.path(), Some(&agents::section(false)));
        assert_eq!(link_state(&paths, Some(&receipt(true))), LinkState::Drifted);

        std::fs::write(&paths.codex_agents, agents::known_legacy_section(false)).unwrap();
        assert_eq!(
            link_state(&paths, Some(&receipt(false))),
            LinkState::KnownLegacy
        );

        let drifted = "<!-- fastctx:begin -->\nuser-owned drift\n<!-- fastctx:end -->";
        std::fs::write(&paths.codex_agents, drifted).unwrap();
        assert_eq!(
            link_state(&paths, Some(&receipt(false))),
            LinkState::Drifted
        );

        let damaged = format!("{}\n{}", agents::section(false), agents::section(false));
        std::fs::write(&paths.codex_agents, damaged).unwrap();
        assert_eq!(
            link_state(&paths, Some(&receipt(false))),
            LinkState::Malformed
        );

        let empty_home = tempfile::tempdir().unwrap();
        let empty = paths_with_agents(empty_home.path(), None);
        assert_eq!(
            link_state(&empty, Some(&receipt(false))),
            LinkState::Missing
        );
    }

    #[test]
    fn only_absent_and_current_are_complete_connection_states() {
        assert!(!LinkState::Absent.requires_apply());
        assert!(!LinkState::Current.requires_apply());
        for state in [
            LinkState::ApplyRequired,
            LinkState::KnownLegacy,
            LinkState::Missing,
            LinkState::Drifted,
            LinkState::Malformed,
            LinkState::Unreadable,
        ] {
            assert!(state.requires_apply(), "{state:?}");
        }
    }
}
