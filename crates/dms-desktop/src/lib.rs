use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use dms_core::{
    Document, DocumentControl, EffectiveConfidentiality, EffectiveWorkflowRoles, LibraryEntry,
    LibraryFolder, LibraryFolderNode, Lifecycle, SourceState, Workspace,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub mod export;

const PREFERENCES_FILENAME: &str = "preferences.json";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct Preferences {
    pub sidebar_expanded: bool,
    pub saved_views: Vec<SavedView>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            sidebar_expanded: true,
            saved_views: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SavedView {
    pub id: String,
    pub workspace_id: String,
    pub destination: String,
    pub task: String,
    pub label: String,
    #[serde(default)]
    pub document_id: Option<String>,
    #[serde(default)]
    pub route_state: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceSummary {
    pub workspace_id: String,
    pub edit_root: String,
    pub publish_root: String,
    pub document_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct LibrarySnapshot {
    pub tree: Vec<LibraryFolderNode>,
    pub folder: LibraryFolder,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentSelection {
    pub document_id: Uuid,
    pub source_name: String,
    pub relative_path: String,
    pub folder: String,
    pub source_exists: bool,
    pub source_state: SourceState,
    pub lifecycle: Lifecycle,
    pub control: DocumentControl,
    pub effective_confidentiality: Option<EffectiveConfidentiality>,
    pub effective_workflow_roles: Option<EffectiveWorkflowRoles>,
    pub permalink: String,
}

#[tauri::command]
fn load_preferences(app: AppHandle) -> Result<Preferences, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("cannot resolve the app-config directory: {error}"))?
        .join(PREFERENCES_FILENAME);
    load_preferences_at(&path)
}

#[tauri::command]
fn save_preferences(app: AppHandle, preferences: Preferences) -> Result<(), String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("cannot resolve the app-config directory: {error}"))?
        .join(PREFERENCES_FILENAME);
    save_preferences_at(&path, &preferences)
}

#[tauri::command]
fn open_workspace(edit_root: String) -> Result<WorkspaceSummary, String> {
    workspace_summary(Path::new(&edit_root))
}

#[tauri::command]
fn load_library(edit_root: String, folder: String) -> Result<LibrarySnapshot, String> {
    library_snapshot(Path::new(&edit_root), Path::new(&folder))
}

#[tauri::command]
fn search_library(
    edit_root: String,
    folder: String,
    query: String,
) -> Result<Vec<LibraryEntry>, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .search_library(Path::new(&folder), &query)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn add_library_documents(edit_root: String, paths: Vec<String>) -> Result<Vec<Document>, String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let paths = paths.into_iter().map(PathBuf::from).collect::<Vec<_>>();
    let documents = workspace
        .add_documents(&paths)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    Ok(documents)
}

#[tauri::command]
fn unregister_library_documents(
    edit_root: String,
    document_ids: Vec<Uuid>,
) -> Result<Vec<Document>, String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let documents = workspace
        .unregister_documents(&document_ids)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    Ok(documents)
}

#[tauri::command]
fn reassociate_library_document(
    edit_root: String,
    document_id: Uuid,
    path: String,
) -> Result<Document, String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let document = workspace
        .reassociate_document(document_id, Path::new(&path))
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    Ok(document)
}

#[tauri::command]
fn load_document_selection(
    edit_root: String,
    document_id: Uuid,
) -> Result<DocumentSelection, String> {
    document_selection(Path::new(&edit_root), document_id)
}

fn load_preferences_at(path: &Path) -> Result<Preferences, String> {
    if !path.exists() {
        return Ok(Preferences::default());
    }

    let content = fs::read_to_string(path)
        .map_err(|error| format!("cannot read preferences at {}: {error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("preferences at {} are invalid: {error}", path.display()))
}

fn save_preferences_at(path: &Path, preferences: &Preferences) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("preferences path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "cannot create app-config directory {}: {error}",
            parent.display()
        )
    })?;
    let content = serde_json::to_vec_pretty(preferences)
        .map_err(|error| format!("cannot encode preferences: {error}"))?;
    fs::write(path, content)
        .map_err(|error| format!("cannot write preferences at {}: {error}", path.display()))
}

fn workspace_summary(edit_root: &Path) -> Result<WorkspaceSummary, String> {
    let workspace = Workspace::open(edit_root).map_err(|error| error.to_string())?;
    Ok(WorkspaceSummary {
        workspace_id: workspace.workspace_id.to_string(),
        edit_root: workspace.edit_root.to_string_lossy().into_owned(),
        publish_root: workspace.publish_root.to_string_lossy().into_owned(),
        document_count: workspace.documents().len(),
    })
}

fn library_snapshot(edit_root: &Path, folder: &Path) -> Result<LibrarySnapshot, String> {
    let workspace = Workspace::open(edit_root).map_err(|error| error.to_string())?;
    Ok(LibrarySnapshot {
        tree: workspace
            .library_tree()
            .map_err(|error| error.to_string())?,
        folder: workspace
            .library_folder(folder)
            .map_err(|error| error.to_string())?,
    })
}

fn document_selection(edit_root: &Path, document_id: Uuid) -> Result<DocumentSelection, String> {
    let workspace = Workspace::open(edit_root).map_err(|error| error.to_string())?;
    let document = workspace
        .document(document_id)
        .map_err(|error| error.to_string())?;
    let source_name = document
        .relative_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "document source name is not valid UTF-8".to_owned())?
        .to_owned();
    let folder = document
        .relative_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(path_text)
        .unwrap_or_else(|| ".".to_owned());
    Ok(DocumentSelection {
        document_id,
        source_name,
        relative_path: path_text(&document.relative_path),
        folder,
        source_exists: workspace.edit_root.join(&document.relative_path).is_file(),
        source_state: document.source_state,
        lifecycle: document.lifecycle,
        control: document.control.clone(),
        effective_confidentiality: workspace.effective_confidentiality(document_id).ok(),
        effective_workflow_roles: workspace.effective_workflow_roles(document_id).ok(),
        permalink: workspace
            .document_permalink(document_id)
            .map_err(|error| error.to_string())?,
    })
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if std::env::var_os("DMS_DESKTOP_SMOKE").is_some() {
                app.handle().exit(0);
            } else if std::env::var_os("DMS_DESKTOP_EXPORT_SMOKE").is_some() {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    let code = match export::platform_pdf_smoke(handle.clone()) {
                        Ok(()) => 0,
                        Err(error) => {
                            eprintln!("DMS Desktop PDF export smoke failed: {error}");
                            1
                        }
                    };
                    handle.exit(code);
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_preferences,
            save_preferences,
            open_workspace,
            load_library,
            search_library,
            add_library_documents,
            unregister_library_documents,
            reassociate_library_document,
            load_document_selection
        ])
        .run(tauri::generate_context!())
        .expect("DMS Desktop failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_preferences_use_expanded_sidebar_and_no_saved_views() {
        let directory = tempfile::tempdir().unwrap();
        let preferences = load_preferences_at(&directory.path().join("missing.json")).unwrap();

        assert_eq!(preferences, Preferences::default());
    }

    #[test]
    fn preferences_round_trip_outside_workspace_metadata() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config").join(PREFERENCES_FILENAME);
        let preferences = Preferences {
            sidebar_expanded: false,
            saved_views: vec![SavedView {
                id: "ws-1:Library".into(),
                workspace_id: "ws-1".into(),
                destination: "Library".into(),
                task: "Library".into(),
                label: "Library · policies/HR".into(),
                document_id: None,
                route_state: [("folder".into(), "policies/HR".into())]
                    .into_iter()
                    .collect(),
            }],
        };

        save_preferences_at(&path, &preferences).unwrap();

        assert_eq!(load_preferences_at(&path).unwrap(), preferences);
    }

    #[test]
    fn desktop_adapter_opens_the_shared_core_workspace() {
        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        let workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();

        let summary = workspace_summary(edit_root.path()).unwrap();

        assert_eq!(summary.workspace_id, workspace.workspace_id.to_string());
        assert_eq!(summary.document_count, 0);
        assert_eq!(summary.edit_root, workspace.edit_root.to_string_lossy());
        assert_eq!(
            summary.publish_root,
            workspace.publish_root.to_string_lossy()
        );
    }

    #[test]
    fn desktop_adapter_lists_and_selects_library_documents() {
        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        fs::create_dir_all(edit_root.path().join("Policies/Empty")).unwrap();
        let source = edit_root.path().join("Policies/Handbook.md");
        fs::write(&source, "# Handbook").unwrap();
        let document = workspace.add_document(&source).unwrap();
        workspace.save().unwrap();

        let snapshot = library_snapshot(edit_root.path(), Path::new("Policies")).unwrap();
        assert_eq!(snapshot.folder.entries.len(), 2);
        assert!(snapshot
            .tree
            .iter()
            .any(|folder| folder.relative_path == Path::new("Policies/Empty")));

        let selection = document_selection(edit_root.path(), document.id).unwrap();
        assert_eq!(selection.source_name, "Handbook.md");
        assert_eq!(selection.relative_path, "Policies/Handbook.md");
        assert_eq!(selection.document_id, document.id);
        assert!(selection.permalink.ends_with(&document.id.to_string()));
    }
}
