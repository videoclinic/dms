use std::{fs, process::Command};

use serde_json::Value;

fn dms() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dms"))
}

#[test]
fn cli_requires_confirmation_before_initializing_a_workspace() {
    let edit_root = tempfile::tempdir().expect("edit root");
    let publish_root = tempfile::tempdir().expect("publish root");
    let output = dms()
        .args(["workspace", "init", "--edit-root"])
        .arg(edit_root.path())
        .arg("--publish-root")
        .arg(publish_root.path())
        .output()
        .expect("run dms");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires --confirm"));
    assert!(!edit_root.path().join(".dms").exists());
}

#[test]
fn cli_initializes_registers_and_lists_a_document_as_json() {
    let edit_root = tempfile::tempdir().expect("edit root");
    let publish_root = tempfile::tempdir().expect("publish root");
    let source = edit_root.path().join("policies/Access.md");
    fs::create_dir_all(source.parent().expect("source parent")).expect("create source parent");
    fs::write(&source, "# Access\n").expect("write source");

    let init = dms()
        .arg("--json")
        .args(["workspace", "init", "--edit-root"])
        .arg(edit_root.path())
        .arg("--publish-root")
        .arg(publish_root.path())
        .arg("--confirm")
        .output()
        .expect("initialize workspace");
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let workspace: Value = serde_json::from_slice(&init.stdout).expect("workspace JSON");
    assert!(workspace["workspace_id"].is_string());

    let add = dms()
        .arg("--json")
        .args(["document", "add", "--edit-root"])
        .arg(edit_root.path())
        .arg("--path")
        .arg(&source)
        .output()
        .expect("register document");
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let document: Value = serde_json::from_slice(&add.stdout).expect("document JSON");
    assert_eq!(document["control"]["title"], "Access");

    let list = dms()
        .arg("--json")
        .args(["document", "list", "--edit-root"])
        .arg(edit_root.path())
        .output()
        .expect("list documents");
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let documents: Value = serde_json::from_slice(&list.stdout).expect("list JSON");
    assert_eq!(documents.as_array().expect("document array").len(), 1);
    assert_eq!(documents[0]["relative_path"], "policies/Access.md");
}

#[test]
fn cli_adds_and_lists_document_notes() {
    let edit_root = tempfile::tempdir().expect("edit root");
    let publish_root = tempfile::tempdir().expect("publish root");
    let source = edit_root.path().join("draft.md");
    fs::write(&source, "# Draft\n").expect("write source");

    let init = dms()
        .args(["workspace", "init", "--edit-root"])
        .arg(edit_root.path())
        .arg("--publish-root")
        .arg(publish_root.path())
        .arg("--confirm")
        .output()
        .expect("initialize workspace");
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );

    let add = dms()
        .arg("--json")
        .args(["document", "add", "--edit-root"])
        .arg(edit_root.path())
        .arg("--path")
        .arg(&source)
        .output()
        .expect("register document");
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let document: Value = serde_json::from_slice(&add.stdout).expect("document JSON");
    let document_id = document["id"].as_str().expect("document ID");

    let note = dms()
        .arg("--json")
        .args(["note", "add", "--edit-root"])
        .arg(edit_root.path())
        .args([
            "--document",
            document_id,
            "--body",
            "Initial note",
            "--author",
            "Raphael",
        ])
        .output()
        .expect("add note");
    assert!(
        note.status.success(),
        "{}",
        String::from_utf8_lossy(&note.stderr)
    );

    let list = dms()
        .arg("--json")
        .args(["note", "list", "--edit-root"])
        .arg(edit_root.path())
        .args(["--document", document_id])
        .output()
        .expect("list notes");
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let notes: Value = serde_json::from_slice(&list.stdout).expect("notes JSON");
    assert_eq!(notes[0]["body"], "Initial note");
    assert_eq!(notes[0]["author"], "Raphael");
}
