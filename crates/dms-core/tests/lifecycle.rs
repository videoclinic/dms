use std::{
    collections::VecDeque,
    fs,
    io::{Read, Write},
    path::Path,
};

use dms_core::{
    AssistanceEvidence, AuthenticatedActor, CandidateRequest, CandidateStatus, ControlUpdate,
    DeliveryReceipt, DeliveryStatus, DmsError, EntraIdentitySource, EntraPerson, GraphClient,
    Lifecycle, MarkerStatus, NotificationClient, NotificationMessage, NotificationSettings,
    NotificationTransport, PdfExporter, PeriodicReviewResult, PeriodicReviewStatus, ReleaseOutcome,
    ReleaseVerificationStatus, ReviewDecision, RoleUpdate, SmtpSettings, TargetSelection, Version,
    WorkflowEventType, WorkflowVerification, Workspace, SCHEMA_VERSION,
};
use tempfile::TempDir;
use uuid::Uuid;
use zip::{write::SimpleFileOptions, ZipWriter};

struct Fixture {
    _temp: TempDir,
    workspace: Workspace,
    document_id: Uuid,
    source_path: std::path::PathBuf,
    tenant_id: Uuid,
    editor_id: Uuid,
    approver_id: Uuid,
    requester_id: Uuid,
    people: Vec<EntraPerson>,
}

impl Fixture {
    fn new(markdown: &str, transport: NotificationTransport) -> Self {
        let temp = tempfile::tempdir().expect("temporary directory");
        let edit_root = temp.path().join("edit");
        let publish_root = temp.path().join("publish");
        fs::create_dir_all(edit_root.join("Policies")).expect("edit root");
        let mut workspace = Workspace::init(&edit_root, &publish_root).expect("workspace init");
        workspace
            .configure_document_type("procedure", "Procedure", true)
            .expect("document type");
        workspace
            .configure_confidentiality_type("internal", "Internal", true)
            .expect("confidentiality type");
        workspace
            .set_confidentiality_policy(".", "internal")
            .expect("root confidentiality");

        let tenant_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();
        let editor_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();
        let people = vec![
            EntraPerson::eligible(editor_id, "Eva Editor", "editor@example.test"),
            EntraPerson::eligible(approver_id, "Ada Approver", "approver@example.test"),
            EntraPerson::eligible(requester_id, "Rita Requester", "requester@example.test"),
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
            .expect("workflow policy");
        workspace
            .configure_notifications(
                transport,
                (transport == NotificationTransport::Smtp).then(|| SmtpSettings {
                    relay_host: "smtp.example.test".to_owned(),
                    relay_port: 587,
                    sender: "dms@example.test".to_owned(),
                }),
            )
            .expect("notification settings");

        let source_path = edit_root.join("Policies/Handbook.md");
        fs::write(&source_path, markdown).expect("source");
        let document = workspace.add_document(&source_path).expect("document");
        workspace
            .update_control(
                document.id,
                ControlUpdate {
                    title: Some("Employee handbook".to_owned()),
                    document_type: Some(Some("procedure".to_owned())),
                    owner: Some(Some("People team".to_owned())),
                    ..ControlUpdate::default()
                },
            )
            .expect("control data");

        Self {
            _temp: temp,
            workspace,
            document_id: document.id,
            source_path,
            tenant_id,
            editor_id,
            approver_id,
            requester_id,
            people,
        }
    }

    fn graph(&self) -> FakeGraph {
        FakeGraph {
            people: self.people.clone(),
            actor: AuthenticatedActor {
                tenant_id: self.tenant_id,
                object_id: self.approver_id,
            },
            refresh_error: None,
        }
    }

    fn candidate_request(&self, selection: TargetSelection) -> CandidateRequest {
        CandidateRequest {
            document_id: self.document_id,
            selection,
            changelog: "Clarify onboarding responsibilities".to_owned(),
            requester_object_id: self.requester_id,
            review_override_reason: None,
            assistance: None,
        }
    }
}

struct FakeGraph {
    people: Vec<EntraPerson>,
    actor: AuthenticatedActor,
    refresh_error: Option<String>,
}

impl GraphClient for FakeGraph {
    fn direct_user_members(
        &mut self,
        _source: &EntraIdentitySource,
    ) -> std::result::Result<Vec<EntraPerson>, String> {
        if let Some(error) = self.refresh_error.clone() {
            Err(error)
        } else {
            Ok(self.people.clone())
        }
    }

    fn authenticated_actor(
        &mut self,
        _source: &EntraIdentitySource,
    ) -> std::result::Result<AuthenticatedActor, String> {
        Ok(self.actor.clone())
    }
}

#[derive(Default)]
struct FakeNotifier {
    receipts: VecDeque<std::result::Result<DeliveryReceipt, String>>,
    messages: Vec<NotificationMessage>,
}

impl FakeNotifier {
    fn accepted() -> Self {
        Self {
            receipts: [Ok(DeliveryReceipt::accepted(250, "accepted"))]
                .into_iter()
                .collect(),
            messages: Vec::new(),
        }
    }

    fn confirmed() -> Self {
        Self {
            receipts: [Ok(DeliveryReceipt::confirmed("operator confirmed send"))]
                .into_iter()
                .collect(),
            messages: Vec::new(),
        }
    }
}

impl NotificationClient for FakeNotifier {
    fn send(
        &mut self,
        _settings: &NotificationSettings,
        message: &NotificationMessage,
    ) -> std::result::Result<DeliveryReceipt, String> {
        self.messages.push(message.clone());
        self.receipts
            .pop_front()
            .unwrap_or_else(|| Err("transport unavailable".to_owned()))
    }
}

struct FakeExporter {
    fail: Option<String>,
}

impl FakeExporter {
    fn successful() -> Self {
        Self { fail: None }
    }
}

impl PdfExporter for FakeExporter {
    fn export(&mut self, request: &dms_core::ExportRequest) -> std::result::Result<(), String> {
        assert_eq!(
            request
                .temporary_pdf_path
                .extension()
                .and_then(|value| value.to_str()),
            Some("pdf")
        );
        if let Some(error) = self.fail.clone() {
            return Err(error);
        }
        fs::write(&request.temporary_pdf_path, b"%PDF-1.7\nfake export")
            .map_err(|error| error.to_string())
    }
}

fn approve_first_release(fixture: &mut Fixture) -> FakeGraph {
    let mut graph = fixture.graph();
    let mut review_notifier = FakeNotifier::accepted();
    let submission = fixture
        .workspace
        .submit_candidate(
            fixture.candidate_request(TargetSelection::NextMajor),
            &mut graph,
            &mut review_notifier,
        )
        .expect("review request");
    assert_eq!(submission.version, Version::V1_0);
    assert_eq!(submission.status, CandidateStatus::InReview);

    let mut outcome_notifier = FakeNotifier::accepted();
    fixture
        .workspace
        .decide_review(
            fixture.document_id,
            ReviewDecision::Approved,
            Some("Ready for release"),
            &mut graph,
            &mut outcome_notifier,
        )
        .expect("approval");
    graph
}

fn release_first(fixture: &mut Fixture) -> (FakeGraph, ReleaseOutcome) {
    let mut graph = approve_first_release(fixture);
    let mut exporter = FakeExporter::successful();
    let outcome = fixture
        .workspace
        .release_candidate(
            fixture.document_id,
            None,
            &mut graph,
            &mut FakeNotifier::default(),
            &mut exporter,
        )
        .expect("release");
    (graph, outcome)
}

#[test]
fn accepted_assistance_is_explicit_evidence_without_granting_lifecycle_authority() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let mut graph = fixture.graph();
    let mut request = fixture.candidate_request(TargetSelection::NextMajor);
    request.assistance = Some(AssistanceEvidence::claude_desktop());

    fixture
        .workspace
        .submit_candidate(request, &mut graph, &mut FakeNotifier::accepted())
        .unwrap();

    let candidate = fixture.workspace.candidates(fixture.document_id).unwrap()[0];
    assert_eq!(candidate.status, CandidateStatus::InReview);
    assert_eq!(
        candidate
            .assistance
            .as_ref()
            .map(|evidence| evidence.provider.as_str()),
        Some("Claude Desktop")
    );
    let history = fixture
        .workspace
        .workflow_history(fixture.document_id)
        .unwrap();
    let event = history.last().unwrap();
    assert_eq!(
        event
            .body
            .assistance
            .as_ref()
            .map(|evidence| evidence.provider.as_str()),
        Some("Claude Desktop")
    );
    assert_eq!(
        fixture
            .workspace
            .document(fixture.document_id)
            .unwrap()
            .lifecycle,
        Lifecycle::InReview
    );
}

#[test]
fn major_review_requires_graph_refresh_transport_success_and_verified_actor() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let mut graph = fixture.graph();
    let mut invalid_request = fixture.candidate_request(TargetSelection::NextMajor);
    invalid_request.changelog = "   ".to_owned();
    let mut invalid_notifier = FakeNotifier::accepted();
    assert!(matches!(
        fixture.workspace.submit_candidate(
            invalid_request,
            &mut graph,
            &mut invalid_notifier,
        ),
        Err(DmsError::InvalidConfiguration(field)) if field == "release changelog"
    ));
    let mut notifier = FakeNotifier {
        receipts: [
            Err("relay refused".to_owned()),
            Ok(DeliveryReceipt::accepted(250, "accepted")),
        ]
        .into_iter()
        .collect(),
        messages: Vec::new(),
    };

    let pending = fixture
        .workspace
        .submit_candidate(
            fixture.candidate_request(TargetSelection::NextMajor),
            &mut graph,
            &mut notifier,
        )
        .expect("candidate remains retryable");
    assert_eq!(pending.version, Version::V1_0);
    assert_eq!(pending.status, CandidateStatus::ReviewDeliveryFailed);
    assert_eq!(
        fixture
            .workspace
            .document(fixture.document_id)
            .expect("document")
            .lifecycle,
        Lifecycle::Draft
    );

    let retried = fixture
        .workspace
        .retry_review_notification(fixture.document_id, &mut notifier)
        .expect("retry");
    assert_eq!(retried.status, CandidateStatus::InReview);
    assert_eq!(notifier.messages.len(), 2);
    let message = &notifier.messages[0];
    assert_eq!(message.recipient, "approver@example.test");
    assert_eq!(
        message.subject,
        "[Internal] Review requested: Employee handbook (V1.0)"
    );
    for expected in [
        "Title: Employee handbook",
        "Source: Policies/Handbook.md",
        "Requested by: Rita Requester",
        "Target version: V1.0",
        "Confidentiality: Internal",
        "Review and decide:",
        "dms://open?workspace=",
        "&target=review&review=",
    ] {
        assert!(message.body.contains(expected), "missing {expected:?}");
    }

    graph.actor.object_id = fixture.editor_id;
    let mut outcome_notifier = FakeNotifier::accepted();
    assert!(matches!(
        fixture.workspace.decide_review(
            fixture.document_id,
            ReviewDecision::Approved,
            None,
            &mut graph,
            &mut outcome_notifier,
        ),
        Err(DmsError::DecisionActorMismatch)
    ));

    graph.actor.object_id = fixture.approver_id;
    let outcome = fixture
        .workspace
        .decide_review(
            fixture.document_id,
            ReviewDecision::Approved,
            None,
            &mut graph,
            &mut outcome_notifier,
        )
        .expect("eligible approver");
    assert_eq!(outcome.status, CandidateStatus::Approved);
    assert_eq!(outcome.delivery.status, DeliveryStatus::Accepted);
    assert_eq!(
        fixture
            .workspace
            .document(fixture.document_id)
            .expect("document")
            .lifecycle,
        Lifecycle::Approved
    );
}

#[test]
fn approved_release_is_atomic_mirrors_tree_persists_chain_and_refuses_overwrite() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let mut graph = approve_first_release(&mut fixture);
    let review_id = fixture.workspace.candidates(fixture.document_id).unwrap()[0]
        .review_id
        .expect("review ID");
    let review_link = fixture
        .workspace
        .review_permalink(fixture.document_id, review_id)
        .expect("review permalink");

    let mut no_notification = FakeNotifier::default();
    let mut failed_exporter = FakeExporter {
        fail: Some("Office conversion failed".to_owned()),
    };
    assert!(matches!(
        fixture.workspace.release_candidate(
            fixture.document_id,
            None,
            &mut graph,
            &mut no_notification,
            &mut failed_exporter,
        ),
        Err(DmsError::ExportFailed(message)) if message == "Office conversion failed"
    ));
    let candidate = fixture.workspace.candidates(fixture.document_id).unwrap()[0];
    assert_eq!(candidate.status, CandidateStatus::Approved);
    assert_eq!(candidate.export_failures, vec!["Office conversion failed"]);

    let mut exporter = FakeExporter::successful();
    let occupied_path = fixture
        .workspace
        .publish_root
        .join("Policies/Handbook_V1.0_internal.pdf");
    fs::create_dir_all(occupied_path.parent().unwrap()).unwrap();
    fs::write(&occupied_path, b"existing release").unwrap();
    assert!(matches!(
        fixture.workspace.release_candidate(
            fixture.document_id,
            None,
            &mut graph,
            &mut no_notification,
            &mut exporter,
        ),
        Err(DmsError::ReleasePathExists(path)) if path == occupied_path
    ));
    fs::remove_file(&occupied_path).unwrap();
    let outcome = fixture
        .workspace
        .release_candidate(
            fixture.document_id,
            None,
            &mut graph,
            &mut no_notification,
            &mut exporter,
        )
        .expect("release");
    assert_eq!(outcome.release.version, Version::V1_0);
    assert_eq!(
        outcome.release.relative_pdf_path,
        Path::new("Policies/Handbook_V1.0_internal.pdf")
    );
    assert!(fixture
        .workspace
        .publish_root
        .join(&outcome.release.relative_pdf_path)
        .is_file());
    assert_eq!(
        fixture
            .workspace
            .verify_workflow(fixture.document_id)
            .unwrap(),
        WorkflowVerification::Valid
    );

    let renamed = fixture
        .workspace
        .edit_root
        .join("Policies/Renamed-Handbook.md");
    fs::rename(&fixture.source_path, &renamed).expect("external rename");
    fixture
        .workspace
        .reassociate_document(fixture.document_id, &renamed)
        .expect("reassociate");
    assert_eq!(
        fixture
            .workspace
            .resolve_permalink(&review_link)
            .unwrap()
            .review_id,
        Some(review_id)
    );
    fixture.workspace.save().expect("persist release");
    let reopened = Workspace::open(&fixture.workspace.edit_root).expect("reopen");
    assert_eq!(reopened.releases(fixture.document_id).unwrap().len(), 1);
    assert_eq!(
        reopened.verify_workflow(fixture.document_id).unwrap(),
        WorkflowVerification::Valid
    );

    fixture
        .workspace
        .begin_revision(fixture.document_id)
        .unwrap();
    fs::write(
        &renamed,
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
    )
    .unwrap();
    let manual = fixture.candidate_request(TargetSelection::Manual(Version::V1_0));
    let mut notifier = FakeNotifier::accepted();
    assert!(matches!(
        fixture
            .workspace
            .submit_candidate(manual, &mut graph, &mut notifier),
        Err(DmsError::InvalidTargetVersion) | Err(DmsError::VersionAlreadyReleased(_))
    ));
}

#[test]
fn minor_release_skips_review_and_notification_failure_does_not_reverse_commit() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let mut graph = approve_first_release(&mut fixture);
    let mut exporter = FakeExporter::successful();
    let mut unused_notifier = FakeNotifier::default();
    fixture
        .workspace
        .release_candidate(
            fixture.document_id,
            None,
            &mut graph,
            &mut unused_notifier,
            &mut exporter,
        )
        .expect("first release");
    fixture
        .workspace
        .begin_revision(fixture.document_id)
        .unwrap();
    fs::write(
        &fixture.source_path,
        "# Handbook\n\nVersion: 1.1\n\nVertraulichkeitsstufe: Internal\n",
    )
    .unwrap();

    let mut notifier = FakeNotifier::default();
    let candidate = fixture
        .workspace
        .submit_candidate(
            fixture.candidate_request(TargetSelection::NextMinor),
            &mut graph,
            &mut notifier,
        )
        .expect("minor candidate");
    assert!(!candidate.approval_required);
    assert_eq!(candidate.status, CandidateStatus::Draft);
    assert!(notifier.messages.is_empty());

    let outcome: ReleaseOutcome = fixture
        .workspace
        .release_candidate(
            fixture.document_id,
            None,
            &mut graph,
            &mut notifier,
            &mut exporter,
        )
        .expect("minor release remains committed");
    assert_eq!(outcome.release.version, Version { major: 1, minor: 1 });
    assert_eq!(
        outcome.minor_notification.expect("delivery attempt").status,
        DeliveryStatus::Failed
    );
    assert_eq!(
        notifier.messages[0].subject,
        "[Internal] Minor release published: Employee handbook (V1.1)"
    );
    assert!(notifier.messages[0]
        .body
        .contains("Source: Policies/Handbook.md"));
    assert_eq!(
        fixture
            .workspace
            .releases(fixture.document_id)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        fixture
            .workspace
            .document(fixture.document_id)
            .expect("document")
            .lifecycle,
        Lifecycle::Released
    );
    let mut retry_notifier = FakeNotifier::accepted();
    let retry = fixture
        .workspace
        .retry_minor_publication_notification(
            fixture.document_id,
            outcome.release.id,
            &mut retry_notifier,
        )
        .expect("retry minor publication notification");
    assert_eq!(retry.status, DeliveryStatus::Accepted);
    assert_eq!(
        fixture
            .workspace
            .workflow_history(fixture.document_id)
            .unwrap()[0]
            .body
            .delivery
            .as_ref()
            .unwrap()
            .status,
        DeliveryStatus::Accepted
    );
}

#[test]
fn approver_policy_change_invalidates_open_review_and_target_remains_reusable() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let mut graph = fixture.graph();
    let mut notifier = FakeNotifier::accepted();
    fixture
        .workspace
        .submit_candidate(
            fixture.candidate_request(TargetSelection::NextMajor),
            &mut graph,
            &mut notifier,
        )
        .expect("review");
    fixture
        .workspace
        .update_workflow_policy(
            ".",
            RoleUpdate::Unchanged,
            RoleUpdate::replace(fixture.editor_id),
        )
        .expect("approver policy update");
    assert_eq!(
        fixture.workspace.candidates(fixture.document_id).unwrap()[0].status,
        CandidateStatus::Invalidated
    );
    assert_eq!(
        fixture
            .workspace
            .document(fixture.document_id)
            .unwrap()
            .lifecycle,
        Lifecycle::Draft
    );

    let mut notifier = FakeNotifier::accepted();
    let resubmitted = fixture
        .workspace
        .submit_candidate(
            fixture.candidate_request(TargetSelection::NextMajor),
            &mut graph,
            &mut notifier,
        )
        .expect("same uncommitted V1.0 target is reusable");
    assert_eq!(resubmitted.version, Version::V1_0);
}

#[test]
fn approved_candidate_is_invalidated_by_control_or_source_changes() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let mut graph = approve_first_release(&mut fixture);
    fixture
        .workspace
        .update_control(
            fixture.document_id,
            ControlUpdate {
                owner: Some(Some("Compliance team".to_owned())),
                ..ControlUpdate::default()
            },
        )
        .expect("control change");
    assert_eq!(
        fixture.workspace.candidates(fixture.document_id).unwrap()[0].status,
        CandidateStatus::Invalidated
    );

    let mut notifier = FakeNotifier::accepted();
    fixture
        .workspace
        .submit_candidate(
            fixture.candidate_request(TargetSelection::NextMajor),
            &mut graph,
            &mut notifier,
        )
        .expect("replacement review");
    let mut outcome_notifier = FakeNotifier::accepted();
    fixture
        .workspace
        .decide_review(
            fixture.document_id,
            ReviewDecision::Approved,
            None,
            &mut graph,
            &mut outcome_notifier,
        )
        .expect("replacement approval");
    fs::write(
        &fixture.source_path,
        "# changed\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
    )
    .unwrap();
    let mut exporter = FakeExporter::successful();
    assert!(matches!(
        fixture.workspace.release_candidate(
            fixture.document_id,
            None,
            &mut graph,
            &mut outcome_notifier,
            &mut exporter,
        ),
        Err(DmsError::CandidateInvalidated)
    ));
    let candidates = fixture.workspace.candidates(fixture.document_id).unwrap();
    assert_eq!(candidates[0].status, CandidateStatus::Invalidated);
    assert_eq!(
        fixture
            .workspace
            .workflow_history(fixture.document_id)
            .unwrap()[0]
            .body
            .event_type,
        WorkflowEventType::CandidateInvalidated
    );
    let reopened = Workspace::open(&fixture.workspace.edit_root).unwrap();
    assert_eq!(
        reopened.candidates(fixture.document_id).unwrap()[0].status,
        CandidateStatus::Invalidated
    );
}

#[test]
fn marker_scanners_ignore_non_rendered_markdown_scan_docx_surfaces_and_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let markdown = temp.path().join("policy.md");
    fs::write(
        &markdown,
        "---\nVersion: 9.9\n---\n<!-- Vertraulichkeitsstufe: Public -->\n```text\nVersion: 8.8\n```\n**Version:** 1.0\n**Vertraulichkeitsstufe:** Internal\n",
    )
    .unwrap();
    let markdown_check = dms_core::scan_content_markers(&markdown, Version::V1_0, "Internal")
        .expect("Markdown scan");
    assert_eq!(markdown_check.version.status, MarkerStatus::Match);
    assert_eq!(markdown_check.confidentiality.status, MarkerStatus::Match);

    let missing = temp.path().join("missing.md");
    fs::write(&missing, "# No controlled markers\n").unwrap();
    let missing_check =
        dms_core::scan_content_markers(&missing, Version::V1_0, "Internal").unwrap();
    assert_eq!(missing_check.version.status, MarkerStatus::Missing);
    assert_eq!(missing_check.confidentiality.status, MarkerStatus::Missing);

    let conflicting = temp.path().join("conflicting.md");
    fs::write(
        &conflicting,
        "Version: 1.0\nVersion: 2.0\nVertraulichkeitsstufe: Internal\n",
    )
    .unwrap();
    let conflicting_check =
        dms_core::scan_content_markers(&conflicting, Version::V1_0, "Internal").unwrap();
    assert_eq!(conflicting_check.version.status, MarkerStatus::Conflicting);

    let mismatch = temp.path().join("mismatch.md");
    fs::write(&mismatch, "Version: 9.9\nVertraulichkeitsstufe: Public\n").unwrap();
    let mismatch_check =
        dms_core::scan_content_markers(&mismatch, Version::V1_0, "Internal").unwrap();
    assert_eq!(mismatch_check.version.status, MarkerStatus::Mismatch);
    assert_eq!(
        mismatch_check.confidentiality.status,
        MarkerStatus::Mismatch
    );

    let docx = temp.path().join("policy.docx");
    let file = fs::File::create(&docx).unwrap();
    let mut archive = ZipWriter::new(file);
    archive
        .start_file("word/document.xml", SimpleFileOptions::default())
        .unwrap();
    archive
        .write_all(br#"<w:document><w:body><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Version: 1.0</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:txbxContent><w:p><w:r><w:t>Version: 1.0</w:t></w:r></w:p></w:txbxContent></w:body></w:document>"#)
        .unwrap();
    archive
        .start_file("word/header1.xml", SimpleFileOptions::default())
        .unwrap();
    archive
        .write_all(
            br#"<w:hdr><w:p><w:r><w:t>Vertraulichkeitsstufe: Internal</w:t></w:r></w:p></w:hdr>"#,
        )
        .unwrap();
    archive
        .start_file("word/footer1.xml", SimpleFileOptions::default())
        .unwrap();
    archive
        .write_all(
            br#"<w:ftr><w:p><w:r><w:t>Vertraulichkeitsstufe: Internal</w:t></w:r></w:p></w:ftr>"#,
        )
        .unwrap();
    archive.finish().unwrap();
    let docx_check =
        dms_core::scan_content_markers(&docx, Version::V1_0, "Internal").expect("DOCX scan");
    assert_eq!(docx_check.version.status, MarkerStatus::Match);
    assert_eq!(docx_check.confidentiality.status, MarkerStatus::Match);
    assert!(docx_check
        .confidentiality
        .locations
        .iter()
        .any(|location| location.contains("header1.xml")));
    assert!(docx_check
        .confidentiality
        .locations
        .iter()
        .any(|location| location.contains("footer1.xml")));

    let unsupported = temp.path().join("sheet.xlsx");
    fs::write(&unsupported, "not scanned").unwrap();
    assert!(matches!(
        dms_core::scan_content_markers(&unsupported, Version::V1_0, "Internal"),
        Err(DmsError::UnsupportedContentScanner(_))
    ));
}

#[test]
fn false_positive_override_is_revision_bound_and_hash_chained() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 9.9\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Mailto,
    );
    let mut graph = fixture.graph();
    let mut notifier = FakeNotifier::confirmed();
    let mut request = fixture.candidate_request(TargetSelection::NextMajor);
    request.review_override_reason = Some("Legacy appendix shows an example version".to_owned());
    let submission = fixture
        .workspace
        .submit_candidate(request, &mut graph, &mut notifier)
        .expect("reasoned override");
    assert_eq!(submission.status, CandidateStatus::InReview);
    let candidate = fixture.workspace.candidates(fixture.document_id).unwrap()[0];
    assert_eq!(candidate.content_overrides.len(), 1);
    assert_eq!(candidate.content_overrides[0].version, Version::V1_0);
    assert_eq!(
        candidate.content_overrides[0].draft_digest,
        candidate.source_digest
    );
    assert_eq!(
        fixture
            .workspace
            .verify_workflow(fixture.document_id)
            .unwrap(),
        WorkflowVerification::Valid
    );
    assert!(notifier.messages[0]
        .mailto_uri
        .starts_with("mailto:approver@example.test?"));
}

#[test]
fn graph_refresh_failure_blocks_candidate_without_using_stale_cache() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let mut graph = fixture.graph();
    graph.refresh_error = Some("offline".to_owned());
    let mut notifier = FakeNotifier::accepted();
    assert!(matches!(
        fixture.workspace.submit_candidate(
            fixture.candidate_request(TargetSelection::NextMajor),
            &mut graph,
            &mut notifier,
        ),
        Err(DmsError::GraphRefreshFailed(message)) if message == "offline"
    ));
    assert!(fixture
        .workspace
        .candidates(fixture.document_id)
        .unwrap()
        .is_empty());
}

#[test]
fn schema_v3_migrates_to_v4_with_empty_evidence_and_backup() {
    let fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    fixture.workspace.save().unwrap();
    let metadata_path = fixture.workspace.edit_root.join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
    metadata["schema_version"] = serde_json::Value::from(3);
    metadata
        .as_object_mut()
        .unwrap()
        .remove("notification_settings");
    let document = metadata["documents"][fixture.document_id.to_string()]
        .as_object_mut()
        .unwrap();
    for field in [
        "candidates",
        "active_candidate_id",
        "releases",
        "workflow_events",
    ] {
        document.remove(field);
    }
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(&fixture.workspace.edit_root).expect("migration");
    assert_eq!(migrated.schema_version, SCHEMA_VERSION);
    assert!(migrated.candidates(fixture.document_id).unwrap().is_empty());
    assert!(migrated.releases(fixture.document_id).unwrap().is_empty());
    assert_eq!(
        migrated.verify_workflow(fixture.document_id).unwrap(),
        WorkflowVerification::Missing
    );
    assert!(fixture
        .workspace
        .edit_root
        .join(".dms/workspace.v3.json.bak")
        .is_file());
}

#[test]
fn release_verification_reports_match_mismatch_and_missing_without_modifying_pdf() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let (_, outcome) = release_first(&mut fixture);
    let pdf = fixture
        .workspace
        .publish_root
        .join(&outcome.release.relative_pdf_path);
    let original = fs::read(&pdf).unwrap();

    assert_eq!(
        fixture
            .workspace
            .verify_release(fixture.document_id, outcome.release.id)
            .unwrap()
            .status,
        ReleaseVerificationStatus::Match
    );
    fs::write(&pdf, b"%PDF-1.7\ntampered").unwrap();
    assert_eq!(
        fixture
            .workspace
            .verify_release(fixture.document_id, outcome.release.id)
            .unwrap()
            .status,
        ReleaseVerificationStatus::Mismatch
    );
    assert_eq!(fs::read(&pdf).unwrap(), b"%PDF-1.7\ntampered");
    fs::remove_file(&pdf).unwrap();
    assert_eq!(
        fixture
            .workspace
            .verify_release(fixture.document_id, outcome.release.id)
            .unwrap()
            .status,
        ReleaseVerificationStatus::MissingFile
    );
    assert_ne!(original, b"%PDF-1.7\ntampered");
}

#[test]
fn periodic_review_binds_release_requires_integrity_and_records_result_transitions() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    fixture
        .workspace
        .configure_default_review_interval(6)
        .unwrap();
    let (mut graph, outcome) = release_first(&mut fixture);
    let marker = fixture
        .workspace
        .periodic_review_markers(chrono::Utc::now().date_naive())
        .unwrap()
        .remove(0);
    assert_eq!(marker.release_id, Some(outcome.release.id));
    assert!(marker.next_review_due.is_some());

    let review = fixture
        .workspace
        .start_periodic_review(fixture.document_id)
        .unwrap();
    assert_eq!(review.release_id, outcome.release.id);
    assert_eq!(review.pdf_digest, outcome.release.pdf_digest);
    assert_eq!(review.confidentiality, outcome.release.confidentiality);
    let completed = fixture
        .workspace
        .complete_periodic_review(
            fixture.document_id,
            review.id,
            PeriodicReviewResult::ConfirmedCurrent,
            "The released content remains current",
            &mut graph,
        )
        .unwrap();
    assert_eq!(completed.status, PeriodicReviewStatus::Completed);
    assert_eq!(
        completed.result,
        Some(PeriodicReviewResult::ConfirmedCurrent)
    );
    assert_eq!(
        fixture
            .workspace
            .verify_workflow(fixture.document_id)
            .unwrap(),
        WorkflowVerification::Valid
    );

    let mut changed = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let (mut graph, _) = release_first(&mut changed);
    let review = changed
        .workspace
        .start_periodic_review(changed.document_id)
        .unwrap();
    changed
        .workspace
        .complete_periodic_review(
            changed.document_id,
            review.id,
            PeriodicReviewResult::ChangesRequired,
            "Responsibilities changed",
            &mut graph,
        )
        .unwrap();
    assert_eq!(
        changed
            .workspace
            .document(changed.document_id)
            .unwrap()
            .lifecycle,
        Lifecycle::Draft
    );

    let mut obsolete = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let (mut graph, _) = release_first(&mut obsolete);
    let review = obsolete
        .workspace
        .start_periodic_review(obsolete.document_id)
        .unwrap();
    obsolete
        .workspace
        .complete_periodic_review(
            obsolete.document_id,
            review.id,
            PeriodicReviewResult::Obsolete,
            "This policy no longer applies",
            &mut graph,
        )
        .unwrap();
    assert_eq!(
        obsolete
            .workspace
            .document(obsolete.document_id)
            .unwrap()
            .lifecycle,
        Lifecycle::Obsolete
    );
}

#[test]
fn backup_archive_contains_metadata_controlled_sources_releases_and_verified_manifest() {
    let mut fixture = Fixture::new(
        "# Handbook\n\nVersion: 1.0\n\nVertraulichkeitsstufe: Internal\n",
        NotificationTransport::Smtp,
    );
    let (_, outcome) = release_first(&mut fixture);
    fixture.workspace.save().unwrap();
    let archive_path = fixture
        .workspace
        .edit_root
        .parent()
        .unwrap()
        .join("workspace.zip");

    let backup = fixture.workspace.backup_workspace(&archive_path).unwrap();
    assert_eq!(backup.entry_count, 3);
    assert_eq!(backup.manifest_digest.len(), 64);
    let file = fs::File::open(&archive_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut manifest = String::new();
    archive
        .by_name("manifest.json")
        .unwrap()
        .read_to_string(&mut manifest)
        .unwrap();
    let manifest: dms_core::BackupManifest = serde_json::from_str(&manifest).unwrap();
    let paths = manifest
        .entries
        .iter()
        .map(|entry| entry.archive_path.as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"edit/.dms/workspace.json"));
    assert!(paths.contains(&"edit/Policies/Handbook.md"));
    let pdf_archive =
        format!("publish/{}", outcome.release.relative_pdf_path.display()).replace('\\', "/");
    assert!(
        paths.contains(&pdf_archive.as_str()),
        "manifest entries: {paths:#?}"
    );
}

#[test]
fn schema_v4_migrates_periodic_review_defaults_and_creates_a_versioned_backup() {
    let fixture = Fixture::new("# Handbook\n", NotificationTransport::Smtp);
    fixture.workspace.save().unwrap();
    let metadata_path = fixture.workspace.edit_root.join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
    metadata["schema_version"] = serde_json::Value::from(4);
    metadata
        .as_object_mut()
        .unwrap()
        .remove("default_review_interval_months");
    let document = metadata["documents"][fixture.document_id.to_string()]
        .as_object_mut()
        .unwrap();
    for field in [
        "review_interval_months",
        "review_exemption_reason",
        "next_review_due",
        "periodic_reviews",
    ] {
        document.remove(field);
    }
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(&fixture.workspace.edit_root).unwrap();
    migrated.save().unwrap();
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
    assert_eq!(persisted["schema_version"], SCHEMA_VERSION);
    assert_eq!(persisted["default_review_interval_months"], 12);
    assert!(
        persisted["documents"][fixture.document_id.to_string()]["periodic_reviews"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(fixture
        .workspace
        .edit_root
        .join(".dms/workspace.v4.json.bak")
        .is_file());
}

#[test]
fn schema_v5_migrates_disabled_claude_assistance_policy_and_creates_backup() {
    let fixture = Fixture::new("# Handbook\n", NotificationTransport::Smtp);
    fixture.workspace.save().unwrap();
    let metadata_path = fixture.workspace.edit_root.join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
    metadata["schema_version"] = serde_json::Value::from(5);
    metadata
        .as_object_mut()
        .unwrap()
        .remove("claude_assistance");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(&fixture.workspace.edit_root).unwrap();
    assert!(!migrated.claude_assistance_policy().enabled);
    assert!(migrated
        .claude_assistance_policy()
        .allowed_confidentiality_type_ids
        .is_empty());
    assert_eq!(
        migrated.claude_assistance_policy().max_payload_chars,
        dms_core::DEFAULT_CLAUDE_PAYLOAD_LIMIT
    );
    assert!(fixture
        .workspace
        .edit_root
        .join(".dms/workspace.v5.json.bak")
        .is_file());
}
