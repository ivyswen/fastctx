mod common;

use common::{McpSession, mcp_text, normalized, write};
use serde_json::{Value, json};
use std::path::Path;
use std::process::Command;

#[test]
fn every_file_tool_accepts_local_file_uris_without_echoing_them() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("URI space 中文");
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("sample file.txt");
    write(&file, b"hit one\nhit two\nhit three");
    let plain_file = normalized(&file);
    let plain_root = normalized(&root);
    let uri_file = file_uri(&file);
    let uri_root = file_uri(&root);
    let mut session = session(temp.path(), &root);

    assert_same_success(
        &mut session,
        "inspect_local_file",
        json!({"file_path": plain_file}),
        json!({"file_path": uri_file}),
    );
    let batch = assert_same_success(
        &mut session,
        "inspect_local_file",
        json!({"files": [{"path": plain_file, "limit": 2}]}),
        json!({"files": [{"path": uri_file, "limit": 2}]}),
    );
    assert!(batch.contains("Continue with files="), "{batch}");
    assert!(!batch.contains("file:"), "{batch}");

    assert_same_success(
        &mut session,
        "grep",
        json!({"pattern": "hit", "path": plain_root, "output_mode": "content"}),
        json!({"pattern": "hit", "path": uri_root, "output_mode": "content"}),
    );
    assert_same_success(
        &mut session,
        "glob",
        json!({"pattern": "*.txt", "path": plain_root}),
        json!({"pattern": "*.txt", "path": uri_root}),
    );
    assert_same_success(
        &mut session,
        "replace",
        json!({
            "path": plain_file, "pattern": "hit", "replacement": "MISS", "dry_run": true
        }),
        json!({
            "path": uri_file, "pattern": "hit", "replacement": "MISS", "dry_run": true
        }),
    );
    let written = session.call(
        "replace",
        json!({"path": uri_file, "pattern": "hit", "replacement": "MISS"}),
    );
    assert_eq!(written["result"]["isError"], false, "{written}");
    assert!(!mcp_text(&written).contains("file:"), "{written}");
    let replaced = b"MISS one\nMISS two\nMISS three";
    assert_eq!(std::fs::read(&file).unwrap(), replaced);

    let unsupported = "https://example.invalid/file.txt";
    for (tool, arguments) in [
        ("inspect_local_file", json!({"file_path": unsupported})),
        (
            "inspect_local_file",
            json!({"files": [{"path": unsupported}]}),
        ),
        ("grep", json!({"pattern": "hit", "path": unsupported})),
        ("glob", json!({"pattern": "*", "path": unsupported})),
        (
            "replace",
            json!({"path": unsupported, "pattern": "hit", "replacement": "MISS"}),
        ),
    ] {
        let response = session.call(tool, arguments);
        assert_eq!(response["result"]["isError"], true, "{tool}: {response}");
        assert_eq!(
            mcp_text(&response),
            "Unsupported URI scheme \"https\" for a local filesystem path.",
            "{tool}"
        );
        assert!(!mcp_text(&response).contains(unsupported));
    }
    assert_eq!(
        std::fs::read(&file).unwrap(),
        replaced,
        "unsupported URI inputs must not mutate an existing local target"
    );

    assert!(session.close().success());
}

#[cfg(unix)]
#[test]
fn batch_uri_continuations_match_the_equivalent_plain_symlink_path() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    let target = root.join("target.txt");
    let alias = root.join("alias.txt");
    write(&target, b"one\ntwo\nthree");
    symlink(&target, &alias).unwrap();
    let plain_alias = normalized(&alias);
    let uri_alias = file_uri(&alias);
    let mut session = session(temp.path(), &root);

    let response = assert_same_success(
        &mut session,
        "inspect_local_file",
        json!({"files": [{"path": plain_alias, "limit": 1}]}),
        json!({"files": [{"path": uri_alias, "limit": 1}]}),
    );
    assert!(response.contains(&normalized(&alias)), "{response}");
    assert!(!response.contains("file:"), "{response}");
    assert!(session.close().success());
}

fn assert_same_success(
    session: &mut McpSession,
    tool: &str,
    plain_arguments: Value,
    uri_arguments: Value,
) -> String {
    let plain = session.call(tool, plain_arguments);
    let uri = session.call(tool, uri_arguments);
    assert_eq!(plain["result"]["isError"], false, "{tool}: {plain}");
    assert_eq!(uri["result"]["isError"], false, "{tool}: {uri}");
    assert_eq!(mcp_text(&uri), mcp_text(&plain), "{tool}");
    assert!(!mcp_text(&uri).contains("file:"), "{tool}: {uri}");
    mcp_text(&uri).to_string()
}

fn file_uri(path: &Path) -> String {
    url::Url::from_file_path(path).unwrap().to_string()
}

fn session(home: &Path, cwd: &Path) -> McpSession {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastctx"));
    command
        .arg("serve")
        .current_dir(cwd)
        .env("HOME", home)
        .env("USERPROFILE", home);
    #[cfg(unix)]
    {
        let runtime = home.join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        command.env("XDG_RUNTIME_DIR", runtime);
    }
    #[cfg(windows)]
    command
        .env("LOCALAPPDATA", home)
        .env("TEMP", home)
        .env("TMP", home);
    McpSession::start(command)
}
