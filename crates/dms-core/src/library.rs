use std::{
    cmp::Ordering,
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    is_metadata_path, is_supported_source, DmsError, Document, DocumentControl, Lifecycle, Result,
    Workspace, METADATA_DIRECTORY,
};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceState {
    #[default]
    Registered,
    Unregistered,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LibraryEntryKind {
    Folder,
    File,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LibraryMembership {
    InLibrary { document_id: Uuid },
    NotInLibrary,
    Unsupported,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryDocumentSummary {
    pub id: Uuid,
    pub lifecycle: Lifecycle,
    pub control: DocumentControl,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub name: String,
    #[serde(with = "library_path_serde")]
    pub relative_path: PathBuf,
    pub kind: LibraryEntryKind,
    pub membership: Option<LibraryMembership>,
    pub document: Option<LibraryDocumentSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryFolderNode {
    pub name: String,
    #[serde(with = "library_path_serde")]
    pub relative_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryFolder {
    #[serde(with = "library_path_serde")]
    pub relative_path: PathBuf,
    #[serde(with = "optional_library_path_serde")]
    pub parent: Option<PathBuf>,
    pub entries: Vec<LibraryEntry>,
}

impl Workspace {
    pub fn library_tree(&self) -> Result<Vec<LibraryFolderNode>> {
        let mut folders = vec![LibraryFolderNode {
            name: self
                .edit_root
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Library")
                .to_owned(),
            relative_path: PathBuf::from("."),
        }];
        self.collect_library_folders(&self.edit_root, &mut folders)?;
        folders.sort_by(|left, right| path_order(&left.relative_path, &right.relative_path));
        Ok(folders)
    }

    pub fn library_folder(&self, relative_folder: &Path) -> Result<LibraryFolder> {
        let registered = self.registered_document_index();
        self.library_folder_with_documents(relative_folder, &registered)
    }

    fn library_folder_with_documents(
        &self,
        relative_folder: &Path,
        registered: &HashMap<PathBuf, &Document>,
    ) -> Result<LibraryFolder> {
        let (folder, relative_folder) = self.resolve_library_folder(relative_folder)?;
        let mut entries = Vec::new();
        for entry in fs::read_dir(&folder).map_err(|source| DmsError::Io {
            path: folder.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| DmsError::Io {
                path: folder.clone(),
                source,
            })?;
            let name = entry
                .file_name()
                .to_str()
                .ok_or_else(|| DmsError::InvalidLibraryEntry(entry.path()))?
                .to_owned();
            if (relative_folder == Path::new(".") && name == METADATA_DIRECTORY)
                || name.starts_with("~$")
            {
                continue;
            }
            let file_type = entry.file_type().map_err(|source| DmsError::Io {
                path: entry.path(),
                source,
            })?;
            let relative_path = relative_join(&relative_folder, &name);
            if file_type.is_dir() {
                entries.push(LibraryEntry {
                    name,
                    relative_path,
                    kind: LibraryEntryKind::Folder,
                    membership: None,
                    document: None,
                });
            } else if file_type.is_file() {
                entries.push(self.file_entry(name, relative_path, registered));
            }
        }
        entries.sort_by(entry_order);
        let parent = if relative_folder == Path::new(".") {
            None
        } else {
            relative_folder
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map(Path::to_path_buf)
                .or_else(|| Some(PathBuf::from(".")))
        };
        Ok(LibraryFolder {
            relative_path: relative_folder,
            parent,
            entries,
        })
    }

    pub fn search_library(&self, relative_folder: &Path, query: &str) -> Result<Vec<LibraryEntry>> {
        let (folder, _) = self.resolve_library_folder(relative_folder)?;
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let mut results = Vec::new();
        let registered = self.registered_document_index();
        self.collect_search_results(&folder, &query, &registered, &mut results)?;
        results.sort_by(|left, right| path_order(&left.relative_path, &right.relative_path));
        Ok(results)
    }

    pub fn add_documents(&mut self, source_paths: &[PathBuf]) -> Result<Vec<Document>> {
        let mut candidate = self.clone();
        let mut added = Vec::with_capacity(source_paths.len());
        for source_path in source_paths {
            added.push(candidate.add_document(source_path)?);
        }
        *self = candidate;
        Ok(added)
    }

    pub fn unregister_document(&mut self, document_id: Uuid) -> Result<Document> {
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?;
        document.source_state = SourceState::Unregistered;
        Ok(document.clone())
    }

    pub fn unregister_documents(&mut self, document_ids: &[Uuid]) -> Result<Vec<Document>> {
        let mut candidate = self.clone();
        let mut unregistered = Vec::with_capacity(document_ids.len());
        for document_id in document_ids {
            unregistered.push(candidate.unregister_document(*document_id)?);
        }
        *self = candidate;
        Ok(unregistered)
    }

    pub fn reassociate_document(
        &mut self,
        document_id: Uuid,
        source_path: &Path,
    ) -> Result<Document> {
        let (absolute_path, relative_path) = self.resolve_source_path(source_path)?;
        if !is_supported_source(&absolute_path) {
            return Err(DmsError::UnsupportedSource(absolute_path));
        }
        if self
            .documents
            .values()
            .any(|document| document.id != document_id && document.relative_path == relative_path)
        {
            return Err(DmsError::DocumentAlreadyRegistered(relative_path));
        }
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(DmsError::DocumentNotFound(document_id))?;
        document.relative_path = relative_path;
        document.source_state = SourceState::Registered;
        Ok(document.clone())
    }

    pub fn document_permalink(&self, document_id: Uuid) -> Result<String> {
        self.document(document_id)?;
        Ok(format!(
            "dms://open?workspace={}&document={document_id}",
            self.workspace_id
        ))
    }

    fn collect_library_folders(
        &self,
        folder: &Path,
        folders: &mut Vec<LibraryFolderNode>,
    ) -> Result<()> {
        for entry in fs::read_dir(folder).map_err(|source| DmsError::Io {
            path: folder.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| DmsError::Io {
                path: folder.to_path_buf(),
                source,
            })?;
            let file_type = entry.file_type().map_err(|source| DmsError::Io {
                path: entry.path(),
                source,
            })?;
            if !file_type.is_dir() {
                continue;
            }
            let path = entry.path();
            let relative_path = path
                .strip_prefix(&self.edit_root)
                .map_err(|_| DmsError::OutsideEditRoot(path.clone()))?
                .to_path_buf();
            if is_metadata_path(&relative_path) {
                continue;
            }
            let name = entry
                .file_name()
                .to_str()
                .ok_or_else(|| DmsError::InvalidLibraryEntry(path.clone()))?
                .to_owned();
            folders.push(LibraryFolderNode {
                name,
                relative_path,
            });
            self.collect_library_folders(&path, folders)?;
        }
        Ok(())
    }

    fn collect_search_results(
        &self,
        folder: &Path,
        query: &str,
        registered: &HashMap<PathBuf, &Document>,
        results: &mut Vec<LibraryEntry>,
    ) -> Result<()> {
        let relative_folder = folder
            .strip_prefix(&self.edit_root)
            .map_err(|_| DmsError::OutsideEditRoot(folder.to_path_buf()))?;
        let listing = self.library_folder_with_documents(
            if relative_folder.as_os_str().is_empty() {
                Path::new(".")
            } else {
                relative_folder
            },
            registered,
        )?;
        for entry in listing.entries {
            if entry.kind == LibraryEntryKind::Folder {
                self.collect_search_results(
                    &self.edit_root.join(&entry.relative_path),
                    query,
                    registered,
                    results,
                )?;
                continue;
            }
            let matches_file = entry.name.to_lowercase().contains(query)
                || path_text(&entry.relative_path)
                    .to_lowercase()
                    .contains(query);
            let matches_control = entry.document.as_ref().is_some_and(|document| {
                document.control.title.to_lowercase().contains(query)
                    || document
                        .control
                        .document_number
                        .as_deref()
                        .is_some_and(|number| number.to_lowercase().contains(query))
            });
            if matches_file || matches_control {
                results.push(entry);
            }
        }
        Ok(())
    }

    fn resolve_library_folder(&self, relative_folder: &Path) -> Result<(PathBuf, PathBuf)> {
        if relative_folder.is_absolute()
            || relative_folder.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(DmsError::InvalidLibraryFolder(
                relative_folder.to_path_buf(),
            ));
        }
        let normalized =
            if relative_folder.as_os_str().is_empty() || relative_folder == Path::new(".") {
                PathBuf::from(".")
            } else {
                relative_folder.to_path_buf()
            };
        if is_metadata_path(&normalized) {
            return Err(DmsError::InvalidLibraryFolder(normalized));
        }
        let requested = if normalized == Path::new(".") {
            self.edit_root.clone()
        } else {
            self.edit_root.join(&normalized)
        };
        let canonical = fs::canonicalize(&requested).map_err(|source| DmsError::Io {
            path: requested,
            source,
        })?;
        if !canonical.is_dir() || !canonical.starts_with(&self.edit_root) {
            return Err(DmsError::InvalidLibraryFolder(normalized));
        }
        let relative = canonical
            .strip_prefix(&self.edit_root)
            .map_err(|_| DmsError::InvalidLibraryFolder(normalized.clone()))?
            .to_path_buf();
        Ok((
            canonical,
            if relative.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                relative.to_path_buf()
            },
        ))
    }

    fn registered_document_index(&self) -> HashMap<PathBuf, &Document> {
        self.documents
            .values()
            .filter(|document| document.source_state == SourceState::Registered)
            .map(|document| (document.relative_path.clone(), document))
            .collect()
    }

    fn file_entry(
        &self,
        name: String,
        relative_path: PathBuf,
        registered: &HashMap<PathBuf, &Document>,
    ) -> LibraryEntry {
        let document = registered.get(&relative_path).copied();
        let membership = if let Some(document) = document {
            LibraryMembership::InLibrary {
                document_id: document.id,
            }
        } else if is_supported_source(&relative_path) {
            LibraryMembership::NotInLibrary
        } else {
            LibraryMembership::Unsupported
        };
        LibraryEntry {
            name,
            relative_path,
            kind: LibraryEntryKind::File,
            membership: Some(membership),
            document: document.map(|document| LibraryDocumentSummary {
                id: document.id,
                lifecycle: document.lifecycle,
                control: document.control.clone(),
            }),
        }
    }
}

fn relative_join(folder: &Path, name: &str) -> PathBuf {
    if folder == Path::new(".") {
        PathBuf::from(name)
    } else {
        folder.join(name)
    }
}

fn entry_order(left: &LibraryEntry, right: &LibraryEntry) -> Ordering {
    let kind = match (left.kind, right.kind) {
        (LibraryEntryKind::Folder, LibraryEntryKind::File) => Ordering::Less,
        (LibraryEntryKind::File, LibraryEntryKind::Folder) => Ordering::Greater,
        _ => Ordering::Equal,
    };
    kind.then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        .then_with(|| left.name.cmp(&right.name))
}

fn path_order(left: &Path, right: &Path) -> Ordering {
    path_text(left)
        .to_lowercase()
        .cmp(&path_text(right).to_lowercase())
        .then_with(|| path_text(left).cmp(&path_text(right)))
}

fn path_text(path: &Path) -> String {
    if path == Path::new(".") {
        return ".".to_owned();
    }
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

mod library_path_serde {
    use std::path::{Component, Path, PathBuf};

    use serde::{de::Error as _, Deserialize, Deserializer, Serializer};

    use super::path_text;

    pub fn serialize<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&path_text(path))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == "." {
            return Ok(PathBuf::from("."));
        }
        let path = PathBuf::from(&value);
        if value.is_empty()
            || value.contains('\\')
            || path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(D::Error::custom(
                "library path must be clean, relative, and '/'-separated",
            ));
        }
        Ok(path)
    }
}

mod optional_library_path_serde {
    use std::path::PathBuf;

    use serde::{Deserialize, Deserializer, Serializer};

    use super::{library_path_serde, path_text};

    pub fn serialize<S>(path: &Option<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match path {
            Some(path) => serializer.serialize_some(&path_text(path)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        value
            .map(|value| {
                let deserializer = serde::de::value::StringDeserializer::<D::Error>::new(value);
                library_path_serde::deserialize(deserializer)
            })
            .transpose()
    }
}
