use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    default_author, DmsError, ReleaseVerificationStatus, Result, WorkflowEvent, WorkflowEventBody,
    WorkflowEventType, WorkflowVerification, Workspace,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditReportFormat {
    Csv,
    Pdf,
}

impl AuditReportFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Pdf => "pdf",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditReportFilter {
    #[serde(default)]
    pub document_ids: Vec<Uuid>,
    #[serde(default)]
    pub approver_object_ids: Vec<Uuid>,
    #[serde(default)]
    pub confidentiality_type_ids: Vec<String>,
    pub from: Option<DateTime<Utc>>,
    pub through: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditReportRequest {
    pub format: AuditReportFormat,
    pub relative_path: Option<PathBuf>,
    #[serde(default)]
    pub filter: AuditReportFilter,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditReportEvidence {
    pub format: AuditReportFormat,
    pub relative_path: String,
    pub filter: AuditReportFilter,
    pub sha256: String,
    pub size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditReportRecord {
    pub event_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub local_os_user: String,
    pub event_hash: String,
    pub format: AuditReportFormat,
    pub relative_path: String,
    pub filter: AuditReportFilter,
    pub sha256: String,
    pub size: u64,
}

impl AuditReportRecord {
    pub fn relative_path(&self) -> PathBuf {
        PathBuf::from(&self.relative_path)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditReportVerificationStatus {
    Match,
    Mismatch,
    MissingFile,
    InvalidEvidence,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditReportVerification {
    pub event_id: Uuid,
    pub relative_path: PathBuf,
    pub expected_sha256: String,
    pub actual_sha256: Option<String>,
    pub status: AuditReportVerificationStatus,
}

#[derive(Clone, Debug)]
struct AuditRow {
    record_type: &'static str,
    document_id: Uuid,
    title: String,
    relative_path: String,
    timestamp: Option<DateTime<Utc>>,
    event_type: String,
    actor: String,
    approver: String,
    confidentiality: String,
    version: String,
    target_mode: String,
    detail: String,
    content_digest: String,
    predecessor_hash: String,
    evidence_hash: String,
    verification: String,
}

impl Workspace {
    pub fn preview_audit_report(
        &self,
        format: AuditReportFormat,
        filter: &AuditReportFilter,
    ) -> Result<Vec<u8>> {
        let filter = normalize_filter(filter)?;
        let rows = self.audit_rows(&filter)?;
        match format {
            AuditReportFormat::Csv => Ok(render_csv(self.workspace_id, &filter, &rows)),
            AuditReportFormat::Pdf => Ok(render_pdf(self.workspace_id, &filter, &rows)),
        }
    }

    pub fn generate_audit_report(
        &mut self,
        request: AuditReportRequest,
    ) -> Result<AuditReportRecord> {
        let filter = normalize_filter(&request.filter)?;
        let bytes = self.preview_audit_report(request.format, &filter)?;
        let event_id = Uuid::new_v4();
        let generated_at = Utc::now();
        let relative_path = self.resolve_report_path(
            request.relative_path.as_deref(),
            request.format,
            generated_at,
            event_id,
        )?;
        let absolute_path = self.edit_root.join(&relative_path);
        write_report_atomically(&self.edit_root, &absolute_path, &bytes, event_id)?;

        let evidence = AuditReportEvidence {
            format: request.format,
            relative_path: portable_path(&relative_path)?,
            filter,
            sha256: digest_bytes(&bytes),
            size: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        };
        let body = WorkflowEventBody {
            event_id,
            document_id: Uuid::nil(),
            event_type: WorkflowEventType::ReportGenerated,
            predecessor_hash: self
                .workspace_events
                .last()
                .map(|event| event.event_hash.clone()),
            timestamp: generated_at,
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
            report: Some(evidence.clone()),
        };
        let event = WorkflowEvent {
            event_hash: crate::lifecycle::hash_event_body(&body)?,
            body,
        };
        self.workspace_events.push(event.clone());
        if let Err(error) = self.save() {
            self.workspace_events.pop();
            let _ = fs::remove_file(&absolute_path);
            return Err(error);
        }
        Ok(report_record(&event).expect("report event was just constructed"))
    }

    pub fn recent_reports(&self) -> Vec<AuditReportRecord> {
        self.workspace_events
            .iter()
            .rev()
            .filter_map(report_record)
            .collect()
    }

    pub fn verify_report(&self, event_id: Uuid) -> Result<AuditReportVerification> {
        let event = self
            .workspace_events
            .iter()
            .find(|event| event.body.event_id == event_id)
            .ok_or(DmsError::ReportNotFound(event_id))?;
        self.verify_report_event(event, self.verify_report_chain().is_valid())
    }

    pub fn verify_reports(&self) -> Result<Vec<AuditReportVerification>> {
        let chain_valid = self.verify_report_chain().is_valid();
        self.workspace_events
            .iter()
            .rev()
            .filter(|event| event.body.report.is_some())
            .map(|event| self.verify_report_event(event, chain_valid))
            .collect()
    }

    fn verify_report_event(
        &self,
        event: &WorkflowEvent,
        chain_valid: bool,
    ) -> Result<AuditReportVerification> {
        let evidence = event
            .body
            .report
            .as_ref()
            .ok_or(DmsError::ReportNotFound(event.body.event_id))?;
        let relative_path = PathBuf::from(&evidence.relative_path);
        let path = self.edit_root.join(&relative_path);
        let (actual_sha256, status) = if !chain_valid {
            (None, AuditReportVerificationStatus::InvalidEvidence)
        } else {
            match fs::symlink_metadata(&path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    (None, AuditReportVerificationStatus::MissingFile)
                }
                Err(source) => {
                    return Err(DmsError::Io {
                        path: path.clone(),
                        source,
                    });
                }
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                    (None, AuditReportVerificationStatus::InvalidEvidence)
                }
                Ok(_) => {
                    let actual = digest_file(&path)?;
                    let status = if actual == evidence.sha256 {
                        AuditReportVerificationStatus::Match
                    } else {
                        AuditReportVerificationStatus::Mismatch
                    };
                    (Some(actual), status)
                }
            }
        };
        Ok(AuditReportVerification {
            event_id: event.body.event_id,
            relative_path,
            expected_sha256: evidence.sha256.clone(),
            actual_sha256,
            status,
        })
    }

    pub fn verify_report_chain(&self) -> WorkflowVerification {
        verify_chain(&self.workspace_events)
    }

    pub(crate) fn validate_workspace_events(&self) -> Result<()> {
        if self.workspace_events.iter().any(|event| {
            event.body.document_id != Uuid::nil()
                || event.body.event_type != WorkflowEventType::ReportGenerated
                || event.body.report.is_none()
        }) {
            return Err(DmsError::LifecycleIntegrity(
                "workspace workflow events must be report-generated evidence".to_owned(),
            ));
        }
        for event in &self.workspace_events {
            validate_report_evidence(event.body.report.as_ref().expect("checked above"))?;
        }
        if matches!(
            self.verify_report_chain(),
            WorkflowVerification::TamperedAt(_)
        ) {
            return Err(DmsError::LifecycleIntegrity(
                "workspace report event chain is invalid".to_owned(),
            ));
        }
        Ok(())
    }

    fn resolve_report_path(
        &self,
        requested: Option<&Path>,
        format: AuditReportFormat,
        generated_at: DateTime<Utc>,
        event_id: Uuid,
    ) -> Result<PathBuf> {
        let path = requested.map(Path::to_path_buf).unwrap_or_else(|| {
            PathBuf::from(format!(
                ".dms/exports/audit-{}-{}.{}",
                generated_at.format("%Y%m%dT%H%M%S%.3fZ"),
                event_id.simple(),
                format.extension()
            ))
        });
        let relative = if path.is_absolute() {
            path.strip_prefix(&self.edit_root)
                .map_err(|_| DmsError::InvalidReportPath(path.clone()))?
                .to_path_buf()
        } else {
            path
        };
        crate::validate_relative_source_path(&relative)
            .map_err(|_| DmsError::InvalidReportPath(relative.clone()))?;
        if relative
            .extension()
            .and_then(|value| value.to_str())
            .is_none_or(|value| !value.eq_ignore_ascii_case(format.extension()))
        {
            return Err(DmsError::InvalidReportPath(relative));
        }
        Ok(relative)
    }

    fn audit_rows(&self, filter: &AuditReportFilter) -> Result<Vec<AuditRow>> {
        let mut rows = Vec::new();
        for document in self.documents() {
            if !filter.document_ids.is_empty() && !filter.document_ids.contains(&document.id) {
                continue;
            }
            let effective = self.effective_confidentiality(document.id)?;
            let workflow_verification =
                workflow_verification_text(self.verify_workflow(document.id)?);
            if matches_confidentiality(filter, &effective.type_id) {
                rows.push(AuditRow {
                    record_type: "classification",
                    document_id: document.id,
                    title: document.control.title.clone(),
                    relative_path: portable_path(&document.relative_path)?,
                    timestamp: None,
                    event_type: format!("{:?}", document.lifecycle).to_ascii_lowercase(),
                    actor: String::new(),
                    approver: String::new(),
                    confidentiality: effective.type_id.clone(),
                    version: String::new(),
                    target_mode: String::new(),
                    detail: format!(
                        "classification={} ({}) source={} override={}",
                        effective.type_id,
                        effective.label,
                        effective.source_folder,
                        effective.document_override
                    ),
                    content_digest: String::new(),
                    predecessor_hash: String::new(),
                    evidence_hash: String::new(),
                    verification: workflow_verification.clone(),
                });
            }

            for event in &document.workflow_events {
                if !matches_timestamp(filter, event.body.timestamp)
                    || !matches_approver(
                        filter,
                        event.body.approver.as_ref().map(|person| person.object_id),
                    )
                    || !matches_confidentiality(
                        filter,
                        event
                            .body
                            .confidentiality
                            .as_ref()
                            .map_or("", |value| &value.type_id),
                    )
                {
                    continue;
                }
                let mut details = [
                    event.body.changelog.clone(),
                    event.body.decision_comment.clone(),
                    event.body.operator_comment.clone(),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
                if let Some(delivery) = &event.body.delivery {
                    details.push(format!(
                        "delivery_status={} delivery_detail={}",
                        delivery_status_text(delivery.status),
                        delivery.detail
                    ));
                }
                if let Some(review) = &event.body.periodic_review {
                    details.push(format!(
                        "review_id={} release_id={} result={}",
                        review.review_id,
                        review.release_id,
                        review.result.map(periodic_result_text).unwrap_or("pending")
                    ));
                }
                rows.push(AuditRow {
                    record_type: "workflow_event",
                    document_id: document.id,
                    title: document.control.title.clone(),
                    relative_path: portable_path(&document.relative_path)?,
                    timestamp: Some(event.body.timestamp),
                    event_type: event_type_text(event.body.event_type).to_owned(),
                    actor: event.body.authenticated_actor.as_ref().map_or_else(
                        || event.body.local_os_user.clone(),
                        |actor| actor.object_id.to_string(),
                    ),
                    approver: event
                        .body
                        .approver
                        .as_ref()
                        .map_or_else(String::new, |person| person.object_id.to_string()),
                    confidentiality: event
                        .body
                        .confidentiality
                        .as_ref()
                        .map_or_else(String::new, |value| value.type_id.clone()),
                    version: event
                        .body
                        .target_version
                        .map_or_else(String::new, |version| version.to_string()),
                    target_mode: event
                        .body
                        .target_mode
                        .map_or_else(String::new, |mode| target_mode_text(mode).to_owned()),
                    detail: details.join(" | "),
                    content_digest: event
                        .body
                        .pdf_digest
                        .clone()
                        .or_else(|| event.body.revision_digest.clone())
                        .unwrap_or_default(),
                    predecessor_hash: event.body.predecessor_hash.clone().unwrap_or_default(),
                    evidence_hash: event.event_hash.clone(),
                    verification: workflow_verification.clone(),
                });
            }

            for release in &document.releases {
                if !matches_timestamp(filter, release.released_at)
                    || !matches_approver(filter, Some(release.approver.object_id))
                    || !matches_confidentiality(filter, &release.confidentiality.type_id)
                {
                    continue;
                }
                let verification = match self.verify_release(document.id, release.id)?.status {
                    ReleaseVerificationStatus::Match => "match",
                    ReleaseVerificationStatus::Mismatch => "mismatch",
                    ReleaseVerificationStatus::MissingFile => "missing_file",
                };
                rows.push(AuditRow {
                    record_type: "release",
                    document_id: document.id,
                    title: document.control.title.clone(),
                    relative_path: portable_path(&release.relative_pdf_path)?,
                    timestamp: Some(release.released_at),
                    event_type: if release.withdrawn {
                        "withdrawn"
                    } else {
                        "released"
                    }
                    .to_owned(),
                    actor: release.requester.object_id.to_string(),
                    approver: release.approver.object_id.to_string(),
                    confidentiality: release.confidentiality.type_id.clone(),
                    version: release.version.to_string(),
                    target_mode: target_mode_text(release.mode).to_owned(),
                    detail: release.changelog.clone(),
                    content_digest: release.pdf_digest.clone(),
                    predecessor_hash: release.approval_chain_head.clone().unwrap_or_default(),
                    evidence_hash: release.workflow_chain_head.clone(),
                    verification: verification.to_owned(),
                });
            }
        }
        rows.sort_by(|left, right| {
            left.relative_path
                .cmp(&right.relative_path)
                .then_with(|| left.timestamp.cmp(&right.timestamp))
                .then_with(|| left.record_type.cmp(right.record_type))
                .then_with(|| left.evidence_hash.cmp(&right.evidence_hash))
        });
        Ok(rows)
    }
}

fn normalize_filter(filter: &AuditReportFilter) -> Result<AuditReportFilter> {
    if matches!((filter.from, filter.through), (Some(from), Some(through)) if from > through) {
        return Err(DmsError::InvalidAuditFilter(
            "from must not be after through".to_owned(),
        ));
    }
    let mut normalized = filter.clone();
    normalized.document_ids.sort_unstable();
    normalized.document_ids.dedup();
    normalized.approver_object_ids.sort_unstable();
    normalized.approver_object_ids.dedup();
    normalized.confidentiality_type_ids = normalized
        .confidentiality_type_ids
        .into_iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect();
    normalized.confidentiality_type_ids.sort();
    normalized.confidentiality_type_ids.dedup();
    Ok(normalized)
}

fn matches_timestamp(filter: &AuditReportFilter, value: DateTime<Utc>) -> bool {
    filter.from.is_none_or(|from| value >= from)
        && filter.through.is_none_or(|through| value <= through)
}

fn matches_approver(filter: &AuditReportFilter, value: Option<Uuid>) -> bool {
    filter.approver_object_ids.is_empty()
        || value.is_some_and(|value| filter.approver_object_ids.contains(&value))
}

fn matches_confidentiality(filter: &AuditReportFilter, value: &str) -> bool {
    filter.confidentiality_type_ids.is_empty()
        || filter
            .confidentiality_type_ids
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(value))
}

fn report_record(event: &WorkflowEvent) -> Option<AuditReportRecord> {
    let evidence = event.body.report.clone()?;
    Some(AuditReportRecord {
        event_id: event.body.event_id,
        generated_at: event.body.timestamp,
        local_os_user: event.body.local_os_user.clone(),
        event_hash: event.event_hash.clone(),
        format: evidence.format,
        relative_path: evidence.relative_path,
        filter: evidence.filter,
        sha256: evidence.sha256,
        size: evidence.size,
    })
}

fn verify_chain(events: &[WorkflowEvent]) -> WorkflowVerification {
    if events.is_empty() {
        return WorkflowVerification::Missing;
    }
    let mut predecessor = None;
    for event in events {
        let hash = match crate::lifecycle::hash_event_body(&event.body) {
            Ok(hash) => hash,
            Err(_) => return WorkflowVerification::TamperedAt(event.body.event_id),
        };
        if event.body.predecessor_hash != predecessor || event.event_hash != hash {
            return WorkflowVerification::TamperedAt(event.body.event_id);
        }
        predecessor = Some(event.event_hash.clone());
    }
    WorkflowVerification::Valid
}

fn validate_report_evidence(evidence: &AuditReportEvidence) -> Result<()> {
    let path = PathBuf::from(&evidence.relative_path);
    crate::validate_relative_source_path(&path)
        .map_err(|_| DmsError::InvalidReportPath(path.clone()))?;
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_none_or(|value| !value.eq_ignore_ascii_case(evidence.format.extension()))
        || evidence.sha256.len() != 64
        || !evidence.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(DmsError::InvalidReportPath(path));
    }
    normalize_filter(&evidence.filter)?;
    Ok(())
}

fn render_csv(workspace_id: Uuid, filter: &AuditReportFilter, rows: &[AuditRow]) -> Vec<u8> {
    let mut output = String::from(
        "record_type,workspace_id,document_id,title,relative_path,timestamp,event_type,actor,approver,confidentiality,version,target_mode,detail,content_digest,predecessor_hash,evidence_hash,verification,filter\n",
    );
    let filter_text = serde_json::to_string(filter).expect("audit filters are serializable");
    for row in rows {
        let values = [
            row.record_type.to_owned(),
            workspace_id.to_string(),
            row.document_id.to_string(),
            row.title.clone(),
            row.relative_path.clone(),
            row.timestamp.map_or_else(String::new, canonical_time),
            row.event_type.clone(),
            row.actor.clone(),
            row.approver.clone(),
            row.confidentiality.clone(),
            row.version.clone(),
            row.target_mode.clone(),
            row.detail.clone(),
            row.content_digest.clone(),
            row.predecessor_hash.clone(),
            row.evidence_hash.clone(),
            row.verification.clone(),
            filter_text.clone(),
        ];
        output.push_str(
            &values
                .into_iter()
                .map(csv_cell)
                .collect::<Vec<_>>()
                .join(","),
        );
        output.push('\n');
    }
    output.into_bytes()
}

fn render_pdf(workspace_id: Uuid, filter: &AuditReportFilter, rows: &[AuditRow]) -> Vec<u8> {
    let filter_text = serde_json::to_string(filter).expect("audit filters are serializable");
    let mut lines = vec![
        "DMS Workspace Audit Report".to_owned(),
        format!("Workspace: {workspace_id}"),
        format!("Filter: {filter_text}"),
        format!("Records: {}", rows.len()),
        String::new(),
    ];
    for row in rows {
        lines.push(format!(
            "{} | {} | {} | {} | {} | {} | {}",
            row.record_type,
            row.timestamp
                .map_or_else(|| "current".to_owned(), canonical_time),
            row.title,
            row.event_type,
            row.version,
            row.target_mode,
            row.verification
        ));
        lines.push(format!(
            "  document={} path={} confidentiality={} approver={}",
            row.document_id, row.relative_path, row.confidentiality, row.approver
        ));
        if !row.detail.is_empty() {
            lines.push(format!("  detail={}", row.detail));
        }
        if !row.content_digest.is_empty() {
            lines.push(format!("  content-sha256={}", row.content_digest));
        }
        if !row.evidence_hash.is_empty() {
            lines.push(format!("  evidence-sha256={}", row.evidence_hash));
        }
    }
    build_pdf(&lines)
}

fn build_pdf(lines: &[String]) -> Vec<u8> {
    let wrapped = lines
        .iter()
        .flat_map(|line| wrap_ascii(line, 105))
        .collect::<Vec<_>>();
    let pages = wrapped.chunks(56).collect::<Vec<_>>();
    let page_count = pages.len().max(1);
    let mut objects = Vec::<Vec<u8>>::new();
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    let kids = (0..page_count)
        .map(|index| format!("{} 0 R", 4 + index * 2))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push(format!("<< /Type /Pages /Count {page_count} /Kids [{kids}] >>").into_bytes());
    objects.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );
    for index in 0..page_count {
        let page_object = 4 + index * 2;
        let content_object = page_object + 1;
        objects.push(
            format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R >> >> /Contents {content_object} 0 R >>")
                .into_bytes(),
        );
        let page_lines = pages.get(index).copied().unwrap_or(&[]);
        let mut stream = String::from("BT\n/F1 8 Tf\n45 800 Td\n");
        for line in page_lines {
            stream.push_str(&format!("({}) Tj\n0 -13 Td\n", pdf_escape(line)));
        }
        stream.push_str("ET\n");
        objects.push(
            format!(
                "<< /Length {} >>\nstream\n{}endstream",
                stream.len(),
                stream
            )
            .into_bytes(),
        );
    }

    let mut output = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (index, object) in objects.iter().enumerate() {
        offsets.push(output.len());
        output.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        output.extend_from_slice(object);
        output.extend_from_slice(b"\nendobj\n");
    }
    let xref = output.len();
    output.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    output.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        output.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    output.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    output
}

fn wrap_ascii(value: &str, width: usize) -> Vec<String> {
    let ascii = value
        .chars()
        .map(|character| if character.is_ascii() { character } else { '?' })
        .collect::<String>();
    if ascii.is_empty() {
        return vec![String::new()];
    }
    ascii
        .as_bytes()
        .chunks(width)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect()
}

fn pdf_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn csv_cell(value: String) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

fn canonical_time(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Nanos, true)
}

fn workflow_verification_text(value: WorkflowVerification) -> String {
    match value {
        WorkflowVerification::Valid => "valid".to_owned(),
        WorkflowVerification::Missing => "missing".to_owned(),
        WorkflowVerification::TamperedAt(event_id) => format!("tampered_at:{event_id}"),
    }
}

fn target_mode_text(value: crate::TargetVersionMode) -> &'static str {
    match value {
        crate::TargetVersionMode::NextMinor => "next_minor",
        crate::TargetVersionMode::NextMajor => "next_major",
        crate::TargetVersionMode::Manual => "manual",
    }
}

fn delivery_status_text(value: crate::DeliveryStatus) -> &'static str {
    match value {
        crate::DeliveryStatus::Queued => "queued",
        crate::DeliveryStatus::Accepted => "accepted",
        crate::DeliveryStatus::Confirmed => "confirmed",
        crate::DeliveryStatus::Failed => "failed",
    }
}

fn periodic_result_text(value: crate::PeriodicReviewResult) -> &'static str {
    match value {
        crate::PeriodicReviewResult::ConfirmedCurrent => "confirmed_current",
        crate::PeriodicReviewResult::ChangesRequired => "changes_required",
        crate::PeriodicReviewResult::Obsolete => "obsolete",
    }
}

fn event_type_text(value: WorkflowEventType) -> &'static str {
    match value {
        WorkflowEventType::ReviewRequested => "review_requested",
        WorkflowEventType::ReviewDecisionApproved => "review_decision_approved",
        WorkflowEventType::ReviewDecisionRejected => "review_decision_rejected",
        WorkflowEventType::ReviewDecisionChangedRequested => "review_decision_changes_requested",
        WorkflowEventType::Release => "release",
        WorkflowEventType::MinorPublicationNotified => "minor_publication_notified",
        WorkflowEventType::ReviewCancelled => "review_cancelled",
        WorkflowEventType::CandidateInvalidated => "candidate_invalidated",
        WorkflowEventType::RevisionBegun => "revision_begun",
        WorkflowEventType::DocumentObsoleted => "document_obsoleted",
        WorkflowEventType::ContentConformanceOverridden => "content_conformance_overridden",
        WorkflowEventType::PeriodicReviewRequested => "periodic_review_requested",
        WorkflowEventType::PeriodicReviewCompleted => "periodic_review_completed",
        WorkflowEventType::PeriodicReviewCancelled => "periodic_review_cancelled",
        WorkflowEventType::PeriodicReviewReminder => "periodic_review_reminder",
        WorkflowEventType::ReportGenerated => "report_generated",
    }
}

fn write_report_atomically(root: &Path, path: &Path, bytes: &[u8], event_id: Uuid) -> Result<()> {
    if fs::symlink_metadata(path).is_ok() {
        return Err(DmsError::ReportPathExists(path.to_path_buf()));
    }
    let parent = path
        .parent()
        .ok_or_else(|| DmsError::InvalidReportPath(path.to_path_buf()))?;
    create_safe_directories(root, parent)?;
    let temporary = parent.join(format!(".audit-{}.tmp", event_id.simple()));
    let result = (|| -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|source| DmsError::Io {
                path: temporary.clone(),
                source,
            })?;
        file.write_all(bytes).map_err(|source| DmsError::Io {
            path: temporary.clone(),
            source,
        })?;
        file.sync_all().map_err(|source| DmsError::Io {
            path: temporary.clone(),
            source,
        })?;
        fs::hard_link(&temporary, path).map_err(|source| {
            if source.kind() == std::io::ErrorKind::AlreadyExists {
                DmsError::ReportPathExists(path.to_path_buf())
            } else {
                DmsError::Io {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })?;
        if let Err(source) = fs::remove_file(&temporary) {
            let _ = fs::remove_file(path);
            return Err(DmsError::Io {
                path: temporary.clone(),
                source,
            });
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn create_safe_directories(root: &Path, parent: &Path) -> Result<()> {
    let relative = parent
        .strip_prefix(root)
        .map_err(|_| DmsError::InvalidReportPath(parent.to_path_buf()))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return Err(DmsError::InvalidReportPath(parent.to_path_buf()));
        };
        current.push(value);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(DmsError::InvalidReportPath(current));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|source| DmsError::Io {
                    path: current.clone(),
                    source,
                })?;
            }
            Err(source) => {
                return Err(DmsError::Io {
                    path: current,
                    source,
                });
            }
        }
    }
    Ok(())
}

fn portable_path(path: &Path) -> Result<String> {
    let components = path
        .components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| DmsError::InvalidReportPath(path.to_path_buf())),
            Component::CurDir => Ok(String::new()),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                Err(DmsError::InvalidReportPath(path.to_path_buf()))
            }
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(components
        .into_iter()
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>()
        .join("/"))
}

fn digest_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(digest_bytes(&bytes))
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
