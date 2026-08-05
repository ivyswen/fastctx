//! Whether FastCtx is currently connected to the host, as the control terminal reports it.
//!
//! Apply writes three things: the stable binary, the `mcp_servers` entry, and the marker block in
//! the host's `AGENTS.md`. Only the last one changes between releases without the user doing
//! anything, because upgrades replace the binary but never rewrite shared host files. This module
//! answers the one question the main menu needs from that: does the connection still match the
//! build the user is running?

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
    /// A connection was recorded, but its guidance block no longer matches this build.
    Stale,
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
    // An outdated block, a damaged one, and an unreadable file all call for the same user action,
    // so they share one state instead of adding a fourth nobody could act on differently.
    match fs::read(&paths.codex_agents) {
        Ok(bytes) => match agents::has_exact_section_for(&bytes, record.fastshell_enabled) {
            Ok(true) => LinkState::Current,
            _ => LinkState::Stale,
        },
        Err(_) => LinkState::Stale,
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

    /// The upgrade path users actually hit: the binary advances while the host file keeps the
    /// guidance an older release wrote, and nothing in the update flow rewrites it.
    #[test]
    fn a_block_that_no_longer_matches_this_build_reads_as_stale() {
        let home = tempfile::tempdir().unwrap();
        let stale = agents::section(false).replace("Read only what the task needs.", "Old text.");
        let paths = paths_with_agents(home.path(), Some(&stale));
        assert_eq!(link_state(&paths, Some(&receipt(false))), LinkState::Stale);
    }

    #[test]
    fn a_shell_state_that_disagrees_with_the_receipt_reads_as_stale() {
        let home = tempfile::tempdir().unwrap();
        let paths = paths_with_agents(home.path(), Some(&agents::section(false)));
        assert_eq!(link_state(&paths, Some(&receipt(true))), LinkState::Stale);
    }

    /// Damage and absence both resolve to Stale so the menu never shows a state whose only
    /// remedy is the one Stale already names.
    #[test]
    fn damaged_markers_and_a_missing_file_both_read_as_stale() {
        let home = tempfile::tempdir().unwrap();
        let damaged = format!("{}\n{}", agents::section(false), agents::section(false));
        let paths = paths_with_agents(home.path(), Some(&damaged));
        assert_eq!(link_state(&paths, Some(&receipt(false))), LinkState::Stale);

        let empty_home = tempfile::tempdir().unwrap();
        let empty = paths_with_agents(empty_home.path(), None);
        assert_eq!(link_state(&empty, Some(&receipt(false))), LinkState::Stale);
    }
}
