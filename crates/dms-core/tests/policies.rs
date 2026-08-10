use std::{fs, path::Path};

use dms_core::{
    ControlUpdate, DmsError, EntraPerson, ResolutionState, RoleUpdate, Workspace, SCHEMA_VERSION,
};
use serde_json::Value;
use tempfile::TempDir;
use uuid::Uuid;

fn initialized_workspace() -> (TempDir, Workspace) {
    let temp = TempDir::new().expect("temporary directory");
    let edit_root = temp.path().join("edit");
    let publish_root = temp.path().join("publish");
    fs::create_dir_all(&edit_root).expect("edit root");
    let workspace = Workspace::init(&edit_root, &publish_root).expect("workspace init");
    (temp, workspace)
}

fn configure_confidentiality(workspace: &mut Workspace) {
    workspace
        .configure_confidentiality_type("internal", "Internal", true)
        .expect("internal type");
    workspace
        .configure_confidentiality_type("restricted", "Restricted", true)
        .expect("restricted type");
    workspace
        .set_confidentiality_policy(".", "internal")
        .expect("root policy");
}

#[test]
fn schema_v1_metadata_migrates_to_the_current_store_shape() {
    let temp = TempDir::new().expect("temporary directory");
    let edit_root = temp.path().join("edit");
    let publish_root = temp.path().join("publish");
    fs::create_dir_all(edit_root.join(".dms")).expect("metadata directory");
    fs::create_dir_all(&publish_root).expect("publish root");

    let fixture = include_str!("fixtures/workspace-v1.json");
    let mut value: Value = serde_json::from_str(fixture).expect("fixture JSON");
    value["edit_root"] = Value::String(edit_root.to_string_lossy().into_owned());
    value["publish_root"] = Value::String(publish_root.to_string_lossy().into_owned());
    fs::write(
        edit_root.join(".dms/workspace.json"),
        serde_json::to_vec_pretty(&value).expect("fixture serialization"),
    )
    .expect("workspace fixture");

    let workspace = Workspace::open(&edit_root).expect("migrated workspace");
    assert_eq!(workspace.schema_version, SCHEMA_VERSION);
    assert_eq!(
        workspace.workspace_id.to_string(),
        "4fc6f944-813c-4fed-8c14-72cb956fa683"
    );
    assert!(workspace.confidentiality_types().is_empty());
    assert_eq!(workspace.document_types()[0].id, "procedure");
    assert_eq!(workspace.documents().len(), 1);
    assert!(workspace.identity_source().is_none());

    let persisted: Value =
        serde_json::from_slice(&fs::read(edit_root.join(".dms/workspace.json")).expect("metadata"))
            .expect("persisted JSON");
    assert_eq!(persisted["schema_version"], SCHEMA_VERSION);
    assert!(persisted["workflow_policies"].is_object());
    assert!(persisted["identity_cache"].is_object());
    let backup: Value = serde_json::from_slice(
        &fs::read(edit_root.join(".dms/workspace.v1.json.bak")).expect("migration backup"),
    )
    .expect("backup JSON");
    assert_eq!(backup["schema_version"], 1);
}

#[test]
fn migration_and_newer_schema_fail_closed_without_rewriting_metadata() {
    let temp = TempDir::new().expect("temporary directory");
    let edit_root = temp.path().join("edit");
    let publish_root = temp.path().join("publish");
    fs::create_dir_all(edit_root.join(".dms")).expect("metadata directory");
    fs::create_dir_all(&publish_root).expect("publish root");
    let metadata_path = edit_root.join(".dms/workspace.json");

    let mut value: Value =
        serde_json::from_str(include_str!("fixtures/workspace-v1.json")).expect("fixture JSON");
    value["edit_root"] = Value::String(edit_root.to_string_lossy().into_owned());
    value["publish_root"] = Value::String(publish_root.to_string_lossy().into_owned());
    value["unknown_phase_1_field"] = Value::Bool(true);
    let unknown_bytes = serde_json::to_vec_pretty(&value).expect("unknown-field fixture");
    fs::write(&metadata_path, &unknown_bytes).expect("workspace fixture");

    assert!(matches!(
        Workspace::open(&edit_root),
        Err(DmsError::InvalidMetadata { .. })
    ));
    assert_eq!(
        fs::read(&metadata_path).expect("unchanged metadata"),
        unknown_bytes
    );

    value
        .as_object_mut()
        .expect("workspace object")
        .remove("unknown_phase_1_field");
    value["schema_version"] = Value::from(999);
    let newer_bytes = serde_json::to_vec_pretty(&value).expect("newer fixture");
    fs::write(&metadata_path, &newer_bytes).expect("newer workspace fixture");
    assert!(matches!(
        Workspace::open(&edit_root),
        Err(DmsError::UnsupportedSchema {
            expected: SCHEMA_VERSION,
            found: 999
        })
    ));
    assert_eq!(
        fs::read(&metadata_path).expect("unchanged metadata"),
        newer_bytes
    );
}

#[test]
fn document_type_catalogue_is_persisted_and_controls_document_values() {
    let (_temp, mut workspace) = initialized_workspace();
    workspace
        .configure_document_type("procedure", "Procedure", true)
        .expect("document type");
    fs::write(workspace.edit_root.join("Onboarding.md"), "# Onboarding").expect("source");
    let document = workspace
        .add_document(Path::new("Onboarding.md"))
        .expect("document");
    workspace
        .update_control(
            document.id,
            ControlUpdate {
                document_type: Some(Some("procedure".to_owned())),
                ..ControlUpdate::default()
            },
        )
        .expect("document control");
    workspace.save().expect("persist document type");

    let reopened = Workspace::open(&workspace.edit_root).expect("reopen workspace");
    assert_eq!(reopened.document_types()[0].id, "procedure");
    assert_eq!(
        reopened
            .document(document.id)
            .expect("document")
            .control
            .document_type
            .as_deref(),
        Some("procedure")
    );
    assert!(matches!(
        reopened.clone().update_control(
            document.id,
            ControlUpdate {
                document_type: Some(Some("unknown".to_owned())),
                ..ControlUpdate::default()
            }
        ),
        Err(DmsError::UnknownDocumentType(value)) if value == "unknown"
    ));
    assert!(matches!(
        reopened
            .clone()
            .configure_document_type("procedure", "Procedure", false),
        Err(DmsError::DocumentTypeInUse(value)) if value == "procedure"
    ));
}

#[test]
fn policy_folder_tree_contains_root_and_empty_folders_but_not_metadata() {
    let (_temp, workspace) = initialized_workspace();
    fs::create_dir_all(workspace.edit_root.join("policies/HR/empty")).expect("empty folder");
    fs::create_dir_all(workspace.edit_root.join("procedures")).expect("procedures");
    fs::create_dir_all(workspace.edit_root.join(".dms/private")).expect("metadata child");

    let folders = workspace.policy_folders().expect("policy folders");
    let paths = folders
        .iter()
        .map(|folder| folder.relative_path.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        paths,
        vec![
            ".",
            "policies",
            "policies/HR",
            "policies/HR/empty",
            "procedures"
        ]
    );
}

#[test]
fn confidentiality_policies_inherit_and_document_overrides_survive_policy_changes() {
    let (_temp, mut workspace) = initialized_workspace();
    configure_confidentiality(&mut workspace);
    fs::create_dir_all(workspace.edit_root.join("policies/HR")).expect("policy folders");
    fs::write(
        workspace.edit_root.join("policies/HR/Handbook.md"),
        "# Handbook",
    )
    .expect("source");
    let document = workspace
        .add_document(Path::new("policies/HR/Handbook.md"))
        .expect("document");

    workspace
        .set_confidentiality_policy("policies", "restricted")
        .expect("folder policy");
    let inherited = workspace
        .effective_confidentiality(document.id)
        .expect("effective confidentiality");
    assert_eq!(inherited.type_id, "restricted");
    assert_eq!(inherited.source_folder, "policies");
    assert!(!inherited.document_override);

    workspace
        .set_document_confidentiality(document.id, Some("internal"))
        .expect("document override");
    workspace
        .set_confidentiality_policy("policies", "internal")
        .expect("replace folder policy");
    workspace
        .remove_confidentiality_policy("policies")
        .expect("remove folder policy");
    assert!(matches!(
        workspace.configure_confidentiality_type("internal", "Internal", false),
        Err(DmsError::ConfidentialityTypeInUse(value)) if value == "internal"
    ));
    let overridden = workspace
        .effective_confidentiality(document.id)
        .expect("overridden confidentiality");
    assert_eq!(overridden.type_id, "internal");
    assert!(overridden.document_override);

    assert!(matches!(
        workspace.remove_confidentiality_policy("."),
        Err(DmsError::RequiredRootPolicy)
    ));
    workspace.save().expect("persist policies");
    let reopened = Workspace::open(&workspace.edit_root).expect("reopen workspace");
    assert!(
        reopened
            .effective_confidentiality(document.id)
            .expect("reopened confidentiality")
            .document_override
    );
}

#[test]
fn workflow_roles_inherit_independently_and_binding_replacement_unresolves_live_roles() {
    let (_temp, mut workspace) = initialized_workspace();
    configure_confidentiality(&mut workspace);
    fs::create_dir_all(workspace.edit_root.join("policies/IT")).expect("policy folders");
    fs::write(
        workspace.edit_root.join("policies/IT/Access.md"),
        "# Access",
    )
    .expect("source");
    let document = workspace
        .add_document(Path::new("policies/IT/Access.md"))
        .expect("document");

    let tenant_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let editor_id = Uuid::new_v4();
    let approver_id = Uuid::new_v4();
    let replacement_approver_id = Uuid::new_v4();
    let people = vec![
        EntraPerson::eligible(editor_id, "Lukas Roth", "lukas@example.test"),
        EntraPerson::eligible(approver_id, "Anna Berg", "anna@example.test"),
        EntraPerson::eligible(replacement_approver_id, "Mara Klein", "mara@example.test"),
    ];
    workspace
        .replace_identity_source(
            tenant_id,
            "Example tenant",
            group_id,
            "DMS workflow",
            people.clone(),
        )
        .expect("identity source");
    workspace
        .update_workflow_policy(
            ".",
            RoleUpdate::replace(editor_id),
            RoleUpdate::replace(approver_id),
        )
        .expect("root routing");
    assert!(matches!(
        workspace.update_workflow_policy(".", RoleUpdate::Clear, RoleUpdate::Unchanged),
        Err(DmsError::RequiredRootWorkflowPolicy)
    ));
    workspace
        .update_workflow_policy(
            "policies/IT",
            RoleUpdate::replace(editor_id),
            RoleUpdate::Unchanged,
        )
        .expect("folder editor");
    workspace
        .set_document_workflow_roles(
            document.id,
            RoleUpdate::Unchanged,
            RoleUpdate::replace(replacement_approver_id),
        )
        .expect("document approver override");

    let routing = workspace
        .effective_workflow_roles(document.id)
        .expect("effective routing");
    assert_eq!(routing.editor.expect("editor").source_folder, "policies/IT");
    let approver = routing.approver.expect("approver");
    assert!(approver.document_override);
    assert_eq!(approver.object_id, replacement_approver_id);
    assert_eq!(approver.state, ResolutionState::Resolved);

    workspace
        .replace_identity_source(
            tenant_id,
            "Example tenant",
            Uuid::new_v4(),
            "Replacement workflow",
            people,
        )
        .expect("replacement source");
    let unresolved = workspace
        .effective_workflow_roles(document.id)
        .expect("unresolved routing");
    assert_eq!(
        unresolved.editor.expect("editor remains assigned").state,
        ResolutionState::Unresolved
    );
    assert_eq!(
        unresolved
            .approver
            .expect("approver remains assigned")
            .state,
        ResolutionState::Unresolved
    );

    workspace.save().expect("persist routing");
    let metadata =
        fs::read_to_string(workspace.edit_root.join(".dms/workspace.json")).expect("metadata");
    assert!(metadata.contains(&tenant_id.to_string()));
    assert!(!metadata.contains("client_secret"));
    assert!(!metadata.contains("access_token"));
}
