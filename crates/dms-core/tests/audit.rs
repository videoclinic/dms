use std::fs;
use std::path::PathBuf;

use chrono::{Duration, Utc};
use dms_core::{
    AuditReportFilter, AuditReportFormat, AuditReportRequest, AuditReportVerificationStatus,
    DmsError, Workspace, SCHEMA_VERSION,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use uuid::Uuid;

fn fixture() -> (TempDir, Workspace, Uuid) {
    let temp = tempfile::tempdir().expect("temporary directory");
    let edit_root = temp.path().join("edit");
    let publish_root = temp.path().join("publish");
    fs::create_dir_all(edit_root.join("Policies")).expect("edit root");
    let mut workspace = Workspace::init(&edit_root, &publish_root).expect("workspace");
    workspace
        .configure_confidentiality_type("internal", "Internal", true)
        .expect("confidentiality type");
    workspace
        .set_confidentiality_policy(".", "internal")
        .expect("root policy");
    let source = edit_root.join("Policies/Handbook.md");
    fs::write(&source, "# Handbook\n\nTOP SECRET SOURCE BYTES\n").expect("source");
    let document = workspace.add_document(&source).expect("document");
    workspace.save().expect("save fixture");
    (temp, workspace, document.id)
}

#[test]
fn audit_reports_are_deterministic_filtered_and_never_embed_source_bytes() {
    let (_temp, mut workspace, document_id) = fixture();
    let filter = AuditReportFilter {
        document_ids: vec![document_id],
        confidentiality_type_ids: vec!["internal".to_owned()],
        ..AuditReportFilter::default()
    };

    let csv_before = workspace
        .preview_audit_report(AuditReportFormat::Csv, &filter)
        .expect("CSV preview");
    let pdf_before = workspace
        .preview_audit_report(AuditReportFormat::Pdf, &filter)
        .expect("PDF preview");
    assert!(String::from_utf8_lossy(&csv_before).contains("Handbook"));
    assert!(!String::from_utf8_lossy(&csv_before).contains("TOP SECRET SOURCE BYTES"));
    assert!(pdf_before.starts_with(b"%PDF-1.7"));
    assert!(!String::from_utf8_lossy(&pdf_before).contains("TOP SECRET SOURCE BYTES"));
    assert!(pdf_extract::extract_text_from_mem(&pdf_before)
        .expect("valid PDF")
        .contains("DMS Workspace Audit Report"));
    let future_filtered = workspace
        .preview_audit_report(
            AuditReportFormat::Csv,
            &AuditReportFilter {
                document_ids: vec![document_id],
                approver_object_ids: vec![Uuid::new_v4()],
                confidentiality_type_ids: vec!["internal".to_owned()],
                from: Some(Utc::now() + Duration::days(1)),
                through: Some(Utc::now() + Duration::days(2)),
            },
        )
        .expect("filtered classification summary");
    assert!(String::from_utf8_lossy(&future_filtered).contains("classification"));

    let csv_report = workspace
        .generate_audit_report(AuditReportRequest {
            format: AuditReportFormat::Csv,
            relative_path: Some(PathBuf::from(".dms/exports/handbook.csv")),
            filter: filter.clone(),
        })
        .expect("CSV report");
    let pdf_report = workspace
        .generate_audit_report(AuditReportRequest {
            format: AuditReportFormat::Pdf,
            relative_path: Some(PathBuf::from(".dms/exports/handbook.pdf")),
            filter: filter.clone(),
        })
        .expect("PDF report");

    assert_eq!(
        csv_before,
        workspace
            .preview_audit_report(AuditReportFormat::Csv, &filter)
            .expect("repeat CSV preview")
    );
    assert_eq!(
        pdf_before,
        workspace
            .preview_audit_report(AuditReportFormat::Pdf, &filter)
            .expect("repeat PDF preview")
    );
    assert_eq!(
        csv_report.sha256,
        format!("{:x}", Sha256::digest(&csv_before))
    );
    assert_eq!(
        pdf_report.sha256,
        format!("{:x}", Sha256::digest(&pdf_before))
    );
    assert_eq!(
        workspace
            .verify_report(csv_report.event_id)
            .expect("verify report")
            .status,
        AuditReportVerificationStatus::Match
    );
    assert_eq!(workspace.recent_reports().len(), 2);
    assert_eq!(workspace.recent_reports()[0].event_id, pdf_report.event_id);
    let verifications = workspace.verify_reports().expect("verify all reports");
    assert_eq!(verifications.len(), 2);
    assert!(verifications
        .iter()
        .all(|verification| verification.status == AuditReportVerificationStatus::Match));
    assert!(workspace.verify_report_chain().is_valid());

    fs::write(
        workspace.edit_root.join(&csv_report.relative_path),
        b"tampered",
    )
    .unwrap();
    assert_eq!(
        workspace
            .verify_report(csv_report.event_id)
            .expect("tampered report verdict")
            .status,
        AuditReportVerificationStatus::Mismatch
    );
    fs::remove_file(workspace.edit_root.join(&csv_report.relative_path)).unwrap();
    assert_eq!(
        workspace
            .verify_report(csv_report.event_id)
            .expect("missing report verdict")
            .status,
        AuditReportVerificationStatus::MissingFile
    );
    #[cfg(unix)]
    {
        let outside = _temp.path().join("outside.txt");
        fs::write(&outside, b"outside bytes").unwrap();
        std::os::unix::fs::symlink(
            &outside,
            workspace.edit_root.join(&csv_report.relative_path),
        )
        .unwrap();
        assert_eq!(
            workspace
                .verify_report(csv_report.event_id)
                .expect("symlink verdict")
                .status,
            AuditReportVerificationStatus::InvalidEvidence
        );
    }

    assert!(matches!(
        workspace.generate_audit_report(AuditReportRequest {
            format: AuditReportFormat::Pdf,
            relative_path: Some(PathBuf::from("../outside.pdf")),
            filter,
        }),
        Err(DmsError::InvalidReportPath(_))
    ));
    assert!(matches!(
        workspace.generate_audit_report(AuditReportRequest {
            format: AuditReportFormat::Pdf,
            relative_path: Some(PathBuf::from(".dms/exports/handbook.pdf")),
            filter: AuditReportFilter::default(),
        }),
        Err(DmsError::ReportPathExists(_))
    ));
}

#[test]
fn schema_v6_migrates_workspace_report_history_and_creates_backup() {
    let (_temp, workspace, _document_id) = fixture();
    let metadata_path = workspace.edit_root.join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
    metadata["schema_version"] = serde_json::Value::from(6);
    metadata.as_object_mut().unwrap().remove("workspace_events");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(&workspace.edit_root).expect("migration");
    assert_eq!(migrated.schema_version, SCHEMA_VERSION);
    assert!(migrated.recent_reports().is_empty());
    assert!(workspace
        .edit_root
        .join(".dms/workspace.v6.json.bak")
        .is_file());
}
