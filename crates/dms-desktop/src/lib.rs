use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

use dms_core::{
    BackupOutcome, ClaudeAssistancePayload, ClaudeAssistancePolicy, Document, DocumentControl,
    EffectiveConfidentiality, EffectiveWorkflowRoles, LibraryEntry, LibraryFolder,
    LibraryFolderNode, Lifecycle, Note, PeriodicReview, PeriodicReviewMarker,
    ReleaseVerificationStatus, SourceState, Workspace,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

mod assistance;
pub mod export;

use assistance::ClaudeDesktopApp;

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

#[derive(Clone, Debug, Serialize)]
pub struct DocumentNotes {
    pub document_id: Uuid,
    pub title: String,
    pub document_number: Option<String>,
    pub notes: Vec<Note>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReleaseRow {
    pub document_id: Uuid,
    pub document_title: String,
    pub release_id: Uuid,
    pub version: String,
    pub relative_pdf_path: String,
    pub pdf_digest: String,
    pub confidentiality_id: String,
    pub confidentiality_label: String,
    pub workflow_chain_head: String,
    pub approval_chain_head: Option<String>,
    pub released_at: String,
    pub withdrawn: bool,
    pub verification: ReleaseVerificationStatus,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReleaseMaintenance {
    pub rows: Vec<ReleaseRow>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ClaudeAssistanceAvailability {
    pub available: bool,
    pub policy_enabled: bool,
    pub confidentiality_permitted: bool,
    pub app_installed: bool,
    pub reason: String,
    pub privacy_notice: String,
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
fn initialize_workspace(
    edit_root: String,
    publish_root: String,
    confirmed: bool,
) -> Result<WorkspaceSummary, String> {
    if !confirmed {
        return Err("workspace initialization requires explicit confirmation".to_owned());
    }
    Workspace::init(Path::new(&edit_root), Path::new(&publish_root))
        .map_err(|error| error.to_string())?;
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

#[tauri::command]
fn load_document_notes(edit_root: String, document_id: Uuid) -> Result<DocumentNotes, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    document_notes(&workspace, document_id)
}

#[tauri::command]
fn load_releases(edit_root: String) -> Result<ReleaseMaintenance, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    release_maintenance(&workspace)
}

#[tauri::command]
fn verify_release(
    edit_root: String,
    document_id: Uuid,
    release_id: Uuid,
) -> Result<ReleaseMaintenance, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .verify_release(document_id, release_id)
        .map_err(|error| error.to_string())?;
    release_maintenance(&workspace)
}

#[tauri::command]
fn verify_all_releases(edit_root: String) -> Result<ReleaseMaintenance, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .verify_all_releases()
        .map_err(|error| error.to_string())?;
    release_maintenance(&workspace)
}

#[tauri::command]
fn load_periodic_reviews(edit_root: String) -> Result<Vec<PeriodicReviewMarker>, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .periodic_review_markers(chrono::Utc::now().date_naive())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn start_periodic_review(edit_root: String, document_id: Uuid) -> Result<PeriodicReview, String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .start_periodic_review(document_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn backup_workspace(edit_root: String, archive_path: String) -> Result<BackupOutcome, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .backup_workspace(Path::new(&archive_path))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn load_claude_assistance_policy(edit_root: String) -> Result<ClaudeAssistancePolicy, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    Ok(workspace.claude_assistance_policy().clone())
}

#[tauri::command]
fn configure_claude_assistance(
    edit_root: String,
    enabled: bool,
    allowed_confidentiality_type_ids: Vec<String>,
    max_payload_chars: usize,
) -> Result<(), String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let allowed = allowed_confidentiality_type_ids
        .into_iter()
        .collect::<BTreeSet<_>>();
    workspace
        .configure_claude_assistance(enabled, allowed, max_payload_chars)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn claude_assistance_availability(
    edit_root: String,
    document_id: Uuid,
) -> Result<ClaudeAssistanceAvailability, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    assistance_availability(
        &workspace,
        document_id,
        ClaudeDesktopApp::locate().is_some(),
    )
}

#[tauri::command]
fn preview_claude_assistance(
    edit_root: String,
    document_id: Uuid,
) -> Result<ClaudeAssistancePayload, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let availability = assistance_availability(
        &workspace,
        document_id,
        ClaudeDesktopApp::locate().is_some(),
    )?;
    if !availability.available {
        return Err(availability.reason);
    }
    workspace
        .prepare_claude_assistance(document_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn launch_claude_assistance(
    edit_root: String,
    document_id: Uuid,
    payload_digest: String,
    confirmed: bool,
) -> Result<(), String> {
    if !confirmed {
        return Err(
            "explicit confirmation is required before handing data to Claude Desktop".to_owned(),
        );
    }
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let payload = workspace
        .prepare_claude_assistance(document_id)
        .map_err(|error| error.to_string())?;
    if payload.payload_digest != payload_digest {
        return Err(
            "the previewed Claude Desktop payload has changed; preview it again".to_owned(),
        );
    }
    ClaudeDesktopApp::locate()
        .ok_or_else(|| "Claude Desktop is not installed in a supported location".to_owned())?
        .launch()
}

#[tauri::command]
fn add_document_note(
    edit_root: String,
    document_id: Uuid,
    body: String,
    author: Option<String>,
) -> Result<DocumentNotes, String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .add_note(document_id, &body, author.as_deref())
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_notes(&workspace, document_id)
}

#[tauri::command]
fn edit_document_note(
    edit_root: String,
    document_id: Uuid,
    note_id: Uuid,
    body: String,
) -> Result<DocumentNotes, String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .edit_note(document_id, note_id, &body)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_notes(&workspace, document_id)
}

#[tauri::command]
fn remove_document_note(
    edit_root: String,
    document_id: Uuid,
    note_id: Uuid,
) -> Result<DocumentNotes, String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .remove_note(document_id, note_id)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_notes(&workspace, document_id)
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

fn document_notes(workspace: &Workspace, document_id: Uuid) -> Result<DocumentNotes, String> {
    let document = workspace
        .document(document_id)
        .map_err(|error| error.to_string())?;
    Ok(DocumentNotes {
        document_id,
        title: document.control.title.clone(),
        document_number: document.control.document_number.clone(),
        notes: workspace
            .notes(document_id)
            .map_err(|error| error.to_string())?,
    })
}

fn assistance_availability(
    workspace: &Workspace,
    document_id: Uuid,
    app_installed: bool,
) -> Result<ClaudeAssistanceAvailability, String> {
    workspace
        .document(document_id)
        .map_err(|error| error.to_string())?;
    let policy = workspace.claude_assistance_policy();
    const PRIVACY_NOTICE: &str = "Claude Desktop is a local client, but model processing may send the displayed payload to Anthropic. Only the exact previewed payload is copied after explicit confirmation.";
    if !policy.enabled {
        return Ok(ClaudeAssistanceAvailability {
            available: false,
            policy_enabled: false,
            confidentiality_permitted: false,
            app_installed,
            reason: "Claude Desktop assistance is disabled for this workspace".to_owned(),
            privacy_notice: PRIVACY_NOTICE.to_owned(),
        });
    }
    let confidentiality = workspace
        .effective_confidentiality(document_id)
        .map_err(|error| error.to_string())?;
    let confidentiality_permitted = policy
        .allowed_confidentiality_type_ids
        .contains(&confidentiality.type_id);
    let has_release = workspace
        .releases(document_id)
        .map_err(|error| error.to_string())?
        .iter()
        .any(|release| !release.withdrawn);
    let reason = if !confidentiality_permitted {
        "workspace policy does not permit this confidentiality type"
    } else if !app_installed {
        "Claude Desktop is not installed in a supported location"
    } else if !has_release {
        "a current released PDF is required before evaluating changes"
    } else {
        "Available"
    };
    Ok(ClaudeAssistanceAvailability {
        available: confidentiality_permitted && app_installed && has_release,
        policy_enabled: true,
        confidentiality_permitted,
        app_installed,
        reason: reason.to_owned(),
        privacy_notice: PRIVACY_NOTICE.to_owned(),
    })
}

fn release_maintenance(workspace: &Workspace) -> Result<ReleaseMaintenance, String> {
    let mut rows = Vec::new();
    for document in workspace.documents() {
        for release in workspace
            .releases(document.id)
            .map_err(|error| error.to_string())?
        {
            let verification = workspace
                .verify_release(document.id, release.id)
                .map_err(|error| error.to_string())?;
            rows.push(ReleaseRow {
                document_id: document.id,
                document_title: document.control.title.clone(),
                release_id: release.id,
                version: release.version.to_string(),
                relative_pdf_path: path_text(&release.relative_pdf_path),
                pdf_digest: release.pdf_digest.clone(),
                confidentiality_id: release.confidentiality.type_id.clone(),
                confidentiality_label: release.confidentiality.label.clone(),
                workflow_chain_head: release.workflow_chain_head.clone(),
                approval_chain_head: release.approval_chain_head.clone(),
                released_at: release.released_at.to_rfc3339(),
                withdrawn: release.withdrawn,
                verification: verification.status,
            });
        }
    }
    rows.sort_by(|left, right| {
        right
            .released_at
            .cmp(&left.released_at)
            .then_with(|| left.document_title.cmp(&right.document_title))
    });
    Ok(ReleaseMaintenance { rows })
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
            initialize_workspace,
            open_workspace,
            load_library,
            search_library,
            add_library_documents,
            unregister_library_documents,
            reassociate_library_document,
            load_document_selection,
            load_document_notes,
            load_releases,
            verify_release,
            verify_all_releases,
            load_periodic_reviews,
            start_periodic_review,
            backup_workspace,
            load_claude_assistance_policy,
            configure_claude_assistance,
            claude_assistance_availability,
            preview_claude_assistance,
            launch_claude_assistance,
            add_document_note,
            edit_document_note,
            remove_document_note
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
    fn desktop_adapter_refuses_unconfirmed_workspace_initialization_without_touching_roots() {
        let directory = tempfile::tempdir().unwrap();
        let edit_root = directory.path().join("edit");
        let publish_root = directory.path().join("publish");

        let error = initialize_workspace(
            edit_root.to_string_lossy().into_owned(),
            publish_root.to_string_lossy().into_owned(),
            false,
        )
        .unwrap_err();

        assert!(error.contains("explicit confirmation"));
        assert!(!edit_root.exists());
        assert!(!publish_root.exists());
    }

    #[test]
    fn desktop_adapter_initializes_and_reopens_confirmed_dual_root_workspace() {
        let directory = tempfile::tempdir().unwrap();
        let edit_root = directory.path().join("edit");
        let publish_root = directory.path().join("publish");
        fs::create_dir(&edit_root).unwrap();
        let root = edit_root.to_string_lossy().into_owned();

        let initialized = initialize_workspace(
            root.clone(),
            publish_root.to_string_lossy().into_owned(),
            true,
        )
        .unwrap();
        let reopened = open_workspace(root).unwrap();

        assert_eq!(reopened, initialized);
        assert!(edit_root.join(".dms/workspace.json").is_file());
        assert!(publish_root.is_dir());
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

    #[test]
    fn disabled_or_missing_claude_desktop_is_reported_without_blocking_workspace_use() {
        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        let source = edit_root.path().join("Handbook.md");
        fs::write(&source, "# Handbook").unwrap();
        let document = workspace.add_document(&source).unwrap();
        workspace.save().unwrap();

        let disabled = assistance_availability(&workspace, document.id, false).unwrap();
        assert!(!disabled.available);
        assert!(!disabled.policy_enabled);
        assert!(disabled.reason.contains("disabled"));

        workspace
            .configure_confidentiality_type("internal", "Internal", true)
            .unwrap();
        workspace
            .set_confidentiality_policy(".", "internal")
            .unwrap();
        workspace
            .configure_claude_assistance(
                true,
                ["internal".to_owned()].into_iter().collect(),
                dms_core::DEFAULT_CLAUDE_PAYLOAD_LIMIT,
            )
            .unwrap();
        let missing = assistance_availability(&workspace, document.id, false).unwrap();
        assert!(!missing.available);
        assert!(missing.policy_enabled);
        assert!(missing.confidentiality_permitted);
        assert!(missing.reason.contains("not installed"));
    }

    #[test]
    fn launch_refuses_unconfirmed_handoff_before_any_workspace_or_app_access() {
        let error = launch_claude_assistance(
            "missing".to_owned(),
            Uuid::nil(),
            "digest".to_owned(),
            false,
        )
        .unwrap_err();
        assert!(error.contains("explicit confirmation"));
    }

    #[test]
    fn desktop_note_commands_persist_create_edit_delete_and_newest_first_order() {
        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        let source = edit_root.path().join("Notes.md");
        fs::write(&source, "# Notes").unwrap();
        let document = workspace.add_document(&source).unwrap();
        workspace.save().unwrap();
        let root = edit_root.path().to_string_lossy().into_owned();

        let first = add_document_note(
            root.clone(),
            document.id,
            "First note".into(),
            Some("Raphael".into()),
        )
        .unwrap();
        let first_id = first.notes[0].id;
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second =
            add_document_note(root.clone(), document.id, "Second note".into(), None).unwrap();
        let second_id = second.notes[0].id;
        assert_eq!(
            second.notes.iter().map(|note| note.id).collect::<Vec<_>>(),
            vec![second_id, first_id]
        );

        let edited = edit_document_note(
            root.clone(),
            document.id,
            first_id,
            "Edited first note".into(),
        )
        .unwrap();
        assert_eq!(
            edited
                .notes
                .iter()
                .find(|note| note.id == first_id)
                .unwrap()
                .body,
            "Edited first note"
        );
        let remaining = remove_document_note(root.clone(), document.id, second_id).unwrap();
        assert_eq!(remaining.notes.len(), 1);

        let reopened = load_document_notes(root, document.id).unwrap();
        assert_eq!(reopened.notes.len(), 1);
        assert_eq!(reopened.notes[0].id, first_id);
        assert_eq!(reopened.notes[0].body, "Edited first note");
    }

    #[test]
    fn desktop_maintenance_commands_list_releases_and_create_a_manifest_backup() {
        let directory = tempfile::tempdir().unwrap();
        let edit_root = directory.path().join("edit");
        let publish_root = directory.path().join("publish");
        fs::create_dir_all(&edit_root).unwrap();
        fs::create_dir_all(&publish_root).unwrap();
        Workspace::init(&edit_root, &publish_root).unwrap();
        let root = edit_root.to_string_lossy().into_owned();

        assert!(load_releases(root.clone()).unwrap().rows.is_empty());
        assert!(load_periodic_reviews(root.clone()).unwrap().is_empty());
        let archive = directory.path().join("workspace.zip");
        let outcome = backup_workspace(root, archive.to_string_lossy().into_owned()).unwrap();
        assert_eq!(outcome.entry_count, 1);
        assert_eq!(outcome.manifest_digest.len(), 64);
        assert!(archive.is_file());
    }
}
