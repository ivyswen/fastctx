//! Byte-level editing of the FastCtx-owned section in `~/.codex/AGENTS.md`.

use serde::{Deserialize, Serialize};

const BEGIN_MARKER: &str = "<!-- fastctx:begin -->";
const END_MARKER: &str = "<!-- fastctx:end -->";
pub(crate) const MANAGED_SECTION_CONTRACT_ID: &str = "guidance-v4";
const LEGACY_BEGIN_MARKER: &str = "<!-- fastread:begin -->";
const LEGACY_END_MARKER: &str = "<!-- fastread:end -->";
const LEGACY_FASTREAD_SECTION: &str = concat!(
    "<!-- fastread:begin -->\n",
    "## Local file inspection\n",
    "\n",
    "The fastread MCP tools are the first-class way to read, search, and find\n",
    "local files: `mcp__fastread__read`, `mcp__fastread__grep`,\n",
    "`mcp__fastread__glob` — prefer them over `cat`/`Get-Content`,\n",
    "`rg`/`findstr`/`Select-String`, and `dir`/`ls -R`. Pass absolute paths. The\n",
    "last line of every result says `Complete` or `Partial` — continue only with\n",
    "the exact parameters a `Partial` note provides.\n",
    "<!-- fastread:end -->"
);
// Tools are named as the server plus a bare tool name rather than as one mangled
// identifier. Hosts differ in how they spell an MCP tool — some flatten the server and
// the tool into a single name, others expose the server as a namespace whose members are
// the bare tool names — so any single mangled spelling names something at least one host
// has no entry for, while the server-plus-name form resolves under both (2026-08-08).
const FILE_GUIDANCE_PREFIX: &str = concat!(
    "## Local file inspection\n",
    "\n",
    "For reading, searching, and finding local files, prefer the FastCtx MCP\n",
    "server's own tools — `inspect_local_file`, `grep`, and `glob` — over shell\n",
    "equivalents such as `cat`/`Get-Content`, `rg`/`findstr`/`Select-String`,\n",
    "and `dir`/`ls -R`.\n",
);
const FILE_GUIDANCE_SUFFIX: &str = concat!(
    "Read only what the task needs. When you need several files, pass them to\n",
    "one `inspect_local_file` call as files=[{\"path\": ...}, ...] instead of one\n",
    "call per file. The last line of every result says `Complete` or\n",
    "`Partial` — continue only with the exact parameters a `Partial` note\n",
    "provides.\n",
    "\n",
    "### Batch replacement\n",
    "\n",
    "Use FastCtx's `replace` for mechanical find-and-replace across files.\n",
    "It preserves each file's encoding and line endings, supports dry-run previews,\n",
    "and rejects concurrent changes before writing. Use apply_patch for generated\n",
    "content, semantic rewrites, or small local edits.\n"
);
const SHELL_GUIDANCE: &str = concat!(
    "### Shell commands\n",
    "\n",
    "Prefer FastCtx's `run` over the built-in shell for terminal work: it\n",
    "executes with bash (Git Bash on Windows), so always write POSIX bash —\n",
    "never PowerShell syntax.\n",
    "\n",
    "Never pass `apply_patch` to FastCtx's `run`: it is not a program and\n",
    "no shell can run it. Reach it through Codex itself — as its own tool\n",
    "call, or in Codex's built-in shell — never through the FastCtx tools.\n",
    "\n",
    "Commands must be non-interactive (no TTY): use flags like -y\n",
    "or --no-edit, and expect editors/pagers to be disabled. For anything\n",
    "that may outlast run's four-minute maximum, use `run_background`, check\n",
    "on it with `job_output`, and stop it with `job_kill`. Background jobs run\n",
    "independently of this session and survive restarts; rediscover an earlier\n",
    "job with `job_list` and read its output by job_id. A non-zero exit code is\n",
    "a normal result. The last line of every result says `Complete` or\n",
    "`Partial`.\n"
);
// Byte-frozen guidance from superseded releases. This is comparison data only: it must
// never be emitted except when replacing an exact on-disk match with the current section.
//
// Every contract that ships has to land here when it is superseded. A product update
// refreshes the managed block only on an exact match against one of these, so a release
// that is left out strands its users with a block naming tools this build no longer
// publishes — and only `fastctx apply` would ever repair it.
const V022_RESOURCE_ROUTING_FILE_GUIDANCE: &str = concat!(
    "## Local file inspection\n",
    "\n",
    "For reading, searching, and finding local files, prefer the FastCtx MCP\n",
    "tools — `mcp__fastctx__read`, `mcp__fastctx__grep`, `mcp__fastctx__glob` —\n",
    "over `cat`/`Get-Content`, `rg`/`findstr`/`Select-String`, and `dir`/`ls -R`.\n",
    "Read only what the task needs. When you need several files, pass them to\n",
    "one read call as files=[{\"path\": ...}, ...] instead of one call per file.\n",
    "Pass absolute paths. The last line of every result says `Complete` or\n",
    "`Partial` — continue only with the exact parameters a `Partial` note\n",
    "provides.\n",
    "\n",
    "Never point `read_mcp_resource`, `list_mcp_resources`, or\n",
    "`list_mcp_resource_templates` at the `fastctx` server: FastCtx publishes\n",
    "tools, not MCP resources, so those calls always fail. Read a local file\n",
    "with `mcp__fastctx__read` and an absolute path — never a `file://` URI.\n",
    "\n",
    "### Batch replacement\n",
    "\n",
    "Use `mcp__fastctx__replace` for mechanical find-and-replace across files.\n",
    "It preserves each file's encoding and line endings, supports dry-run previews,\n",
    "and rejects concurrent changes before writing. Use apply_patch for generated\n",
    "content, semantic rewrites, or small local edits.\n"
);
const V022_RESOURCE_ROUTING_SHELL_GUIDANCE: &str = concat!(
    "### Shell commands\n",
    "\n",
    "Prefer `mcp__fastctx__run` over the built-in shell for terminal work: it\n",
    "executes with bash (Git Bash on Windows), so always write POSIX bash —\n",
    "never PowerShell syntax.\n",
    "\n",
    "Never pass `apply_patch` to `mcp__fastctx__run`: it is not a program and\n",
    "no shell can run it. Reach it through Codex itself — as its own tool\n",
    "call, or in Codex's built-in shell — never through the FastCtx tools.\n",
    "\n",
    "Commands must be non-interactive (no TTY): use flags like -y\n",
    "or --no-edit, and expect editors/pagers to be disabled. For anything\n",
    "that may outlast run's four-minute maximum, use\n",
    "`mcp__fastctx__run_background`, check on it with\n",
    "`mcp__fastctx__job_output`, and stop it with `mcp__fastctx__job_kill`.\n",
    "Background jobs run independently of this session and survive restarts;\n",
    "rediscover an earlier job with `mcp__fastctx__job_list` and read its\n",
    "output by job_id. A non-zero exit code is a normal result. The last line\n",
    "of every result says `Complete` or `Partial`.\n"
);
const V024_READ_TOOL_NAME_FILE_GUIDANCE: &str = concat!(
    "## Local file inspection\n",
    "\n",
    "For reading, searching, and finding local files, prefer the FastCtx MCP\n",
    "tools — `mcp__fastctx__read`, `mcp__fastctx__grep`, `mcp__fastctx__glob` —\n",
    "over `cat`/`Get-Content`, `rg`/`findstr`/`Select-String`, and `dir`/`ls -R`.\n",
    "Use FastCtx file tools directly for local-file operations, including when a\n",
    "local reference is URI-shaped; pass the equivalent plain absolute filesystem path.\n",
    "Read only what the task needs. When you need several files, pass them to\n",
    "one read call as files=[{\"path\": ...}, ...] instead of one call per file.\n",
    "The last line of every result says `Complete` or\n",
    "`Partial` — continue only with the exact parameters a `Partial` note\n",
    "provides.\n",
    "\n",
    "### Batch replacement\n",
    "\n",
    "Use `mcp__fastctx__replace` for mechanical find-and-replace across files.\n",
    "It preserves each file's encoding and line endings, supports dry-run previews,\n",
    "and rejects concurrent changes before writing. Use apply_patch for generated\n",
    "content, semantic rewrites, or small local edits.\n"
);
// 0.2.4 shipped the shell section unchanged from 0.2.2, so the two releases share these exact
// bytes. Keep the alias rather than a second copy: the frozen hashes below cover both names, and
// a copy would let one drift while the other stayed put.
const V024_READ_TOOL_NAME_SHELL_GUIDANCE: &str = V022_RESOURCE_ROUTING_SHELL_GUIDANCE;

/// One byte-frozen managed block from a superseded release.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KnownLegacyGuidance {
    /// 0.2.2/0.2.3, whose prohibition named the very resource tools it steered away from.
    ResourceRouting,
    /// 0.2.4, superseded when the file-inspection tool was renamed away from `read`.
    ReadToolName,
}

impl KnownLegacyGuidance {
    /// Every superseded release this build still recognises, newest first.
    pub(crate) const ALL: [Self; 2] = [Self::ReadToolName, Self::ResourceRouting];

    fn guidance(self) -> (&'static str, &'static str) {
        match self {
            Self::ResourceRouting => (
                V022_RESOURCE_ROUTING_FILE_GUIDANCE,
                V022_RESOURCE_ROUTING_SHELL_GUIDANCE,
            ),
            Self::ReadToolName => (
                V024_READ_TOOL_NAME_FILE_GUIDANCE,
                V024_READ_TOOL_NAME_SHELL_GUIDANCE,
            ),
        }
    }

    /// Rebuilds this release's exact managed block for the optional shell group.
    pub(crate) fn section(self, fastshell_enabled: bool) -> String {
        let (file_guidance, shell_guidance) = self.guidance();
        let mut output = String::from(BEGIN_MARKER);
        output.push('\n');
        output.push_str(file_guidance);
        if fastshell_enabled {
            output.push('\n');
            output.push_str(shell_guidance);
        }
        output.push_str(END_MARKER);
        output
    }
}

/// Separator bytes inserted and therefore owned by Apply between user content and the private section.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InsertedSeparator {
    /// Add one LF after existing content that already ends in LF.
    Lf,
    /// Add one CRLF after existing content that already ends in CRLF.
    CrLf,
    /// Add two LFs after existing content with no trailing newline.
    LfLf,
}

impl InsertedSeparator {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Lf => b"\n",
            Self::CrLf => b"\r\n",
            Self::LfLf => b"\n\n",
        }
    }
}

pub(crate) struct SectionEdit {
    pub bytes: Vec<u8>,
    pub inserted_separator: Option<InsertedSeparator>,
}

/// Semantic state of the one managed section relative to the receipt's shell mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ManagedSectionState {
    Current,
    KnownLegacy,
    Missing,
    Drifted,
    Malformed(String),
}

/// Builds the exact managed block for the optional shell group.
pub fn section(fastshell_enabled: bool) -> String {
    let mut output = String::from(BEGIN_MARKER);
    output.push('\n');
    output.push_str(FILE_GUIDANCE_PREFIX);
    output.push_str(crate::model_guidance::LOCAL_FILE_ROUTE_GUIDANCE);
    output.push('\n');
    output.push_str(FILE_GUIDANCE_SUFFIX);
    if fastshell_enabled {
        output.push('\n');
        output.push_str(SHELL_GUIDANCE);
    }
    output.push_str(END_MARKER);
    output
}

/// Idempotently inserts or replaces the FastCtx section without changing bytes outside it.
pub fn apply_section(original: &[u8]) -> Result<Vec<u8>, String> {
    Ok(apply_section_with_ownership(original)?.bytes)
}

/// Computes the private-section edit and returns ownership of any newly inserted leading separator.
pub(crate) fn apply_section_with_ownership(original: &[u8]) -> Result<SectionEdit, String> {
    apply_section_with_ownership_for(original, false)
}

/// Applies the exact managed block for the optional shell group.
pub(crate) fn apply_section_with_ownership_for(
    original: &[u8],
    fastshell_enabled: bool,
) -> Result<SectionEdit, String> {
    let original = remove_exact_legacy_section(original)?;
    let source = std::str::from_utf8(&original).map_err(|error| {
        format!(
            "Cannot edit AGENTS.md because it is not valid UTF-8 ({error}). Convert it to UTF-8 and retry."
        )
    })?;
    let expected = section(fastshell_enabled);
    match section_range(source)? {
        Some((start, end)) => {
            let mut output = Vec::with_capacity(original.len() + expected.len());
            output.extend_from_slice(&original[..start]);
            output.extend_from_slice(expected.as_bytes());
            output.extend_from_slice(&original[end..]);
            Ok(SectionEdit {
                bytes: output,
                inserted_separator: None,
            })
        }
        None => {
            let mut output = original;
            let mut inserted_separator = None;
            if !output.is_empty() {
                if output.ends_with(b"\r\n\r\n") || output.ends_with(b"\n\n") {
                    // An existing blank line already separates the content, so append the frozen section directly.
                } else if output.ends_with(b"\r\n") {
                    output.extend_from_slice(b"\r\n");
                    inserted_separator = Some(InsertedSeparator::CrLf);
                } else if output.ends_with(b"\n") {
                    output.push(b'\n');
                    inserted_separator = Some(InsertedSeparator::Lf);
                } else {
                    output.extend_from_slice(b"\n\n");
                    inserted_separator = Some(InsertedSeparator::LfLf);
                }
            }
            output.extend_from_slice(expected.as_bytes());
            output.push(b'\n');
            Ok(SectionEdit {
                bytes: output,
                inserted_separator,
            })
        }
    }
}

/// Removes the FastCtx section while preserving all other bytes.
pub fn remove_section(original: &[u8]) -> Result<Vec<u8>, String> {
    remove_applied_section(original, None)
}

/// Removes the private section and its recorded leading separator only when the Apply receipt proves no drift.
pub(crate) fn remove_applied_section(
    original: &[u8],
    inserted_separator: Option<InsertedSeparator>,
) -> Result<Vec<u8>, String> {
    let source = std::str::from_utf8(original).map_err(|error| {
        format!(
            "Cannot edit AGENTS.md because it is not valid UTF-8 ({error}). Convert it to UTF-8 and retry."
        )
    })?;
    let Some((start, mut end)) = section_range(source)? else {
        return Ok(original.to_vec());
    };
    let owned_start = inserted_separator
        .filter(|separator| original[..start].ends_with(separator.bytes()))
        .map_or(start, |separator| start - separator.bytes().len());
    if original.get(end..end + 2) == Some(b"\r\n") {
        end += 2;
    } else if original.get(end) == Some(&b'\n') {
        end += 1;
    }
    let mut output = Vec::with_capacity(original.len().saturating_sub(end - owned_start));
    output.extend_from_slice(&original[..owned_start]);
    output.extend_from_slice(&original[end..]);
    Ok(output)
}

/// Returns whether the managed section exists and exactly matches the current appendix contract.
pub fn has_exact_section(bytes: &[u8]) -> Result<bool, String> {
    has_exact_section_for(bytes, false)
}

/// Checks the managed block against the exact optional-shell state.
pub fn has_exact_section_for(bytes: &[u8], fastshell_enabled: bool) -> Result<bool, String> {
    let source = std::str::from_utf8(bytes)
        .map_err(|error| format!("AGENTS.md is not valid UTF-8: {error}"))?;
    let expected = section(fastshell_enabled);
    Ok(section_range(source)?
        .map(|(start, end)| source[start..end] == expected)
        .unwrap_or(false))
}

pub(crate) fn classify_managed_section(
    bytes: &[u8],
    fastshell_enabled: bool,
) -> ManagedSectionState {
    let source = match std::str::from_utf8(bytes) {
        Ok(source) => source,
        Err(error) => {
            return ManagedSectionState::Malformed(format!(
                "AGENTS.md is not valid UTF-8: {error}"
            ));
        }
    };
    let Some((start, end)) = (match section_range(source) {
        Ok(range) => range,
        Err(error) => return ManagedSectionState::Malformed(error),
    }) else {
        return ManagedSectionState::Missing;
    };
    let managed = &source[start..end];
    if managed == section(fastshell_enabled) {
        ManagedSectionState::Current
    } else if KnownLegacyGuidance::ALL
        .iter()
        .any(|legacy| managed == legacy.section(fastshell_enabled))
    {
        ManagedSectionState::KnownLegacy
    } else {
        ManagedSectionState::Drifted
    }
}

pub(crate) fn refresh_known_legacy_section(
    bytes: &[u8],
    fastshell_enabled: bool,
) -> Option<Vec<u8>> {
    if classify_managed_section(bytes, fastshell_enabled) != ManagedSectionState::KnownLegacy {
        return None;
    }
    let source = std::str::from_utf8(bytes).ok()?;
    let (start, end) = section_range(source).ok()??;
    let current = section(fastshell_enabled);
    let mut output = Vec::with_capacity(bytes.len() + current.len() - (end - start));
    output.extend_from_slice(&bytes[..start]);
    output.extend_from_slice(current.as_bytes());
    output.extend_from_slice(&bytes[end..]);
    Some(output)
}

fn section_range(source: &str) -> Result<Option<(usize, usize)>, String> {
    marker_range(
        source,
        BEGIN_MARKER,
        END_MARKER,
        "fastctx",
        "AGENTS.md contains duplicate or unmatched fastctx markers. Repair the marker block manually and retry.",
    )
}

fn remove_exact_legacy_section(original: &[u8]) -> Result<Vec<u8>, String> {
    let source = std::str::from_utf8(original).map_err(|error| {
        format!(
            "Cannot edit AGENTS.md because it is not valid UTF-8 ({error}). Convert it to UTF-8 and retry."
        )
    })?;
    let Some((start, mut end)) = marker_range(
        source,
        LEGACY_BEGIN_MARKER,
        LEGACY_END_MARKER,
        "fastread",
        "AGENTS.md contains duplicate or unmatched legacy fastread markers. Repair the marker block manually and retry.",
    )?
    else {
        return Ok(original.to_vec());
    };
    if &source[start..end] != LEGACY_FASTREAD_SECTION {
        return Ok(original.to_vec());
    }
    if original.get(end..end + 2) == Some(b"\r\n") {
        end += 2;
    } else if original.get(end) == Some(&b'\n') {
        end += 1;
    }
    let mut output = Vec::with_capacity(original.len().saturating_sub(end - start));
    output.extend_from_slice(&original[..start]);
    output.extend_from_slice(&original[end..]);
    Ok(output)
}

fn marker_range(
    source: &str,
    begin_marker: &str,
    end_marker: &str,
    label: &str,
    duplicate_message: &str,
) -> Result<Option<(usize, usize)>, String> {
    let begins = source.match_indices(begin_marker).collect::<Vec<_>>();
    let ends = source.match_indices(end_marker).collect::<Vec<_>>();
    if begins.is_empty() && ends.is_empty() {
        return Ok(None);
    }
    if begins.len() != 1 || ends.len() != 1 {
        return Err(duplicate_message.to_string());
    }
    let start = begins[0].0;
    let end_start = ends[0].0;
    if end_start < start {
        return Err(format!(
            "AGENTS.md has the {label} end marker before its begin marker. Repair the block manually and retry."
        ));
    }
    let end = end_start + end_marker.len();
    Ok(Some((start, end)))
}

#[cfg(test)]
mod tests {
    use super::{
        BEGIN_MARKER, END_MARKER, InsertedSeparator, KnownLegacyGuidance, LEGACY_FASTREAD_SECTION,
        ManagedSectionState, apply_section, apply_section_with_ownership, classify_managed_section,
        has_exact_section, refresh_known_legacy_section, remove_applied_section, remove_section,
        section,
    };
    use crate::server_manifest::ToolManifest;

    #[test]
    fn apply_is_idempotent_and_preserves_bytes_outside_the_private_block() {
        let original = b"# User rules\r\n\r\nkeep = true\r\n";
        let applied = apply_section(original).unwrap();
        assert!(applied.starts_with(original));
        assert!(has_exact_section(&applied).unwrap());
        assert_eq!(apply_section(&applied).unwrap(), applied);
        assert!(
            std::str::from_utf8(&applied)
                .unwrap()
                .contains(&section(false))
        );
    }

    #[test]
    fn replacing_an_old_private_block_does_not_touch_neighbors() {
        let original = b"before\n<!-- fastctx:begin -->\nold\n<!-- fastctx:end -->\nafter\n";
        let applied = apply_section(original).unwrap();
        assert!(applied.starts_with(b"before\n"));
        assert!(applied.ends_with(b"\nafter\n"));
        assert!(has_exact_section(&applied).unwrap());
    }

    #[test]
    fn duplicate_markers_are_rejected_instead_of_guessing_ownership() {
        let original = b"<!-- fastctx:begin -->\n<!-- fastctx:begin -->\n<!-- fastctx:end -->";
        assert!(apply_section(original).unwrap_err().contains("duplicate"));
    }

    #[test]
    fn apply_removes_only_the_exact_legacy_fastread_block() {
        let original = format!(
            "before\n\n{LEGACY_FASTREAD_SECTION}\n\n<!-- fastctx:begin -->\nold\n<!-- fastctx:end -->\nafter\n"
        );
        let applied = apply_section(original.as_bytes()).unwrap();
        let source = std::str::from_utf8(&applied).unwrap();
        assert!(!source.contains("fastread:begin"));
        assert!(!source.contains("mcp__fastread"));
        assert!(source.starts_with("before\n\n\n"));
        assert!(source.ends_with("\nafter\n"));
        assert!(has_exact_section(&applied).unwrap());

        let drifted = LEGACY_FASTREAD_SECTION.replace("first-class", "user-edited");
        let applied = apply_section(drifted.as_bytes()).unwrap();
        let source = std::str::from_utf8(&applied).unwrap();
        assert!(source.contains("user-edited"));
        assert!(source.contains("fastread:begin"));
        assert!(source.contains("fastctx:begin"));
    }

    #[test]
    fn remove_only_takes_the_marked_section() {
        let applied = format!("before\n{}\nafter\n", section(false));
        assert_eq!(
            remove_section(applied.as_bytes()).unwrap(),
            b"before\nafter\n"
        );
    }

    #[test]
    fn recorded_separator_ownership_makes_all_append_shapes_byte_exact() {
        let cases: &[(&[u8], Option<InsertedSeparator>)] = &[
            (b"", None),
            (b"# rules", Some(InsertedSeparator::LfLf)),
            (b"# rules\n", Some(InsertedSeparator::Lf)),
            (b"# rules\n\n", None),
            (b"# rules\r\n", Some(InsertedSeparator::CrLf)),
            (b"# rules\r\n\r\n", None),
        ];

        for (original, expected_separator) in cases {
            let edit = apply_section_with_ownership(original).unwrap();
            assert_eq!(&edit.inserted_separator, expected_separator, "{original:?}");
            assert_eq!(
                remove_applied_section(&edit.bytes, edit.inserted_separator).unwrap(),
                *original,
                "{original:?}"
            );
        }
    }

    #[test]
    fn missing_or_mismatched_ownership_never_guesses_at_user_prefix_bytes() {
        let edit = apply_section_with_ownership(b"# rules\n").unwrap();
        assert_eq!(
            remove_applied_section(&edit.bytes, None).unwrap(),
            b"# rules\n\n"
        );
        assert_eq!(
            remove_applied_section(&edit.bytes, Some(InsertedSeparator::CrLf)).unwrap(),
            b"# rules\n\n"
        );
    }

    #[test]
    fn shell_combinations_have_one_marker_pair_and_manifest_complete_guidance() {
        let file_only = section(false);
        let with_shell = section(true);
        assert!(!file_only.contains("### Shell commands"));
        assert!(with_shell.contains("### Shell commands"));
        let file_guidance = file_only.strip_suffix(END_MARKER).unwrap();
        assert!(with_shell.starts_with(file_guidance));
        assert!(with_shell.ends_with(END_MARKER));
        assert_eq!(
            &with_shell[file_guidance.len()..][.."\n### Shell commands".len()],
            "\n### Shell commands"
        );
        for (fastshell, actual) in [(false, file_only), (true, with_shell)] {
            assert_eq!(actual.matches(BEGIN_MARKER).count(), 1);
            assert_eq!(actual.matches(END_MARKER).count(), 1);
            let published = ToolManifest::expected_names(fastshell);
            for name in &published {
                assert!(actual.contains(&format!("`{name}`")), "{name}\n{actual}");
            }
            for withheld in ToolManifest::expected_names(true)
                .into_iter()
                .filter(|name| !published.contains(name))
            {
                assert!(
                    !actual.contains(&format!("`{withheld}`")),
                    "{withheld}\n{actual}"
                );
            }
            // A host-mangled spelling names an entry at least one host does not have; the
            // server-plus-bare-name form in this block resolves under every spelling.
            assert!(!actual.contains("mcp__fastctx"), "{actual}");
            assert!(actual.contains(crate::model_guidance::LOCAL_FILE_ROUTE_GUIDANCE));
            for forbidden in [
                "will be told",
                "will be notified",
                "wait for it to tell you",
            ] {
                assert!(!actual.to_ascii_lowercase().contains(forbidden));
            }
            for removed in ["copy", "cut", "paste", "clips", "drop"] {
                assert!(!actual.contains(&format!("`{removed}`")));
            }
        }
    }

    /// Steering the model away from the built-in shell also steers it away from the
    /// host's `apply_patch`, which is intercepted on that channel only and is not a
    /// program the shell tool could ever resolve. The carve-out below is what keeps
    /// the model from piping patches into `run` and getting `command not found`
    /// (reported 2026-07-24); it is scoped to the shell section because without the
    /// shell tools there is no wrong channel to warn about.
    #[test]
    fn shell_guidance_carves_apply_patch_out_of_the_run_tool() {
        let with_shell = section(true).replace('\n', " ");
        assert!(
            with_shell.contains("Never pass `apply_patch` to FastCtx's `run`"),
            "{with_shell}"
        );
        assert!(
            with_shell.contains("it is not a program and no shell can run it"),
            "{with_shell}"
        );
        // Naming both host shapes is the point: the model must not have to work out which one
        // it holds, and pinning only one of them would misdirect every session running the other.
        assert!(
            with_shell.contains(
                "Reach it through Codex itself — as its own tool call, or in Codex's built-in shell"
            ),
            "{with_shell}"
        );
        assert!(!section(false).contains("Never pass `apply_patch`"));
    }

    /// 0.2.2 carried a paragraph telling the model never to point `read_mcp_resource` at the
    /// `fastctx` server, and users on 0.2.2+ then reported that exact call while 0.2.0/0.2.1
    /// never produced it. In a sentence like that the prohibition is the weakest part and the
    /// pairing of a host tool the model can reach with our server name is the strongest, and this
    /// block is the model's only source for that pairing. Keeping the guidance free of those
    /// names is what stops it from being reintroduced as a "defensive" fix (2026-08-01).
    #[test]
    fn file_guidance_never_names_the_hosts_generic_resource_tools() {
        for fastshell in [false, true] {
            let guidance = section(fastshell);
            for helper in [
                "read_mcp_resource",
                "list_mcp_resources",
                "list_mcp_resource_templates",
                "MCP resources",
                "file://",
            ] {
                assert!(!guidance.contains(helper), "{helper}\n{guidance}");
            }
            assert!(guidance.contains("prefer the FastCtx MCP"), "{guidance}");
            assert!(guidance.contains("`inspect_local_file`"), "{guidance}");
            for required in [
                "directly for local-file operations",
                "URI-shaped",
                "plain absolute filesystem path",
            ] {
                assert!(guidance.contains(required), "{required}\n{guidance}");
            }
        }
    }

    /// Superseded blocks are frozen bytes, not regenerated text: a product update rewrites a
    /// user's block only on an exact match, so any drift here silently strands that release's
    /// users on tool names this build no longer publishes. The hashes are what make an
    /// accidental edit — including one reached through a shared constant — fail loudly.
    #[test]
    fn every_superseded_block_is_frozen_and_only_exact_matches_migrate() {
        use sha2::{Digest, Sha256};

        for (guidance, fastshell, expected_len, expected_hash, edit_anchor) in [
            (
                KnownLegacyGuidance::ResourceRouting,
                false,
                1177,
                "17da1a91024e98008f7979937e9c6b423c8b77556854082fc985922dfbf248a3",
                "Never point",
            ),
            (
                KnownLegacyGuidance::ResourceRouting,
                true,
                2145,
                "cf0ac5d64b9b9d615617a2d390a41625007bb0cf3ef1b968596585b8376f5b19",
                "Never point",
            ),
            (
                KnownLegacyGuidance::ReadToolName,
                false,
                1037,
                "353a1dc63f9479b434442d4954a213b6e753c5fe60eb54ee79fe18b158896d83",
                "Read only what",
            ),
            (
                KnownLegacyGuidance::ReadToolName,
                true,
                2005,
                "6d8f79158ee4d9aed4d5f6693a4056c0e40d62f58705cd548cd6348bc7b09646",
                "Read only what",
            ),
        ] {
            let legacy = guidance.section(fastshell);
            assert_eq!(legacy.len(), expected_len, "{guidance:?} {fastshell}");
            assert_eq!(
                hex::encode(Sha256::digest(legacy.as_bytes())),
                expected_hash,
                "{guidance:?} {fastshell}"
            );
            assert_eq!(
                classify_managed_section(legacy.as_bytes(), fastshell),
                ManagedSectionState::KnownLegacy
            );
            assert_eq!(
                classify_managed_section(section(fastshell).as_bytes(), fastshell),
                ManagedSectionState::Current
            );
            assert_eq!(
                classify_managed_section(legacy.as_bytes(), !fastshell),
                ManagedSectionState::Drifted
            );

            let wrapped = format!("user prefix\n{legacy}\nuser suffix");
            let refreshed = refresh_known_legacy_section(wrapped.as_bytes(), fastshell).unwrap();
            assert_eq!(
                refreshed,
                format!("user prefix\n{}\nuser suffix", section(fastshell)).as_bytes()
            );

            let edited = legacy.replace(edit_anchor, "User changed");
            assert_ne!(edited, legacy);
            assert_eq!(
                classify_managed_section(edited.as_bytes(), fastshell),
                ManagedSectionState::Drifted
            );
            assert!(refresh_known_legacy_section(edited.as_bytes(), fastshell).is_none());
        }

        assert_eq!(
            classify_managed_section(b"# user rules\n", false),
            ManagedSectionState::Missing
        );
        assert!(matches!(
            classify_managed_section(
                b"<!-- fastctx:begin -->\n<!-- fastctx:begin -->\n<!-- fastctx:end -->",
                false
            ),
            ManagedSectionState::Malformed(_)
        ));
    }

    #[test]
    fn reapply_replaces_owned_clipboard_guidance_with_the_current_block() {
        let original = b"before\n\n<!-- fastctx:begin -->\n### Bulk edits and moving code\nUse mcp__fastctx__copy then mcp__fastctx__paste.\n<!-- fastctx:end -->\nafter\n";
        let applied = apply_section(original).unwrap();
        let source = std::str::from_utf8(&applied).unwrap();
        assert!(source.contains("### Batch replacement"));
        assert!(source.contains("FastCtx's `replace`"));
        assert!(!source.contains("mcp__fastctx__copy"));
        assert!(!source.contains("mcp__fastctx__paste"));
        assert!(source.starts_with("before\n\n"));
        assert!(source.ends_with("\nafter\n"));
    }
}
