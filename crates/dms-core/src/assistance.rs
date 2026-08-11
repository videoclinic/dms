use std::collections::BTreeSet;
use std::fs;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{
    configured_text, visible_source_text, DmsError, ReleaseVerificationStatus, Result, Version,
    Workspace,
};

pub const DEFAULT_CLAUDE_PAYLOAD_LIMIT: usize = 24_000;
pub const CLAUDE_DESKTOP_PROVIDER: &str = "Claude Desktop";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClaudeAssistancePolicy {
    pub enabled: bool,
    pub allowed_confidentiality_type_ids: BTreeSet<String>,
    pub max_payload_chars: usize,
}

impl Default for ClaudeAssistancePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_confidentiality_type_ids: BTreeSet::new(),
            max_payload_chars: DEFAULT_CLAUDE_PAYLOAD_LIMIT,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AssistanceEvidence {
    pub provider: String,
}

impl AssistanceEvidence {
    pub fn claude_desktop() -> Self {
        Self {
            provider: CLAUDE_DESKTOP_PROVIDER.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeExcerpt {
    pub line: usize,
    pub released: Option<String>,
    pub current: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeAssistancePayload {
    pub document_id: Uuid,
    pub title: String,
    pub document_number: Option<String>,
    pub document_type: Option<String>,
    pub confidentiality_type_id: String,
    pub confidentiality_label: String,
    pub release_id: Uuid,
    pub release_version: Version,
    pub release_source_digest: String,
    pub current_source_digest: String,
    pub released_pdf_digest: String,
    pub excerpts: Vec<ChangeExcerpt>,
    pub prompt: String,
    pub payload_digest: String,
}

impl Workspace {
    pub fn claude_assistance_policy(&self) -> &ClaudeAssistancePolicy {
        &self.claude_assistance
    }

    pub fn configure_claude_assistance(
        &mut self,
        enabled: bool,
        allowed_confidentiality_type_ids: BTreeSet<String>,
        max_payload_chars: usize,
    ) -> Result<()> {
        if max_payload_chars == 0 {
            return Err(DmsError::InvalidClaudePayloadLimit);
        }
        for type_id in &allowed_confidentiality_type_ids {
            if !self.confidentiality_types.contains_key(type_id) {
                return Err(DmsError::UnknownConfidentialityType(type_id.clone()));
            }
        }
        self.claude_assistance = ClaudeAssistancePolicy {
            enabled,
            allowed_confidentiality_type_ids,
            max_payload_chars,
        };
        self.save()
    }

    pub fn prepare_claude_assistance(&self, document_id: Uuid) -> Result<ClaudeAssistancePayload> {
        if !self.claude_assistance.enabled {
            return Err(DmsError::ClaudeAssistanceDisabled);
        }
        let document = self.document(document_id)?;
        let confidentiality = self.effective_confidentiality(document_id)?;
        if !self
            .claude_assistance
            .allowed_confidentiality_type_ids
            .contains(&confidentiality.type_id)
        {
            return Err(DmsError::ClaudeAssistanceNotPermitted(
                confidentiality.type_id,
            ));
        }
        let release = self
            .current_release(document_id)?
            .ok_or(DmsError::NoCurrentRelease)?;
        let verification = self.verify_release(document_id, release.id)?;
        if verification.status != ReleaseVerificationStatus::Match {
            return Err(DmsError::ReleaseIntegrityRequired(release.id));
        }

        let source_path = self.edit_root.join(&document.relative_path);
        let current_text = visible_source_text(&source_path)?;
        let pdf_path = self.publish_root.join(&release.relative_pdf_path);
        let released_text =
            pdf_extract::extract_text(&pdf_path).map_err(|error| DmsError::PdfTextExtraction {
                path: pdf_path,
                detail: error.to_string(),
            })?;
        let current_source_digest =
            digest_bytes(&fs::read(&source_path).map_err(|source| DmsError::Io {
                path: source_path,
                source,
            })?);

        build_payload(
            document_id,
            document.control.title.clone(),
            document.control.document_number.clone(),
            document.control.document_type.clone(),
            confidentiality.type_id,
            confidentiality.label,
            release.id,
            release.version,
            release.source_digest.clone(),
            current_source_digest,
            release.pdf_digest.clone(),
            &released_text,
            &current_text,
            self.claude_assistance.max_payload_chars,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn build_payload(
    document_id: Uuid,
    title: String,
    document_number: Option<String>,
    document_type: Option<String>,
    confidentiality_type_id: String,
    confidentiality_label: String,
    release_id: Uuid,
    release_version: Version,
    release_source_digest: String,
    current_source_digest: String,
    released_pdf_digest: String,
    released_text: &str,
    current_text: &str,
    max_payload_chars: usize,
) -> Result<ClaudeAssistancePayload> {
    let excerpts = comparison_excerpts(released_text, current_text);
    let mut prompt = String::new();
    prompt.push_str("Review this operator-previewed document change comparison.\n");
    prompt
        .push_str("Use only the supplied metadata and excerpts. Do not infer omitted content.\n\n");
    prompt.push_str(&format!("Title: {}\n", configured_text(&title, "title")?));
    if let Some(number) = &document_number {
        prompt.push_str(&format!("Document number: {number}\n"));
    }
    if let Some(document_type) = &document_type {
        prompt.push_str(&format!("Document type: {document_type}\n"));
    }
    prompt.push_str(&format!(
        "Confidentiality: {confidentiality_label} ({confidentiality_type_id})\nCurrent release: {release_version}\nRelease source digest: {release_source_digest}\nCurrent source digest: {current_source_digest}\nReleased PDF digest: {released_pdf_digest}\n\n",
    ));
    prompt.push_str("Deterministic local comparison excerpts:\n");
    if excerpts.is_empty() {
        prompt.push_str("No line differences were found in extracted text.\n");
    } else {
        for excerpt in &excerpts {
            prompt.push_str(&format!("Line {}:\n", excerpt.line));
            prompt.push_str(&format!(
                "- released: {}\n+ current: {}\n",
                excerpt.released.as_deref().unwrap_or("<no line>"),
                excerpt.current.as_deref().unwrap_or("<no line>"),
            ));
        }
    }
    prompt.push_str("\nReturn two advisory sections:\n");
    prompt.push_str("1. Suggested target-version mode: minor version change, major version change, or manual version set; include rationale.\n");
    prompt.push_str("2. Proposed concise changelog grounded only in the supplied differences.\n");
    prompt.push_str(
        "The operator, not you, chooses the target and edits any accepted changelog text.\n",
    );

    let actual_chars = prompt.chars().count();
    if actual_chars > max_payload_chars {
        return Err(DmsError::ClaudePayloadTooLarge {
            actual_chars,
            max_chars: max_payload_chars,
        });
    }
    let payload_digest = digest_bytes(prompt.as_bytes());
    Ok(ClaudeAssistancePayload {
        document_id,
        title,
        document_number,
        document_type,
        confidentiality_type_id,
        confidentiality_label,
        release_id,
        release_version,
        release_source_digest,
        current_source_digest,
        released_pdf_digest,
        excerpts,
        prompt,
        payload_digest,
    })
}

fn comparison_excerpts(released_text: &str, current_text: &str) -> Vec<ChangeExcerpt> {
    let released = released_text.lines().collect::<Vec<_>>();
    let current = current_text.lines().collect::<Vec<_>>();
    let count = released.len().max(current.len());
    (0..count)
        .filter_map(|index| {
            let old = released.get(index).copied();
            let new = current.get(index).copied();
            (old != new).then(|| ChangeExcerpt {
                line: index + 1,
                released: old.map(str::to_owned),
                current: new.map(str::to_owned),
            })
        })
        .collect()
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_is_deterministic_bounded_and_contains_only_selected_metadata_and_excerpts() {
        let id = Uuid::nil();
        let payload = build_payload(
            id,
            "Handbook".to_owned(),
            Some("DOC-1".to_owned()),
            Some("policy".to_owned()),
            "internal".to_owned(),
            "Internal".to_owned(),
            id,
            Version::V1_0,
            "source-old".to_owned(),
            "source-new".to_owned(),
            "pdf".to_owned(),
            "old line\nunchanged",
            "new line\nunchanged",
            10_000,
        )
        .unwrap();
        assert_eq!(payload.excerpts.len(), 1);
        assert!(payload.prompt.contains("- released: old line"));
        assert!(payload.prompt.contains("+ current: new line"));
        assert!(!payload.prompt.contains("approver"));
        assert!(!payload.prompt.contains("/"));
        assert_eq!(payload.payload_digest.len(), 64);

        assert!(matches!(
            build_payload(
                id,
                "Handbook".to_owned(),
                None,
                None,
                "internal".to_owned(),
                "Internal".to_owned(),
                id,
                Version::V1_0,
                "old".to_owned(),
                "new".to_owned(),
                "pdf".to_owned(),
                "old",
                "new",
                10,
            ),
            Err(DmsError::ClaudePayloadTooLarge { .. })
        ));
    }
}
