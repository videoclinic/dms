use std::{fs, io::Write, path::Path};

use dms_core::{
    SourceChangeKind, SourceHistoryFormat, SourceHistoryScanOutcome, SourceState, Workspace,
    SCHEMA_VERSION,
};
use tempfile::TempDir;
use zip::{write::SimpleFileOptions, ZipWriter};

fn initialized_workspace() -> (TempDir, TempDir, Workspace) {
    let edit_root = tempfile::tempdir().expect("edit root");
    let publish_root = tempfile::tempdir().expect("publish root");
    let workspace = Workspace::init(edit_root.path(), publish_root.path()).expect("workspace init");
    (edit_root, publish_root, workspace)
}

fn write_package(path: &Path, parts: &[(&str, &str)]) {
    let file = fs::File::create(path).expect("package file");
    let mut archive = ZipWriter::new(file);
    for (name, content) in parts {
        archive
            .start_file(*name, SimpleFileOptions::default())
            .expect("package entry");
        archive
            .write_all(content.as_bytes())
            .expect("package content");
    }
    archive.finish().expect("finish package");
}

fn word_package(revisions: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="urn:test"><w:body>{revisions}</w:body></w:document>"#
    )
}

#[test]
fn first_docx_import_groups_and_caps_attributable_revisions_without_content() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let source = edit_root.path().join("Handbook.docx");
    write_package(
        &source,
        &[(
            "word/document.xml",
            &word_package(
                r#"<w:ins w:author="Alex" w:date="2026-01-04T10:00:00Z">secret text</w:ins>
                   <w:del w:author="Alex" w:date="2026-01-04T10:00:00Z">other text</w:del>
                   <w:pPrChange w:author="Beth" w:date="2026-01-05T10:00:00Z" />
                   <w:rPrChange w:author="Casey" w:date="2026-01-03T10:00:00Z" />
                   <w:moveTo w:author="Dana" w:date="2026-01-02T10:00:00Z" />"#,
            ),
        )],
    );

    let document = workspace.add_document(&source).expect("first import");
    let history = document.source_history.expect("source history");
    assert_eq!(history.source_format, SourceHistoryFormat::Docx);
    assert_eq!(
        history.scan_outcome,
        SourceHistoryScanOutcome::AttributableRevisions
    );
    assert_eq!(history.observations.len(), 3);
    assert_eq!(history.observations[0].display_author, "Beth");
    assert_eq!(history.observations[1].display_author, "Alex");
    assert_eq!(history.observations[1].record_count, 2);
    assert_eq!(
        history.observations[1].kind,
        SourceChangeKind::WordMixedRevision
    );
    assert_eq!(history.observations[2].display_author, "Casey");
    assert!(!history
        .observations
        .iter()
        .any(|row| row.display_author == "Dana"));

    let serialized = serde_json::to_string(&history).expect("serialized history");
    assert!(!serialized.contains("secret text"));
    assert!(!serialized.contains("other text"));
    assert!(workspace.workflow_history(document.id).unwrap().is_empty());
    assert!(workspace.releases(document.id).unwrap().is_empty());
}

#[test]
fn xlsx_resolves_user_names_and_pptx_retains_only_unattributed_outcome() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let spreadsheet = edit_root.path().join("Budget.xlsx");
    write_package(
        &spreadsheet,
        &[
            (
                "xl/revisions/userNames.xml",
                r#"<users><user userName="Alice"/><user userName="Bob"/></users>"#,
            ),
            (
                "xl/revisions/revisionLog1.xml",
                r#"<revisions><revision userId="2" date="2026-01-06T10:00:00Z"/><revision userId="2" date="2026-01-06T10:00:00Z"/></revisions>"#,
            ),
        ],
    );
    let spreadsheet_document = workspace
        .add_document(&spreadsheet)
        .expect("spreadsheet import");
    let spreadsheet_history = spreadsheet_document
        .source_history
        .expect("spreadsheet history");
    assert_eq!(spreadsheet_history.source_format, SourceHistoryFormat::Xlsx);
    assert_eq!(spreadsheet_history.observations.len(), 1);
    assert_eq!(spreadsheet_history.observations[0].display_author, "Bob");
    assert_eq!(
        spreadsheet_history.observations[0].kind,
        SourceChangeKind::SpreadsheetRevision
    );
    assert_eq!(spreadsheet_history.observations[0].record_count, 2);

    let presentation = edit_root.path().join("Status.pptx");
    write_package(
        &presentation,
        &[(
            "ppt/revisionInfo/revisionInfo.xml",
            "<revisionInfo clientId=\"opaque-client\" />",
        )],
    );
    let presentation_document = workspace
        .add_document(&presentation)
        .expect("presentation import");
    let presentation_history = presentation_document
        .source_history
        .expect("presentation history");
    assert_eq!(
        presentation_history.source_format,
        SourceHistoryFormat::Pptx
    );
    assert_eq!(
        presentation_history.scan_outcome,
        SourceHistoryScanOutcome::UnattributedRevisionData
    );
    assert!(presentation_history.observations.is_empty());
    assert!(!serde_json::to_string(&presentation_history)
        .unwrap()
        .contains("opaque-client"));
}

#[test]
fn malformed_and_no_data_office_packages_remain_addable_with_sanitized_outcomes() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let malformed = edit_root.path().join("Broken.docx");
    fs::write(&malformed, "not a zip package").expect("malformed source");
    let malformed_document = workspace
        .add_document(&malformed)
        .expect("malformed import");
    let malformed_history = malformed_document
        .source_history
        .expect("malformed history");
    assert_eq!(
        malformed_history.scan_outcome,
        SourceHistoryScanOutcome::MalformedPackage
    );
    assert!(malformed_history.observations.is_empty());

    let empty = edit_root.path().join("Empty.docx");
    write_package(&empty, &[("[Content_Types].xml", "<Types/>")]);
    let empty_document = workspace.add_document(&empty).expect("empty import");
    let empty_history = empty_document.source_history.expect("empty history");
    assert_eq!(
        empty_history.scan_outcome,
        SourceHistoryScanOutcome::NoRevisionData
    );
    assert!(empty_history.observations.is_empty());
}

#[test]
fn v16_migration_adds_null_source_history_and_retains_backup() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let source = edit_root.path().join("Legacy.md");
    fs::write(&source, "# Legacy\n").expect("source");
    let document = workspace.add_document(&source).expect("add source");
    workspace.save().expect("save workspace");

    let metadata_path = edit_root.path().join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).expect("metadata"))
            .expect("metadata JSON");
    metadata["schema_version"] = serde_json::Value::from(16);
    metadata["documents"][document.id.to_string()]
        .as_object_mut()
        .expect("document metadata")
        .remove("source_history");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(edit_root.path()).expect("v16 migration");
    assert_eq!(migrated.schema_version, SCHEMA_VERSION);
    assert!(migrated
        .document(document.id)
        .unwrap()
        .source_history
        .is_none());
    assert!(edit_root
        .path()
        .join(".dms/workspace.v16.json.bak")
        .is_file());
    let current: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).expect("current metadata")).unwrap();
    assert!(current["documents"][document.id.to_string()]
        .get("source_history")
        .is_some_and(serde_json::Value::is_null));
}

#[test]
fn re_registration_preserves_the_original_capture_after_source_bytes_change() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let source = edit_root.path().join("Procedure.docx");
    write_package(
        &source,
        &[(
            "word/document.xml",
            &word_package(r#"<w:ins w:author="Original" w:date="2026-01-02T10:00:00Z"/>"#),
        )],
    );
    let first = workspace.add_document(&source).expect("first import");
    let first_history = first.source_history.clone().expect("first history");
    workspace.unregister_document(first.id).expect("unregister");

    write_package(
        &source,
        &[(
            "word/document.xml",
            &word_package(r#"<w:ins w:author="Replacement" w:date="2026-01-07T10:00:00Z"/>"#),
        )],
    );
    let restored = workspace.add_document(&source).expect("re-register");
    assert_eq!(restored.id, first.id);
    assert_eq!(restored.source_state, SourceState::Registered);
    assert_eq!(restored.source_history.as_ref(), Some(&first_history));
    assert!(workspace.workflow_history(first.id).unwrap().is_empty());
    assert!(workspace.releases(first.id).unwrap().is_empty());
}

#[test]
fn batch_add_captures_every_office_source_before_mutating_the_workspace() {
    let (edit_root, _publish_root, mut workspace) = initialized_workspace();
    let document = edit_root.path().join("First.docx");
    write_package(
        &document,
        &[(
            "word/document.xml",
            &word_package(r#"<w:ins w:author="Alex" w:date="2026-01-01T10:00:00Z"/>"#),
        )],
    );
    let unsupported = edit_root.path().join("not-a-draft.pdf");
    fs::write(&unsupported, "pdf").expect("unsupported source");

    assert!(workspace.add_documents(&[document, unsupported]).is_err());
    assert!(workspace.documents().is_empty());
}
