use std::{fs, path::PathBuf, thread, time::Duration};

use dms_core::{
    ControlUpdate, DmsError, WorkflowEventType, Workspace, METADATA_DIRECTORY, METADATA_FILENAME,
};
use tempfile::TempDir;

fn initialized_workspace() -> (TempDir, TempDir, Workspace) {
    let edit_root = tempfile::tempdir().expect("edit root");
    let publish_root = tempfile::tempdir().expect("publish root");
    let workspace = Workspace::init(edit_root.path(), publish_root.path()).expect("workspace init");
    (edit_root, publish_root, workspace)
}

fn add_markdown_document(
    workspace: &mut Workspace,
    edit_root: &TempDir,
    name: &str,
) -> dms_core::Document {
    let source_path = edit_root.path().join(name);
    if let Some(parent) = source_path.parent() {
        fs::create_dir_all(parent).expect("source parent");
    }
    fs::write(&source_path, "# Draft\n").expect("source draft");
    workspace.add_document(&source_path).expect("add document")
}

#[test]
fn workspace_init_persists_canonical_roots_and_stable_id() {
    let (edit_root, publish_root, workspace) = initialized_workspace();
    let metadata_path = edit_root
        .path()
        .join(METADATA_DIRECTORY)
        .join(METADATA_FILENAME);
    assert!(metadata_path.is_file());

    let reopened = Workspace::open(edit_root.path()).expect("reopen workspace");
    assert_eq!(reopened.workspace_id, workspace.workspace_id);
    assert_eq!(
        reopened.edit_root,
        fs::canonicalize(edit_root.path()).expect("canonical edit root")
    );
    assert_eq!(
        reopened.publish_root,
        fs::canonicalize(publish_root.path()).expect("canonical publish root")
    );
}

#[test]
fn document_control_is_persisted_independently_from_source_locator() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let document = add_markdown_document(&mut workspace, &edit_root, "procedures/Onboarding.md");
    let relative_path = PathBuf::from("procedures").join("Onboarding.md");
    assert_eq!(document.relative_path, relative_path);
    assert_eq!(document.control.title, "Onboarding");
    workspace
        .configure_document_type("procedure", "Procedure", true)
        .expect("document type");

    let updated = workspace
        .update_control(
            document.id,
            ControlUpdate {
                title: Some("New hire onboarding".to_owned()),
                document_number: Some(Some("PR-001".to_owned())),
                document_type: Some(Some("procedure".to_owned())),
                ..ControlUpdate::default()
            },
        )
        .expect("update control data");
    workspace.save().expect("save workspace");

    let metadata = fs::read_to_string(
        edit_root
            .path()
            .join(METADATA_DIRECTORY)
            .join(METADATA_FILENAME),
    )
    .expect("read workspace metadata");
    assert!(metadata.contains(r#""relative_path": "procedures/Onboarding.md""#));

    let reopened = Workspace::open(edit_root.path()).expect("reopen workspace");
    let stored = reopened.document(document.id).expect("stored document");
    assert_eq!(stored.relative_path, relative_path);
    assert_eq!(stored.control.title, "New hire onboarding");
    assert_eq!(stored.control.document_number.as_deref(), Some("PR-001"));
    assert!(stored.control.owner.is_none());
    assert_eq!(updated.id, stored.id);
    let history = workspace
        .workflow_history(document.id)
        .expect("document-control evidence");
    assert_eq!(history.len(), 1);
    assert_eq!(
        history[0].body.event_type,
        WorkflowEventType::DocumentControlDataChanged
    );
    let change = history[0]
        .body
        .control_change
        .as_ref()
        .expect("before/after control data");
    assert_eq!(change.before.title, "Onboarding");
    assert_eq!(change.after, stored.control);
    assert!(workspace.verify_workflow(document.id).unwrap().is_valid());
}

#[test]
fn schema_v11_migration_preserves_legacy_owner_without_assigning_authority() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let document = add_markdown_document(&mut workspace, &edit_root, "Legacy.md");
    workspace.save().expect("current metadata");

    let metadata_path = edit_root
        .path()
        .join(METADATA_DIRECTORY)
        .join(METADATA_FILENAME);
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).expect("metadata"))
            .expect("metadata JSON");
    metadata["schema_version"] = serde_json::Value::from(11);
    metadata["documents"][document.id.to_string()]["control"]["owner"] =
        serde_json::Value::String("Legacy Quality Team".to_owned());
    metadata["documents"][document.id.to_string()]["control"]["effective_date"] =
        serde_json::Value::String("2026-08-11".to_owned());
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(edit_root.path()).expect("schema v11 migration");
    let control = &migrated.document(document.id).unwrap().control;
    assert!(control.owner.is_none());
    assert_eq!(
        control.legacy_owner_label.as_deref(),
        Some("Legacy Quality Team")
    );
    assert!(edit_root
        .path()
        .join(".dms/workspace.v11.json.bak")
        .is_file());
}

#[test]
fn document_registration_rejects_out_of_root_temp_unsupported_and_duplicate_sources() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let document = add_markdown_document(&mut workspace, &edit_root, "policy.md");
    let source_path = edit_root.path().join("policy.md");
    assert!(matches!(
        workspace.add_document(&source_path),
        Err(DmsError::DocumentAlreadyRegistered(_))
    ));

    let temporary_path = edit_root.path().join("~$policy.docx");
    fs::write(&temporary_path, "temporary").expect("temporary draft");
    assert!(matches!(
        workspace.add_document(&temporary_path),
        Err(DmsError::OfficeTemporaryFile(_))
    ));

    let unsupported_path = edit_root.path().join("policy.pdf");
    fs::write(&unsupported_path, "pdf").expect("unsupported file");
    assert!(matches!(
        workspace.add_document(&unsupported_path),
        Err(DmsError::UnsupportedSource(_))
    ));

    let outside_root = tempfile::tempdir().expect("outside root");
    let outside_path = outside_root.path().join("other.md");
    fs::write(&outside_path, "# other").expect("outside file");
    assert!(matches!(
        workspace.add_document(&outside_path),
        Err(DmsError::OutsideEditRoot(_))
    ));

    let second = add_markdown_document(&mut workspace, &edit_root, "second.md");
    workspace
        .update_control(
            document.id,
            ControlUpdate {
                document_number: Some(Some("POL-1".to_owned())),
                ..ControlUpdate::default()
            },
        )
        .expect("first document number");
    assert!(matches!(
        workspace.update_control(
            second.id,
            ControlUpdate {
                document_number: Some(Some("pol-1".to_owned())),
                ..ControlUpdate::default()
            },
        ),
        Err(DmsError::DuplicateDocumentNumber(_))
    ));
}

#[test]
fn notes_are_newest_first_editable_removable_and_persistent() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let document = add_markdown_document(&mut workspace, &edit_root, "notes.md");
    let first = workspace
        .add_note(document.id, "First note", Some("Raphael"))
        .expect("first note");
    thread::sleep(Duration::from_millis(2));
    let second = workspace
        .add_note(document.id, "Second note", Some("Raphael"))
        .expect("second note");
    let newest_first = workspace.notes(document.id).expect("list notes");
    assert_eq!(
        newest_first.iter().map(|note| note.id).collect::<Vec<_>>(),
        vec![second.id, first.id]
    );

    let edited = workspace
        .edit_note(document.id, first.id, "Edited first note")
        .expect("edit note");
    assert_eq!(edited.body, "Edited first note");
    workspace
        .remove_note(document.id, second.id)
        .expect("remove note");
    workspace.save().expect("save workspace");

    let reopened = Workspace::open(edit_root.path()).expect("reopen workspace");
    let notes = reopened.notes(document.id).expect("reopened notes");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, first.id);
    assert_eq!(notes[0].body, "Edited first note");
}
