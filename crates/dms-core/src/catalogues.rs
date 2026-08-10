use serde::{Deserialize, Serialize};

use super::{DmsError, Result, Workspace};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DocumentType {
    pub id: String,
    pub label: String,
    pub enabled: bool,
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
        if !enabled
            && self
                .documents
                .values()
                .any(|document| document.control.document_type.as_deref() == Some(id))
        {
            return Err(DmsError::DocumentTypeInUse(id.to_owned()));
        }
        let configured = DocumentType {
            id: id.to_owned(),
            label: configured_text(label, "document type label")?,
            enabled,
        };
        self.document_types
            .insert(configured.id.clone(), configured.clone());
        Ok(configured)
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
