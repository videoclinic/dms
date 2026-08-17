use std::{
    fmt, fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use chrono::{DateTime, NaiveDate, Utc};
use quick_xml::{events::Event as XmlEvent, Reader};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zip::ZipArchive;

use super::{
    configured_text, default_author, AssistanceEvidence, DmsError, DocumentControl,
    EffectiveWorkflowRole, EntraIdentitySource, EntraPerson, Lifecycle, PeriodicReviewResult,
    ResolutionState, Result, SourceState, Workspace,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl Version {
    pub const V1_0: Self = Self { major: 1, minor: 0 };
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "V{}.{}", self.major, self.minor)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetVersionMode {
    NextMinor,
    NextMajor,
    Manual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetSelection {
    NextMinor,
    NextMajor,
    Manual(Version),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonSnapshot {
    pub tenant_id: Uuid,
    pub object_id: Uuid,
    pub display_name: String,
    pub email: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateStatus {
    Draft,
    ReviewDeliveryFailed,
    InReview,
    Approved,
    Rejected,
    ChangesRequested,
    Cancelled,
    Invalidated,
    Released,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfidentialitySnapshot {
    pub type_id: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateMetadataSnapshot {
    pub control: DocumentControl,
    pub confidentiality: ConfidentialitySnapshot,
    #[serde(default)]
    pub owner: Option<PersonSnapshot>,
    pub editor: PersonSnapshot,
    pub approver: PersonSnapshot,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Queued,
    Accepted,
    Confirmed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryReceipt {
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub detail: String,
}

impl DeliveryReceipt {
    pub fn accepted(response_code: u16, detail: &str) -> Self {
        Self {
            status: DeliveryStatus::Accepted,
            response_code: Some(response_code),
            detail: detail.to_owned(),
        }
    }

    pub fn confirmed(detail: &str) -> Self {
        Self {
            status: DeliveryStatus::Confirmed,
            response_code: None,
            detail: detail.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryAttempt {
    pub kind: NotificationKind,
    pub recipient: String,
    pub transport: NotificationTransport,
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub detail: String,
    pub attempted_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    ReviewRequest,
    DecisionOutcome,
    MinorPublication,
    PeriodicReviewReminder,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationTransport {
    Smtp,
    Mailto,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SmtpSettings {
    pub relay_host: String,
    pub relay_port: u16,
    pub login_user: String,
    pub from_mailbox: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NotificationSettings {
    pub transport: NotificationTransport,
    pub smtp: Option<SmtpSettings>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationMessage {
    pub kind: NotificationKind,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    /// HTML alternative part for rich mail clients; the plain `body` remains
    /// the canonical plain-text contract and the `mailto:` draft content.
    pub html_body: String,
    pub mailto_uri: String,
}

pub trait NotificationClient {
    fn send(
        &mut self,
        settings: &NotificationSettings,
        message: &NotificationMessage,
    ) -> std::result::Result<DeliveryReceipt, String>;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckPhase {
    Review,
    Release,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarkerStatus {
    Match,
    Missing,
    Mismatch,
    Conflicting,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkerVerdict {
    pub status: MarkerStatus,
    pub expected: String,
    pub detected: Vec<String>,
    pub locations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentCheck {
    pub version: MarkerVerdict,
    pub confidentiality: MarkerVerdict,
    #[serde(default)]
    pub title: Option<MarkerVerdict>,
    #[serde(default)]
    pub document_number: Option<MarkerVerdict>,
}

impl ContentCheck {
    pub fn passes(&self) -> bool {
        self.version.status == MarkerStatus::Match
            && self.confidentiality.status == MarkerStatus::Match
            && self
                .title
                .as_ref()
                .is_none_or(|verdict| verdict.status == MarkerStatus::Match)
            && self
                .document_number
                .as_ref()
                .is_none_or(|verdict| verdict.status == MarkerStatus::Match)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentOverride {
    pub phase: CheckPhase,
    pub reason: String,
    pub draft_digest: String,
    pub version: Version,
    pub confidentiality: ConfidentialitySnapshot,
    pub check: ContentCheck,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseCandidate {
    pub id: Uuid,
    pub review_id: Option<Uuid>,
    pub version: Version,
    pub mode: TargetVersionMode,
    pub approval_required: bool,
    pub changelog: String,
    #[serde(default)]
    pub assistance: Option<AssistanceEvidence>,
    pub requester: PersonSnapshot,
    pub metadata: CandidateMetadataSnapshot,
    #[serde(default)]
    pub effective_date: Option<NaiveDate>,
    #[serde(default)]
    pub staged_owner: Option<super::OwnerReference>,
    #[serde(default)]
    pub staged_editor: Option<super::WorkflowRoleRef>,
    pub source_digest: String,
    pub source_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub status: CandidateStatus,
    pub decision_comment: Option<String>,
    pub approval_event_hash: Option<String>,
    pub delivery_attempts: Vec<DeliveryAttempt>,
    pub content_overrides: Vec<ContentOverride>,
    #[serde(default)]
    pub export_failures: Vec<String>,
    pub release_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseRecord {
    pub id: Uuid,
    pub version: Version,
    pub relative_pdf_path: PathBuf,
    pub source_digest: String,
    pub pdf_digest: String,
    pub changelog: String,
    #[serde(default)]
    pub assistance: Option<AssistanceEvidence>,
    pub mode: TargetVersionMode,
    pub approval_required: bool,
    pub approval_chain_head: Option<String>,
    pub workflow_chain_head: String,
    pub confidentiality: ConfidentialitySnapshot,
    #[serde(default)]
    pub owner: Option<PersonSnapshot>,
    pub editor: PersonSnapshot,
    pub approver: PersonSnapshot,
    pub requester: PersonSnapshot,
    #[serde(default)]
    pub effective_date: Option<NaiveDate>,
    #[serde(default)]
    pub control: Option<DocumentControl>,
    pub released_at: DateTime<Utc>,
    pub withdrawn: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowEventType {
    ReviewRequested,
    ReviewDecisionApproved,
    ReviewDecisionRejected,
    ReviewDecisionChangedRequested,
    DecisionOutcomeNotified,
    Release,
    ReleaseWithdrawn,
    MinorPublicationNotified,
    ReviewCancelled,
    CandidateInvalidated,
    DocumentControlDataChanged,
    RevisionBegun,
    DocumentObsoleted,
    ContentConformanceOverridden,
    PeriodicReviewRequested,
    PeriodicReviewCompleted,
    PeriodicReviewCancelled,
    PeriodicReviewReminder,
    ReportGenerated,
    SourceReassociated,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceReassociation {
    pub previous_relative_path: String,
    pub new_relative_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absorbed_document_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeriodicReviewEventDetails {
    pub review_id: Uuid,
    pub release_id: Uuid,
    pub result: Option<PeriodicReviewResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentControlChange {
    pub before: DocumentControl,
    pub after: DocumentControl,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthenticatedActor {
    pub tenant_id: Uuid,
    pub object_id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowEventBody {
    pub event_id: Uuid,
    pub document_id: Uuid,
    pub event_type: WorkflowEventType,
    pub predecessor_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub requester: Option<PersonSnapshot>,
    pub editor: Option<PersonSnapshot>,
    pub approver: Option<PersonSnapshot>,
    pub authenticated_actor: Option<AuthenticatedActor>,
    pub local_os_user: String,
    pub revision_digest: Option<String>,
    pub confidentiality: Option<ConfidentialitySnapshot>,
    pub target_version: Option<Version>,
    pub target_mode: Option<TargetVersionMode>,
    pub changelog: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistance: Option<AssistanceEvidence>,
    pub decision_comment: Option<String>,
    pub operator_comment: Option<String>,
    pub delivery: Option<DeliveryAttempt>,
    pub content_override: Option<ContentOverride>,
    pub pdf_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub periodic_review: Option<PeriodicReviewEventDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_change: Option<DocumentControlChange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<crate::AuditReportEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_reassociation: Option<SourceReassociation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowEvent {
    pub body: WorkflowEventBody,
    pub event_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowVerification {
    Valid,
    TamperedAt(Uuid),
    Missing,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct LifecycleActionAvailability {
    pub available: bool,
    pub reason: Option<&'static str>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct LocalLifecycleActions {
    pub cancel_review: LifecycleActionAvailability,
    pub mark_obsolete: LifecycleActionAvailability,
}

impl WorkflowVerification {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

pub trait GraphClient {
    fn tenant_id(&self) -> std::result::Result<Uuid, String>;

    fn direct_user_members(
        &mut self,
        source: &EntraIdentitySource,
    ) -> std::result::Result<Vec<EntraPerson>, String>;

    fn authenticated_actor(
        &mut self,
        source: &EntraIdentitySource,
    ) -> std::result::Result<AuthenticatedActor, String>;
}

#[derive(Clone, Debug)]
pub struct CandidateRequest {
    pub document_id: Uuid,
    pub selection: TargetSelection,
    pub changelog: String,
    pub effective_date: NaiveDate,
    pub requester_object_id: Uuid,
    pub staged_owner_object_id: Option<Uuid>,
    pub staged_editor_object_id: Option<Uuid>,
    pub review_override_reason: Option<String>,
    pub assistance: Option<AssistanceEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateSubmission {
    pub candidate_id: Uuid,
    pub review_id: Option<Uuid>,
    pub version: Version,
    pub approval_required: bool,
    pub status: CandidateStatus,
    pub delivery: Option<DeliveryAttempt>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewDecision {
    Approved,
    Rejected,
    ChangesRequested,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionOutcome {
    pub candidate_id: Uuid,
    pub status: CandidateStatus,
    pub delivery: DeliveryAttempt,
}

#[derive(Clone, Debug)]
pub struct ExportChrome {
    pub version_label: String,
    pub confidentiality: ConfidentialitySnapshot,
    pub title: String,
    pub document_number: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ExportRequest {
    pub document_id: Uuid,
    pub source_path: PathBuf,
    pub markdown_template_path: Option<PathBuf>,
    pub temporary_pdf_path: PathBuf,
    pub final_pdf_path: PathBuf,
    pub chrome: ExportChrome,
}

pub trait PdfExporter {
    fn export(&mut self, request: &ExportRequest) -> std::result::Result<(), String>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseOutcome {
    pub release: ReleaseRecord,
    pub minor_notification: Option<DeliveryAttempt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPermalink {
    pub document_id: Uuid,
    pub target: PermalinkTarget,
    pub review_id: Option<Uuid>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermalinkTarget {
    Document,
    Review,
    Notes,
}

#[derive(Default)]
struct CandidateEventDetails {
    actor: Option<AuthenticatedActor>,
    decision_comment: Option<String>,
    delivery: Option<DeliveryAttempt>,
    operator_comment: Option<String>,
    pdf_digest: Option<String>,
}

impl Workspace {
    pub fn notification_settings(&self) -> Option<&NotificationSettings> {
        self.notification_settings.as_ref()
    }

    pub fn configure_notifications(
        &mut self,
        transport: NotificationTransport,
        smtp: Option<SmtpSettings>,
    ) -> Result<NotificationSettings> {
        let smtp = match transport {
            NotificationTransport::Smtp => {
                let mut smtp = smtp.ok_or(DmsError::SmtpConfigurationRequired)?;
                smtp.relay_host = configured_text(&smtp.relay_host, "SMTP relay host")?;
                smtp.login_user = configured_text(&smtp.login_user, "SMTP login user")?;
                smtp.from_mailbox = configured_text(&smtp.from_mailbox, "SMTP From address")?;
                smtp.from_mailbox
                    .parse::<lettre::message::Mailbox>()
                    .map_err(|error| {
                        DmsError::InvalidConfiguration(format!(
                            "SMTP From address is not a valid mailbox: {error}"
                        ))
                    })?;
                if smtp.relay_port == 0 {
                    return Err(DmsError::InvalidConfiguration("SMTP relay port".to_owned()));
                }
                Some(smtp)
            }
            NotificationTransport::Mailto => None,
        };
        let settings = NotificationSettings { transport, smtp };
        self.notification_settings = Some(settings.clone());
        Ok(settings)
    }

    pub fn refresh_eligible_people<G: GraphClient + ?Sized>(
        &mut self,
        graph: &mut G,
    ) -> Result<()> {
        graph.tenant_id().map_err(DmsError::GraphRefreshFailed)?;
        let source = self
            .identity_source
            .clone()
            .ok_or(DmsError::IdentitySourceRequired)?;
        let people = graph
            .direct_user_members(&source)
            .map_err(DmsError::GraphRefreshFailed)?;
        let mut refreshed = std::collections::BTreeMap::new();
        for mut person in people {
            person.display_name = configured_text(&person.display_name, "person display name")?;
            person.email = configured_text(&person.email, "person email")?;
            if person.account_enabled {
                refreshed.insert(person.object_id, person);
            }
        }
        self.identity_cache = refreshed;
        self.identity_source
            .as_mut()
            .expect("identity source was checked before refresh")
            .last_refreshed_at = Some(Utc::now());
        self.invalidate_stale_candidates();
        Ok(())
    }

    pub fn submit_candidate<G: GraphClient, N: NotificationClient>(
        &mut self,
        request: CandidateRequest,
        graph: &mut G,
        notifier: &mut N,
    ) -> Result<CandidateSubmission> {
        let tenant_id = graph.tenant_id().map_err(DmsError::GraphRefreshFailed)?;
        self.refresh_eligible_people(graph)?;
        let settings = self
            .notification_settings
            .clone()
            .ok_or(DmsError::NotificationSettingsRequired)?;
        self.ensure_document_can_start_candidate(request.document_id)?;
        let document = self.document(request.document_id)?.clone();
        if request.staged_owner_object_id.is_some() != request.staged_editor_object_id.is_some() {
            return Err(DmsError::InvalidConfiguration(
                "staged owner and editor must be supplied together".to_owned(),
            ));
        }
        let binding_id = self
            .identity_source()
            .ok_or(DmsError::IdentitySourceRequired)?
            .binding_id;
        let staged_owner = request
            .staged_owner_object_id
            .map(|object_id| super::OwnerReference {
                binding_id,
                object_id,
            });
        if let Some(reference) = staged_owner {
            self.require_eligible_owner(reference)?;
        }
        let staged_editor =
            request
                .staged_editor_object_id
                .map(|object_id| super::WorkflowRoleRef {
                    binding_id,
                    object_id,
                });
        let confidentiality = self.effective_confidentiality(request.document_id)?;
        let roles = self.effective_workflow_roles(request.document_id)?;
        let mut control = document.control.clone();
        if let Some(owner) = staged_owner {
            control.owner = Some(owner);
            control.legacy_owner_label = None;
        }
        validate_release_control(&control)?;
        let owner = self.person_snapshot(
            control
                .owner
                .expect("release control validation requires an owner")
                .object_id,
            tenant_id,
        )?;
        let metadata = CandidateMetadataSnapshot {
            control,
            confidentiality: ConfidentialitySnapshot {
                type_id: confidentiality.type_id,
                label: confidentiality.label,
            },
            owner: Some(owner),
            editor: match staged_editor {
                Some(editor) => self.person_snapshot(editor.object_id, tenant_id)?,
                None => self.role_snapshot(roles.editor, tenant_id)?,
            },
            approver: self.role_snapshot(roles.approver, tenant_id)?,
        };
        let version = self.resolve_target_version(request.document_id, request.selection)?;
        let current = self.current_release(request.document_id)?;
        let approval_required = current
            .map(|release| version.major > release.version.major)
            .unwrap_or(true);
        let mode = match request.selection {
            TargetSelection::NextMinor => TargetVersionMode::NextMinor,
            TargetSelection::NextMajor => TargetVersionMode::NextMajor,
            TargetSelection::Manual(_) => TargetVersionMode::Manual,
        };
        let changelog = configured_text(&request.changelog, "release changelog")?;
        // DMS is authoritative for controlled Markdown frontmatter: rewrite the
        // candidate target version first so digest and content checks match.
        self.sync_markdown_control_frontmatter_with_version(
            request.document_id,
            Some(&format!("{}.{}", version.major, version.minor)),
        )?;
        let source_path = self.edit_root.join(&document.relative_path);
        let source_digest = sha256_file(&source_path)?;
        let requester = self.person_snapshot(request.requester_object_id, tenant_id)?;
        let review_id = approval_required.then(Uuid::new_v4);
        let mut candidate = ReleaseCandidate {
            id: Uuid::new_v4(),
            review_id,
            version,
            mode,
            approval_required,
            changelog,
            assistance: request.assistance.clone(),
            requester,
            metadata,
            effective_date: Some(request.effective_date),
            staged_owner,
            staged_editor,
            source_digest,
            source_path: document.relative_path.clone(),
            created_at: Utc::now(),
            status: CandidateStatus::Draft,
            decision_comment: None,
            approval_event_hash: None,
            delivery_attempts: Vec::new(),
            content_overrides: Vec::new(),
            export_failures: Vec::new(),
            release_id: None,
        };

        if approval_required {
            let check = scan_content_conformance(
                &source_path,
                version,
                &candidate.metadata.confidentiality.type_id,
                &candidate.metadata.confidentiality.label,
                Some(&candidate.metadata.control.title),
                Some(candidate.metadata.control.document_number.as_deref()),
            )?;
            self.accept_or_reject_content_check(
                request.document_id,
                &mut candidate,
                CheckPhase::Review,
                check,
                request.review_override_reason.as_deref(),
            )?;
        }

        let candidate_id = candidate.id;
        self.documents
            .get_mut(&request.document_id)
            .expect("document checked above")
            .candidates
            .push(candidate);
        self.documents
            .get_mut(&request.document_id)
            .expect("document checked above")
            .active_candidate_id = Some(candidate_id);

        if !approval_required {
            return Ok(CandidateSubmission {
                candidate_id,
                review_id: None,
                version,
                approval_required: false,
                status: CandidateStatus::Draft,
                delivery: None,
            });
        }

        let message = self.review_message(request.document_id, candidate_id)?;
        let attempt = delivery_attempt(&settings, &message, notifier);
        let successful = delivery_advances_workflow(settings.transport, attempt.status);
        {
            let candidate = self.candidate_mut(request.document_id, candidate_id)?;
            candidate.delivery_attempts.push(attempt.clone());
            candidate.status = if successful {
                CandidateStatus::InReview
            } else {
                CandidateStatus::ReviewDeliveryFailed
            };
        }
        if successful {
            self.documents
                .get_mut(&request.document_id)
                .expect("document checked above")
                .lifecycle = Lifecycle::InReview;
            let candidate = self.candidate(request.document_id, candidate_id)?.clone();
            self.append_candidate_event(
                request.document_id,
                WorkflowEventType::ReviewRequested,
                &candidate,
                CandidateEventDetails {
                    delivery: Some(attempt.clone()),
                    ..CandidateEventDetails::default()
                },
            )?;
        }
        Ok(CandidateSubmission {
            candidate_id,
            review_id,
            version,
            approval_required: true,
            status: self.candidate(request.document_id, candidate_id)?.status,
            delivery: Some(attempt),
        })
    }

    pub fn retry_review_notification<N: NotificationClient>(
        &mut self,
        document_id: Uuid,
        notifier: &mut N,
    ) -> Result<CandidateSubmission> {
        let settings = self
            .notification_settings
            .clone()
            .ok_or(DmsError::NotificationSettingsRequired)?;
        let candidate_id = self.active_candidate_id(document_id)?;
        let candidate = self.candidate(document_id, candidate_id)?;
        if candidate.status != CandidateStatus::ReviewDeliveryFailed {
            return Err(DmsError::InvalidLifecycleTransition(
                "review notification is not awaiting retry".to_owned(),
            ));
        }
        let message = self.review_message(document_id, candidate_id)?;
        let attempt = delivery_attempt(&settings, &message, notifier);
        let successful = delivery_advances_workflow(settings.transport, attempt.status);
        {
            let candidate = self.candidate_mut(document_id, candidate_id)?;
            candidate.delivery_attempts.push(attempt.clone());
            if successful {
                candidate.status = CandidateStatus::InReview;
            }
        }
        if successful {
            self.documents
                .get_mut(&document_id)
                .expect("document checked above")
                .lifecycle = Lifecycle::InReview;
            let candidate = self.candidate(document_id, candidate_id)?.clone();
            self.append_candidate_event(
                document_id,
                WorkflowEventType::ReviewRequested,
                &candidate,
                CandidateEventDetails {
                    delivery: Some(attempt.clone()),
                    ..CandidateEventDetails::default()
                },
            )?;
        }
        let candidate = self.candidate(document_id, candidate_id)?;
        Ok(CandidateSubmission {
            candidate_id,
            review_id: candidate.review_id,
            version: candidate.version,
            approval_required: true,
            status: candidate.status,
            delivery: Some(attempt),
        })
    }

    pub fn decide_review<G: GraphClient, N: NotificationClient>(
        &mut self,
        document_id: Uuid,
        decision: ReviewDecision,
        comment: Option<&str>,
        graph: &mut G,
        notifier: &mut N,
    ) -> Result<DecisionOutcome> {
        let candidate_id = self.active_candidate_id(document_id)?;
        let candidate = self.candidate(document_id, candidate_id)?.clone();
        let tenant_id = graph.tenant_id().map_err(DmsError::GraphRefreshFailed)?;
        self.refresh_eligible_people(graph)?;
        if self.document(document_id)?.active_candidate_id != Some(candidate_id) {
            self.save()?;
            return Err(DmsError::CandidateInvalidated);
        }
        let settings = self
            .notification_settings
            .clone()
            .ok_or(DmsError::NotificationSettingsRequired)?;
        if candidate.status != CandidateStatus::InReview
            || self.document(document_id)?.lifecycle != Lifecycle::InReview
        {
            return Err(DmsError::InvalidLifecycleTransition(
                "document is not in review".to_owned(),
            ));
        }
        if self
            .ensure_candidate_current(document_id, &candidate, tenant_id)
            .is_err()
        {
            self.invalidate_candidate(document_id, candidate_id, "draft or metadata changed")?;
            self.save()?;
            return Err(DmsError::CandidateInvalidated);
        }
        let source = self
            .identity_source
            .clone()
            .ok_or(DmsError::IdentitySourceRequired)?;
        let actor = graph
            .authenticated_actor(&source)
            .map_err(DmsError::InteractiveSignInFailed)?;
        if actor.tenant_id != candidate.metadata.approver.tenant_id
            || actor.object_id != candidate.metadata.approver.object_id
            || !self.identity_cache.contains_key(&actor.object_id)
        {
            return Err(DmsError::DecisionActorMismatch);
        }
        let comment = comment
            .map(|value| validate_comment(value, false))
            .transpose()?;
        let (event_type, status, lifecycle) = match decision {
            ReviewDecision::Approved => (
                WorkflowEventType::ReviewDecisionApproved,
                CandidateStatus::Approved,
                Lifecycle::Approved,
            ),
            ReviewDecision::Rejected => (
                WorkflowEventType::ReviewDecisionRejected,
                CandidateStatus::Rejected,
                Lifecycle::Draft,
            ),
            ReviewDecision::ChangesRequested => (
                WorkflowEventType::ReviewDecisionChangedRequested,
                CandidateStatus::ChangesRequested,
                Lifecycle::Draft,
            ),
        };
        let event = self.append_candidate_event(
            document_id,
            event_type,
            &candidate,
            CandidateEventDetails {
                actor: Some(actor),
                decision_comment: comment.clone(),
                ..CandidateEventDetails::default()
            },
        )?;
        {
            let candidate = self.candidate_mut(document_id, candidate_id)?;
            candidate.status = status;
            candidate.decision_comment = comment;
            if status == CandidateStatus::Approved {
                candidate.approval_event_hash = Some(event.event_hash);
            }
        }
        let document = self
            .documents
            .get_mut(&document_id)
            .expect("document checked above");
        document.lifecycle = lifecycle;
        if status != CandidateStatus::Approved {
            document.active_candidate_id = None;
        }

        let message = decision_message(
            &candidate,
            decision,
            self.review_permalink(document_id, candidate.review_id.expect("review candidate"))?,
        );
        let attempt = delivery_attempt(&settings, &message, notifier);
        self.candidate_mut(document_id, candidate_id)?
            .delivery_attempts
            .push(attempt.clone());
        self.append_candidate_event(
            document_id,
            WorkflowEventType::DecisionOutcomeNotified,
            &candidate,
            CandidateEventDetails {
                delivery: Some(attempt.clone()),
                ..CandidateEventDetails::default()
            },
        )?;
        Ok(DecisionOutcome {
            candidate_id,
            status,
            delivery: attempt,
        })
    }

    pub fn release_candidate<G: GraphClient, N: NotificationClient, E: PdfExporter>(
        &mut self,
        document_id: Uuid,
        release_override_reason: Option<&str>,
        graph: &mut G,
        notifier: &mut N,
        exporter: &mut E,
    ) -> Result<ReleaseOutcome> {
        let tenant_id = graph.tenant_id().map_err(DmsError::GraphRefreshFailed)?;
        let candidate_id = self.active_candidate_id(document_id)?;
        let mut candidate = self.candidate(document_id, candidate_id)?.clone();
        let expected_status = if candidate.approval_required {
            CandidateStatus::Approved
        } else {
            CandidateStatus::Draft
        };
        if candidate.status != expected_status {
            return Err(DmsError::InvalidLifecycleTransition(
                "candidate is not ready for release".to_owned(),
            ));
        }
        let effective_date = candidate
            .effective_date
            .ok_or_else(|| DmsError::InvalidConfiguration("candidate effective date".to_owned()))?;
        self.refresh_eligible_people(graph)?;
        if self.document(document_id)?.active_candidate_id != Some(candidate_id) {
            self.save()?;
            return Err(DmsError::CandidateInvalidated);
        }
        if self
            .ensure_candidate_current(document_id, &candidate, tenant_id)
            .is_err()
        {
            self.invalidate_candidate(document_id, candidate_id, "draft or metadata changed")?;
            self.save()?;
            return Err(DmsError::CandidateInvalidated);
        }
        let source_path = self
            .edit_root
            .join(&self.document(document_id)?.relative_path);
        let check = scan_content_conformance(
            &source_path,
            candidate.version,
            &candidate.metadata.confidentiality.type_id,
            &candidate.metadata.confidentiality.label,
            Some(&candidate.metadata.control.title),
            Some(candidate.metadata.control.document_number.as_deref()),
        )?;
        self.accept_or_reject_content_check(
            document_id,
            &mut candidate,
            CheckPhase::Release,
            check,
            release_override_reason,
        )?;
        self.candidate_mut(document_id, candidate_id)?
            .content_overrides = candidate.content_overrides.clone();

        let markdown_template_path = source_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
            .then(|| self.markdown_template_path_for_export())
            .transpose()?;

        let relative_path = release_relative_path(
            &self.document(document_id)?.relative_path,
            candidate.version,
            &candidate.metadata.confidentiality.type_id,
        )?;
        let final_path = self.publish_root.join(&relative_path);
        if final_path.exists() {
            return Err(DmsError::ReleasePathExists(final_path));
        }
        let parent = final_path
            .parent()
            .ok_or_else(|| DmsError::InvalidReleasePath(final_path.clone()))?;
        fs::create_dir_all(parent).map_err(|source| DmsError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
        let temporary_path = parent.join(format!(".release-{}.tmp.pdf", Uuid::new_v4()));
        let export_request = ExportRequest {
            document_id,
            source_path,
            markdown_template_path,
            temporary_pdf_path: temporary_path.clone(),
            final_pdf_path: final_path.clone(),
            chrome: ExportChrome {
                version_label: candidate.version.to_string(),
                confidentiality: candidate.metadata.confidentiality.clone(),
                title: candidate.metadata.control.title.clone(),
                document_number: candidate.metadata.control.document_number.clone(),
            },
        };
        if let Err(error) = exporter.export(&export_request) {
            let _ = fs::remove_file(&temporary_path);
            self.candidate_mut(document_id, candidate_id)?
                .export_failures
                .push(error.clone());
            self.save()?;
            return Err(DmsError::ExportFailed(error));
        }
        validate_exported_pdf(&temporary_path)?;
        let pdf_digest = sha256_file(&temporary_path)?;
        fs::rename(&temporary_path, &final_path).map_err(|source| DmsError::Io {
            path: final_path.clone(),
            source,
        })?;

        let before_commit = self.clone();
        let commit_result = (|| -> Result<ReleaseOutcome> {
            let release_id = Uuid::new_v4();
            let placeholder_chain = self
                .document(document_id)?
                .workflow_events
                .last()
                .map(|event| event.event_hash.clone())
                .unwrap_or_default();
            let mut release = ReleaseRecord {
                id: release_id,
                version: candidate.version,
                relative_pdf_path: relative_path,
                source_digest: candidate.source_digest.clone(),
                pdf_digest: pdf_digest.clone(),
                changelog: candidate.changelog.clone(),
                assistance: candidate.assistance.clone(),
                mode: candidate.mode,
                approval_required: candidate.approval_required,
                approval_chain_head: candidate.approval_event_hash.clone(),
                workflow_chain_head: placeholder_chain,
                confidentiality: candidate.metadata.confidentiality.clone(),
                owner: candidate.metadata.owner.clone(),
                editor: candidate.metadata.editor.clone(),
                approver: candidate.metadata.approver.clone(),
                requester: candidate.requester.clone(),
                effective_date: Some(effective_date),
                control: Some(candidate.metadata.control.clone()),
                released_at: Utc::now(),
                withdrawn: false,
            };
            let release_event = self.append_candidate_event(
                document_id,
                WorkflowEventType::Release,
                &candidate,
                CandidateEventDetails {
                    pdf_digest: Some(pdf_digest),
                    ..CandidateEventDetails::default()
                },
            )?;
            release.workflow_chain_head = release_event.event_hash;
            {
                let stored = self.candidate_mut(document_id, candidate_id)?;
                stored.status = CandidateStatus::Released;
                stored.release_id = Some(release_id);
            }
            {
                let document = self
                    .documents
                    .get_mut(&document_id)
                    .expect("document checked above");
                document.releases.push(release.clone());
                document.active_candidate_id = None;
                document.lifecycle = Lifecycle::Released;
                if let Some(owner) = candidate.staged_owner {
                    document.control.owner = Some(owner);
                    document.control.legacy_owner_label = None;
                }
                if let Some(editor) = candidate.staged_editor {
                    document.workflow_overrides.editor = Some(editor);
                }
            }
            self.schedule_next_review(document_id, effective_date)?;

            let mut minor_notification = None;
            if !candidate.approval_required {
                let settings = self
                    .notification_settings
                    .clone()
                    .ok_or(DmsError::NotificationSettingsRequired)?;
                let message =
                    minor_publication_message(&candidate, self.document_permalink(document_id)?);
                let attempt = delivery_attempt(&settings, &message, notifier);
                self.candidate_mut(document_id, candidate_id)?
                    .delivery_attempts
                    .push(attempt.clone());
                self.append_candidate_event(
                    document_id,
                    WorkflowEventType::MinorPublicationNotified,
                    &candidate,
                    CandidateEventDetails {
                        delivery: Some(attempt.clone()),
                        ..CandidateEventDetails::default()
                    },
                )?;
                minor_notification = Some(attempt);
            }
            self.save()?;
            Ok(ReleaseOutcome {
                release,
                minor_notification,
            })
        })();
        if commit_result.is_err() {
            *self = before_commit;
            let _ = fs::remove_file(&final_path);
        }
        commit_result
    }

    pub fn retry_minor_publication_notification<N: NotificationClient>(
        &mut self,
        document_id: Uuid,
        release_id: Uuid,
        notifier: &mut N,
    ) -> Result<DeliveryAttempt> {
        let settings = self
            .notification_settings
            .clone()
            .ok_or(DmsError::NotificationSettingsRequired)?;
        let candidate = self
            .document(document_id)?
            .candidates
            .iter()
            .find(|candidate| candidate.release_id == Some(release_id))
            .cloned()
            .ok_or(DmsError::CandidateNotFound(release_id))?;
        if candidate.approval_required || candidate.status != CandidateStatus::Released {
            return Err(DmsError::InvalidLifecycleTransition(
                "only a committed minor publication notification can be retried".to_owned(),
            ));
        }
        let message = minor_publication_message(&candidate, self.document_permalink(document_id)?);
        let attempt = delivery_attempt(&settings, &message, notifier);
        self.candidate_mut(document_id, candidate.id)?
            .delivery_attempts
            .push(attempt.clone());
        self.append_candidate_event(
            document_id,
            WorkflowEventType::MinorPublicationNotified,
            &candidate,
            CandidateEventDetails {
                delivery: Some(attempt.clone()),
                ..CandidateEventDetails::default()
            },
        )?;
        self.save()?;
        Ok(attempt)
    }

    pub fn retry_decision_notification<N: NotificationClient>(
        &mut self,
        document_id: Uuid,
        candidate_id: Uuid,
        notifier: &mut N,
    ) -> Result<DeliveryAttempt> {
        let settings = self
            .notification_settings
            .clone()
            .ok_or(DmsError::NotificationSettingsRequired)?;
        let candidate = self.candidate(document_id, candidate_id)?.clone();
        let decision = match candidate.status {
            CandidateStatus::Approved => ReviewDecision::Approved,
            CandidateStatus::Rejected => ReviewDecision::Rejected,
            CandidateStatus::ChangesRequested => ReviewDecision::ChangesRequested,
            _ => {
                return Err(DmsError::InvalidLifecycleTransition(
                    "only a recorded review decision notification can be retried".to_owned(),
                ));
            }
        };
        let review_id = candidate
            .review_id
            .ok_or(DmsError::ReviewNotFound(candidate_id))?;
        let message = decision_message(
            &candidate,
            decision,
            self.review_permalink(document_id, review_id)?,
        );
        let attempt = delivery_attempt(&settings, &message, notifier);
        self.candidate_mut(document_id, candidate_id)?
            .delivery_attempts
            .push(attempt.clone());
        self.append_candidate_event(
            document_id,
            WorkflowEventType::DecisionOutcomeNotified,
            &candidate,
            CandidateEventDetails {
                delivery: Some(attempt.clone()),
                ..CandidateEventDetails::default()
            },
        )?;
        self.save()?;
        Ok(attempt)
    }

    pub fn local_lifecycle_actions(&self, document_id: Uuid) -> Result<LocalLifecycleActions> {
        let document = self.document(document_id)?;
        let source_lost = document.source_state == SourceState::Registered
            && !self.edit_root.join(&document.relative_path).is_file();
        let availability = |available, reason| LifecycleActionAvailability {
            available,
            reason: (!available).then_some(reason),
        };
        if source_lost {
            return Ok(LocalLifecycleActions {
                cancel_review: availability(
                    false,
                    "Source file is Lost source; reassociate the source first.",
                ),
                mark_obsolete: availability(
                    false,
                    "Source file is Lost source; reassociate the source first.",
                ),
            });
        }
        Ok(LocalLifecycleActions {
            cancel_review: availability(
                document.lifecycle == Lifecycle::InReview,
                "Available only while a review is open.",
            ),
            mark_obsolete: availability(
                document.lifecycle != Lifecycle::Obsolete,
                "The document is already obsolete.",
            ),
        })
    }

    /// Set Draft/Released from the current draft digest versus the latest
    /// non-withdrawn release. Never-released registered documents are Draft.
    /// Matching digest keeps Released; mismatch becomes Draft. Open review,
    /// approved candidates, and obsolete stay unchanged.
    pub fn sync_lifecycle_from_source(&mut self, document_id: Uuid) -> Result<bool> {
        let document = self.document(document_id)?;
        if matches!(
            document.lifecycle,
            Lifecycle::Obsolete | Lifecycle::InReview | Lifecycle::Approved
        ) {
            return Ok(false);
        }
        if document.active_candidate_id.is_some() {
            if document.lifecycle != Lifecycle::Draft {
                self.documents
                    .get_mut(&document_id)
                    .expect("document checked above")
                    .lifecycle = Lifecycle::Draft;
                self.sync_markdown_control_frontmatter(document_id)?;
                return Ok(true);
            }
            return Ok(false);
        }
        let desired = self.lifecycle_from_source_digest(document_id)?;
        if document.lifecycle == desired {
            return Ok(false);
        }
        self.documents
            .get_mut(&document_id)
            .expect("document checked above")
            .lifecycle = desired;
        self.sync_markdown_control_frontmatter(document_id)?;
        Ok(true)
    }

    /// Reconcile every registered document. Returns how many lifecycles changed.
    pub fn sync_all_registered_lifecycles(&mut self) -> Result<usize> {
        let ids: Vec<Uuid> = self
            .documents
            .values()
            .filter(|document| document.source_state == SourceState::Registered)
            .map(|document| document.id)
            .collect();
        let mut changed = 0;
        for document_id in ids {
            if self.sync_lifecycle_from_source(document_id)? {
                changed += 1;
            }
        }
        Ok(changed)
    }

    fn lifecycle_from_source_digest(&self, document_id: Uuid) -> Result<Lifecycle> {
        let Some(release) = self.current_release(document_id)? else {
            return Ok(Lifecycle::Draft);
        };
        let document = self.document(document_id)?;
        let source_path = self.edit_root.join(&document.relative_path);
        if !source_path.is_file() {
            // Missing source keeps Released when a current release exists.
            return Ok(Lifecycle::Released);
        }
        let digest = sha256_file(&source_path)?;
        if digest == release.source_digest {
            Ok(Lifecycle::Released)
        } else {
            Ok(Lifecycle::Draft)
        }
    }

    pub fn withdraw_release(
        &mut self,
        document_id: Uuid,
        release_id: Uuid,
        reason: &str,
    ) -> Result<ReleaseRecord> {
        let reason = validate_comment(reason, true)?;
        let release = self
            .document(document_id)?
            .releases
            .iter()
            .find(|release| release.id == release_id)
            .cloned()
            .ok_or(DmsError::ReleaseNotFound(release_id))?;
        if release.withdrawn {
            return Err(DmsError::InvalidLifecycleTransition(
                "release is already withdrawn".to_owned(),
            ));
        }
        let body = WorkflowEventBody {
            event_id: Uuid::new_v4(),
            document_id,
            event_type: WorkflowEventType::ReleaseWithdrawn,
            predecessor_hash: self
                .document(document_id)?
                .workflow_events
                .last()
                .map(|event| event.event_hash.clone()),
            timestamp: Utc::now(),
            requester: None,
            editor: Some(release.editor.clone()),
            approver: Some(release.approver.clone()),
            authenticated_actor: None,
            local_os_user: default_author(),
            revision_digest: Some(release.source_digest.clone()),
            confidentiality: Some(release.confidentiality.clone()),
            target_version: Some(release.version),
            target_mode: Some(release.mode),
            changelog: None,
            assistance: None,
            decision_comment: None,
            operator_comment: Some(reason),
            delivery: None,
            content_override: None,
            pdf_digest: Some(release.pdf_digest.clone()),
            periodic_review: None,
            control_change: None,
            report: None,
            source_reassociation: None,
        };
        self.append_event(document_id, body)?;
        let stored = self
            .documents
            .get_mut(&document_id)
            .expect("document checked above")
            .releases
            .iter_mut()
            .find(|stored| stored.id == release_id)
            .expect("release checked above");
        stored.withdrawn = true;
        Ok(stored.clone())
    }

    pub fn cancel_review(&mut self, document_id: Uuid, reason: &str) -> Result<()> {
        let reason = validate_comment(reason, true)?;
        let candidate_id = self.active_candidate_id(document_id)?;
        let candidate = self.candidate(document_id, candidate_id)?.clone();
        if candidate.status != CandidateStatus::InReview {
            return Err(DmsError::InvalidLifecycleTransition(
                "document is not in review".to_owned(),
            ));
        }
        self.append_candidate_event(
            document_id,
            WorkflowEventType::ReviewCancelled,
            &candidate,
            CandidateEventDetails {
                operator_comment: Some(reason),
                ..CandidateEventDetails::default()
            },
        )?;
        self.candidate_mut(document_id, candidate_id)?.status = CandidateStatus::Cancelled;
        let document = self
            .documents
            .get_mut(&document_id)
            .expect("document checked above");
        document.lifecycle = Lifecycle::Draft;
        document.active_candidate_id = None;
        self.sync_markdown_control_frontmatter(document_id)?;
        Ok(())
    }

    pub fn mark_obsolete(&mut self, document_id: Uuid, reason: &str) -> Result<()> {
        let reason = validate_comment(reason, true)?;
        if self.document(document_id)?.lifecycle == Lifecycle::Obsolete {
            return Err(DmsError::InvalidLifecycleTransition(
                "document is already obsolete".to_owned(),
            ));
        }
        self.append_simple_event(
            document_id,
            WorkflowEventType::DocumentObsoleted,
            Some(reason),
        )?;
        let document = self
            .documents
            .get_mut(&document_id)
            .expect("document checked above");
        if let Some(candidate_id) = document.active_candidate_id.take() {
            if let Some(candidate) = document
                .candidates
                .iter_mut()
                .find(|candidate| candidate.id == candidate_id)
            {
                candidate.status = CandidateStatus::Cancelled;
            }
        }
        document.lifecycle = Lifecycle::Obsolete;
        Ok(())
    }

    pub fn candidates(&self, document_id: Uuid) -> Result<Vec<&ReleaseCandidate>> {
        let document = self.document(document_id)?;
        let mut candidates = document.candidates.iter().collect::<Vec<_>>();
        candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.created_at));
        Ok(candidates)
    }

    pub fn active_candidate(&self, document_id: Uuid) -> Result<Option<&ReleaseCandidate>> {
        let document = self.document(document_id)?;
        document
            .active_candidate_id
            .map(|candidate_id| {
                document
                    .candidates
                    .iter()
                    .find(|candidate| candidate.id == candidate_id)
                    .ok_or(DmsError::CandidateNotFound(candidate_id))
            })
            .transpose()
    }

    pub fn releases(&self, document_id: Uuid) -> Result<Vec<&ReleaseRecord>> {
        let document = self.document(document_id)?;
        let mut releases = document.releases.iter().collect::<Vec<_>>();
        releases.sort_by_key(|release| std::cmp::Reverse(release.released_at));
        Ok(releases)
    }

    pub fn workflow_history(&self, document_id: Uuid) -> Result<Vec<&WorkflowEvent>> {
        let document = self.document(document_id)?;
        Ok(document.workflow_events.iter().rev().collect())
    }

    pub fn verify_workflow(&self, document_id: Uuid) -> Result<WorkflowVerification> {
        let events = &self.document(document_id)?.workflow_events;
        if events.is_empty() {
            return Ok(WorkflowVerification::Missing);
        }
        let mut predecessor = None;
        for event in events {
            if event.body.predecessor_hash != predecessor
                || hash_event_body(&event.body)? != event.event_hash
            {
                return Ok(WorkflowVerification::TamperedAt(event.body.event_id));
            }
            predecessor = Some(event.event_hash.clone());
        }
        Ok(WorkflowVerification::Valid)
    }

    pub fn review_permalink(&self, document_id: Uuid, review_id: Uuid) -> Result<String> {
        self.document(document_id)?;
        Ok(format!(
            "dms://open?workspace={}&document={document_id}&target=review&review={review_id}",
            self.workspace_id
        ))
    }

    pub fn resolve_permalink(&self, uri: &str) -> Result<ResolvedPermalink> {
        let query = uri
            .strip_prefix("dms://open?")
            .ok_or(DmsError::InvalidPermalink)?;
        let values = query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .collect::<std::collections::BTreeMap<_, _>>();
        let workspace_id = values
            .get("workspace")
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or(DmsError::InvalidPermalink)?;
        if workspace_id != self.workspace_id {
            return Err(DmsError::PermalinkWorkspaceMismatch(workspace_id));
        }
        let document_id = values
            .get("document")
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or(DmsError::InvalidPermalink)?;
        let document = self.document(document_id)?;
        let target = match values.get("target").copied() {
            Some("review") => PermalinkTarget::Review,
            Some("notes") => PermalinkTarget::Notes,
            _ => PermalinkTarget::Document,
        };
        let review_id = if target == PermalinkTarget::Review {
            let review_id = values
                .get("review")
                .and_then(|value| Uuid::parse_str(value).ok())
                .ok_or(DmsError::InvalidPermalink)?;
            if !document
                .candidates
                .iter()
                .any(|candidate| candidate.review_id == Some(review_id))
                && !document
                    .periodic_reviews
                    .iter()
                    .any(|review| review.id == review_id)
            {
                return Err(DmsError::ReviewNotFound(review_id));
            }
            Some(review_id)
        } else {
            None
        };
        Ok(ResolvedPermalink {
            document_id,
            target,
            review_id,
        })
    }

    pub(crate) fn invalidate_stale_candidates(&mut self) {
        let document_ids = self.documents.keys().copied().collect::<Vec<_>>();
        for document_id in document_ids {
            let Some(candidate_id) = self
                .documents
                .get(&document_id)
                .and_then(|document| document.active_candidate_id)
            else {
                continue;
            };
            let stale = self
                .candidate(document_id, candidate_id)
                .and_then(|candidate| {
                    self.ensure_candidate_metadata_current(
                        document_id,
                        candidate,
                        candidate.metadata.approver.tenant_id,
                    )
                })
                .is_err();
            if stale {
                self.invalidate_candidate(
                    document_id,
                    candidate_id,
                    "effective workflow metadata changed",
                )
                .expect("active candidate invalidation has serializable evidence");
            }
        }
    }

    fn invalidate_candidate(
        &mut self,
        document_id: Uuid,
        candidate_id: Uuid,
        reason: &str,
    ) -> Result<()> {
        let candidate = self.candidate(document_id, candidate_id)?.clone();
        self.append_candidate_event(
            document_id,
            WorkflowEventType::CandidateInvalidated,
            &candidate,
            CandidateEventDetails {
                operator_comment: Some(reason.to_owned()),
                ..CandidateEventDetails::default()
            },
        )?;
        self.candidate_mut(document_id, candidate_id)?.status = CandidateStatus::Invalidated;
        let document = self
            .documents
            .get_mut(&document_id)
            .expect("candidate document exists");
        document.lifecycle = Lifecycle::Draft;
        document.active_candidate_id = None;
        Ok(())
    }

    pub(crate) fn validate_lifecycle_records(&self) -> Result<()> {
        let mut candidate_ids = std::collections::BTreeSet::new();
        let mut review_ids = std::collections::BTreeSet::new();
        let mut release_ids = std::collections::BTreeSet::new();
        for document in self.documents.values() {
            for candidate in &document.candidates {
                if !candidate_ids.insert(candidate.id) {
                    return Err(DmsError::LifecycleIntegrity(format!(
                        "duplicate candidate ID {}",
                        candidate.id
                    )));
                }
                if let Some(review_id) = candidate.review_id {
                    if !review_ids.insert(review_id) {
                        return Err(DmsError::LifecycleIntegrity(format!(
                            "duplicate review ID {review_id}"
                        )));
                    }
                }
            }
            if let Some(active_id) = document.active_candidate_id {
                if !document
                    .candidates
                    .iter()
                    .any(|candidate| candidate.id == active_id)
                {
                    return Err(DmsError::LifecycleIntegrity(format!(
                        "active candidate {active_id} is missing"
                    )));
                }
            }
            for release in &document.releases {
                if !release_ids.insert(release.id) {
                    return Err(DmsError::LifecycleIntegrity(format!(
                        "duplicate release ID {}",
                        release.id
                    )));
                }
                let withdrawal_events = document
                    .workflow_events
                    .iter()
                    .filter(|event| {
                        event.body.event_type == WorkflowEventType::ReleaseWithdrawn
                            && event.body.target_version == Some(release.version)
                            && event.body.pdf_digest.as_deref() == Some(&release.pdf_digest)
                    })
                    .count();
                if withdrawal_events != usize::from(release.withdrawn) {
                    return Err(DmsError::LifecycleIntegrity(format!(
                        "release {} withdrawal state does not match canonical evidence",
                        release.id
                    )));
                }
            }
            if document
                .workflow_events
                .iter()
                .any(|event| event.body.document_id != document.id)
            {
                return Err(DmsError::LifecycleIntegrity(format!(
                    "workflow event belongs to another document than {}",
                    document.id
                )));
            }
            if matches!(
                self.verify_workflow(document.id)?,
                WorkflowVerification::TamperedAt(_)
            ) {
                return Err(DmsError::LifecycleIntegrity(format!(
                    "workflow event chain for {} is invalid",
                    document.id
                )));
            }
        }
        self.validate_workspace_events()?;
        Ok(())
    }

    fn ensure_document_can_start_candidate(&mut self, document_id: Uuid) -> Result<()> {
        self.sync_lifecycle_from_source(document_id)?;
        let document = self.document(document_id)?;
        if document.source_state != SourceState::Registered {
            return Err(DmsError::InvalidLifecycleTransition(
                "document is not registered".to_owned(),
            ));
        }
        if !self.edit_root.join(&document.relative_path).is_file() {
            return Err(DmsError::InvalidLifecycleTransition(
                "source file is Lost source; reassociate the source first".to_owned(),
            ));
        }
        if document.lifecycle != Lifecycle::Draft || document.active_candidate_id.is_some() {
            return Err(DmsError::InvalidLifecycleTransition(
                "document must be an idle draft".to_owned(),
            ));
        }
        Ok(())
    }

    pub(crate) fn resolve_target_version(
        &self,
        document_id: Uuid,
        selection: TargetSelection,
    ) -> Result<Version> {
        let latest_committed = self
            .document(document_id)?
            .releases
            .iter()
            .map(|release| release.version)
            .max();
        let target = match (latest_committed, selection) {
            (None, TargetSelection::NextMinor | TargetSelection::NextMajor) => Version::V1_0,
            (None, TargetSelection::Manual(version)) if version == Version::V1_0 => version,
            (None, TargetSelection::Manual(_)) => return Err(DmsError::InvalidTargetVersion),
            (Some(current), TargetSelection::NextMinor) => Version {
                major: current.major,
                minor: current
                    .minor
                    .checked_add(1)
                    .ok_or(DmsError::InvalidTargetVersion)?,
            },
            (Some(current), TargetSelection::NextMajor) => Version {
                major: current
                    .major
                    .checked_add(1)
                    .ok_or(DmsError::InvalidTargetVersion)?,
                minor: 0,
            },
            (Some(current), TargetSelection::Manual(version)) if version > current => version,
            (Some(_), TargetSelection::Manual(_)) => return Err(DmsError::InvalidTargetVersion),
        };
        if self
            .document(document_id)?
            .releases
            .iter()
            .any(|release| release.version == target)
        {
            return Err(DmsError::VersionAlreadyReleased(target.to_string()));
        }
        Ok(target)
    }

    pub fn current_release(&self, document_id: Uuid) -> Result<Option<&ReleaseRecord>> {
        Ok(self
            .document(document_id)?
            .releases
            .iter()
            .filter(|release| !release.withdrawn)
            .max_by_key(|release| release.version))
    }

    fn person_snapshot(&self, object_id: Uuid, tenant_id: Uuid) -> Result<PersonSnapshot> {
        let person = self
            .identity_cache
            .get(&object_id)
            .filter(|person| person.account_enabled)
            .ok_or(DmsError::IneligibleEntraPerson(object_id))?;
        Ok(PersonSnapshot {
            tenant_id,
            object_id,
            display_name: person.display_name.clone(),
            email: person.email.clone(),
        })
    }

    fn role_snapshot(
        &self,
        role: Option<EffectiveWorkflowRole>,
        tenant_id: Uuid,
    ) -> Result<PersonSnapshot> {
        let role = role.ok_or(DmsError::UnresolvedWorkflowRole)?;
        if role.state != ResolutionState::Resolved {
            return Err(DmsError::UnresolvedWorkflowRole);
        }
        self.person_snapshot(role.object_id, tenant_id)
    }

    fn active_candidate_id(&self, document_id: Uuid) -> Result<Uuid> {
        self.document(document_id)?
            .active_candidate_id
            .ok_or(DmsError::NoActiveCandidate)
    }

    fn candidate(&self, document_id: Uuid, candidate_id: Uuid) -> Result<&ReleaseCandidate> {
        self.document(document_id)?
            .candidates
            .iter()
            .find(|candidate| candidate.id == candidate_id)
            .ok_or(DmsError::CandidateNotFound(candidate_id))
    }

    fn candidate_mut(
        &mut self,
        document_id: Uuid,
        candidate_id: Uuid,
    ) -> Result<&mut ReleaseCandidate> {
        self.documents
            .get_mut(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?
            .candidates
            .iter_mut()
            .find(|candidate| candidate.id == candidate_id)
            .ok_or(DmsError::CandidateNotFound(candidate_id))
    }

    fn ensure_candidate_metadata_current(
        &self,
        document_id: Uuid,
        candidate: &ReleaseCandidate,
        tenant_id: Uuid,
    ) -> Result<()> {
        let document = self.document(document_id)?;
        let mut control = document.control.clone();
        if let Some(owner) = candidate.staged_owner {
            self.require_eligible_owner(owner)?;
            control.owner = Some(owner);
            control.legacy_owner_label = None;
        }
        let confidentiality = self.effective_confidentiality(document_id)?;
        let roles = self.effective_workflow_roles(document_id)?;
        let editor = match candidate.staged_editor {
            Some(editor) => self.person_snapshot(editor.object_id, tenant_id)?,
            None => self.role_snapshot(roles.editor, tenant_id)?,
        };
        let approver = self.role_snapshot(roles.approver, tenant_id)?;
        let owner = self.person_snapshot(
            control
                .owner
                .ok_or_else(|| DmsError::InvalidConfiguration("owner".to_owned()))?
                .object_id,
            tenant_id,
        )?;
        let same_identity = |current: &PersonSnapshot, snapshot: &PersonSnapshot| {
            current.tenant_id == snapshot.tenant_id && current.object_id == snapshot.object_id
        };
        if control != candidate.metadata.control
            || confidentiality.type_id != candidate.metadata.confidentiality.type_id
            || confidentiality.label != candidate.metadata.confidentiality.label
            || candidate
                .metadata
                .owner
                .as_ref()
                .is_none_or(|snapshot| !same_identity(&owner, snapshot))
            || !same_identity(&editor, &candidate.metadata.editor)
            || !same_identity(&approver, &candidate.metadata.approver)
        {
            return Err(DmsError::CandidateInvalidated);
        }
        Ok(())
    }

    fn ensure_candidate_current(
        &self,
        document_id: Uuid,
        candidate: &ReleaseCandidate,
        tenant_id: Uuid,
    ) -> Result<()> {
        self.ensure_candidate_metadata_current(document_id, candidate, tenant_id)?;
        let source_path = self
            .edit_root
            .join(&self.document(document_id)?.relative_path);
        if sha256_file(&source_path)? != candidate.source_digest {
            return Err(DmsError::CandidateInvalidated);
        }
        Ok(())
    }

    fn accept_or_reject_content_check(
        &mut self,
        document_id: Uuid,
        candidate: &mut ReleaseCandidate,
        phase: CheckPhase,
        check: ContentCheck,
        reason: Option<&str>,
    ) -> Result<()> {
        if check.passes() {
            return Ok(());
        }
        let reason = reason
            .map(|value| validate_comment(value, true))
            .transpose()?
            .ok_or_else(|| DmsError::ContentConformanceFailed(Box::new(check.clone())))?;
        let evidence = ContentOverride {
            phase,
            reason,
            draft_digest: candidate.source_digest.clone(),
            version: candidate.version,
            confidentiality: candidate.metadata.confidentiality.clone(),
            check,
        };
        candidate.content_overrides.push(evidence.clone());
        self.append_candidate_event(
            document_id,
            WorkflowEventType::ContentConformanceOverridden,
            candidate,
            CandidateEventDetails::default(),
        )?;
        let event = self
            .documents
            .get_mut(&document_id)
            .expect("document checked above")
            .workflow_events
            .last_mut()
            .expect("event just appended");
        event.body.content_override = Some(evidence);
        event.event_hash = hash_event_body(&event.body)?;
        Ok(())
    }

    fn append_candidate_event(
        &mut self,
        document_id: Uuid,
        event_type: WorkflowEventType,
        candidate: &ReleaseCandidate,
        details: CandidateEventDetails,
    ) -> Result<WorkflowEvent> {
        let body = WorkflowEventBody {
            event_id: Uuid::new_v4(),
            document_id,
            event_type,
            predecessor_hash: self
                .document(document_id)?
                .workflow_events
                .last()
                .map(|event| event.event_hash.clone()),
            timestamp: Utc::now(),
            requester: Some(candidate.requester.clone()),
            editor: Some(candidate.metadata.editor.clone()),
            approver: Some(candidate.metadata.approver.clone()),
            authenticated_actor: details.actor,
            local_os_user: default_author(),
            revision_digest: Some(candidate.source_digest.clone()),
            confidentiality: Some(candidate.metadata.confidentiality.clone()),
            target_version: Some(candidate.version),
            target_mode: Some(candidate.mode),
            changelog: Some(candidate.changelog.clone()),
            assistance: candidate.assistance.clone(),
            decision_comment: details.decision_comment,
            operator_comment: details.operator_comment,
            delivery: details.delivery,
            content_override: None,
            pdf_digest: details.pdf_digest,
            periodic_review: None,
            control_change: None,
            report: None,
            source_reassociation: None,
        };
        self.append_event(document_id, body)
    }

    fn append_simple_event(
        &mut self,
        document_id: Uuid,
        event_type: WorkflowEventType,
        operator_comment: Option<String>,
    ) -> Result<WorkflowEvent> {
        let body = WorkflowEventBody {
            event_id: Uuid::new_v4(),
            document_id,
            event_type,
            predecessor_hash: self
                .document(document_id)?
                .workflow_events
                .last()
                .map(|event| event.event_hash.clone()),
            timestamp: Utc::now(),
            requester: None,
            editor: None,
            approver: None,
            authenticated_actor: None,
            local_os_user: default_author(),
            revision_digest: None,
            confidentiality: None,
            target_version: None,
            target_mode: None,
            changelog: None,
            assistance: None,
            decision_comment: None,
            operator_comment,
            delivery: None,
            content_override: None,
            pdf_digest: None,
            periodic_review: None,
            control_change: None,
            report: None,
            source_reassociation: None,
        };
        self.append_event(document_id, body)
    }

    pub(crate) fn append_control_change_event(
        &mut self,
        document_id: Uuid,
        before: DocumentControl,
        after: DocumentControl,
    ) -> Result<WorkflowEvent> {
        let body = WorkflowEventBody {
            event_id: Uuid::new_v4(),
            document_id,
            event_type: WorkflowEventType::DocumentControlDataChanged,
            predecessor_hash: self
                .document(document_id)?
                .workflow_events
                .last()
                .map(|event| event.event_hash.clone()),
            timestamp: Utc::now(),
            requester: None,
            editor: None,
            approver: None,
            authenticated_actor: None,
            local_os_user: default_author(),
            revision_digest: None,
            confidentiality: None,
            target_version: None,
            target_mode: None,
            changelog: None,
            assistance: None,
            decision_comment: None,
            operator_comment: None,
            delivery: None,
            content_override: None,
            pdf_digest: None,
            periodic_review: None,
            control_change: Some(DocumentControlChange { before, after }),
            report: None,
            source_reassociation: None,
        };
        self.append_event(document_id, body)
    }

    pub(crate) fn append_event(
        &mut self,
        document_id: Uuid,
        body: WorkflowEventBody,
    ) -> Result<WorkflowEvent> {
        let event = WorkflowEvent {
            event_hash: hash_event_body(&body)?,
            body,
        };
        self.documents
            .get_mut(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?
            .workflow_events
            .push(event.clone());
        Ok(event)
    }

    fn review_message(&self, document_id: Uuid, candidate_id: Uuid) -> Result<NotificationMessage> {
        let candidate = self.candidate(document_id, candidate_id)?;
        let review_id = candidate.review_id.ok_or(DmsError::NoActiveReview)?;
        Ok(review_request_message(
            candidate,
            self.review_permalink(document_id, review_id)?,
        ))
    }
}

pub fn scan_content_markers(
    source_path: &Path,
    version: Version,
    confidentiality_value: &str,
) -> Result<ContentCheck> {
    scan_content_conformance(
        source_path,
        version,
        confidentiality_value,
        confidentiality_value,
        None,
        None,
    )
}

fn scan_content_conformance(
    source_path: &Path,
    version: Version,
    confidentiality_type_id: &str,
    confidentiality_label: &str,
    title: Option<&str>,
    document_number: Option<Option<&str>>,
) -> Result<ContentCheck> {
    let extension = source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| DmsError::UnsupportedContentScanner(source_path.to_path_buf()))?;
    if extension == "md" {
        let markdown = fs::read_to_string(source_path).map_err(|source| DmsError::Io {
            path: source_path.to_path_buf(),
            source,
        })?;
        let check = super::check_markdown_frontmatter(
            &markdown,
            title.unwrap_or_default(),
            document_number.flatten(),
            &format!("{}.{}", version.major, version.minor),
            confidentiality_type_id,
        )?;
        return Ok(ContentCheck {
            version: check.version,
            confidentiality: check.confidentiality,
            title: title.and(check.title),
            document_number: document_number.and(check.document_number),
        });
    }
    let sections = match extension.as_str() {
        "docx" => docx_visible_text(source_path)?,
        _ => {
            return Err(DmsError::UnsupportedContentScanner(
                source_path.to_path_buf(),
            ))
        }
    };
    let mut versions = Vec::new();
    let mut version_locations = Vec::new();
    let mut confidentialities = Vec::new();
    let mut confidentiality_locations = Vec::new();
    for (location, text) in sections {
        for (line_index, line) in text.lines().enumerate() {
            if let Some(value) = marker_value(line, "version") {
                versions.push(value);
                version_locations.push(format!("{location}:{}", line_index + 1));
            }
            if let Some(value) = marker_value(line, "vertraulichkeitsstufe") {
                confidentialities.push(value);
                confidentiality_locations.push(format!("{location}:{}", line_index + 1));
            }
        }
    }
    Ok(ContentCheck {
        version: marker_verdict(
            format!("{}.{}", version.major, version.minor),
            versions,
            version_locations,
        ),
        confidentiality: marker_verdict(
            normalize_whitespace(confidentiality_label),
            confidentialities,
            confidentiality_locations,
        ),
        title: None,
        document_number: None,
    })
}

pub(crate) fn visible_source_text(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "md" => markdown_visible_text(path),
        "docx" => Ok(docx_visible_text(path)?
            .into_iter()
            .map(|(_, text)| text)
            .collect::<Vec<_>>()
            .join("\n")),
        _ => Err(DmsError::UnsupportedContentScanner(path.to_path_buf())),
    }
}

fn markdown_visible_text(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut output = String::new();
    let mut in_front_matter = content.starts_with("---\n") || content.starts_with("---\r\n");
    let mut front_matter_closed = !in_front_matter;
    let mut in_fence = false;
    let mut in_comment = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if in_front_matter {
            if trimmed == "---" && !output.is_empty() {
                in_front_matter = false;
                front_matter_closed = true;
            } else if trimmed == "---" && output.is_empty() {
                output.push('\n');
            }
            continue;
        }
        if !front_matter_closed {
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || line.starts_with("    ") || line.starts_with('\t') {
            continue;
        }
        let mut visible = String::new();
        let mut rest = line;
        loop {
            if in_comment {
                if let Some(end) = rest.find("-->") {
                    rest = &rest[end + 3..];
                    in_comment = false;
                    continue;
                }
                break;
            }
            if let Some(start) = rest.find("<!--") {
                visible.push_str(&rest[..start]);
                rest = &rest[start + 4..];
                in_comment = true;
                continue;
            }
            visible.push_str(rest);
            break;
        }
        output.push_str(&visible);
        output.push('\n');
    }
    Ok(output)
}

fn docx_visible_text(path: &Path) -> Result<Vec<(String, String)>> {
    let bytes = fs::read(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
    let mut sections = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
        let name = file.name().to_owned();
        let relevant = name == "word/document.xml"
            || (name.starts_with("word/header") && name.ends_with(".xml"))
            || (name.starts_with("word/footer") && name.ends_with(".xml"));
        if !relevant {
            continue;
        }
        let mut xml = String::new();
        file.read_to_string(&mut xml)
            .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
        sections.push((name, visible_xml_text(&xml)?));
    }
    if sections.is_empty() {
        return Err(DmsError::InvalidDocx(
            "DOCX has no document, header, or footer XML".to_owned(),
        ));
    }
    Ok(sections)
}

fn visible_xml_text(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut output = String::new();
    loop {
        match reader.read_event() {
            Ok(XmlEvent::Text(text)) => {
                let text = text
                    .unescape()
                    .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
                output.push_str(&text);
            }
            Ok(XmlEvent::End(end)) if end.name().as_ref() == b"w:p" => output.push('\n'),
            Ok(XmlEvent::Empty(empty)) if matches!(empty.name().as_ref(), b"w:br" | b"w:tab") => {
                output.push(' ')
            }
            Ok(XmlEvent::Eof) => break,
            Err(error) => return Err(DmsError::InvalidDocx(error.to_string())),
            _ => {}
        }
    }
    Ok(output)
}

fn marker_value(line: &str, caption: &str) -> Option<String> {
    let rendered = line.replace(['*', '_', '`'], "");
    let normalized = normalize_whitespace(&rendered);
    let lower = normalized.to_lowercase();
    let caption = caption.to_lowercase();
    let start = lower.find(&caption)? + caption.len();
    let after = normalized.get(start..)?.trim_start();
    let after = after.strip_prefix(':')?.trim();
    let lower_after = after.to_lowercase();
    let end = [" version:", " vertraulichkeitsstufe:"]
        .iter()
        .filter_map(|next| lower_after.find(next))
        .min()
        .unwrap_or(after.len());
    Some(normalize_whitespace(&after[..end]))
}

fn marker_verdict(
    expected: String,
    detected: Vec<String>,
    locations: Vec<String>,
) -> MarkerVerdict {
    let status = if detected.is_empty() {
        MarkerStatus::Missing
    } else if detected.iter().all(|value| value == &expected) {
        MarkerStatus::Match
    } else if detected.windows(2).any(|pair| pair[0] != pair[1]) {
        MarkerStatus::Conflicting
    } else {
        MarkerStatus::Mismatch
    };
    MarkerVerdict {
        status,
        expected,
        detected,
        locations,
    }
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn review_request_message(candidate: &ReleaseCandidate, permalink: String) -> NotificationMessage {
    let subject = format!(
        "[{}] DMS review requested — {} — {}",
        candidate.metadata.confidentiality.label,
        candidate.metadata.control.title,
        candidate.version
    );
    let (body, html_body) = notification_bodies(
        &[
            "A review decision is requested.",
            "",
            "Action: Review and decide",
            &format!("Title: {}", candidate.metadata.control.title),
            &format!("Document: {}", path_text(&candidate.source_path)),
            &format!("Requested by: {}", candidate.requester.display_name),
            &format!("Target version: {}", candidate.version),
            &format!(
                "Confidentiality: {}",
                candidate.metadata.confidentiality.label
            ),
            "",
            "Open review task:",
        ],
        &permalink,
    );
    notification_message(
        NotificationKind::ReviewRequest,
        candidate.metadata.approver.email.clone(),
        subject,
        body,
        html_body,
    )
}

fn decision_message(
    candidate: &ReleaseCandidate,
    decision: ReviewDecision,
    permalink: String,
) -> NotificationMessage {
    let outcome = match decision {
        ReviewDecision::Approved => "approved",
        ReviewDecision::Rejected => "rejected",
        ReviewDecision::ChangesRequested => "changes requested",
    };
    let subject = format!(
        "[{}] DMS review {} — {} — {}",
        candidate.metadata.confidentiality.label,
        outcome,
        candidate.metadata.control.title,
        candidate.version
    );
    let (body, html_body) = notification_bodies(
        &[
            "A review decision was recorded.",
            "",
            &format!("Title: {}", candidate.metadata.control.title),
            &format!("Document: {}", path_text(&candidate.source_path)),
            &format!("Decision: {outcome}"),
            &format!("Target version: {}", candidate.version),
            &format!(
                "Confidentiality: {}",
                candidate.metadata.confidentiality.label
            ),
            "",
            "Open review detail:",
        ],
        &permalink,
    );
    notification_message(
        NotificationKind::DecisionOutcome,
        candidate.requester.email.clone(),
        subject,
        body,
        html_body,
    )
}

fn minor_publication_message(
    candidate: &ReleaseCandidate,
    permalink: String,
) -> NotificationMessage {
    let subject = format!(
        "[{}] DMS minor version released — {} — {}",
        candidate.metadata.confidentiality.label,
        candidate.metadata.control.title,
        candidate.version
    );
    let (body, html_body) = notification_bodies(
        &[
            "A new minor version of your assigned document has been released.",
            "",
            &format!("Title: {}", candidate.metadata.control.title),
            &format!("Document: {}", path_text(&candidate.source_path)),
            &format!("Released by: {}", candidate.requester.display_name),
            &format!("Released version: {}", candidate.version),
            &format!(
                "Confidentiality: {}",
                candidate.metadata.confidentiality.label
            ),
            "",
            "Open document:",
        ],
        &permalink,
    );
    notification_message(
        NotificationKind::MinorPublication,
        candidate.metadata.approver.email.clone(),
        subject,
        body,
        html_body,
    )
}

pub(crate) fn notification_message(
    kind: NotificationKind,
    recipient: String,
    subject: String,
    body: String,
    html_body: String,
) -> NotificationMessage {
    let mailto_uri = format!(
        "mailto:{}?subject={}&body={}",
        percent_encode(&recipient),
        percent_encode(&subject),
        percent_encode(&body)
    );
    NotificationMessage {
        kind,
        recipient,
        subject,
        body,
        html_body,
        mailto_uri,
    }
}

/// Renders one notification body in both representations from the same
/// values: the canonical plain-text lines (joined with `"\n"`, followed by
/// the bare permalink line) and an HTML alternative that wraps the lines in
/// `<pre>` so the visible copy is identical, with the permalink rendered as
/// a clickable `<a>` element.
pub(crate) fn notification_bodies(lines: &[&str], permalink: &str) -> (String, String) {
    let body = format!("{}\n{}", lines.join("\n"), permalink);
    let mut html_body = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
    html_body.push_str("<meta charset=\"utf-8\">\n</head>\n<body>\n");
    for line in lines {
        if line.is_empty() {
            html_body.push_str("<br>\n");
        } else {
            html_body.push_str(&format!("<pre>{}</pre>\n", escape_html(line)));
        }
    }
    html_body.push_str(&format!(
        "<a href=\"{}\">{}</a>\n",
        escape_html(permalink),
        escape_html(permalink)
    ));
    html_body.push_str("</body>\n</html>\n");
    (body, html_body)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn delivery_attempt<N: NotificationClient + ?Sized>(
    settings: &NotificationSettings,
    message: &NotificationMessage,
    notifier: &mut N,
) -> DeliveryAttempt {
    let receipt = notifier
        .send(settings, message)
        .unwrap_or_else(|error| DeliveryReceipt {
            status: DeliveryStatus::Failed,
            response_code: None,
            detail: error,
        });
    DeliveryAttempt {
        kind: message.kind,
        recipient: message.recipient.clone(),
        transport: settings.transport,
        status: receipt.status,
        response_code: receipt.response_code,
        detail: receipt.detail,
        attempted_at: Utc::now(),
    }
}

fn delivery_advances_workflow(transport: NotificationTransport, status: DeliveryStatus) -> bool {
    matches!(
        (transport, status),
        (NotificationTransport::Smtp, DeliveryStatus::Accepted)
            | (NotificationTransport::Mailto, DeliveryStatus::Confirmed)
    )
}

fn validate_release_control(control: &DocumentControl) -> Result<()> {
    configured_text(&control.title, "title")?;
    control
        .document_type
        .as_deref()
        .ok_or_else(|| DmsError::InvalidConfiguration("document type".to_owned()))?;
    control
        .owner
        .ok_or_else(|| DmsError::InvalidConfiguration("owner".to_owned()))?;
    Ok(())
}

fn validate_comment(value: &str, required: bool) -> Result<String> {
    let value = value.trim();
    if required && value.is_empty() {
        return Err(DmsError::InvalidWorkflowComment(
            "comment cannot be empty".to_owned(),
        ));
    }
    if value.lines().any(|line| line.chars().count() > 500) {
        return Err(DmsError::InvalidWorkflowComment(
            "comment lines cannot exceed 500 characters".to_owned(),
        ));
    }
    if value.chars().any(|character| character == '\0') {
        return Err(DmsError::InvalidWorkflowComment(
            "comment cannot contain NUL".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

fn release_relative_path(
    source_path: &Path,
    version: Version,
    confidentiality_type_id: &str,
) -> Result<PathBuf> {
    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| DmsError::InvalidReleasePath(source_path.to_path_buf()))?;
    let filename = format!("{stem}_{version}_{confidentiality_type_id}.pdf");
    Ok(source_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.join(&filename))
        .unwrap_or_else(|| PathBuf::from(filename)))
}

fn validate_exported_pdf(path: &Path) -> Result<()> {
    let bytes = fs::read(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if bytes.len() < 5 || !bytes.starts_with(b"%PDF-") {
        let _ = fs::remove_file(path);
        return Err(DmsError::InvalidExportedPdf(path.to_path_buf()));
    }
    Ok(())
}

pub(crate) fn hash_event_body(body: &WorkflowEventBody) -> Result<String> {
    let canonical =
        serde_json::to_vec(body).map_err(|error| DmsError::CanonicalEvent(error.to_string()))?;
    Ok(sha256_bytes(&canonical))
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}

fn percent_encode(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~' | b'@') {
            output.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(&mut output, "%{byte:02X}").expect("writing to a string cannot fail");
        }
    }
    output
}

fn path_text(path: &Path) -> String {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}
