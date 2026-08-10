use std::{fs, process::Command};

use serde_json::Value;
use uuid::Uuid;

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

#[test]
fn cli_configures_catalogues_folder_policies_and_identity_routing() {
    let edit_root = tempfile::tempdir().expect("edit root");
    let publish_root = tempfile::tempdir().expect("publish root");
    fs::create_dir_all(edit_root.path().join("policies/empty")).expect("policy folders");
    let init = dms()
        .args(["workspace", "init", "--edit-root"])
        .arg(edit_root.path())
        .arg("--publish-root")
        .arg(publish_root.path())
        .arg("--confirm")
        .output()
        .expect("initialize workspace");
    assert!(init.status.success());

    for args in [
        vec![
            "policy",
            "configure-confidentiality-type",
            "--id",
            "internal",
            "--label",
            "Internal",
            "--root",
        ],
        vec![
            "policy",
            "configure-document-type",
            "--id",
            "procedure",
            "--label",
            "Procedure",
        ],
    ] {
        let output = dms()
            .args(args)
            .arg("--edit-root")
            .arg(edit_root.path())
            .output()
            .expect("configure policy");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let folders = dms()
        .arg("--json")
        .args(["policy", "folders", "--edit-root"])
        .arg(edit_root.path())
        .output()
        .expect("list policy folders");
    let folders: Value = serde_json::from_slice(&folders.stdout).expect("folder JSON");
    assert_eq!(folders[0]["relative_path"], ".");
    assert!(folders
        .as_array()
        .expect("folders")
        .iter()
        .any(|folder| folder["relative_path"] == "policies/empty"));

    let remove_root = dms()
        .args([
            "policy",
            "remove-confidentiality",
            "--folder",
            ".",
            "--edit-root",
        ])
        .arg(edit_root.path())
        .output()
        .expect("reject root removal");
    assert!(!remove_root.status.success());
    assert!(String::from_utf8_lossy(&remove_root.stderr).contains("required"));

    let tenant_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let editor_id = Uuid::new_v4();
    let approver_id = Uuid::new_v4();
    let people_path = edit_root.path().join("eligible-people.json");
    fs::write(
        &people_path,
        serde_json::to_vec(&serde_json::json!([
            {"object_id": editor_id, "display_name": "Editor", "email": "editor@example.test", "account_enabled": true},
            {"object_id": approver_id, "display_name": "Approver", "email": "approver@example.test", "account_enabled": true}
        ]))
        .expect("people JSON"),
    )
    .expect("people file");
    let identity = dms()
        .arg("--json")
        .args(["policy", "replace-identity-source", "--edit-root"])
        .arg(edit_root.path())
        .args(["--tenant-id", &tenant_id.to_string()])
        .args(["--tenant-display", "Example tenant"])
        .args(["--group-id", &group_id.to_string()])
        .args(["--group-label", "DMS workflow"])
        .args([
            "--eligible-people",
            &format!("@file:{}", people_path.display()),
        ])
        .args(["--root-editor", &editor_id.to_string()])
        .args(["--root-approver", &approver_id.to_string()])
        .output()
        .expect("replace identity source");
    assert!(
        identity.status.success(),
        "{}",
        String::from_utf8_lossy(&identity.stderr)
    );
    let metadata =
        fs::read_to_string(edit_root.path().join(".dms/workspace.json")).expect("metadata");
    assert!(metadata.contains(&tenant_id.to_string()));
    assert!(!metadata.contains("client_secret"));
    assert!(!metadata.contains("access_token"));
}

#[test]
fn cli_lists_searches_unregisters_and_reassociates_library_files() {
    let edit_root = tempfile::tempdir().expect("edit root");
    let publish_root = tempfile::tempdir().expect("publish root");
    let source = edit_root.path().join("Policies/Handbook.md");
    fs::create_dir_all(source.parent().expect("source parent")).expect("source folder");
    fs::write(&source, "# Handbook").expect("source");
    assert!(dms()
        .args(["workspace", "init", "--edit-root"])
        .arg(edit_root.path())
        .arg("--publish-root")
        .arg(publish_root.path())
        .arg("--confirm")
        .status()
        .expect("init")
        .success());

    let listing = dms()
        .arg("--json")
        .args(["library", "list", "--edit-root"])
        .arg(edit_root.path())
        .args(["--folder", "Policies"])
        .output()
        .expect("list folder");
    assert!(listing.status.success());
    let listing: Value = serde_json::from_slice(&listing.stdout).expect("listing JSON");
    assert_eq!(listing["entries"][0]["name"], "Handbook.md");
    assert_eq!(listing["entries"][0]["membership"], "not_in_library");

    let add = dms()
        .arg("--json")
        .args(["document", "add", "--edit-root"])
        .arg(edit_root.path())
        .arg("--path")
        .arg(&source)
        .output()
        .expect("add document");
    assert!(add.status.success());
    let document: Value = serde_json::from_slice(&add.stdout).expect("document JSON");
    let document_id = document["id"].as_str().expect("document ID");

    let search = dms()
        .arg("--json")
        .args(["library", "search", "--edit-root"])
        .arg(edit_root.path())
        .args(["--query", "HANDBOOK.MD"])
        .output()
        .expect("search library");
    assert!(search.status.success());
    let results: Value = serde_json::from_slice(&search.stdout).expect("search JSON");
    assert_eq!(results[0]["document"]["id"], document_id);

    let unregister = dms()
        .arg("--json")
        .args(["document", "unregister", "--edit-root"])
        .arg(edit_root.path())
        .args(["--document", document_id])
        .output()
        .expect("unregister");
    assert!(unregister.status.success());
    let unregistered: Value = serde_json::from_slice(&unregister.stdout).expect("unregister JSON");
    assert_eq!(unregistered["source_state"], "unregistered");

    let renamed = edit_root.path().join("Policies/Staff-Handbook.md");
    fs::rename(&source, &renamed).expect("external rename");
    let reassociate = dms()
        .arg("--json")
        .args(["document", "reassociate", "--edit-root"])
        .arg(edit_root.path())
        .args(["--document", document_id, "--path"])
        .arg(&renamed)
        .output()
        .expect("reassociate");
    assert!(reassociate.status.success());
    let reassociated: Value =
        serde_json::from_slice(&reassociate.stdout).expect("reassociate JSON");
    assert_eq!(reassociated["id"], document_id);
    assert_eq!(reassociated["relative_path"], "Policies/Staff-Handbook.md");
}

#[test]
fn cli_exposes_periodic_review_closure_commands_and_requires_confirmation_first() {
    let help = dms()
        .args(["periodic-review", "--help"])
        .output()
        .expect("periodic-review help");
    assert!(help.status.success());
    let help = String::from_utf8_lossy(&help.stdout);
    for command in ["list", "start", "result", "cancel", "reminder"] {
        assert!(help.contains(command), "missing {command} in {help}");
    }

    let id = "00000000-0000-0000-0000-000000000000";
    let commands = [
        vec![
            "periodic-review",
            "result",
            "--edit-root",
            "missing",
            "--document",
            id,
            "--review",
            id,
            "--result",
            "confirmed-current",
            "--comment",
            "Current",
        ],
        vec![
            "periodic-review",
            "cancel",
            "--edit-root",
            "missing",
            "--document",
            id,
            "--review",
            id,
            "--comment",
            "Cancelled",
        ],
        vec![
            "periodic-review",
            "reminder",
            "--edit-root",
            "missing",
            "--document",
            id,
            "--review",
            id,
        ],
    ];
    for args in commands {
        let output = dms().args(args).output().expect("periodic-review command");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("requires --confirm"));
    }
}
