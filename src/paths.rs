//! Cross-platform input parsing, output normalization, and filesystem error translation.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Converts either user-facing separator style into a path the current platform can parse.
pub fn parse_input_path(input: &str) -> PathBuf {
    if std::path::MAIN_SEPARATOR == '/' {
        PathBuf::from(input.replace('\\', "/"))
    } else {
        PathBuf::from(input)
    }
}

/// Accepts either a plain path or a strictly local file URI and returns a native path.
///
/// URI normalization is deliberately limited to the `file` scheme. Other schemes and
/// ambiguous file-URI shapes fail before filesystem lookup so callers never guess a path.
pub(crate) fn parse_local_path_input(input: &str) -> Result<PathBuf, String> {
    let Some(scheme) = uri_scheme(input) else {
        return Ok(parse_input_path(input));
    };
    if !scheme.eq_ignore_ascii_case("file") {
        return Err(format!(
            "Unsupported URI scheme \"{}\" for a local filesystem path.",
            scheme.to_ascii_lowercase()
        ));
    }
    if !input[scheme.len() + 1..].starts_with("//") {
        return Err(
            "Invalid local file URI: expected an absolute URI with a local authority.".to_string(),
        );
    }
    if !has_valid_percent_escapes(input) {
        return Err("Invalid local file URI: malformed percent escape.".to_string());
    }

    let uri =
        url::Url::parse(input).map_err(|error| format!("Invalid local file URI: {error}."))?;
    if !uri.username().is_empty() || uri.password().is_some() || uri.port().is_some() {
        return Err(
            "Invalid local file URI: user information and ports are not supported.".to_string(),
        );
    }
    if uri.query().is_some() || uri.fragment().is_some() {
        return Err(
            "Invalid local file URI: query and fragment components are not supported.".to_string(),
        );
    }
    if uri
        .host_str()
        .is_some_and(|host| !host.eq_ignore_ascii_case("localhost"))
    {
        return Err("Invalid local file URI: remote authorities are not supported.".to_string());
    }

    let path = uri.to_file_path().map_err(|_| {
        "Invalid local file URI: it cannot be represented as a local filesystem path.".to_string()
    })?;
    if path.as_os_str().to_string_lossy().contains('\0') {
        return Err("Invalid local file URI: the decoded path contains a NUL byte.".to_string());
    }
    if !path.is_absolute() {
        return Err("Invalid local file URI: the decoded path is not absolute.".to_string());
    }
    Ok(path)
}

pub(crate) fn is_local_file_uri_input(input: &str) -> bool {
    uri_scheme(input).is_some_and(|scheme| scheme.eq_ignore_ascii_case("file"))
}

/// Returns the RFC 3986 scheme of an input that cannot instead be a Windows drive designator.
///
/// One-character schemes are reported as absent: no registered scheme is that short, while
/// `C:\repo`, `C:/repo`, and the drive-relative `C:repo` all open with one. Reading those as
/// schemes would reject real paths as unsupported protocols.
fn uri_scheme(input: &str) -> Option<&str> {
    let end = input.find(':')?;
    let scheme = &input[..end];
    if scheme.len() < 2 {
        return None;
    }
    let mut bytes = scheme.bytes();
    bytes.next()?.is_ascii_alphabetic().then_some(())?;
    bytes
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.'))
        .then_some(scheme)
}

fn has_valid_percent_escapes(input: &str) -> bool {
    let bytes = input.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            index += 1;
            continue;
        }
        let Some(pair) = bytes.get(index + 1..index + 3) else {
            return false;
        };
        if !pair.iter().all(u8::is_ascii_hexdigit) {
            return false;
        }
        index += 3;
    }
    true
}

/// Returns an absolute display path that never depends on platform backslashes.
pub fn display_path(path: &Path) -> String {
    let mut value = path.to_string_lossy().replace('\\', "/");
    if let Some(rest) = value.strip_prefix("//?/UNC/") {
        value = format!("//{rest}");
    } else if let Some(rest) = value.strip_prefix("//?/") {
        value = rest.to_string();
    }
    value
}

/// Canonicalizes an existing path to a stable absolute form without the Windows `\\?\` prefix.
pub fn canonical_existing(path: &Path) -> io::Result<PathBuf> {
    dunce::canonicalize(path)
}

/// Normalized display value of the current session working directory.
pub fn current_dir_display() -> String {
    crate::session::current_dir()
        .ok()
        .and_then(|path| canonical_existing(&path).ok().or(Some(path)))
        .map(|path| display_path(&path))
        .unwrap_or_else(|| ".".to_string())
}

/// Translates open/read failures into the frozen permission or lock messages.
pub fn io_error_message(path: &Path, error: &io::Error) -> String {
    let display = display_path(path);
    #[cfg(windows)]
    if matches!(error.raw_os_error(), Some(32 | 33)) {
        return format!("Cannot open file (locked by another process): {display}");
    }
    if error.kind() == io::ErrorKind::PermissionDenied {
        return format!("Permission denied: {display}");
    }
    format!("Cannot open file: {display} ({error})")
}

/// Selects a high-confidence sibling candidate for a missing file.
pub fn nearest_existing_name(path: &Path) -> Option<PathBuf> {
    let parent = path.parent()?;
    let wanted = path.file_name()?.to_string_lossy();
    let mut candidates = fs::read_dir(parent)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let score = strsim::jaro_winkler(&wanted, &name);
            (score, name, entry.path())
        })
        .filter(|(_, name, _)| name != wanted.as_ref())
        .filter(|(score, _, _)| *score >= 0.80)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .0
            .total_cmp(&left.0)
            .then_with(|| left.1.as_bytes().cmp(right.1.as_bytes()))
    });
    candidates.into_iter().next().map(|(_, _, path)| path)
}

/// Builds the read error for missing or relative paths, including a recovery step when possible.
pub fn missing_file_message(input: &str) -> String {
    let cwd = crate::session::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let parsed = parse_input_path(input);
    let resolved = if parsed.is_absolute() {
        parsed.clone()
    } else {
        cwd.join(&parsed)
    };
    let requested = if parsed.is_absolute() {
        display_path(&parsed)
    } else {
        input.replace('\\', "/")
    };
    let cwd_display = current_dir_display();
    let mut note = format!("Note: the session working directory is {cwd_display}.");
    if !parsed.is_absolute() && resolved.exists() {
        let resolved_display = canonical_existing(&resolved).unwrap_or_else(|_| resolved.clone());
        note.push_str(&format!(
            " Use the absolute path {}.",
            display_path(&resolved_display)
        ));
    }
    let mut message = format!("File does not exist: {requested}\n{note}");
    if let Some(candidate) = nearest_existing_name(&resolved) {
        let candidate = canonical_existing(&candidate).unwrap_or(candidate);
        message.push_str(&format!("\nDid you mean: {}?", display_path(&candidate)));
    }
    message
}

/// Builds read's missing-file error and explains why a lossy U+FFFD filename cannot round-trip as text.
pub fn missing_read_file_message(input: &str) -> String {
    let mut message = missing_file_message(input);
    if input.contains('\u{FFFD}') {
        message.push_str(
            "\nNote: this path contains U+FFFD (a placeholder for bytes that are not valid text); it looks like the lossy rendering of a filename that cannot be represented as text and cannot be opened by name.",
        );
    }
    message
}

/// Builds the missing-path error shared by grep and glob.
pub fn missing_search_path_message(input: &str) -> String {
    let cwd = crate::session::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let parsed = parse_input_path(input);
    let resolved = if parsed.is_absolute() {
        parsed
    } else {
        cwd.join(&parsed)
    };
    let mut note = format!(
        "Note: the session working directory is {}.",
        current_dir_display()
    );
    if !Path::new(input).is_absolute() && resolved.exists() {
        let absolute = canonical_existing(&resolved).unwrap_or(resolved);
        note.push_str(&format!(
            " Use the absolute path {}.",
            display_path(&absolute)
        ));
    }
    format!("Path does not exist: {}\n{note}", input.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::{display_path, parse_input_path, parse_local_path_input};

    #[test]
    fn local_file_uri_input_round_trips_and_rejects_every_ambiguous_shape() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("space 中文.txt");
        let plain = display_path(&path);
        assert_eq!(
            parse_local_path_input(&plain).unwrap(),
            parse_input_path(&plain)
        );

        let uri = url::Url::from_file_path(&path).unwrap().to_string();
        assert!(uri.contains("%20"), "{uri}");
        assert_eq!(parse_local_path_input(&uri).unwrap(), path);
        let mut uppercase = uri.clone();
        uppercase.replace_range(..4, "FILE");
        assert_eq!(parse_local_path_input(&uppercase).unwrap(), path);
        let localhost = uri.replacen("file:///", "file://LOCALHOST/", 1);
        assert_eq!(parse_local_path_input(&localhost).unwrap(), path);

        for (input, expected) in [
            (
                "https://example.invalid/file.txt",
                "Unsupported URI scheme \"https\" for a local filesystem path.",
            ),
            (
                "app://connector/file.txt",
                "Unsupported URI scheme \"app\" for a local filesystem path.",
            ),
            (
                "file:relative.txt",
                "Invalid local file URI: expected an absolute URI with a local authority.",
            ),
            (
                "file:///tmp/bad%ZZ.txt",
                "Invalid local file URI: malformed percent escape.",
            ),
            (
                "file://server/share/file.txt",
                "Invalid local file URI: remote authorities are not supported.",
            ),
            (
                "file:///tmp/file.txt?query",
                "Invalid local file URI: query and fragment components are not supported.",
            ),
            (
                "file:///tmp/file.txt#fragment",
                "Invalid local file URI: query and fragment components are not supported.",
            ),
        ] {
            assert_eq!(
                parse_local_path_input(input).unwrap_err(),
                expected,
                "{input}"
            );
        }
        for input in [
            "file://user@localhost/tmp/file.txt",
            "file://localhost:80/tmp/file.txt",
            "file:///tmp/nul%00.txt",
        ] {
            let error = parse_local_path_input(input).unwrap_err();
            assert!(
                error.starts_with("Invalid local file URI:"),
                "{input}: {error}"
            );
            assert!(!error.contains(input), "the raw URI leaked into: {error}");
        }
    }

    #[test]
    fn a_windows_drive_designator_is_never_read_as_a_uri_scheme() {
        // Rejecting `C:notes.txt` as an unsupported protocol would send the caller after a
        // scheme problem it does not have; a drive-relative path is just a path.
        for input in [
            "C:/repo/notes.txt",
            "C:\\repo\\notes.txt",
            "C:notes.txt",
            "v:relative/notes.txt",
        ] {
            assert_eq!(
                parse_local_path_input(input).unwrap(),
                parse_input_path(input),
                "{input}"
            );
        }
    }
}
