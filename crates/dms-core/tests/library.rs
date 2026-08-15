use std::{fs, path::Path};

use dms_core::{
    ControlUpdate, DmsError, LibraryEntryKind, LibraryMembership, PermalinkTarget, SourceState,
    Workspace,
};

fn initialized_workspace() -> (tempfile::TempDir, Workspace) {
    let temp = tempfile::tempdir().expect("temporary directory");
    let edit_root = temp.path().join("edit");
    let publish_root = temp.path().join("publish");
    fs::create_dir_all(&edit_root).expect("edit root");
    let workspace = Workspace::init(&edit_root, &publish_root).expect("workspace init");
    (temp, workspace)
}

#[test]
fn folder_listing_keeps_exact_files_and_hides_internal_or_office_temporary_entries() {
    let (_temp, workspace) = initialized_workspace();
    fs::create_dir_all(workspace.edit_root.join("Policies/Empty")).expect("empty folder");
    fs::write(
        workspace.edit_root.join("Policies/Handbook.md"),
        "# Handbook",
    )
    .expect("supported draft");
    fs::write(workspace.edit_root.join("Policies/diagram.png"), "image").expect("unsupported file");
    fs::write(workspace.edit_root.join("Policies/~$Handbook.docx"), "lock").expect("Office lock");

    let tree = workspace.library_tree().expect("library tree");
    assert_eq!(tree[0].relative_path, Path::new("."));
    assert!(tree
        .iter()
        .any(|folder| folder.relative_path == Path::new("Policies/Empty")));
    assert!(!tree
        .iter()
        .any(|folder| folder.relative_path.starts_with(".dms")));

    let folder = workspace
        .library_folder(Path::new("Policies"))
        .expect("folder listing");
    assert_eq!(folder.relative_path, Path::new("Policies"));
    assert_eq!(folder.parent.as_deref(), Some(Path::new(".")));
    assert_eq!(folder.entries[0].kind, LibraryEntryKind::Folder);
    assert_eq!(folder.entries[0].name, "Empty");
    assert_eq!(folder.entries[1].name, "diagram.png");
    assert_eq!(
        folder.entries[1].membership,
        Some(LibraryMembership::Unsupported)
    );
    assert_eq!(folder.entries[2].name, "Handbook.md");
    assert_eq!(
        folder.entries[2].membership,
        Some(LibraryMembership::NotInLibrary)
    );
    assert!(folder
        .entries
        .iter()
        .all(|entry| entry.name != "~$Handbook.docx"));
    let json = serde_json::to_value(&folder).expect("machine-readable folder");
    assert_eq!(json["relative_path"], "Policies");
    assert_eq!(json["parent"], ".");
    assert_eq!(json["entries"][0]["relative_path"], "Policies/Empty");
}

#[test]
fn batch_add_is_atomic_and_unregister_reassociate_preserve_document_identity() {
    let (_temp, mut workspace) = initialized_workspace();
    fs::create_dir_all(workspace.edit_root.join("Policies")).expect("folder");
    let original = workspace.edit_root.join("Policies/Handbook.md");
    let unsupported = workspace.edit_root.join("Policies/diagram.png");
    fs::write(&original, "# Handbook").expect("draft");
    fs::write(&unsupported, "image").expect("unsupported");

    assert!(matches!(
        workspace.add_documents(&[original.clone(), unsupported]),
        Err(DmsError::UnsupportedSource(_))
    ));
    assert!(workspace.documents().is_empty());

    let document = workspace
        .add_documents(std::slice::from_ref(&original))
        .expect("batch add")
        .remove(0);
    workspace
        .update_control(
            document.id,
            ControlUpdate {
                title: Some("Employee handbook".into()),
                document_number: Some(Some("HR-001".into())),
                ..ControlUpdate::default()
            },
        )
        .expect("control data");
    workspace
        .add_note(document.id, "Keep this history", Some("Raphael"))
        .expect("note");
    let permalink = workspace
        .document_permalink(document.id)
        .expect("permalink");

    workspace
        .unregister_document(document.id)
        .expect("unregister");
    assert!(original.is_file());
    let unregistered = workspace.document(document.id).expect("retained record");
    assert_eq!(unregistered.source_state, SourceState::Unregistered);
    assert_eq!(unregistered.control.title, "Employee handbook");
    assert_eq!(workspace.notes(document.id).expect("notes").len(), 1);

    let renamed = workspace.edit_root.join("Policies/Staff-Handbook.md");
    fs::rename(&original, &renamed).expect("external rename");
    let reassociated = workspace
        .reassociate_document(document.id, &renamed)
        .expect("reassociate");
    assert_eq!(reassociated.id, document.id);
    assert_eq!(
        reassociated.relative_path,
        Path::new("Policies/Staff-Handbook.md")
    );
    assert_eq!(reassociated.source_state, SourceState::Registered);
    assert_eq!(reassociated.control.title, "Employee handbook");
    assert_eq!(
        workspace
            .document_permalink(document.id)
            .expect("stable link"),
        permalink
    );
    assert_eq!(
        permalink,
        format!(
            "dms://open?workspace={}&document={}",
            workspace.workspace_id, document.id
        )
    );
    let document_target = workspace.resolve_permalink(&permalink).unwrap();
    assert_eq!(document_target.target, PermalinkTarget::Document);
    let notes_target = workspace
        .resolve_permalink(&format!("{permalink}&target=notes&ignored=value"))
        .unwrap();
    assert_eq!(notes_target.document_id, document.id);
    assert_eq!(notes_target.target, PermalinkTarget::Notes);
    assert_eq!(notes_target.review_id, None);
}

#[test]
fn search_matches_file_identity_path_and_control_data_with_explicit_scope() {
    let (_temp, mut workspace) = initialized_workspace();
    fs::create_dir_all(workspace.edit_root.join("Policies/HR")).expect("HR folder");
    fs::create_dir_all(workspace.edit_root.join("Policies/IT")).expect("IT folder");
    let hr = workspace.edit_root.join("Policies/HR/Handbook.md");
    let it = workspace.edit_root.join("Policies/IT/Access.docx");
    fs::write(&hr, "# Handbook").expect("HR draft");
    fs::write(&it, "office").expect("IT draft");
    let hr_document = workspace.add_document(&hr).expect("HR document");
    workspace
        .update_control(
            hr_document.id,
            ControlUpdate {
                title: Some("People guide".into()),
                document_number: Some(Some("HR-042".into())),
                ..ControlUpdate::default()
            },
        )
        .expect("control data");

    let current_scope = workspace
        .search_library(Path::new("Policies/HR"), "hr-042")
        .expect("current folder search");
    assert_eq!(current_scope.len(), 1);
    assert_eq!(current_scope[0].name, "Handbook.md");

    let entire_library = workspace
        .search_library(Path::new("."), "ACCESS.DOCX")
        .expect("entire library search");
    assert_eq!(entire_library.len(), 1);
    assert_eq!(
        entire_library[0].relative_path,
        Path::new("Policies/IT/Access.docx")
    );

    let excluded = workspace
        .search_library(Path::new("Policies/IT"), "people guide")
        .expect("scoped search");
    assert!(excluded.is_empty());
}

#[test]
fn schema_v2_migration_defaults_existing_documents_to_registered() {
    let (_temp, mut workspace) = initialized_workspace();
    let source = workspace.edit_root.join("Legacy.md");
    fs::write(&source, "# Legacy").expect("legacy source");
    let document = workspace.add_document(&source).expect("legacy document");
    workspace.save().expect("schema-v3 metadata");

    let metadata_path = workspace.edit_root.join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).expect("metadata"))
            .expect("metadata JSON");
    metadata["schema_version"] = serde_json::Value::from(2);
    metadata["documents"][document.id.to_string()]
        .as_object_mut()
        .expect("document object")
        .remove("source_state");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).expect("schema-v2 JSON"),
    )
    .expect("schema-v2 metadata");

    let migrated = Workspace::open(&workspace.edit_root).expect("migrated workspace");
    assert_eq!(
        migrated
            .document(document.id)
            .expect("document")
            .source_state,
        SourceState::Registered
    );
    assert!(workspace
        .edit_root
        .join(".dms/workspace.v2.json.bak")
        .is_file());
}
