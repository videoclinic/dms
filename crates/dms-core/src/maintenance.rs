use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Months, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{
    configured_text, default_author, AuthenticatedActor, ConfidentialitySnapshot, DmsError,
    GraphClient, Lifecycle, PeriodicReviewEventDetails, PersonSnapshot, Result, Version,
    WorkflowEventBody, WorkflowEventType, Workspace, METADATA_DIRECTORY,
};

pub const DEFAULT_REVIEW_INTERVAL_MONTHS: u32 = 12;
pub const DUE_SOON_DAYS: i64 = 30;

pub(crate) fn default_review_interval_months() -> u32 {
    DEFAULT_REVIEW_INTERVAL_MONTHS
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseVerificationStatus {
    Match,
    Mismatch,
    MissingFile,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseVerification {
    pub document_id: Uuid,
    pub release_id: Uuid,
    pub version: Version,
    pub relative_pdf_path: PathBuf,
    pub status: ReleaseVerificationStatus,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PeriodicReviewStatus {
    Open,
    Completed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PeriodicReviewResult {
    ConfirmedCurrent,
    ChangesRequired,
    Obsolete,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PeriodicReview {
    pub id: Uuid,
    pub release_id: Uuid,
    pub version: Version,
    pub pdf_digest: String,
    pub confidentiality: ConfidentialitySnapshot,
    pub approver: PersonSnapshot,
    pub requested_at: DateTime<Utc>,
    pub status: PeriodicReviewStatus,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<PeriodicReviewResult>,
    pub comment: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PeriodicReviewDueStatus {
    Current,
    DueSoon,
    Overdue,
    Exempt,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeriodicReviewMarker {
    pub document_id: Uuid,
    pub title: String,
    pub release_id: Option<Uuid>,
    pub version: Option<Version>,
    pub next_review_due: Option<NaiveDate>,
    pub status: PeriodicReviewDueStatus,
    pub open_review_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupManifestEntry {
    pub archive_path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupManifest {
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub entries: Vec<BackupManifestEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupOutcome {
    pub archive_path: PathBuf,
    pub manifest_digest: String,
    pub entry_count: usize,
}

impl Workspace {
    pub fn verify_release(
        &self,
        document_id: Uuid,
        release_id: Uuid,
    ) -> Result<ReleaseVerification> {
        let release = self
            .document(document_id)?
            .releases
            .iter()
            .find(|release| release.id == release_id)
            .ok_or(DmsError::ReleaseNotFound(release_id))?;
        let path = self.publish_root.join(&release.relative_pdf_path);
        let status = if !path.is_file() {
            ReleaseVerificationStatus::MissingFile
        } else if digest_file(&path)? == release.pdf_digest {
            ReleaseVerificationStatus::Match
        } else {
            ReleaseVerificationStatus::Mismatch
        };
        Ok(ReleaseVerification {
            document_id,
            release_id,
            version: release.version,
            relative_pdf_path: release.relative_pdf_path.clone(),
            status,
        })
    }

    pub fn verify_document_releases(&self, document_id: Uuid) -> Result<Vec<ReleaseVerification>> {
        let release_ids = self
            .document(document_id)?
            .releases
            .iter()
            .map(|release| release.id)
            .collect::<Vec<_>>();
        release_ids
            .into_iter()
            .map(|release_id| self.verify_release(document_id, release_id))
            .collect()
    }

    pub fn verify_all_releases(&self) -> Result<Vec<ReleaseVerification>> {
        let mut results = Vec::new();
        for document_id in self.documents.keys().copied() {
            results.extend(self.verify_document_releases(document_id)?);
        }
        Ok(results)
    }

    pub fn configure_default_review_interval(&mut self, months: u32) -> Result<()> {
        if months == 0 {
            return Err(DmsError::InvalidReviewInterval);
        }
        self.default_review_interval_months = months;
        Ok(())
    }

    pub fn set_document_review_interval(
        &mut self,
        document_id: Uuid,
        months: Option<u32>,
    ) -> Result<()> {
        if matches!(months, Some(0)) {
            return Err(DmsError::InvalidReviewInterval);
        }
        self.document_mut(document_id)?.review_interval_months = months;
        Ok(())
    }

    pub fn set_document_review_exemption(
        &mut self,
        document_id: Uuid,
        reason: Option<&str>,
    ) -> Result<()> {
        let reason = reason
            .map(|reason| configured_text(reason, "periodic review exemption reason"))
            .transpose()?;
        let document = self.document_mut(document_id)?;
        document.review_exemption_reason = reason;
        if document.review_exemption_reason.is_some() {
            document.next_review_due = None;
        }
        Ok(())
    }

    pub fn periodic_review_markers(&self, as_of: NaiveDate) -> Result<Vec<PeriodicReviewMarker>> {
        let mut markers = Vec::new();
        for document in self.documents.values() {
            let current = document
                .releases
                .iter()
                .rev()
                .find(|release| !release.withdrawn);
            if current.is_none() && document.review_exemption_reason.is_none() {
                continue;
            }
            let status = if document.review_exemption_reason.is_some() {
                PeriodicReviewDueStatus::Exempt
            } else {
                match document.next_review_due {
                    Some(due) if due < as_of => PeriodicReviewDueStatus::Overdue,
                    Some(due) if due <= as_of + chrono::Duration::days(DUE_SOON_DAYS) => {
                        PeriodicReviewDueStatus::DueSoon
                    }
                    _ => PeriodicReviewDueStatus::Current,
                }
            };
            markers.push(PeriodicReviewMarker {
                document_id: document.id,
                title: document.control.title.clone(),
                release_id: current.map(|release| release.id),
                version: current.map(|release| release.version),
                next_review_due: document.next_review_due,
                status,
                open_review_id: document
                    .periodic_reviews
                    .iter()
                    .rev()
                    .find(|review| review.status == PeriodicReviewStatus::Open)
                    .map(|review| review.id),
            });
        }
        markers.sort_by(|left, right| {
            left.next_review_due
                .cmp(&right.next_review_due)
                .then_with(|| left.title.cmp(&right.title))
        });
        Ok(markers)
    }

    pub fn start_periodic_review(&mut self, document_id: Uuid) -> Result<PeriodicReview> {
        if self
            .document(document_id)?
            .review_exemption_reason
            .is_some()
        {
            return Err(DmsError::PeriodicReviewExempt);
        }
        if self.document(document_id)?.active_candidate_id.is_some()
            || self
                .document(document_id)?
                .periodic_reviews
                .iter()
                .any(|review| review.status == PeriodicReviewStatus::Open)
        {
            return Err(DmsError::PeriodicReviewAlreadyOpen);
        }
        if self.document(document_id)?.lifecycle != Lifecycle::Released {
            return Err(DmsError::InvalidLifecycleTransition(
                "periodic review requires a current released document".to_owned(),
            ));
        }
        let release = self
            .document(document_id)?
            .releases
            .iter()
            .rev()
            .find(|release| !release.withdrawn)
            .cloned()
            .ok_or(DmsError::NoCurrentRelease)?;
        if self.verify_release(document_id, release.id)?.status != ReleaseVerificationStatus::Match
        {
            return Err(DmsError::ReleaseIntegrityRequired(release.id));
        }
        let review = PeriodicReview {
            id: Uuid::new_v4(),
            release_id: release.id,
            version: release.version,
            pdf_digest: release.pdf_digest.clone(),
            confidentiality: release.confidentiality.clone(),
            approver: release.approver.clone(),
            requested_at: Utc::now(),
            status: PeriodicReviewStatus::Open,
            completed_at: None,
            result: None,
            comment: None,
        };
        self.document_mut(document_id)?
            .periodic_reviews
            .push(review.clone());
        self.append_periodic_event(
            document_id,
            WorkflowEventType::PeriodicReviewRequested,
            &review,
            None,
            None,
        )?;
        self.save()?;
        Ok(review)
    }

    pub fn complete_periodic_review<G: GraphClient>(
        &mut self,
        document_id: Uuid,
        review_id: Uuid,
        result: PeriodicReviewResult,
        comment: &str,
        graph: &mut G,
    ) -> Result<PeriodicReview> {
        let comment = configured_text(comment, "periodic review result comment")?;
        let review = self
            .document(document_id)?
            .periodic_reviews
            .iter()
            .find(|review| review.id == review_id)
            .cloned()
            .ok_or(DmsError::PeriodicReviewNotFound(review_id))?;
        if review.status != PeriodicReviewStatus::Open {
            return Err(DmsError::PeriodicReviewNotOpen(review_id));
        }
        if self.verify_release(document_id, review.release_id)?.status
            != ReleaseVerificationStatus::Match
        {
            return Err(DmsError::ReleaseIntegrityRequired(review.release_id));
        }
        let source = self
            .identity_source
            .as_ref()
            .ok_or(DmsError::IdentitySourceRequired)?;
        let actor = graph
            .authenticated_actor(source)
            .map_err(DmsError::InteractiveSignInFailed)?;
        if actor.tenant_id != source.tenant_id || actor.object_id != review.approver.object_id {
            return Err(DmsError::DecisionActorMismatch);
        }
        let completed_at = Utc::now();
        self.append_periodic_event(
            document_id,
            WorkflowEventType::PeriodicReviewCompleted,
            &review,
            Some(actor),
            Some((result, comment.clone())),
        )?;
        {
            let stored = self
                .document_mut(document_id)?
                .periodic_reviews
                .iter_mut()
                .find(|candidate| candidate.id == review_id)
                .expect("periodic review checked above");
            stored.status = PeriodicReviewStatus::Completed;
            stored.completed_at = Some(completed_at);
            stored.result = Some(result);
            stored.comment = Some(comment.clone());
        }
        match result {
            PeriodicReviewResult::ConfirmedCurrent => {
                self.schedule_next_review(document_id, completed_at.date_naive())?;
                self.save()?;
            }
            PeriodicReviewResult::ChangesRequired => {
                self.begin_revision(document_id)?;
            }
            PeriodicReviewResult::Obsolete => {
                self.mark_obsolete(document_id, &comment)?;
            }
        }
        Ok(self
            .document(document_id)?
            .periodic_reviews
            .iter()
            .find(|candidate| candidate.id == review_id)
            .expect("periodic review retained")
            .clone())
    }

    pub fn backup_workspace(&self, archive_path: &Path) -> Result<BackupOutcome> {
        if archive_path.exists() {
            return Err(DmsError::BackupPathExists(archive_path.to_path_buf()));
        }
        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent).map_err(|source| DmsError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let temporary_path = archive_path.with_extension(format!(
            "{}.tmp",
            archive_path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("zip")
        ));
        let files = self.backup_files(archive_path, &temporary_path)?;
        let manifest = BackupManifest {
            workspace_id: self.workspace_id,
            created_at: Utc::now(),
            entries: files
                .iter()
                .map(|file| BackupManifestEntry {
                    archive_path: file.archive_path.clone(),
                    size: file.bytes.len() as u64,
                    sha256: digest_bytes(&file.bytes),
                })
                .collect(),
        };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| {
            DmsError::BackupManifest(format!("cannot encode backup manifest: {error}"))
        })?;
        let manifest_digest = digest_bytes(&manifest_bytes);
        let write_result = (|| -> Result<()> {
            let file = File::create(&temporary_path).map_err(|source| DmsError::Io {
                path: temporary_path.clone(),
                source,
            })?;
            let mut archive = ZipWriter::new(file);
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o600);
            for entry in &files {
                archive
                    .start_file(&entry.archive_path, options)
                    .map_err(|error| DmsError::BackupManifest(error.to_string()))?;
                archive
                    .write_all(&entry.bytes)
                    .map_err(|source| DmsError::Io {
                        path: temporary_path.clone(),
                        source,
                    })?;
            }
            archive
                .start_file("manifest.json", options)
                .map_err(|error| DmsError::BackupManifest(error.to_string()))?;
            archive
                .write_all(&manifest_bytes)
                .map_err(|source| DmsError::Io {
                    path: temporary_path.clone(),
                    source,
                })?;
            archive
                .finish()
                .map_err(|error| DmsError::BackupManifest(error.to_string()))?;
            fs::rename(&temporary_path, archive_path).map_err(|source| DmsError::Io {
                path: archive_path.to_path_buf(),
                source,
            })?;
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        write_result?;
        Ok(BackupOutcome {
            archive_path: archive_path.to_path_buf(),
            manifest_digest,
            entry_count: manifest.entries.len(),
        })
    }

    pub(crate) fn schedule_next_review(
        &mut self,
        document_id: Uuid,
        from: NaiveDate,
    ) -> Result<()> {
        let interval = self
            .document(document_id)?
            .review_interval_months
            .unwrap_or(self.default_review_interval_months);
        if interval == 0 {
            return Err(DmsError::InvalidReviewInterval);
        }
        let document = self.document_mut(document_id)?;
        if document.review_exemption_reason.is_none() {
            document.next_review_due = from.checked_add_months(Months::new(interval));
        }
        Ok(())
    }

    fn append_periodic_event(
        &mut self,
        document_id: Uuid,
        event_type: WorkflowEventType,
        review: &PeriodicReview,
        actor: Option<AuthenticatedActor>,
        outcome: Option<(PeriodicReviewResult, String)>,
    ) -> Result<()> {
        let (result, comment) = outcome
            .map(|(result, comment)| (Some(result), Some(comment)))
            .unwrap_or((None, None));
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
            approver: Some(review.approver.clone()),
            authenticated_actor: actor,
            local_os_user: default_author(),
            revision_digest: None,
            confidentiality: Some(review.confidentiality.clone()),
            target_version: Some(review.version),
            target_mode: None,
            changelog: None,
            assistance: None,
            decision_comment: None,
            operator_comment: comment,
            delivery: None,
            content_override: None,
            pdf_digest: Some(review.pdf_digest.clone()),
            periodic_review: Some(PeriodicReviewEventDetails {
                review_id: review.id,
                release_id: review.release_id,
                result,
            }),
        };
        self.append_event(document_id, body)?;
        Ok(())
    }

    fn backup_files(&self, archive_path: &Path, temporary_path: &Path) -> Result<Vec<BackupFile>> {
        let mut paths = Vec::new();
        collect_regular_files(
            &self.edit_root.join(METADATA_DIRECTORY),
            &mut paths,
            archive_path,
            temporary_path,
        )?;
        let mut files = Vec::new();
        for path in paths {
            let relative = path.strip_prefix(&self.edit_root).map_err(|_| {
                DmsError::BackupManifest(format!("{} is outside the edit root", path.display()))
            })?;
            files.push(read_backup_file(&path, archive_name("edit", relative))?);
        }
        for document in self.documents.values() {
            let source = self.edit_root.join(&document.relative_path);
            files.push(read_backup_file(
                &source,
                archive_name("edit", &document.relative_path),
            )?);
            for release in &document.releases {
                let pdf = self.publish_root.join(&release.relative_pdf_path);
                files.push(read_backup_file(
                    &pdf,
                    archive_name("publish", &release.relative_pdf_path),
                )?);
            }
        }
        files.sort_by(|left, right| left.archive_path.cmp(&right.archive_path));
        files.dedup_by(|left, right| left.archive_path == right.archive_path);
        Ok(files)
    }
}

struct BackupFile {
    archive_path: String,
    bytes: Vec<u8>,
}

fn read_backup_file(path: &Path, archive_path: String) -> Result<BackupFile> {
    let metadata = fs::symlink_metadata(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(DmsError::BackupInputInvalid(path.to_path_buf()));
    }
    let bytes = fs::read(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(BackupFile {
        archive_path,
        bytes,
    })
}

fn collect_regular_files(
    directory: &Path,
    output: &mut Vec<PathBuf>,
    archive_path: &Path,
    temporary_path: &Path,
) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| DmsError::Io {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|source| DmsError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path == archive_path || path == temporary_path {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|source| DmsError::Io {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(DmsError::BackupInputInvalid(path));
        }
        if metadata.is_dir() {
            collect_regular_files(&path, output, archive_path, temporary_path)?;
        } else if metadata.is_file() {
            output.push(path);
        }
    }
    Ok(())
}

fn archive_name(scope: &str, path: &Path) -> String {
    let relative = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    format!("{scope}/{relative}")
}

fn digest_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|source| DmsError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
