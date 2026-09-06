use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{DmsError, Result, Workspace};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DocumentType {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    #[serde(default)]
    pub replacement_type_id: Option<String>,
}

impl Workspace {
    pub fn document_types(&self) -> Vec<&DocumentType> {
        self.document_types.values().collect()
    }

    pub fn configure_document_type(
        &mut self,
        id: &str,
        label: &str,
        enabled: bool,
    ) -> Result<DocumentType> {
        let id = id.trim();
        validate_portable_id(id, DmsError::InvalidDocumentTypeId)?;
        if !enabled && self.document_type_is_in_use(id) {
            return Err(DmsError::DocumentTypeInUse(id.to_owned()));
        }
        let configured = DocumentType {
            id: id.to_owned(),
            label: configured_text(label, "document type label")?,
            enabled,
            replacement_type_id: self
                .document_types
                .get(id)
                .and_then(|configured| configured.replacement_type_id.clone()),
        };
        self.document_types
            .insert(configured.id.clone(), configured.clone());
        Ok(configured)
    }

    pub fn migrate_document_type(
        &mut self,
        source_type_id: &str,
        replacement_type_id: &str,
    ) -> Result<DocumentType> {
        let source_type_id = source_type_id.trim();
        let replacement_type_id = replacement_type_id.trim();
        validate_portable_id(source_type_id, DmsError::InvalidDocumentTypeId)?;
        validate_portable_id(replacement_type_id, DmsError::InvalidDocumentTypeId)?;
        self.validate_document_type_migration(source_type_id, replacement_type_id)?;

        let mut source = self.require_document_type(source_type_id)?.clone();
        source.replacement_type_id = Some(replacement_type_id.to_owned());
        self.document_types
            .insert(source.id.clone(), source.clone());
        self.invalidate_stale_candidates();
        Ok(source)
    }

    pub(crate) fn require_document_type(&self, type_id: &str) -> Result<&DocumentType> {
        self.document_types
            .get(type_id)
            .ok_or_else(|| DmsError::UnknownDocumentType(type_id.to_owned()))
    }

    pub(crate) fn require_enabled_document_type(&self, type_id: &str) -> Result<&DocumentType> {
        let configured = self.require_document_type(type_id)?;
        if !configured.enabled {
            return Err(DmsError::DisabledDocumentType(type_id.to_owned()));
        }
        Ok(configured)
    }

    pub(crate) fn release_document_type(&self, type_id: &str) -> Result<&DocumentType> {
        let mut seen = BTreeSet::new();
        let mut current = self.require_enabled_document_type(type_id)?;
        loop {
            if !seen.insert(current.id.clone()) {
                return Err(DmsError::InvalidDocumentTypeMigration {
                    from_type_id: type_id.to_owned(),
                    replacement: current.id.clone(),
                });
            }
            let Some(next) = current.replacement_type_id.as_deref() else {
                return Ok(current);
            };
            current = self.require_enabled_document_type(next)?;
        }
    }

    pub(crate) fn validate_document_type_migrations(&self) -> Result<()> {
        for (type_id, configured) in &self.document_types {
            if let Some(replacement_type_id) = configured.replacement_type_id.as_deref() {
                self.validate_document_type_migration(type_id, replacement_type_id)?;
            }
        }
        Ok(())
    }

    fn document_type_is_in_use(&self, id: &str) -> bool {
        self.documents
            .values()
            .any(|document| document.control.document_type.as_deref() == Some(id))
            || self
                .document_types
                .values()
                .any(|configured| configured.replacement_type_id.as_deref() == Some(id))
            || self
                .document_types
                .get(id)
                .is_some_and(|configured| configured.replacement_type_id.is_some())
    }

    fn validate_document_type_migration(
        &self,
        source_type_id: &str,
        replacement_type_id: &str,
    ) -> Result<()> {
        self.require_document_type(source_type_id)?;
        if source_type_id == replacement_type_id {
            return Err(DmsError::InvalidDocumentTypeMigration {
                from_type_id: source_type_id.to_owned(),
                replacement: replacement_type_id.to_owned(),
            });
        }

        let mut seen = BTreeSet::new();
        let mut current = replacement_type_id.to_owned();
        loop {
            if !seen.insert(current.clone()) || current == source_type_id {
                return Err(DmsError::InvalidDocumentTypeMigration {
                    from_type_id: source_type_id.to_owned(),
                    replacement: replacement_type_id.to_owned(),
                });
            }
            let configured = self.require_enabled_document_type(&current)?;
            let Some(next) = configured.replacement_type_id.as_deref() else {
                return Ok(());
            };
            current = next.to_owned();
        }
    }
}

pub(crate) fn validate_portable_id(id: &str, error: impl FnOnce(String) -> DmsError) -> Result<()> {
    let valid = !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && id.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && id.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric);
    if valid {
        Ok(())
    } else {
        Err(error(id.to_owned()))
    }
}

pub(crate) fn configured_text(value: &str, field: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        Err(DmsError::InvalidConfiguration(field.to_owned()))
    } else {
        Ok(value.to_owned())
    }
}
