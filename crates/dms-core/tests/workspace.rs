use std::{fs, path::PathBuf, thread, time::Duration};

use dms_core::{
    AuthenticatedActor, ControlUpdate, DmsError, EntraPerson, MutationPrincipal, RoleUpdate,
    WorkflowEventType, Workspace, METADATA_DIRECTORY, METADATA_FILENAME, SCHEMA_VERSION,
};
use tempfile::TempDir;
use uuid::Uuid;

fn initialized_workspace() -> (TempDir, TempDir, Workspace) {
    let edit_root = tempfile::tempdir().expect("edit root");
    let publish_root = tempfile::tempdir().expect("publish root");
    let workspace = Workspace::init(edit_root.path(), publish_root.path()).expect("workspace init");
    (edit_root, publish_root, workspace)
}

fn local_principal() -> MutationPrincipal {
    MutationPrincipal::local_os_user("test-operator")
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
    workspace
        .configure_confidentiality_type("internal", "Internal", true)
        .expect("type");
    workspace
        .set_confidentiality_policy(".", "internal")
        .expect("policy");
    let document = add_markdown_document(&mut workspace, &edit_root, "procedures/Onboarding.md");
    let relative_path = PathBuf::from("procedures").join("Onboarding.md");
    assert_eq!(document.relative_path, relative_path);
    assert_eq!(document.control.title, "Onboarding");
    let source_after_add = fs::read_to_string(edit_root.path().join(&relative_path)).unwrap();
    assert!(source_after_add.contains("title: Onboarding"));
    assert!(source_after_add.contains("version: 1.0"));
    assert!(source_after_add.contains("confidentiality: internal"));
    assert!(source_after_add.contains("# Draft"));
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
            &local_principal(),
        )
        .expect("update control data");
    workspace.save().expect("save workspace");

    let source_after_update = fs::read_to_string(edit_root.path().join(&relative_path)).unwrap();
    assert!(source_after_update.contains("title: New hire onboarding"));
    assert!(source_after_update.contains("document_number: PR-001"));
    assert!(source_after_update.contains("# Draft"));

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
fn group_bound_control_events_require_matching_entra_principal() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let document = add_markdown_document(&mut workspace, &edit_root, "bound.md");
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    workspace
        .replace_identity_source(
            tenant_id,
            Uuid::new_v4(),
            "DMS group",
            vec![EntraPerson::eligible(
                actor_id,
                "Ada Actor",
                "ada@example.test",
            )],
        )
        .expect("identity source");
    workspace
        .update_workflow_policy(
            ".",
            RoleUpdate::replace(actor_id),
            RoleUpdate::replace(actor_id),
        )
        .expect("root workflow policy");

    assert!(matches!(
        workspace.update_control(
            document.id,
            ControlUpdate {
                title: Some("Blocked local mutation".to_owned()),
                ..ControlUpdate::default()
            },
            &local_principal(),
        ),
        Err(DmsError::EntraSessionRequired)
    ));
    assert_eq!(
        workspace.document(document.id).unwrap().control.title,
        "bound"
    );

    assert!(matches!(
        workspace.update_control(
            document.id,
            ControlUpdate {
                title: Some("Blocked tenant mutation".to_owned()),
                ..ControlUpdate::default()
            },
            &MutationPrincipal::authenticated_entra(AuthenticatedActor {
                tenant_id: Uuid::new_v4(),
                object_id: actor_id,
            }),
        ),
        Err(DmsError::EntraTenantMismatch { .. })
    ));
    assert_eq!(
        workspace.document(document.id).unwrap().control.title,
        "bound"
    );

    workspace
        .update_control(
            document.id,
            ControlUpdate {
                title: Some("Verified Entra mutation".to_owned()),
                ..ControlUpdate::default()
            },
            &MutationPrincipal::authenticated_entra(AuthenticatedActor {
                tenant_id,
                object_id: actor_id,
            }),
        )
        .expect("matching principal");
    let event = workspace
        .workflow_history(document.id)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(
        event.body.authenticated_actor.as_ref().unwrap().object_id,
        actor_id
    );
    assert!(event.body.local_os_user.is_none());
    workspace.save().expect("persist group-bound event");
    assert!(Workspace::open(edit_root.path())
        .expect("reopen group-bound workspace")
        .verify_workflow(document.id)
        .expect("verify event chain")
        .is_valid());
}

#[test]
fn schema_v17_group_binding_migrates_without_inferring_a_tenant() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    workspace
        .replace_identity_source(
            tenant_id,
            Uuid::new_v4(),
            "DMS group",
            vec![EntraPerson::eligible(
                actor_id,
                "Ada Actor",
                "ada@example.test",
            )],
        )
        .expect("identity source");
    workspace
        .update_workflow_policy(
            ".",
            RoleUpdate::replace(actor_id),
            RoleUpdate::replace(actor_id),
        )
        .expect("root workflow policy");
    workspace.save().expect("current workspace");

    let metadata_path = edit_root.path().join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).expect("metadata")).expect("JSON");
    metadata["schema_version"] = serde_json::Value::from(17);
    metadata["identity_source"]
        .as_object_mut()
        .unwrap()
        .remove("tenant_id");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(edit_root.path()).expect("v17 migration");
    assert_eq!(migrated.schema_version, SCHEMA_VERSION);
    assert!(migrated.identity_source().unwrap().tenant_id.is_none());
    assert!(edit_root
        .path()
        .join(".dms/workspace.v17.json.bak")
        .is_file());
    assert!(matches!(
        migrated.require_mutation_principal(&MutationPrincipal::authenticated_entra(
            AuthenticatedActor {
                tenant_id,
                object_id: actor_id,
            }
        )),
        Err(DmsError::UnverifiedEntraIdentitySource)
    ));
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
fn schema_v15_migration_removes_workspace_notification_settings_and_retains_backup() {
    let (edit_root, _publish_root, workspace) = initialized_workspace();
    workspace.save().expect("current metadata");

    let metadata_path = edit_root.path().join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).expect("metadata"))
            .expect("metadata JSON");
    metadata["schema_version"] = serde_json::Value::from(15);
    metadata["notification_settings"] = serde_json::json!({
        "transport": "smtp",
        "smtp": {
            "relay_host": "smtp.example.test",
            "relay_port": 587,
            "login_user": "legacy@example.test",
            "from_mailbox": "legacy@example.test"
        }
    });
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(edit_root.path()).expect("schema v15 migration");
    assert_eq!(migrated.schema_version, SCHEMA_VERSION);
    assert!(migrated.notification_settings().is_none());
    let current: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).expect("migrated metadata"))
            .expect("migrated metadata JSON");
    assert!(current.get("notification_settings").is_none());
    let backup: serde_json::Value = serde_json::from_slice(
        &fs::read(edit_root.path().join(".dms/workspace.v15.json.bak")).expect("migration backup"),
    )
    .expect("backup JSON");
    assert_eq!(backup["schema_version"], 15);
    assert_eq!(
        backup["notification_settings"]["smtp"]["login_user"],
        "legacy@example.test"
    );
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
            &local_principal(),
        )
        .expect("first document number");
    assert!(matches!(
        workspace.update_control(
            second.id,
            ControlUpdate {
                document_number: Some(Some("pol-1".to_owned())),
                ..ControlUpdate::default()
            },
            &local_principal(),
        ),
        Err(DmsError::DuplicateDocumentNumber(_))
    ));
}

#[test]
fn notes_are_newest_first_editable_removable_and_persistent() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let document = add_markdown_document(&mut workspace, &edit_root, "notes.md");
    let first = workspace
        .add_note(
            document.id,
            "First note",
            Some("Raphael"),
            &local_principal(),
        )
        .expect("first note");
    thread::sleep(Duration::from_millis(2));
    let second = workspace
        .add_note(
            document.id,
            "Second note",
            Some("Raphael"),
            &local_principal(),
        )
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
