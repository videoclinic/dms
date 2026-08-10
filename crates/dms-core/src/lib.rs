use std::{
    collections::BTreeMap,
    env, fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

mod catalogues;
mod library;
mod policies;

pub use catalogues::*;
pub use library::*;
pub use policies::*;

pub const SCHEMA_VERSION: u32 = 3;
pub const METADATA_DIRECTORY: &str = ".dms";
pub const METADATA_FILENAME: &str = "workspace.json";

pub type Result<T> = std::result::Result<T, DmsError>;

#[derive(Debug, Error)]
pub enum DmsError {
    #[error("I/O failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("metadata at {path} is not valid JSON: {source}")]
    InvalidMetadata {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("workspace metadata already exists at {0}")]
    WorkspaceAlreadyExists(PathBuf),
    #[error("workspace metadata was not found at {0}")]
    WorkspaceNotFound(PathBuf),
    #[error("{0} must be an existing directory")]
    ExpectedDirectory(String),
    #[error("workspace schema version {found} is unsupported; expected {expected}")]
    UnsupportedSchema { expected: u32, found: u32 },
    #[error("stored edit root {stored} does not match requested edit root {requested}")]
    EditRootMismatch { stored: PathBuf, requested: PathBuf },
    #[error("source path {0} resolves outside the edit root")]
    OutsideEditRoot(PathBuf),
    #[error("source path {0} is inside the internal metadata directory")]
    MetadataPath(PathBuf),
    #[error("source path {0} is not a regular file")]
    ExpectedFile(PathBuf),
    #[error("source path {0} has an unsupported draft format")]
    UnsupportedSource(PathBuf),
    #[error("source path {0} is an Office temporary file")]
    OfficeTemporaryFile(PathBuf),
    #[error("document already registered for {0}")]
    DocumentAlreadyRegistered(PathBuf),
    #[error("document {0} was not found")]
    DocumentNotFound(Uuid),
    #[error("note {note_id} was not found on document {document_id}")]
    NoteNotFound { document_id: Uuid, note_id: Uuid },
    #[error("document ID key {key} does not match stored document ID {stored}")]
    DocumentIdMismatch { key: Uuid, stored: Uuid },
    #[error("source path {0} must be a clean path relative to the edit root")]
    InvalidRelativePath(PathBuf),
    #[error("document title cannot be empty")]
    EmptyTitle,
    #[error("note body cannot be empty")]
    EmptyNote,
    #[error("document number {0:?} is already assigned")]
    DuplicateDocumentNumber(String),
    #[error("confidentiality type ID {0:?} must contain lowercase letters, digits, or hyphens")]
    InvalidConfidentialityTypeId(String),
    #[error("confidentiality type {0:?} is not configured")]
    UnknownConfidentialityType(String),
    #[error("confidentiality type {0:?} is disabled")]
    DisabledConfidentialityType(String),
    #[error("the edit-root confidentiality policy is required")]
    RequiredRootPolicy,
    #[error("no effective confidentiality policy is configured")]
    MissingConfidentialityPolicy,
    #[error("policy folder {0:?} must be an existing edit-root-relative folder")]
    InvalidPolicyFolder(String),
    #[error("a Microsoft Entra identity source must be configured first")]
    IdentitySourceRequired,
    #[error("Microsoft Entra person {0} is not an eligible cached group member")]
    IneligibleEntraPerson(Uuid),
    #[error("the edit-root workflow policy must assign both editor and approver")]
    RequiredRootWorkflowPolicy,
    #[error("identity cache key {key} does not match stored person ID {stored}")]
    IdentityCacheKeyMismatch { key: Uuid, stored: Uuid },
    #[error("configuration field {0} cannot be empty")]
    InvalidConfiguration(String),
    #[error("document type ID {0:?} must contain lowercase letters, digits, or hyphens")]
    InvalidDocumentTypeId(String),
    #[error("document type {0:?} is not configured")]
    UnknownDocumentType(String),
    #[error("document type {0:?} is disabled")]
    DisabledDocumentType(String),
    #[error("document type {0:?} is referenced by a document")]
    DocumentTypeInUse(String),
    #[error("confidentiality type {0:?} is referenced by a live policy or document")]
    ConfidentialityTypeInUse(String),
    #[error("migration backup at {0} does not match the workspace being migrated")]
    MigrationBackupConflict(PathBuf),
    #[error("library folder {0} must be an existing edit-root-relative directory")]
    InvalidLibraryFolder(PathBuf),
    #[error("library entry {0} does not have a supported filesystem name")]
    InvalidLibraryEntry(PathBuf),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    pub schema_version: u32,
    pub workspace_id: Uuid,
    pub edit_root: PathBuf,
    pub publish_root: PathBuf,
    pub(crate) documents: BTreeMap<Uuid, Document>,
    #[serde(default)]
    pub(crate) document_types: BTreeMap<String, DocumentType>,
    #[serde(default)]
    pub(crate) confidentiality_types: BTreeMap<String, ConfidentialityType>,
    #[serde(default)]
    pub(crate) confidentiality_policies: BTreeMap<String, ConfidentialityPolicy>,
    #[serde(default)]
    pub(crate) identity_source: Option<EntraIdentitySource>,
    #[serde(default)]
    pub(crate) identity_cache: BTreeMap<Uuid, EntraPerson>,
    #[serde(default)]
    pub(crate) workflow_policies: BTreeMap<String, WorkflowPolicy>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: Uuid,
    #[serde(with = "relative_path_serde")]
    pub relative_path: PathBuf,
    pub lifecycle: Lifecycle,
    #[serde(default)]
    pub source_state: SourceState,
    pub control: DocumentControl,
    #[serde(default)]
    pub(crate) confidentiality_override: Option<String>,
    #[serde(default)]
    pub(crate) workflow_overrides: DocumentWorkflowOverrides,
    notes: Vec<Note>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Draft,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentControl {
    pub title: String,
    pub document_number: Option<String>,
    pub document_type: Option<String>,
    pub owner: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub id: Uuid,
    pub body: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default)]
pub struct ControlUpdate {
    pub title: Option<String>,
    pub document_number: Option<Option<String>>,
    pub document_type: Option<Option<String>>,
    pub owner: Option<Option<String>>,
}

impl Workspace {
    pub fn init(edit_root: &Path, publish_root: &Path) -> Result<Self> {
        let edit_root = canonical_existing_directory(edit_root, "edit root")?;
        if !publish_root.exists() {
            fs::create_dir_all(publish_root).map_err(|source| DmsError::Io {
                path: publish_root.to_path_buf(),
                source,
            })?;
        }
        let publish_root = canonical_existing_directory(publish_root, "publish root")?;
        let metadata_directory = edit_root.join(METADATA_DIRECTORY);
        if metadata_directory.exists() {
            return Err(DmsError::WorkspaceAlreadyExists(metadata_directory));
        }
        fs::create_dir(&metadata_directory).map_err(|source| DmsError::Io {
            path: metadata_directory.clone(),
            source,
        })?;

        let workspace = Self {
            schema_version: SCHEMA_VERSION,
            workspace_id: Uuid::new_v4(),
            edit_root,
            publish_root,
            documents: BTreeMap::new(),
            document_types: BTreeMap::new(),
            confidentiality_types: BTreeMap::new(),
            confidentiality_policies: BTreeMap::new(),
            identity_source: None,
            identity_cache: BTreeMap::new(),
            workflow_policies: BTreeMap::new(),
        };
        workspace.save()?;
        Ok(workspace)
    }

    pub fn open(edit_root: &Path) -> Result<Self> {
        let requested_edit_root = canonical_existing_directory(edit_root, "edit root")?;
        let metadata_path = workspace_metadata_path(&requested_edit_root);
        if !metadata_path.is_file() {
            return Err(DmsError::WorkspaceNotFound(metadata_path));
        }
        let content = fs::read_to_string(&metadata_path).map_err(|source| DmsError::Io {
            path: metadata_path.clone(),
            source,
        })?;
        let mut value: serde_json::Value =
            serde_json::from_str(&content).map_err(|source| DmsError::InvalidMetadata {
                path: metadata_path.clone(),
                source,
            })?;
        let found = value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            .and_then(|version| u32::try_from(version).ok())
            .unwrap_or_default();
        let migrated = matches!(found, 1 | 2);
        if found == 1 {
            migrate_v1_catalogues(&mut value)?;
        }
        if migrated {
            value["schema_version"] = serde_json::Value::from(SCHEMA_VERSION);
        } else if found != SCHEMA_VERSION {
            return Err(DmsError::UnsupportedSchema {
                expected: SCHEMA_VERSION,
                found,
            });
        }
        let workspace: Self =
            serde_json::from_value(value).map_err(|source| DmsError::InvalidMetadata {
                path: workspace_metadata_path(&requested_edit_root),
                source,
            })?;
        if workspace.edit_root != requested_edit_root {
            return Err(DmsError::EditRootMismatch {
                stored: workspace.edit_root,
                requested: requested_edit_root,
            });
        }
        workspace.validate()?;
        if migrated {
            let serialized = serde_json::to_vec_pretty(&workspace).map_err(|source| {
                DmsError::InvalidMetadata {
                    path: metadata_path.clone(),
                    source,
                }
            })?;
            let verified: Self = serde_json::from_slice(&serialized).map_err(|source| {
                DmsError::InvalidMetadata {
                    path: metadata_path.clone(),
                    source,
                }
            })?;
            verified.validate()?;
            retain_migration_backup(&metadata_path, found, content.as_bytes())?;
            verified.save()?;
            return Ok(verified);
        }
        Ok(workspace)
    }

    pub fn save(&self) -> Result<()> {
        self.validate()?;
        let metadata_directory = self.edit_root.join(METADATA_DIRECTORY);
        let metadata_path = workspace_metadata_path(&self.edit_root);
        let serialized =
            serde_json::to_vec_pretty(self).map_err(|source| DmsError::InvalidMetadata {
                path: metadata_path.clone(),
                source,
            })?;
        let temporary_path = metadata_directory.join(format!(".workspace-{}.tmp", Uuid::new_v4()));
        let write_result = (|| -> Result<()> {
            let mut temporary =
                fs::File::create(&temporary_path).map_err(|source| DmsError::Io {
                    path: temporary_path.clone(),
                    source,
                })?;
            temporary
                .write_all(&serialized)
                .map_err(|source| DmsError::Io {
                    path: temporary_path.clone(),
                    source,
                })?;
            temporary.sync_all().map_err(|source| DmsError::Io {
                path: temporary_path.clone(),
                source,
            })?;
            fs::rename(&temporary_path, &metadata_path).map_err(|source| DmsError::Io {
                path: metadata_path.clone(),
                source,
            })?;
            Ok(())
        })();
        if write_result.is_err() && temporary_path.exists() {
            let _ = fs::remove_file(&temporary_path);
        }
        write_result
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(DmsError::UnsupportedSchema {
                expected: SCHEMA_VERSION,
                found: self.schema_version,
            });
        }
        canonical_existing_directory(&self.edit_root, "stored edit root")?;
        canonical_existing_directory(&self.publish_root, "stored publish root")?;

        let mut document_numbers = BTreeMap::new();
        for (key, document) in &self.documents {
            if key != &document.id {
                return Err(DmsError::DocumentIdMismatch {
                    key: *key,
                    stored: document.id,
                });
            }
            validate_relative_source_path(&document.relative_path)?;
            if document.control.title.trim().is_empty() {
                return Err(DmsError::EmptyTitle);
            }
            if let Some(number) = document.control.document_number.as_deref() {
                let normalized = normalized_required(number, "document number")?;
                let key = normalized.to_lowercase();
                if let Some(existing) = document_numbers.insert(key, document.id) {
                    return Err(DmsError::DuplicateDocumentNumber(format!(
                        "{normalized} (documents {existing} and {})",
                        document.id
                    )));
                }
            }
            if let Some(type_id) = document.control.document_type.as_deref() {
                self.require_enabled_document_type(type_id)?;
            }
            for note in &document.notes {
                if note.body.trim().is_empty() {
                    return Err(DmsError::EmptyNote);
                }
            }
        }
        self.validate_policies()?;
        Ok(())
    }

    pub fn documents(&self) -> Vec<&Document> {
        let mut documents = self.documents.values().collect::<Vec<_>>();
        documents.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        documents
    }

    pub fn document(&self, document_id: Uuid) -> Result<&Document> {
        self.documents
            .get(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))
    }

    pub fn add_document(&mut self, source_path: &Path) -> Result<Document> {
        let (absolute_path, relative_path) = self.resolve_source_path(source_path)?;
        if !is_supported_source(&absolute_path) {
            return Err(DmsError::UnsupportedSource(absolute_path));
        }
        if let Some(existing_id) = self
            .documents
            .values()
            .find_map(|document| (document.relative_path == relative_path).then_some(document.id))
        {
            let existing = self
                .documents
                .get_mut(&existing_id)
                .expect("document ID came from the same map");
            if existing.source_state == SourceState::Registered {
                return Err(DmsError::DocumentAlreadyRegistered(relative_path));
            }
            existing.source_state = SourceState::Registered;
            return Ok(existing.clone());
        }
        let title = source_title(&absolute_path)?;
        let document = Document {
            id: Uuid::new_v4(),
            relative_path,
            lifecycle: Lifecycle::Draft,
            source_state: SourceState::Registered,
            control: DocumentControl {
                title,
                document_number: None,
                document_type: None,
                owner: None,
            },
            confidentiality_override: None,
            workflow_overrides: DocumentWorkflowOverrides::default(),
            notes: Vec::new(),
        };
        self.documents.insert(document.id, document.clone());
        Ok(document)
    }

    pub fn update_control(&mut self, document_id: Uuid, update: ControlUpdate) -> Result<Document> {
        if let Some(number) = update.document_number.as_ref() {
            self.ensure_document_number_available(document_id, number.as_deref())?;
        }
        if let Some(Some(type_id)) = update.document_type.as_ref() {
            self.require_enabled_document_type(type_id)?;
        }
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?;
        if let Some(title) = update.title {
            document.control.title = normalized_required(&title, "title")?;
        }
        if let Some(number) = update.document_number {
            document.control.document_number = normalized_optional(number.as_deref());
        }
        if let Some(document_type) = update.document_type {
            document.control.document_type = normalized_optional(document_type.as_deref());
        }
        if let Some(owner) = update.owner {
            document.control.owner = normalized_optional(owner.as_deref());
        }
        Ok(document.clone())
    }

    pub fn add_note(
        &mut self,
        document_id: Uuid,
        body: &str,
        author: Option<&str>,
    ) -> Result<Note> {
        let note = Note {
            id: Uuid::new_v4(),
            body: normalized_required(body, "note body")?,
            author: normalized_optional(author).unwrap_or_else(default_author),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?;
        document.notes.push(note.clone());
        Ok(note)
    }

    pub fn notes(&self, document_id: Uuid) -> Result<Vec<Note>> {
        let document = self.document(document_id)?;
        let mut notes = document.notes.clone();
        notes.sort_by_key(|note| std::cmp::Reverse(note.created_at));
        Ok(notes)
    }

    pub fn edit_note(&mut self, document_id: Uuid, note_id: Uuid, body: &str) -> Result<Note> {
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?;
        let note = document
            .notes
            .iter_mut()
            .find(|note| note.id == note_id)
            .ok_or(DmsError::NoteNotFound {
                document_id,
                note_id,
            })?;
        note.body = normalized_required(body, "note body")?;
        note.updated_at = Utc::now();
        Ok(note.clone())
    }

    pub fn remove_note(&mut self, document_id: Uuid, note_id: Uuid) -> Result<()> {
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?;
        let index = document
            .notes
            .iter()
            .position(|note| note.id == note_id)
            .ok_or(DmsError::NoteNotFound {
                document_id,
                note_id,
            })?;
        document.notes.remove(index);
        Ok(())
    }

    fn resolve_source_path(&self, source_path: &Path) -> Result<(PathBuf, PathBuf)> {
        let requested_path = if source_path.is_absolute() {
            source_path.to_path_buf()
        } else {
            self.edit_root.join(source_path)
        };
        let absolute_path = fs::canonicalize(&requested_path).map_err(|source| DmsError::Io {
            path: requested_path,
            source,
        })?;
        if !absolute_path.is_file() {
            return Err(DmsError::ExpectedFile(absolute_path));
        }
        let relative_path = absolute_path
            .strip_prefix(&self.edit_root)
            .map_err(|_| DmsError::OutsideEditRoot(absolute_path.clone()))?
            .to_path_buf();
        validate_relative_source_path(&relative_path)?;
        if is_metadata_path(&relative_path) {
            return Err(DmsError::MetadataPath(relative_path));
        }
        let filename = absolute_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| DmsError::UnsupportedSource(absolute_path.clone()))?;
        if filename.starts_with("~$") {
            return Err(DmsError::OfficeTemporaryFile(absolute_path));
        }
        Ok((absolute_path, relative_path))
    }

    fn ensure_document_number_available(
        &self,
        document_id: Uuid,
        candidate: Option<&str>,
    ) -> Result<()> {
        let Some(candidate) = normalized_optional(candidate) else {
            return Ok(());
        };
        let candidate_key = candidate.to_lowercase();
        if self.documents.iter().any(|(existing_id, document)| {
            *existing_id != document_id
                && document
                    .control
                    .document_number
                    .as_deref()
                    .and_then(|number| normalized_optional(Some(number)))
                    .is_some_and(|number| number.to_lowercase() == candidate_key)
        }) {
            return Err(DmsError::DuplicateDocumentNumber(candidate));
        }
        Ok(())
    }
}

fn workspace_metadata_path(edit_root: &Path) -> PathBuf {
    edit_root.join(METADATA_DIRECTORY).join(METADATA_FILENAME)
}

fn retain_migration_backup(metadata_path: &Path, from: u32, content: &[u8]) -> Result<()> {
    let backup_path = metadata_path.with_file_name(format!("workspace.v{from}.json.bak"));
    if backup_path.exists() {
        let existing = fs::read(&backup_path).map_err(|source| DmsError::Io {
            path: backup_path.clone(),
            source,
        })?;
        if existing == content {
            return Ok(());
        }
        return Err(DmsError::MigrationBackupConflict(backup_path));
    }
    let mut backup = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup_path)
        .map_err(|source| DmsError::Io {
            path: backup_path.clone(),
            source,
        })?;
    backup.write_all(content).map_err(|source| DmsError::Io {
        path: backup_path.clone(),
        source,
    })?;
    backup.sync_all().map_err(|source| DmsError::Io {
        path: backup_path,
        source,
    })
}

fn migrate_v1_catalogues(value: &mut serde_json::Value) -> Result<()> {
    let mut document_types = serde_json::Map::new();
    if let Some(documents) = value
        .get("documents")
        .and_then(serde_json::Value::as_object)
    {
        for document in documents.values() {
            let Some(type_id) = document
                .get("control")
                .and_then(|control| control.get("document_type"))
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            catalogues::validate_portable_id(type_id, DmsError::InvalidDocumentTypeId)?;
            document_types.insert(
                type_id.to_owned(),
                serde_json::json!({ "id": type_id, "label": type_id, "enabled": true }),
            );
        }
    }
    value["document_types"] = serde_json::Value::Object(document_types);
    Ok(())
}

fn canonical_existing_directory(path: &Path, label: &str) -> Result<PathBuf> {
    if !path.is_dir() {
        return Err(DmsError::ExpectedDirectory(label.to_owned()));
    }
    fs::canonicalize(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_relative_source_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(DmsError::InvalidRelativePath(path.to_path_buf()));
    }
    Ok(())
}

mod relative_path_serde {
    use std::path::{Component, Path, PathBuf};

    use serde::{de::Error as _, ser::Error as _, Deserialize, Deserializer, Serializer};

    use super::validate_relative_source_path;

    pub fn serialize<S>(path: &Path, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        validate_relative_source_path(path).map_err(S::Error::custom)?;
        let components = path
            .components()
            .filter_map(|component| match component {
                Component::Normal(value) => Some(value),
                Component::CurDir => None,
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => None,
            })
            .map(|component| {
                let component = component
                    .to_str()
                    .ok_or_else(|| S::Error::custom("relative source path is not valid UTF-8"))?;
                if component.contains('\\') {
                    return Err(S::Error::custom(
                        "relative source path cannot contain backslashes",
                    ));
                }
                Ok(component)
            })
            .collect::<std::result::Result<Vec<_>, S::Error>>()?;
        serializer.serialize_str(&components.join("/"))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let components = value.split('/').collect::<Vec<_>>();
        if components
            .iter()
            .any(|component| component.is_empty() || *component == "." || *component == "..")
            || value.contains('\\')
        {
            return Err(D::Error::custom(
                "relative source path must use clean '/'-separated components",
            ));
        }
        let path = components.iter().collect::<PathBuf>();
        validate_relative_source_path(&path).map_err(D::Error::custom)?;
        Ok(path)
    }
}

fn is_metadata_path(path: &Path) -> bool {
    path.components()
        .next()
        .and_then(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .is_some_and(|first| first == METADATA_DIRECTORY)
}

fn is_supported_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "md" | "docx" | "xlsx" | "pptx"
            )
        })
}

fn source_title(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| DmsError::UnsupportedSource(path.to_path_buf()))?;
    normalized_required(stem, "source file stem")
}

fn normalized_required(value: &str, field: &str) -> Result<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return if field == "note body" {
            Err(DmsError::EmptyNote)
        } else {
            Err(DmsError::EmptyTitle)
        };
    }
    Ok(normalized.to_owned())
}

fn normalized_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn default_author() -> String {
    env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .ok()
        .and_then(|author| normalized_optional(Some(&author)))
        .unwrap_or_else(|| "local operator".to_owned())
}
