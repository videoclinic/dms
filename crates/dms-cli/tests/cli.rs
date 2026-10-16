use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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
fn cli_exposes_advisory_lock_lifecycle_with_explicit_release() {
    let edit_root = tempfile::tempdir().unwrap();
    let publish_root = tempfile::tempdir().unwrap();
    let initialized = dms()
        .args(["workspace", "init", "--edit-root"])
        .arg(edit_root.path())
        .arg("--publish-root")
        .arg(publish_root.path())
        .arg("--confirm")
        .output()
        .unwrap();
    assert!(initialized.status.success());

    let status = dms()
        .arg("--json")
        .args(["workspace", "lock-status", "--edit-root"])
        .arg(edit_root.path())
        .output()
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&status.stdout).unwrap()["state"],
        "unlocked"
    );
    let acquired = dms()
        .args(["workspace", "lock-acquire", "--edit-root"])
        .arg(edit_root.path())
        .output()
        .unwrap();
    assert!(acquired.status.success());
    let duplicate = dms()
        .args(["workspace", "lock-acquire", "--edit-root"])
        .arg(edit_root.path())
        .output()
        .unwrap();
    assert!(!duplicate.status.success());
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("current advisory lock"));

    let unconfirmed = dms()
        .args(["workspace", "lock-release", "--edit-root"])
        .arg(edit_root.path())
        .output()
        .unwrap();
    assert!(!unconfirmed.status.success());
    assert!(edit_root.path().join(".dms/lock").is_file());
    let released = dms()
        .args(["workspace", "lock-release", "--edit-root"])
        .arg(edit_root.path())
        .arg("--confirm")
        .output()
        .unwrap();
    assert!(released.status.success());
    assert!(!edit_root.path().join(".dms/lock").exists());
}

#[test]
fn cli_backup_and_restore_require_confirmation_and_rewrite_roots() {
    let source_edit = tempfile::tempdir().unwrap();
    let source_publish = tempfile::tempdir().unwrap();
    let archive_directory = tempfile::tempdir().unwrap();
    let archive = archive_directory.path().join("workspace.zip");
    assert!(dms()
        .args(["workspace", "init", "--edit-root"])
        .arg(source_edit.path())
        .arg("--publish-root")
        .arg(source_publish.path())
        .arg("--confirm")
        .status()
        .unwrap()
        .success());
    let backup = dms()
        .arg("--json")
        .args(["workspace", "backup", "--edit-root"])
        .arg(source_edit.path())
        .arg("--archive")
        .arg(&archive)
        .output()
        .unwrap();
    assert!(backup.status.success());
    assert!(archive.is_file());

    let destination = tempfile::tempdir().unwrap();
    let edit_root = destination.path().join("edit");
    let publish_root = destination.path().join("publish");
    fs::create_dir(&edit_root).unwrap();
    fs::create_dir(&publish_root).unwrap();
    let restore = |confirm: bool| {
        let mut command = dms();
        command
            .arg("--json")
            .args(["workspace", "restore", "--archive"])
            .arg(&archive)
            .arg("--edit-root")
            .arg(&edit_root)
            .arg("--publish-root")
            .arg(&publish_root);
        if confirm {
            command.arg("--confirm");
        }
        command.output().unwrap()
    };
    let unconfirmed = restore(false);
    assert!(!unconfirmed.status.success());
    assert!(!edit_root.join(".dms/workspace.json").exists());
    let restored = restore(true);
    assert!(
        restored.status.success(),
        "{}",
        String::from_utf8_lossy(&restored.stderr)
    );
    let restored: Value = serde_json::from_slice(&restored.stdout).unwrap();
    assert_eq!(
        restored["edit_root"],
        fs::canonicalize(&edit_root)
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );
    assert_eq!(
        restored["publish_root"],
        fs::canonicalize(&publish_root)
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );
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
    let instructions =
        fs::read_to_string(edit_root.path().join(".dms/AGENTS.md")).expect("agent instructions");
    assert!(instructions.contains("dms --json workspace verify"));
    assert!(instructions.contains("workspace.json"));

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
    assert!(metadata.contains(&group_id.to_string()));
    assert!(!metadata.contains("tenant_display"));
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

#[test]
fn cli_generates_lists_and_verifies_filtered_audit_reports() {
    let temp = tempfile::tempdir().expect("tempdir");
    let edit = temp.path().join("edit");
    let publish = temp.path().join("publish");
    fs::create_dir_all(edit.join("Policies")).expect("edit root");
    assert!(dms()
        .args([
            "workspace",
            "init",
            "--edit-root",
            edit.to_str().unwrap(),
            "--publish-root",
            publish.to_str().unwrap(),
            "--confirm",
        ])
        .status()
        .unwrap()
        .success());
    assert!(dms()
        .args([
            "policy",
            "configure-confidentiality-type",
            "--edit-root",
            edit.to_str().unwrap(),
            "--id",
            "internal",
            "--label",
            "Internal",
            "--root",
        ])
        .status()
        .unwrap()
        .success());
    let source = edit.join("Policies/Handbook.md");
    fs::write(&source, "# Handbook\n\nPRIVATE SOURCE CONTENT\n").unwrap();
    let added = dms()
        .args([
            "--json",
            "document",
            "add",
            "--edit-root",
            edit.to_str().unwrap(),
            "--path",
            source.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(added.status.success());
    let document: serde_json::Value = serde_json::from_slice(&added.stdout).unwrap();
    let document_id = document["id"].as_str().unwrap();

    let generated = dms()
        .args([
            "--json",
            "report",
            "generate",
            "--edit-root",
            edit.to_str().unwrap(),
            "--format",
            "csv",
            "--output",
            ".dms/exports/handbook.csv",
            "--document",
            document_id,
            "--confidentiality",
            "internal",
        ])
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&generated.stdout).unwrap();
    let event_id = report["event_id"].as_str().unwrap();
    let bytes = fs::read(edit.join(".dms/exports/handbook.csv")).unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("Handbook"));
    assert!(!String::from_utf8_lossy(&bytes).contains("PRIVATE SOURCE CONTENT"));

    let listed = dms()
        .args([
            "--json",
            "report",
            "list",
            "--edit-root",
            edit.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(listed.status.success());
    let reports: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(reports.as_array().unwrap().len(), 1);

    let verified = dms()
        .args([
            "--json",
            "report",
            "verify",
            "--edit-root",
            edit.to_str().unwrap(),
            "--event",
            event_id,
        ])
        .output()
        .unwrap();
    assert!(verified.status.success());
    let verification: serde_json::Value = serde_json::from_slice(&verified.stdout).unwrap();
    assert_eq!(verification["status"], "match");
}

/// The repository's synthetic template; conversion tests never touch a real
/// operator template, corpus or workspace.
const TEMPLATE_FIXTURE: &str = "../dms-core/tests/fixtures/markdown-template.docx";
const TEMPLATE_RELATIVE: &str = "Vorlage.docx";
const TEMPLATE_SHA256: &str = "9904d366848952c1719fe6d4e58a27a7e553ec2f72b403014bea1e02b38f9c68";
const VALID_MARKDOWN: &str = "---\ntitle: Access Control Policy\nversion: 3.0\nconfidentiality: internal\n---\n\n# Access Control Policy\n\nAccess is granted by role.\n";

struct ConversionLayout {
    _temp: tempfile::TempDir,
    config: PathBuf,
    input: PathBuf,
    output: PathBuf,
}

/// A throwaway edit/publish/input/output tree plus a read-only export
/// projection. Unrelated identity, document and workflow fields are present
/// and deliberately invalid: a correct projection must ignore them entirely.
fn workspace_config_value(schema_version: u32, edit: &Path, publish: &Path) -> Value {
    serde_json::json!({
        "schema_version": schema_version,
        "workspace_id": "11111111-1111-1111-1111-111111111111",
        "edit_root": edit,
        "publish_root": publish,
        "markdown_template": {
            "id": "22222222-2222-2222-2222-222222222222",
            "relative_path": TEMPLATE_RELATIVE,
            "sha256": TEMPLATE_SHA256,
            "contract_version": 1
        },
        "confidentiality_types": {
            "internal": {
                "id": "internal",
                "label": "Intern + Unternehmensverbund",
                "enabled": true
            }
        },
        // Unrelated and invalid: no identity, document or workflow data is a
        // conversion input.
        "identity_source": { "binding_id": "not-a-uuid" },
        "identity_cache": { "also-not-a-uuid": { "display_name": 5 } },
        "documents": "not-an-object",
        "workflow_policies": [1, 2, 3],
        "workspace_events": [{ "unknown": true }]
    })
}

fn conversion_layout(schema_version: u32) -> ConversionLayout {
    let temp = tempfile::tempdir().expect("conversion temp dir");
    let base = temp.path().canonicalize().expect("canonical temp dir");
    let edit = base.join("edit");
    let publish = base.join("publish");
    let input = base.join("input");
    fs::create_dir_all(&edit).expect("edit root");
    fs::create_dir_all(&publish).expect("publish root");
    fs::create_dir_all(&input).expect("input root");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(TEMPLATE_FIXTURE),
        edit.join(TEMPLATE_RELATIVE),
    )
    .expect("copy template fixture");
    let metadata_dir = edit.join(".dms");
    fs::create_dir_all(&metadata_dir).expect("metadata dir");
    let config = metadata_dir.join("workspace.json");
    fs::write(
        &config,
        serde_json::to_vec_pretty(&workspace_config_value(schema_version, &edit, &publish))
            .expect("config JSON"),
    )
    .expect("write config");
    ConversionLayout {
        _temp: temp,
        config,
        input,
        output: base.join("out"),
    }
}

fn convert_markdown(layout: &ConversionLayout, extra: &[&str]) -> std::process::Output {
    convert_markdown_to(layout, &layout.output, extra)
}

fn convert_markdown_to(
    layout: &ConversionLayout,
    output: &Path,
    extra: &[&str],
) -> std::process::Output {
    let mut command = dms();
    command
        .arg("--json")
        .args(["convert", "markdown", "--workspace-config"])
        .arg(&layout.config)
        .arg("--input-root")
        .arg(&layout.input)
        .arg("--output-root")
        .arg(output);
    for arg in extra {
        command.arg(arg);
    }
    command.output().expect("run convert markdown")
}

fn result_inputs(manifest: &Value) -> Vec<String> {
    manifest["results"]
        .as_array()
        .expect("results array")
        .iter()
        .map(|result| result["input"].as_str().expect("input path").to_owned())
        .collect()
}

#[test]
fn cli_convert_markdown_documents_and_requires_its_flags() {
    let help = dms()
        .args(["convert", "markdown", "--help"])
        .output()
        .expect("convert markdown help");
    assert!(
        help.status.success(),
        "{}",
        String::from_utf8_lossy(&help.stderr)
    );
    let help = String::from_utf8_lossy(&help.stdout);
    for flag in [
        "--workspace-config",
        "--input-root",
        "--output-root",
        "--recursive",
        "--dry-run",
        "--without-toc",
    ] {
        assert!(help.contains(flag), "missing {flag} in {help}");
    }

    let missing = dms()
        .args(["convert", "markdown"])
        .output()
        .expect("convert markdown without flags");
    assert!(!missing.status.success());
    let stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(stderr.contains("--workspace-config"), "{stderr}");
    assert!(stderr.contains("--input-root"), "{stderr}");
    assert!(stderr.contains("--output-root"), "{stderr}");
}

#[test]
fn cli_convert_markdown_dry_run_lists_exact_selection_and_writes_nothing() {
    let layout = conversion_layout(21);
    fs::write(layout.input.join("policy-a.md"), VALID_MARKDOWN).unwrap();
    fs::write(layout.input.join("policy-b.md"), VALID_MARKDOWN).unwrap();
    fs::create_dir_all(layout.input.join("nested")).unwrap();
    fs::write(layout.input.join("nested/deep.md"), VALID_MARKDOWN).unwrap();
    fs::create_dir_all(layout.input.join(".dms")).unwrap();
    fs::write(layout.input.join(".dms/hidden.md"), VALID_MARKDOWN).unwrap();
    fs::create_dir_all(layout.input.join("assets")).unwrap();
    fs::write(layout.input.join("assets/note.md"), VALID_MARKDOWN).unwrap();
    fs::write(layout.input.join("assets/logo.png"), b"not used").unwrap();
    fs::write(layout.input.join("notes.txt"), b"not markdown").unwrap();

    let output = convert_markdown(&layout, &["--dry-run"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("policy-a.md"), "{stderr}");
    assert!(stderr.contains("policy-b.md"), "{stderr}");
    assert!(!stderr.contains("deep.md"), "{stderr}");
    assert!(!stderr.contains("hidden.md"), "{stderr}");
    assert!(!stderr.contains("note.md"), "{stderr}");

    let manifest: Value = serde_json::from_slice(&output.stdout).expect("manifest JSON");
    assert_eq!(manifest["dry_run"], true);
    assert_eq!(manifest["conversion_only"], true);
    assert_eq!(manifest["configuration_schema"], 21);
    let inputs = result_inputs(&manifest);
    assert_eq!(inputs.len(), 2, "{inputs:?}");
    assert!(inputs.iter().any(|path| path.ends_with("policy-a.md")));
    assert!(inputs.iter().any(|path| path.ends_with("policy-b.md")));
    for result in manifest["results"].as_array().unwrap() {
        assert_eq!(result["status"], "validated");
        assert!(result["docx"].is_null());
        assert!(result["pdf"].is_null());
    }
    assert!(
        !layout.output.exists(),
        "dry-run must not create the output root"
    );
}

#[test]
fn cli_convert_markdown_recursion_is_explicit_and_excludes_metadata_and_assets() {
    let layout = conversion_layout(21);
    fs::write(layout.input.join("policy-a.md"), VALID_MARKDOWN).unwrap();
    fs::create_dir_all(layout.input.join("nested")).unwrap();
    fs::write(layout.input.join("nested/deep.md"), VALID_MARKDOWN).unwrap();
    fs::create_dir_all(layout.input.join(".dms")).unwrap();
    fs::write(layout.input.join(".dms/hidden.md"), VALID_MARKDOWN).unwrap();
    fs::create_dir_all(layout.input.join("assets")).unwrap();
    fs::write(layout.input.join("assets/note.md"), VALID_MARKDOWN).unwrap();

    let output = convert_markdown(&layout, &["--dry-run", "--recursive", "--without-toc"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("nested"), "{stderr}");
    assert!(!stderr.contains("hidden.md"), "{stderr}");
    assert!(!stderr.contains("note.md"), "{stderr}");

    let manifest: Value = serde_json::from_slice(&output.stdout).expect("manifest JSON");
    assert_eq!(manifest["without_toc"], true);
    let inputs = result_inputs(&manifest);
    assert_eq!(inputs.len(), 2, "{inputs:?}");
    assert!(inputs.iter().any(|path| path.ends_with("policy-a.md")));
    assert!(inputs.iter().any(|path| path.ends_with("deep.md")));
}

#[test]
fn cli_convert_markdown_accepts_supported_schemas_and_ignores_unrelated_data() {
    for schema_version in [18_u32, 19, 21, dms_core::SCHEMA_VERSION] {
        let layout = conversion_layout(schema_version);
        fs::write(layout.input.join("policy.md"), VALID_MARKDOWN).unwrap();
        let output = convert_markdown(&layout, &["--dry-run"]);
        assert!(
            output.status.success(),
            "schema {schema_version}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let manifest: Value = serde_json::from_slice(&output.stdout).expect("manifest JSON");
        assert_eq!(manifest["configuration_schema"], schema_version);
        assert_eq!(manifest["results"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn cli_todo_policy_resolution_uses_core_without_a_configuration_command() {
    use dms_core::{ConversionOptions, ConversionPlan, TodoProjection};
    for library in [None, Some("disabled"), Some("enabled")] {
        let layout = conversion_layout(dms_core::SCHEMA_VERSION);
        let mut config: Value = serde_json::from_slice(&fs::read(&layout.config).unwrap()).unwrap();
        if let Some(setting) = library {
            config["conversion_policy"] = serde_json::json!({"todo_projection": setting});
        }
        fs::write(&layout.config, serde_json::to_vec(&config).unwrap()).unwrap();
        let source_path = layout.input.join("policy.md");
        fs::write(&source_path, VALID_MARKDOWN).unwrap();
        let before = fs::read(&layout.config).unwrap();
        // Exercise the public resolver and the exact subprocess conversion path.
        let plan = ConversionPlan::prepare(&ConversionOptions {
            workspace_config: layout.config.clone(),
            input_root: layout.input.clone(),
            output_root: layout.output.clone(),
            recursive: false,
            dry_run: true,
            without_toc: false,
        })
        .unwrap();
        for (extra, expected) in [
            (
                "",
                if library == Some("enabled") {
                    TodoProjection::Enabled
                } else {
                    TodoProjection::Disabled
                },
            ),
            ("todo_projection: enabled\n", TodoProjection::Enabled),
            ("todo_projection: disabled\n", TodoProjection::Disabled),
        ] {
            let source = VALID_MARKDOWN.replacen("title:", &format!("{extra}title:"), 1);
            assert_eq!(plan.resolve_todo_projection(&source).unwrap(), expected);
            fs::write(&source_path, &source).unwrap();
            let output = convert_markdown(&layout, &["--dry-run"]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let manifest: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(manifest["results"][0]["status"], "validated");
            assert_eq!(
                manifest["results"][0]["todo_projection"],
                serde_json::to_value(expected).unwrap()
            );
            assert_eq!(fs::read_to_string(&source_path).unwrap(), source);
        }
        for invalid in ["true", "Enabled", "null", "[]", "PRIVATE_INVALID"] {
            let source = VALID_MARKDOWN.replacen(
                "title:",
                &format!("todo_projection: {invalid}\ntitle:"),
                1,
            );
            let error = plan
                .resolve_todo_projection(&source)
                .unwrap_err()
                .to_string();
            assert!(!error.contains("PRIVATE_INVALID"));
        }
        assert_eq!(fs::read(&layout.config).unwrap(), before);
        assert_eq!(
            fs::read_dir(layout.config.parent().unwrap())
                .unwrap()
                .count(),
            1
        );
        assert!(!layout.output.exists());
    }
}

#[test]
fn cli_todo_projection_reports_mixed_results_effective_policy_and_original_hashes() {
    use sha2::{Digest, Sha256};
    let layout = conversion_layout(dms_core::SCHEMA_VERSION);
    let mut config: Value = serde_json::from_slice(&fs::read(&layout.config).unwrap()).unwrap();
    config["conversion_policy"] = serde_json::json!({"todo_projection": "enabled"});
    let config_bytes = serde_json::to_vec(&config).unwrap();
    fs::write(&layout.config, &config_bytes).unwrap();
    let cases = [
        ("a-inherit.md", "", "TODO: PRIVATE_BODY\n\n# ToDo: PRIVATE_HEADING\n\n- **TODO:** PRIVATE_ITEM\n\n```text\nTODO: PRIVATE_CODE\n```\n\n| A | B |\n| --- | --- |\n| TODO: PRIVATE_CELL | Kept |\n", "validated", Some("enabled")),
        ("b-disabled.md", "todo_projection: disabled\n", "PRIVATE_KEEP TODO: PRIVATE_BODY\n", "validated", Some("disabled")),
        ("c-mixed.md", "", "PRIVATE_KEEP TODO: PRIVATE_BODY\n", "failed", Some("enabled")),
        ("d-cell.md", "", "| A | B |\n| --- | --- |\n| PRIVATE_KEEP TODO: PRIVATE_BODY | kept |\n", "failed", Some("enabled")),
        ("e-invalid.md", "todo_projection: PRIVATE_VALUE\n", "PRIVATE_BODY\n", "failed", None),
        ("f-html.md", "", "<p>PRIVATE_KEEP TODO: PRIVATE_BODY</p>\n", "failed", Some("enabled")),
        ("z-good.md", "", "Kept final body\n", "validated", Some("enabled")),
    ];
    let mut sources = Vec::new();
    for (name, extra, body, _, _) in &cases {
        let source = format!("---\ntitle: Synthetic policy\nversion: 1.0\nconfidentiality: internal\n{extra}---\n{body}");
        fs::write(layout.input.join(name), &source).unwrap();
        sources.push(source);
    }
    let output = convert_markdown(&layout, &["--dry-run"]);
    assert_eq!(output.status.code(), Some(1));
    let manifest: Value = serde_json::from_slice(&output.stdout).unwrap();
    let results = manifest["results"].as_array().unwrap();
    assert_eq!(results.len(), cases.len());
    for ((result, (name, _, _, status, policy)), source) in results.iter().zip(cases).zip(&sources)
    {
        assert_eq!(result["input"], name);
        assert_eq!(result["status"], status);
        assert_eq!(
            result["todo_projection"],
            serde_json::to_value(policy).unwrap()
        );
        assert_eq!(
            result["input_sha256"],
            format!("{:x}", Sha256::digest(source.as_bytes()))
        );
        assert!(result["docx"].is_null() && result["pdf"].is_null());
        assert_eq!(
            fs::read_to_string(layout.input.join(name)).unwrap(),
            *source
        );
    }
    assert!(results[2]["error"]
        .as_str()
        .unwrap()
        .contains("mixed TODO paragraph at line 6, column 1"));
    assert!(results[3]["error"]
        .as_str()
        .unwrap()
        .contains("mixed TODO table cell at line 8, column 2"));
    for bytes in [&output.stdout, &output.stderr] {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains("PRIVATE_") && !text.contains("Kept final body"));
    }
    assert_eq!(fs::read(&layout.config).unwrap(), config_bytes);
    assert!(!layout.output.exists());
}

#[test]
fn cli_todo_projection_checks_visible_metadata_and_definition_titles_without_source_leaks() {
    use sha2::{Digest, Sha256};
    for body in [
        "---\n{entry}\n---\n\nKept body\n",
        "+++\n{entry}\n+++\n\nKept body\n",
        "{entry}\n: Kept body\n",
    ] {
        for (policy, entry, succeeds) in [
            ("enabled", "ToDo: PRIVATE_DROP", true),
            ("enabled", "PRIVATE_KEEP TODO: PRIVATE_DROP", false),
            ("disabled", "ToDo: PRIVATE_DROP", true),
            ("disabled", "PRIVATE_KEEP TODO: PRIVATE_DROP", true),
        ] {
            let layout = conversion_layout(dms_core::SCHEMA_VERSION);
            let source = format!(
                "---\ntitle: Synthetic policy\nversion: 1.0\nconfidentiality: internal\ntodo_projection: {policy}\n---\n{}",
                body.replace("{entry}", entry)
            );
            let path = layout.input.join("policy.md");
            fs::write(&path, &source).unwrap();
            let before_config = fs::read(&layout.config).unwrap();
            let output = convert_markdown(&layout, &["--dry-run"]);
            assert_eq!(output.status.code(), Some(if succeeds { 0 } else { 1 }));
            let manifest: Value = serde_json::from_slice(&output.stdout).unwrap();
            let result = &manifest["results"][0];
            assert_eq!(
                result["status"],
                if succeeds { "validated" } else { "failed" }
            );
            assert_eq!(result["todo_projection"], policy);
            if succeeds {
                assert!(result["error"].is_null());
            } else {
                assert_eq!(result["error"], "mixed TODO paragraph at line 7, column 1: TODO: follows substantive visible content; split retained content and TODO into separate entries");
            }
            assert_eq!(
                result["input_sha256"],
                format!("{:x}", Sha256::digest(source.as_bytes()))
            );
            assert!(result["docx"].is_null() && result["pdf"].is_null());
            for bytes in [&output.stdout, &output.stderr] {
                let text = String::from_utf8_lossy(bytes);
                assert!(!text.contains("PRIVATE_") && !text.contains("Kept body"));
            }
            assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
            assert_eq!(fs::read(&layout.config).unwrap(), before_config);
            assert!(!layout.output.exists());
        }
    }
}

#[test]
fn cli_todo_projection_reports_original_crlf_unicode_cell_location() {
    use sha2::{Digest, Sha256};
    let layout = conversion_layout(dms_core::SCHEMA_VERSION);
    let source = "---\r\ntitle: Synthetic policy\r\nversion: 1.0\r\nconfidentiality: internal\r\ntodo_projection: enabled\r\n---\r\n| A | B |\r\n| --- | --- |\r\n| Grüße | PRIVATE_KEEP **TO**DO: PRIVATE_BODY |\r\n";
    let path = layout.input.join("unicode.md");
    fs::write(&path, source).unwrap();
    let before_config = fs::read(&layout.config).unwrap();
    let output = convert_markdown(&layout, &["--dry-run"]);
    assert_eq!(output.status.code(), Some(1));
    let manifest: Value = serde_json::from_slice(&output.stdout).unwrap();
    let result = &manifest["results"][0];
    assert_eq!(result["todo_projection"], "enabled");
    let column = "| Grüße |".chars().count() + 1;
    assert_eq!(result["error"], format!("mixed TODO table cell at line 9, column {column}: TODO: follows substantive visible content; split retained content and TODO into separate entries"));
    assert_eq!(
        result["input_sha256"],
        format!("{:x}", Sha256::digest(source.as_bytes()))
    );
    assert!(result["docx"].is_null() && result["pdf"].is_null());
    for bytes in [&output.stdout, &output.stderr] {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains("PRIVATE_") && !text.contains("Grüße"));
    }
    assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
    assert_eq!(fs::read(&layout.config).unwrap(), before_config);
    assert!(!layout.output.exists());
}

#[test]
fn cli_todo_projection_normal_output_names_policy_and_sanitized_failure() {
    let layout = conversion_layout(dms_core::SCHEMA_VERSION);
    fs::write(layout.input.join("disabled.md"), VALID_MARKDOWN).unwrap();
    fs::write(
        layout.input.join("enabled.md"),
        VALID_MARKDOWN.replacen("title:", "todo_projection: enabled\ntitle:", 1),
    )
    .unwrap();
    fs::write(
        layout.input.join("invalid.md"),
        VALID_MARKDOWN.replacen("title:", "todo_projection: PRIVATE_VALUE\ntitle:", 1),
    )
    .unwrap();
    fs::write(
        layout.input.join("mixed.md"),
        VALID_MARKDOWN.replacen("title:", "todo_projection: enabled\ntitle:", 1)
            + "\nPRIVATE_KEEP TODO: PRIVATE_BODY\n",
    )
    .unwrap();
    let output = dms()
        .args(["convert", "markdown", "--dry-run", "--workspace-config"])
        .arg(&layout.config)
        .arg("--input-root")
        .arg(&layout.input)
        .arg("--output-root")
        .arg(&layout.output)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "disabled.md: todo_projection=disabled",
        "enabled.md: todo_projection=enabled",
        "invalid.md: todo_projection=unresolved",
        "mixed.md: todo_projection=enabled",
        "mixed TODO paragraph at line",
    ] {
        assert!(stderr.contains(expected), "{stderr}");
    }
    assert!(!stderr.contains("PRIVATE_"));
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("validated 2 of 4 Markdown file(s); 2 failed"));
    assert!(!layout.output.exists());
}

#[test]
fn cli_todo_policy_rejects_invalid_library_values_without_writing_configuration() {
    for invalid in [
        serde_json::json!(true),
        serde_json::json!("Enabled"),
        serde_json::json!("PRIVATE_INVALID"),
    ] {
        let layout = conversion_layout(dms_core::SCHEMA_VERSION);
        fs::write(layout.input.join("policy.md"), VALID_MARKDOWN).unwrap();
        let mut config: Value = serde_json::from_slice(&fs::read(&layout.config).unwrap()).unwrap();
        config["conversion_policy"] = serde_json::json!({"todo_projection": invalid});
        let bytes = serde_json::to_vec(&config).unwrap();
        fs::write(&layout.config, &bytes).unwrap();
        let output = convert_markdown(&layout, &["--dry-run"]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("invalid conversion configuration projection"));
        assert!(!error.contains("PRIVATE_INVALID"));
        assert_eq!(fs::read(&layout.config).unwrap(), bytes);
        assert!(!layout.output.exists());
    }
}

#[test]
fn cli_convert_markdown_rejects_empty_input() {
    let layout = conversion_layout(21);
    fs::write(layout.input.join("notes.txt"), b"not markdown").unwrap();
    fs::create_dir_all(layout.input.join("assets")).unwrap();
    fs::write(layout.input.join("assets/logo.png"), b"not markdown").unwrap();

    let output = convert_markdown(&layout, &["--dry-run"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("no Markdown files"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_convert_markdown_refuses_overwrite_and_output_collisions() {
    // Existing output root: no overwrite or resume.
    let layout = conversion_layout(21);
    fs::write(layout.input.join("policy.md"), VALID_MARKDOWN).unwrap();
    fs::create_dir_all(&layout.output).unwrap();
    let existing = convert_markdown(&layout, &["--dry-run"]);
    assert!(!existing.status.success());
    assert!(
        String::from_utf8_lossy(&existing.stderr).contains("already exists"),
        "{}",
        String::from_utf8_lossy(&existing.stderr)
    );

    // A directory named like another source's DOCX target collides with it.
    let collision = conversion_layout(21);
    fs::write(collision.input.join("guide.md"), VALID_MARKDOWN).unwrap();
    fs::create_dir_all(collision.input.join("guide.docx")).unwrap();
    fs::write(collision.input.join("guide.docx/inner.md"), VALID_MARKDOWN).unwrap();
    let output = convert_markdown(&collision, &["--dry-run", "--recursive"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("collision"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_convert_markdown_refuses_output_inside_the_input_tree() {
    let layout = conversion_layout(21);
    fs::write(layout.input.join("policy.md"), VALID_MARKDOWN).unwrap();
    fs::create_dir_all(layout.input.join("sub")).unwrap();
    let nested_output = layout.input.join("sub/out");
    let output = convert_markdown_to(&layout, &nested_output, &["--dry-run"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("disjoint"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!nested_output.exists());
}

#[test]
fn cli_conversion_without_date_token_accepts_missing_invalid_and_empty_dates() {
    let layout = conversion_layout(21);
    for (name, extra) in [
        ("missing.md", ""),
        ("invalid.md", "last_modified: 2023-02-29\n"),
        ("empty.md", "last_modified:\n"),
    ] {
        fs::write(
            layout.input.join(name),
            VALID_MARKDOWN.replacen("title:", &format!("{extra}title:"), 1),
        )
        .unwrap();
    }
    let config = fs::read(&layout.config).unwrap();
    let output = convert_markdown(&layout, &["--dry-run"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(manifest["conversion_only"], true);
    let results = manifest["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r["status"] == "validated"));
    assert!(!layout.output.exists());
    assert_eq!(fs::read(&layout.config).unwrap(), config);
}

#[test]
fn cli_convert_markdown_reports_the_manifest_then_fails_on_mixed_results() {
    let layout = conversion_layout(21);
    fs::write(layout.input.join("good.md"), VALID_MARKDOWN).unwrap();
    fs::write(
        layout.input.join("bad.md"),
        "---\ntitle: Bad Version\nversion: 1\nconfidentiality: internal\n---\n\n# Bad\n",
    )
    .unwrap();

    let output = convert_markdown(&layout, &["--dry-run"]);
    assert!(!output.status.success(), "mixed results must exit nonzero");

    let manifest: Value = serde_json::from_slice(&output.stdout)
        .expect("the final manifest is still emitted on stdout");
    assert_eq!(manifest["dry_run"], true);
    let results = manifest["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    let validated = results
        .iter()
        .filter(|result| result["status"] == "validated")
        .count();
    let failed = results
        .iter()
        .filter(|result| result["status"] == "failed")
        .count();
    assert_eq!((validated, failed), (1, 1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("failed"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(not(any(windows, target_os = "macos")))]
#[test]
fn cli_convert_markdown_rejects_a_native_host_without_word() {
    let layout = conversion_layout(21);
    fs::write(layout.input.join("policy.md"), VALID_MARKDOWN).unwrap();

    let output = convert_markdown(&layout, &[]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("installed Office export is supported only on Windows and macOS"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // The host preflight fails before any output tree is owned.
    assert!(!layout.output.exists());
    assert!(output.stdout.is_empty());
    let edit = layout
        .config
        .parent()
        .expect("metadata dir")
        .parent()
        .expect("edit root");
    assert!(edit.join(TEMPLATE_RELATIVE).is_file());
}
