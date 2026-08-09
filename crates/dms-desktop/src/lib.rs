use std::{fs, path::Path};

use dms_core::Workspace;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if std::env::var_os("DMS_DESKTOP_SMOKE").is_some() {
                app.handle().exit(0);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_preferences,
            save_preferences,
            open_workspace
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
}
