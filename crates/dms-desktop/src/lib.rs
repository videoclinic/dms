use std::{
    collections::BTreeSet,
    env, fs,
    path::{Component, Path, PathBuf},
    sync::Mutex,
};

use chrono::NaiveDate;
use dms_core::{
    AuditReportRecord, AuditReportRequest, AuditReportVerificationStatus, AuthenticatedActor,
    BackupOutcome, CandidateRequest, ClaudeAssistancePolicy, ClaudeAssistancePreview,
    ConfidentialityPolicy, ConfidentialityType, ControlUpdate, DeliveryAttempt, DmsError, Document,
    DocumentControl, DocumentType, EffectiveConfidentiality, EffectiveWorkflowRoles,
    EntraIdentitySource, EntraPerson, GraphClient, LibraryEntry, LibraryFolder, LibraryFolderNode,
    Lifecycle, LocalLifecycleActions, MarkdownTemplateAsset, MarkdownTemplateValidation, Note,
    NotificationClient, NotificationKind, NotificationMessage, NotificationSettings,
    NotificationTransport, OwnerReference, PdfExporter, PeriodicReview, PeriodicReviewMarker,
    PeriodicReviewResult, PermalinkTarget, PersonSnapshot, PolicyFolder, ReleaseCandidate,
    ReleaseVerificationStatus, RestoreOutcome, RestoreRequest, ReviewDecision, RoleUpdate,
    SmtpSettings, SourceState, TargetSelection, Version, WorkflowEvent, WorkflowPolicyAssignment,
    WorkflowVerification, Workspace, WorkspaceLock, WorkspaceLockStatus, METADATA_DIRECTORY,
};
use lettre::message::Mailbox;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

mod assistance;
pub mod export;
mod graph;
pub mod notify;

use assistance::ClaudeDesktopApp;

const PREFERENCES_FILENAME: &str = "preferences.json";
const GLOBAL_SETTINGS_FILENAME: &str = "global-settings.json";
const RECENT_LIBRARIES_LIMIT: usize = 10;
const REASSOCIATE_SOURCE_FILTER_NAME: &str = "Supported drafts";
const REASSOCIATE_SOURCE_FILTER_EXTENSIONS: &[&str] = &["md", "docx", "xlsx", "pptx"];
const DESKTOP_REASSOCIATE_RULE_LOCATION: &str = "must be a regular file under the workspace edit root (not outside, not a directory, not under .dms)";
const DESKTOP_REASSOCIATE_RULE_FORMAT: &str = "must be a supported draft (.md, .docx, .xlsx, .pptx), not an Office lock/temp sidecar, and not the workspace Word-template asset";
const DESKTOP_REASSOCIATE_RULE_UNREGISTERED: &str =
    "must not already be another registered library document";

struct DesktopIntegrations {
    graph: Mutex<graph::MicrosoftGraphClient>,
    approver_actor: Mutex<Option<AuthenticatedActor>>,
}

impl Default for DesktopIntegrations {
    fn default() -> Self {
        Self {
            graph: Mutex::new(graph::MicrosoftGraphClient::production(None)),
            approver_actor: Mutex::new(None),
        }
    }
}

#[cfg(test)]
struct UnavailableGraphClient;

#[cfg(test)]
impl GraphClient for UnavailableGraphClient {
    fn tenant_id(&self) -> std::result::Result<Uuid, String> {
        Err("live Microsoft Graph integration is not configured".to_owned())
    }

    fn direct_user_members(
        &mut self,
        _source: &EntraIdentitySource,
    ) -> std::result::Result<Vec<EntraPerson>, String> {
        Err("live Microsoft Graph integration is not configured".to_owned())
    }

    fn authenticated_actor(
        &mut self,
        _source: &EntraIdentitySource,
    ) -> std::result::Result<dms_core::AuthenticatedActor, String> {
        Err("live interactive Microsoft Entra sign-in is not configured".to_owned())
    }
}

#[cfg(test)]
struct UnavailableNotificationClient;

#[cfg(test)]
impl NotificationClient for UnavailableNotificationClient {
    fn send(
        &mut self,
        _settings: &NotificationSettings,
        _message: &dms_core::NotificationMessage,
    ) -> std::result::Result<dms_core::DeliveryReceipt, String> {
        Err("notification delivery must not be reached".to_owned())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct Preferences {
    pub sidebar_expanded: bool,
    pub saved_views: Vec<SavedView>,
    #[serde(default)]
    pub recent_libraries: Vec<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            sidebar_expanded: true,
            saved_views: Vec::new(),
            recent_libraries: Vec::new(),
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

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DesktopPermalinkResolution {
    pub workspace: WorkspaceSummary,
    pub document_id: Uuid,
    pub title: String,
    pub document_number: Option<String>,
    pub folder: String,
    pub target: String,
    pub review_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceConfiguration {
    pub workspace: WorkspaceSummary,
    pub markdown_template: Option<MarkdownTemplateAsset>,
    pub markdown_template_validation: Option<MarkdownTemplateValidation>,
    pub default_review_interval_months: u32,
    pub document_types: Vec<DocumentType>,
    pub confidentiality_types: Vec<ConfidentialityType>,
    pub confidentiality_policies: Vec<ConfidentialityPolicy>,
    pub policy_folders: Vec<PolicyFolder>,
    pub identity_source: Option<EntraIdentitySource>,
    pub eligible_people: Vec<EntraPerson>,
    pub workflow_policies: Vec<WorkflowPolicyAssignment>,
    pub notification_settings: Option<NotificationSettings>,
    pub global_entra_configuration: GlobalEntraConfiguration,
    pub smtp_credential_configured: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SmtpTestResult {
    pub recipient: String,
    pub response_code: Option<u16>,
    pub detail: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
struct GlobalSettings {
    entra_client_id: String,
    entra_tenant_id: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct GlobalEntraConfiguration {
    pub client_id: String,
    pub tenant_id: String,
    pub client_id_environment_managed: bool,
    pub tenant_id_environment_managed: bool,
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
    pub current_owner: serde_json::Value,
    pub requires_identity_handover: bool,
    pub review_schedule: DocumentReviewSchedule,
    pub document_types: Vec<DocumentType>,
    pub confidentiality_types: Vec<ConfidentialityType>,
    pub confidentiality_override: Option<String>,
    pub effective_confidentiality: Option<EffectiveConfidentiality>,
    pub effective_workflow_roles: Option<EffectiveWorkflowRoles>,
    pub current_release: Option<CurrentReleaseSelection>,
    pub active_candidate: Option<ReleaseCandidate>,
    pub retryable_decision_candidate: Option<ReleaseCandidate>,
    pub retryable_minor_publication: Option<ReleaseCandidate>,
    pub eligible_people: Vec<EntraPerson>,
    pub lifecycle_actions: LocalLifecycleActions,
    pub workflow_events: Vec<WorkflowEvent>,
    pub workflow_verification: WorkflowVerification,
    pub permalink: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CurrentReleaseSelection {
    pub release_id: Uuid,
    pub version: String,
    pub relative_pdf_path: String,
    pub pdf_exists: bool,
    pub effective_date: Option<NaiveDate>,
    pub profile: Option<ReleaseProfileSelection>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReleaseProfileSelection {
    pub title: String,
    pub document_number: Option<String>,
    pub document_type: Option<String>,
    pub owner: Option<PersonSnapshot>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DocumentReviewSchedule {
    pub workspace_interval_months: u32,
    pub interval_months: Option<u32>,
    pub exemption_reason: Option<String>,
    pub next_due_date: Option<NaiveDate>,
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
    pub effective_date: Option<NaiveDate>,
    pub profile: Option<ReleaseProfileSelection>,
    pub workflow_chain_head: String,
    pub approval_chain_head: Option<String>,
    pub released_at: String,
    pub withdrawn: bool,
    pub orphaned: bool,
    pub verification: ReleaseVerificationStatus,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReleaseMaintenance {
    pub rows: Vec<ReleaseRow>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AuditReportRow {
    #[serde(flatten)]
    pub report: AuditReportRecord,
    pub verification: AuditReportVerificationStatus,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AuditReportSnapshot {
    pub rows: Vec<AuditReportRow>,
    pub evidence_chain: WorkflowVerification,
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
async fn select_directory(app: AppHandle) -> Result<Option<String>, String> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| format!("cannot resolve the OS user's home directory: {error}"))?;
    let selection = app
        .dialog()
        .file()
        .set_title("Choose directory")
        .set_directory(home)
        .blocking_pick_folder();

    match selection {
        Some(path) => path
            .as_path()
            .map(|path| Some(path.to_string_lossy().into_owned()))
            .ok_or_else(|| "the selected directory is not a local filesystem path".to_owned()),
        None => Ok(None),
    }
}

#[tauri::command]
fn open_workspace(edit_root: String) -> Result<WorkspaceSummary, String> {
    workspace_summary(Path::new(&edit_root))
}

#[tauri::command]
fn resolve_registered_permalink(
    app: AppHandle,
    uri: String,
) -> Result<DesktopPermalinkResolution, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("cannot resolve the app-config directory: {error}"))?
        .join(PREFERENCES_FILENAME);
    let preferences = load_preferences_at(&path)?;
    resolve_registered_permalink_from(&preferences, &uri)
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
fn load_workspace_configuration(
    app: AppHandle,
    edit_root: String,
) -> Result<WorkspaceConfiguration, String> {
    workspace_configuration(&app, Path::new(&edit_root))
}

#[tauri::command]
async fn choose_markdown_template(
    app: AppHandle,
    edit_root: String,
) -> Result<Option<WorkspaceConfiguration>, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let selection = app
        .dialog()
        .file()
        .set_title("Choose Markdown Word template")
        .set_directory(workspace.edit_root)
        .add_filter("Word document", &["docx"])
        .blocking_pick_file();
    let Some(selection) = selection else {
        return Ok(None);
    };
    let source_path = selection
        .as_path()
        .ok_or_else(|| "the selected Word template is not a local filesystem path".to_owned())?;
    import_markdown_template_from_path(Path::new(&edit_root), source_path).map(Some)
}

fn path_key(path: &Path) -> String {
    path.iter()
        .map(|component| component.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn absolute_from_edit_root(edit_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        edit_root.join(path)
    }
}

fn relative_to_edit_root(edit_root: &Path, path: &Path) -> Option<PathBuf> {
    let absolute = absolute_from_edit_root(edit_root, path);
    if let (Ok(canonical), Ok(root)) = (absolute.canonicalize(), edit_root.canonicalize()) {
        return canonical.strip_prefix(root).ok().map(Path::to_path_buf);
    }
    if path.is_absolute() {
        path.strip_prefix(edit_root).ok().map(Path::to_path_buf)
    } else {
        Some(path.to_path_buf())
    }
}

fn display_reassociate_path(edit_root: &Path, selected: &Path) -> String {
    relative_to_edit_root(edit_root, selected)
        .map(|relative| path_key(&relative))
        .unwrap_or_else(|| selected.to_string_lossy().into_owned())
}

fn reassociate_source_picker_start_dir(edit_root: &Path, stored_locator: &str) -> PathBuf {
    let stored = Path::new(stored_locator);
    let stored_absolute = absolute_from_edit_root(edit_root, stored);
    let parent = stored_absolute.parent().unwrap_or(edit_root);
    let Ok(root) = edit_root.canonicalize() else {
        return edit_root.to_path_buf();
    };
    match parent.canonicalize() {
        Ok(directory) if directory.starts_with(&root) && directory.is_dir() => directory,
        _ => edit_root.to_path_buf(),
    }
}

fn desktop_reassociate_location_ok(edit_root: &Path, path: &Path) -> bool {
    let absolute = absolute_from_edit_root(edit_root, path);
    let Ok(canonical) = absolute.canonicalize() else {
        return false;
    };
    if !canonical.is_file() {
        return false;
    }
    let Ok(root) = edit_root.canonicalize() else {
        return false;
    };
    let Ok(relative) = canonical.strip_prefix(&root) else {
        return false;
    };
    !relative.components().next().is_some_and(
        |component| matches!(component, Component::Normal(name) if name == METADATA_DIRECTORY),
    )
}

fn desktop_reassociate_format_ok(workspace: &Workspace, path: &Path) -> bool {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if filename.starts_with("~$") {
        return false;
    }
    let supported = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "md" | "docx" | "xlsx" | "pptx"
            )
        });
    if !supported {
        return false;
    }
    let Some(relative) = relative_to_edit_root(&workspace.edit_root, path) else {
        return true;
    };
    workspace
        .markdown_template()
        .is_none_or(|template| path_key(&template.relative_path) != path_key(&relative))
}

fn desktop_reassociate_unregistered_ok(
    workspace: &Workspace,
    document_id: Uuid,
    path: &Path,
) -> bool {
    let Some(relative) = relative_to_edit_root(&workspace.edit_root, path) else {
        return true;
    };
    !workspace.documents().iter().any(|document| {
        document.id != document_id
            && document.source_state == SourceState::Registered
            && path_key(&document.relative_path) == path_key(&relative)
    })
}

fn desktop_reassociate_rule_errors(
    workspace: &Workspace,
    document_id: Uuid,
    path: &Path,
) -> Vec<&'static str> {
    let mut failed = Vec::new();
    if !desktop_reassociate_location_ok(&workspace.edit_root, path) {
        failed.push(DESKTOP_REASSOCIATE_RULE_LOCATION);
    }
    if !desktop_reassociate_format_ok(workspace, path) {
        failed.push(DESKTOP_REASSOCIATE_RULE_FORMAT);
    }
    if !desktop_reassociate_unregistered_ok(workspace, document_id, path) {
        failed.push(DESKTOP_REASSOCIATE_RULE_UNREGISTERED);
    }
    failed
}

fn format_desktop_reassociate_error(failed: &[&str]) -> String {
    let mut message = String::from("Cannot reassociate this path:\n");
    for rule in failed {
        message.push_str("- ");
        message.push_str(rule);
        message.push('\n');
    }
    message.push_str(
        "The selected file must be a supported unregistered source file inside the edit root.",
    );
    message
}

#[tauri::command]
async fn choose_reassociate_source(
    app: AppHandle,
    edit_root: String,
    stored_path: String,
) -> Result<Option<String>, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let selection = app
        .dialog()
        .file()
        .set_title("Choose source file")
        .set_directory(reassociate_source_picker_start_dir(
            &workspace.edit_root,
            &stored_path,
        ))
        .add_filter(
            REASSOCIATE_SOURCE_FILTER_NAME,
            REASSOCIATE_SOURCE_FILTER_EXTENSIONS,
        )
        .blocking_pick_file();
    let Some(selection) = selection else {
        return Ok(None);
    };
    let source_path = selection
        .as_path()
        .ok_or_else(|| "the selected source file is not a local filesystem path".to_owned())?;
    Ok(Some(display_reassociate_path(
        &workspace.edit_root,
        source_path,
    )))
}

fn import_markdown_template_from_path(
    edit_root: &Path,
    source_path: &Path,
) -> Result<WorkspaceConfiguration, String> {
    mutate_workspace_configuration(edit_root, |workspace| {
        workspace.import_markdown_template(source_path).map(|_| ())
    })
}

#[tauri::command]
fn remove_markdown_template(
    edit_root: String,
    confirmed: bool,
) -> Result<WorkspaceConfiguration, String> {
    if !confirmed {
        return Err(
            "removing the Markdown Word template requires explicit confirmation".to_owned(),
        );
    }
    mutate_workspace_configuration(Path::new(&edit_root), |workspace| {
        workspace.remove_markdown_template();
        Ok(())
    })
}

#[tauri::command]
fn configure_global_entra(
    app: AppHandle,
    client_id: String,
    tenant_id: String,
    state: State<'_, DesktopIntegrations>,
) -> Result<GlobalEntraConfiguration, String> {
    let path = global_settings_path(&app)?;
    let mut settings = load_global_settings_at(&path)?;
    let effective = effective_global_entra_configuration(&settings)?;
    if !effective.client_id_environment_managed {
        settings.entra_client_id = client_id.trim().to_owned();
    }
    if !effective.tenant_id_environment_managed {
        settings.entra_tenant_id = tenant_id.trim().to_owned();
    }
    let effective = effective_global_entra_configuration(&settings)?;
    let runtime = runtime_entra_configuration(&effective)?;
    save_global_settings_at(&path, &settings)?;
    *state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())? =
        graph::MicrosoftGraphClient::production(runtime);
    Ok(effective)
}

#[tauri::command]
fn configure_default_review_interval(
    edit_root: String,
    months: u32,
) -> Result<WorkspaceConfiguration, String> {
    mutate_workspace_configuration(Path::new(&edit_root), |workspace| {
        workspace.configure_default_review_interval(months)
    })
}

#[tauri::command]
fn configure_document_type(
    edit_root: String,
    id: String,
    label: String,
    enabled: bool,
) -> Result<WorkspaceConfiguration, String> {
    mutate_workspace_configuration(Path::new(&edit_root), |workspace| {
        workspace
            .configure_document_type(&id, &label, enabled)
            .map(|_| ())
    })
}

#[tauri::command]
fn configure_confidentiality_type(
    edit_root: String,
    id: String,
    label: String,
    enabled: bool,
    workspace_default: bool,
) -> Result<WorkspaceConfiguration, String> {
    mutate_workspace_configuration(Path::new(&edit_root), |workspace| {
        workspace
            .configure_confidentiality_type(&id, &label, enabled)
            .map(|_| ())?;
        if workspace_default {
            workspace.set_confidentiality_policy(".", &id).map(|_| ())?;
        }
        Ok(())
    })
}

#[tauri::command]
fn set_confidentiality_policy(
    edit_root: String,
    folder: String,
    type_id: String,
) -> Result<WorkspaceConfiguration, String> {
    mutate_workspace_configuration(Path::new(&edit_root), |workspace| {
        workspace
            .set_confidentiality_policy(&folder, &type_id)
            .map(|_| ())
    })
}

#[tauri::command]
fn remove_confidentiality_policy(
    edit_root: String,
    folder: String,
) -> Result<WorkspaceConfiguration, String> {
    mutate_workspace_configuration(Path::new(&edit_root), |workspace| {
        workspace.remove_confidentiality_policy(&folder)
    })
}

#[tauri::command]
fn set_workflow_policy(
    edit_root: String,
    folder: String,
    editor: String,
    approver: String,
    state: State<'_, DesktopIntegrations>,
) -> Result<WorkspaceConfiguration, String> {
    let editor = configuration_role_update(&editor)?;
    let approver = configuration_role_update(&approver)?;
    let mut graph = state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?;
    set_workflow_policy_with(
        Path::new(&edit_root),
        &folder,
        editor,
        approver,
        &mut *graph,
    )
}

fn set_workflow_policy_with<G: GraphClient + ?Sized>(
    edit_root: &Path,
    folder: &str,
    editor: RoleUpdate,
    approver: RoleUpdate,
    graph: &mut G,
) -> Result<WorkspaceConfiguration, String> {
    let mut workspace = Workspace::open(edit_root).map_err(|error| error.to_string())?;
    workspace
        .refresh_eligible_people(graph)
        .map_err(|error| error.to_string())?;
    workspace
        .update_workflow_policy(folder, editor, approver)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    workspace_configuration_from(&workspace)
}

#[tauri::command]
fn begin_identity_source_sign_in(
    group_id: String,
    state: State<'_, DesktopIntegrations>,
) -> Result<graph::DeviceLoginChallenge, String> {
    let group_id = Uuid::parse_str(&group_id)
        .map_err(|_| "group ID must be a Microsoft Entra group UUID".to_owned())?;
    state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?
        .begin_identity_source_setup(group_id)
}

#[tauri::command]
fn complete_identity_source_sign_in(
    challenge_id: Uuid,
    state: State<'_, DesktopIntegrations>,
) -> Result<graph::IdentitySourcePreview, String> {
    state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?
        .complete_identity_source_setup(challenge_id)
}

#[tauri::command]
fn begin_approver_sign_in(
    edit_root: String,
    state: State<'_, DesktopIntegrations>,
) -> Result<graph::DeviceLoginChallenge, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace.identity_source().ok_or_else(|| {
        "configure a Microsoft Entra identity source before signing in for approval".to_owned()
    })?;
    let mut graph = state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?;
    let tenant_id = graph.tenant_id()?;
    graph.begin_approver_sign_in(tenant_id)
}

#[tauri::command]
fn complete_approver_sign_in(
    challenge_id: Uuid,
    state: State<'_, DesktopIntegrations>,
) -> Result<dms_core::AuthenticatedActor, String> {
    let actor = state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?
        .complete_approver_sign_in(challenge_id)?;
    *state
        .approver_actor
        .lock()
        .map_err(|_| "interactive approver sign-in state is unavailable".to_owned())? =
        Some(actor.clone());
    Ok(actor)
}

#[tauri::command]
fn apply_identity_source(
    edit_root: String,
    preview_id: Uuid,
    initial_editor_id: Option<Uuid>,
    initial_approver_id: Option<Uuid>,
    confirmed: bool,
    state: State<'_, DesktopIntegrations>,
) -> Result<WorkspaceConfiguration, String> {
    if !confirmed {
        return Err(
            "applying a Microsoft Entra identity source requires explicit confirmation".to_owned(),
        );
    }
    state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?
        .apply_identity_source_preview(
            preview_id,
            |_tenant_id, _tenant_display, group_id, group_label, people| {
                mutate_workspace_configuration(Path::new(&edit_root), |workspace| {
                    apply_identity_source_to_workspace(
                        workspace,
                        group_id,
                        &group_label,
                        people,
                        initial_editor_id,
                        initial_approver_id,
                    )
                })
            },
        )
}

fn apply_identity_source_to_workspace(
    workspace: &mut Workspace,
    group_id: Uuid,
    group_label: &str,
    people: Vec<EntraPerson>,
    initial_editor_id: Option<Uuid>,
    initial_approver_id: Option<Uuid>,
) -> dms_core::Result<()> {
    let successful_empty_initialization =
        workspace.identity_source().is_none() && people.is_empty();
    let initial_roles = if workspace.identity_source().is_none() && !successful_empty_initialization
    {
        Some((
            initial_editor_id.ok_or(DmsError::RequiredRootWorkflowPolicy)?,
            initial_approver_id.ok_or(DmsError::RequiredRootWorkflowPolicy)?,
        ))
    } else {
        None
    };

    workspace.replace_identity_source(group_id, group_label, people)?;
    if let Some((editor_id, approver_id)) = initial_roles {
        workspace.update_workflow_policy(
            ".",
            RoleUpdate::replace(editor_id),
            RoleUpdate::replace(approver_id),
        )?;
    }
    Ok(())
}

#[tauri::command]
fn refresh_identity_source(
    edit_root: String,
    state: State<'_, DesktopIntegrations>,
) -> Result<WorkspaceConfiguration, String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let mut graph = state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?;
    workspace
        .refresh_eligible_people(&mut *graph)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    workspace_configuration_from(&workspace)
}

#[tauri::command]
fn configure_notifications(
    edit_root: String,
    transport: String,
    relay_host: String,
    relay_port: u16,
    login_user: String,
    from_mailbox: String,
    smtp_app_password: String,
) -> Result<WorkspaceConfiguration, String> {
    let credentials = notify::OsCredentialStore;
    configure_notifications_with_credentials(
        &edit_root,
        NotificationConfigurationInput {
            transport,
            relay_host,
            relay_port,
            login_user,
            from_mailbox,
            smtp_app_password,
        },
        &credentials,
    )
}

struct NotificationConfigurationInput {
    transport: String,
    relay_host: String,
    relay_port: u16,
    login_user: String,
    from_mailbox: String,
    smtp_app_password: String,
}

fn configure_notifications_with_credentials<C: notify::CredentialStore>(
    edit_root: &str,
    input: NotificationConfigurationInput,
    credentials: &C,
) -> Result<WorkspaceConfiguration, String> {
    let (transport, smtp) = match input.transport.trim() {
        "smtp" => (
            NotificationTransport::Smtp,
            Some(SmtpSettings {
                relay_host: input.relay_host,
                relay_port: input.relay_port,
                login_user: input.login_user,
                from_mailbox: input.from_mailbox,
            }),
        ),
        "mailto" => (NotificationTransport::Mailto, None),
        value => return Err(format!("unknown notification transport: {value}")),
    };
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    workspace
        .configure_notifications(transport, smtp)
        .map_err(|error| error.to_string())?;
    match transport {
        NotificationTransport::Smtp if !input.smtp_app_password.trim().is_empty() => {
            credentials.set_smtp_password(workspace.workspace_id, &input.smtp_app_password)?
        }
        NotificationTransport::Smtp
            if !credentials.smtp_password_exists(workspace.workspace_id)? =>
        {
            return Err(
                "SMTP configuration requires a Microsoft 365 app password in the OS credential store"
                    .to_owned(),
            );
        }
        NotificationTransport::Mailto => {
            credentials.delete_smtp_password(workspace.workspace_id)?
        }
        _ => {}
    }
    workspace.save().map_err(|error| error.to_string())?;
    workspace_configuration_from_with_global_and_credentials(
        &workspace,
        effective_global_entra_configuration(&GlobalSettings::default())?,
        credentials,
    )
}

#[tauri::command]
fn test_smtp_notification(edit_root: String) -> Result<SmtpTestResult, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let credentials = notify::OsCredentialStore;
    let mut notifier = notify::production_notifier(workspace.workspace_id, false);
    test_smtp_notification_with(&workspace, &credentials, &mut notifier)
}

fn test_smtp_notification_with<C: notify::CredentialStore, N: NotificationClient>(
    workspace: &Workspace,
    credentials: &C,
    notifier: &mut N,
) -> Result<SmtpTestResult, String> {
    let settings = workspace
        .notification_settings()
        .ok_or_else(|| "SMTP test requires saved notification settings".to_owned())?;
    if settings.transport != NotificationTransport::Smtp {
        return Err("SMTP test is unavailable for mailto notification transport".to_owned());
    }
    let smtp = settings
        .smtp
        .as_ref()
        .ok_or_else(|| "SMTP test requires saved relay settings".to_owned())?;
    if !credentials
        .smtp_password_exists(workspace.workspace_id)
        .map_err(|_| "Cannot verify the saved SMTP credential.".to_owned())?
    {
        return Err("SMTP test requires a configured app password".to_owned());
    }
    let from = smtp
        .from_mailbox
        .parse::<Mailbox>()
        .map_err(|error| format!("invalid saved SMTP From address: {error}"))?;
    let recipient = from.email.to_string();
    let message = NotificationMessage {
        kind: NotificationKind::ReviewRequest,
        recipient: recipient.clone(),
        subject: "DMS SMTP configuration test".to_owned(),
        body: "This message confirms that the saved DMS SMTP configuration can deliver email. It contains no document or workflow content.".to_owned(),
        mailto_uri: String::new(),
    };
    let receipt = notifier.send(settings, &message).map_err(|_| {
        "SMTP test delivery failed. Verify the saved relay, identity, From mailbox, and app password."
            .to_owned()
    })?;
    Ok(SmtpTestResult {
        recipient,
        response_code: receipt.response_code,
        detail: "SMTP test message accepted by the configured relay.".to_owned(),
    })
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
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    if workspace
        .sync_all_registered_lifecycles()
        .map_err(|error| error.to_string())?
        > 0
    {
        workspace.save().map_err(|error| error.to_string())?;
    }
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
    let failed = desktop_reassociate_rule_errors(&workspace, document_id, Path::new(&path));
    if !failed.is_empty() {
        return Err(format_desktop_reassociate_error(&failed));
    }
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
fn update_document_control(
    edit_root: String,
    document_id: Uuid,
    title: String,
    document_number: String,
    document_type: String,
    owner_object_id: Uuid,
    state: State<'_, DesktopIntegrations>,
) -> Result<DocumentSelection, String> {
    let mut graph = state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?;
    update_document_control_with(
        &edit_root,
        document_id,
        title,
        document_number,
        document_type,
        owner_object_id,
        &mut *graph,
    )
}

fn update_document_control_with<G: GraphClient + ?Sized>(
    edit_root: &str,
    document_id: Uuid,
    title: String,
    document_number: String,
    document_type: String,
    owner_object_id: Uuid,
    graph: &mut G,
) -> Result<DocumentSelection, String> {
    let optional_text = |value: String| {
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    };
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    workspace
        .refresh_eligible_people(graph)
        .map_err(|error| error.to_string())?;
    let binding_id = workspace
        .identity_source()
        .ok_or_else(|| "owner assignment requires an identity source".to_owned())?
        .binding_id;
    workspace
        .update_control(
            document_id,
            ControlUpdate {
                title: Some(title),
                document_number: Some(optional_text(document_number)),
                document_type: Some(optional_text(document_type)),
                owner: Some(Some(OwnerReference {
                    binding_id,
                    object_id: owner_object_id,
                })),
                ..ControlUpdate::default()
            },
        )
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_selection(Path::new(edit_root), document_id)
}

#[tauri::command]
fn set_document_confidentiality(
    edit_root: String,
    document_id: Uuid,
    confidentiality_type_id: String,
) -> Result<DocumentSelection, String> {
    let confidentiality_type_id = confidentiality_type_id.trim();
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .set_document_confidentiality(
            document_id,
            (!confidentiality_type_id.is_empty()).then_some(confidentiality_type_id),
        )
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_selection(Path::new(&edit_root), document_id)
}

#[tauri::command]
fn update_document_review_schedule(
    edit_root: String,
    document_id: Uuid,
    review_interval_months: Option<u32>,
    review_exemption_reason: Option<String>,
) -> Result<DocumentSelection, String> {
    if review_interval_months.is_some() && review_exemption_reason.is_some() {
        return Err("review interval and exemption cannot both be configured".to_owned());
    }
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .set_document_review_interval(document_id, review_interval_months)
        .map_err(|error| error.to_string())?;
    workspace
        .set_document_review_exemption(document_id, review_exemption_reason.as_deref())
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_selection(Path::new(&edit_root), document_id)
}

#[tauri::command]
fn cancel_document_review(
    edit_root: String,
    document_id: Uuid,
    reason: String,
    confirmed: bool,
) -> Result<DocumentSelection, String> {
    if !confirmed {
        return Err("review cancellation requires explicit confirmation".to_owned());
    }
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .cancel_review(document_id, &reason)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_selection(Path::new(&edit_root), document_id)
}

#[tauri::command]
fn mark_document_obsolete(
    edit_root: String,
    document_id: Uuid,
    reason: String,
    confirmed: bool,
) -> Result<DocumentSelection, String> {
    if !confirmed {
        return Err("mark obsolete requires explicit confirmation".to_owned());
    }
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .mark_obsolete(document_id, &reason)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_selection(Path::new(&edit_root), document_id)
}

struct SignedInActorGraph<'a, G: GraphClient + ?Sized> {
    graph: &'a mut G,
    actor: AuthenticatedActor,
}

impl<G: GraphClient + ?Sized> GraphClient for SignedInActorGraph<'_, G> {
    fn tenant_id(&self) -> std::result::Result<Uuid, String> {
        self.graph.tenant_id()
    }

    fn direct_user_members(
        &mut self,
        source: &EntraIdentitySource,
    ) -> std::result::Result<Vec<EntraPerson>, String> {
        self.graph.direct_user_members(source)
    }

    fn authenticated_actor(
        &mut self,
        _source: &EntraIdentitySource,
    ) -> std::result::Result<AuthenticatedActor, String> {
        Ok(self.actor.clone())
    }
}

fn target_selection(
    target_mode: &str,
    manual_major: Option<u32>,
    manual_minor: Option<u32>,
) -> Result<TargetSelection, String> {
    match target_mode.trim() {
        "next_minor" => Ok(TargetSelection::NextMinor),
        "next_major" => Ok(TargetSelection::NextMajor),
        "manual" => Ok(TargetSelection::Manual(Version {
            major: manual_major
                .ok_or_else(|| "manual target requires a major version".to_owned())?,
            minor: manual_minor
                .ok_or_else(|| "manual target requires a minor version".to_owned())?,
        })),
        value => Err(format!("unknown target version mode: {value}")),
    }
}

fn review_decision(value: &str) -> Result<ReviewDecision, String> {
    match value.trim() {
        "approved" => Ok(ReviewDecision::Approved),
        "rejected" => Ok(ReviewDecision::Rejected),
        "changes_requested" => Ok(ReviewDecision::ChangesRequested),
        value => Err(format!("unknown review decision: {value}")),
    }
}

fn optional_text(value: &str) -> Option<&str> {
    (!value.trim().is_empty()).then_some(value)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CandidateSubmissionInput {
    document_id: Uuid,
    target_mode: String,
    manual_major: Option<u32>,
    manual_minor: Option<u32>,
    changelog: String,
    effective_date: String,
    requester_object_id: Uuid,
    staged_owner_object_id: Option<Uuid>,
    staged_editor_object_id: Option<Uuid>,
    review_override_reason: String,
    mailto_confirmed: bool,
}

fn production_notifier(
    edit_root: &str,
    mailto_confirmed: bool,
) -> Result<notify::DesktopNotifier<notify::OsCredentialStore>, String> {
    let workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    Ok(notify::production_notifier(
        workspace.workspace_id,
        mailto_confirmed,
    ))
}

fn submit_document_candidate_with<G: GraphClient, N: NotificationClient>(
    edit_root: &str,
    input: CandidateSubmissionInput,
    graph: &mut G,
    notifier: &mut N,
) -> Result<DocumentSelection, String> {
    let selection = target_selection(&input.target_mode, input.manual_major, input.manual_minor)?;
    let effective_date = NaiveDate::parse_from_str(input.effective_date.trim(), "%Y-%m-%d")
        .map_err(|_| "effective date must use YYYY-MM-DD".to_owned())?;
    let document_id = input.document_id;
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    workspace
        .submit_candidate(
            CandidateRequest {
                document_id,
                selection,
                changelog: input.changelog,
                effective_date,
                requester_object_id: input.requester_object_id,
                staged_owner_object_id: input.staged_owner_object_id,
                staged_editor_object_id: input.staged_editor_object_id,
                review_override_reason: optional_text(&input.review_override_reason)
                    .map(str::to_owned),
                assistance: None,
            },
            graph,
            notifier,
        )
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_selection(Path::new(edit_root), document_id)
}

fn retry_review_notification_with<N: NotificationClient>(
    edit_root: &str,
    document_id: Uuid,
    notifier: &mut N,
) -> Result<DocumentSelection, String> {
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    workspace
        .retry_review_notification(document_id, notifier)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_selection(Path::new(edit_root), document_id)
}

fn decide_document_review_with<G: GraphClient + ?Sized, N: NotificationClient>(
    edit_root: &str,
    document_id: Uuid,
    decision: ReviewDecision,
    comment: String,
    actor: AuthenticatedActor,
    graph: &mut G,
    notifier: &mut N,
) -> Result<DocumentSelection, String> {
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    let mut signed_in_graph = SignedInActorGraph { graph, actor };
    workspace
        .decide_review(
            document_id,
            decision,
            optional_text(&comment),
            &mut signed_in_graph,
            notifier,
        )
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    document_selection(Path::new(edit_root), document_id)
}

fn release_document_candidate_with<G: GraphClient, N: NotificationClient, E: PdfExporter>(
    edit_root: &str,
    document_id: Uuid,
    release_override_reason: String,
    graph: &mut G,
    notifier: &mut N,
    exporter: &mut E,
) -> Result<DocumentSelection, String> {
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    workspace
        .release_candidate(
            document_id,
            optional_text(&release_override_reason),
            graph,
            notifier,
            exporter,
        )
        .map_err(|error| error.to_string())?;
    document_selection(Path::new(edit_root), document_id)
}

fn retry_decision_notification_with<N: NotificationClient>(
    edit_root: &str,
    document_id: Uuid,
    candidate_id: Uuid,
    notifier: &mut N,
) -> Result<DocumentSelection, String> {
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    workspace
        .retry_decision_notification(document_id, candidate_id, notifier)
        .map_err(|error| error.to_string())?;
    document_selection(Path::new(edit_root), document_id)
}

fn retry_minor_publication_notification_with<N: NotificationClient>(
    edit_root: &str,
    document_id: Uuid,
    release_id: Uuid,
    notifier: &mut N,
) -> Result<DocumentSelection, String> {
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    workspace
        .retry_minor_publication_notification(document_id, release_id, notifier)
        .map_err(|error| error.to_string())?;
    document_selection(Path::new(edit_root), document_id)
}

#[tauri::command]
fn submit_document_candidate(
    edit_root: String,
    input: CandidateSubmissionInput,
    state: State<'_, DesktopIntegrations>,
) -> Result<DocumentSelection, String> {
    let mut notifier = production_notifier(&edit_root, input.mailto_confirmed)?;
    let mut graph = state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?;
    submit_document_candidate_with(&edit_root, input, &mut *graph, &mut notifier)
}

#[tauri::command]
fn retry_review_notification(
    edit_root: String,
    document_id: Uuid,
    mailto_confirmed: bool,
) -> Result<DocumentSelection, String> {
    let mut notifier = production_notifier(&edit_root, mailto_confirmed)?;
    retry_review_notification_with(&edit_root, document_id, &mut notifier)
}

#[tauri::command]
fn decide_document_review(
    edit_root: String,
    document_id: Uuid,
    decision: String,
    comment: String,
    mailto_confirmed: bool,
    state: State<'_, DesktopIntegrations>,
) -> Result<DocumentSelection, String> {
    let decision = review_decision(&decision)?;
    let actor = state
        .approver_actor
        .lock()
        .map_err(|_| "interactive approver sign-in state is unavailable".to_owned())?
        .take()
        .ok_or_else(|| {
            "complete interactive approver sign-in before recording a decision".to_owned()
        })?;
    let mut notifier = production_notifier(&edit_root, mailto_confirmed)?;
    let mut graph = state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?;
    decide_document_review_with(
        &edit_root,
        document_id,
        decision,
        comment,
        actor,
        &mut *graph,
        &mut notifier,
    )
}

#[tauri::command]
fn release_document_candidate(
    edit_root: String,
    document_id: Uuid,
    release_override_reason: String,
    mailto_confirmed: bool,
    state: State<'_, DesktopIntegrations>,
) -> Result<DocumentSelection, String> {
    let mut notifier = production_notifier(&edit_root, mailto_confirmed)?;
    let mut graph = state
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration state is unavailable".to_owned())?;
    let mut exporter = export::LocalPdfExporter::new(export::InstalledOfficeAutomation);
    release_document_candidate_with(
        &edit_root,
        document_id,
        release_override_reason,
        &mut *graph,
        &mut notifier,
        &mut exporter,
    )
}

#[tauri::command]
fn retry_decision_notification(
    edit_root: String,
    document_id: Uuid,
    candidate_id: Uuid,
    mailto_confirmed: bool,
) -> Result<DocumentSelection, String> {
    let mut notifier = production_notifier(&edit_root, mailto_confirmed)?;
    retry_decision_notification_with(&edit_root, document_id, candidate_id, &mut notifier)
}

#[tauri::command]
fn retry_minor_publication_notification(
    edit_root: String,
    document_id: Uuid,
    release_id: Uuid,
    mailto_confirmed: bool,
) -> Result<DocumentSelection, String> {
    let mut notifier = production_notifier(&edit_root, mailto_confirmed)?;
    retry_minor_publication_notification_with(&edit_root, document_id, release_id, &mut notifier)
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
fn withdraw_release(
    edit_root: String,
    document_id: Uuid,
    release_id: Uuid,
    reason: String,
    confirmed: bool,
) -> Result<ReleaseMaintenance, String> {
    if !confirmed {
        return Err("release withdrawal requires explicit confirmation".to_owned());
    }
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .withdraw_release(document_id, release_id, &reason)
        .map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    release_maintenance(&workspace)
}

#[tauri::command]
fn open_document_source(edit_root: String, document_id: Uuid) -> Result<(), String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let path = workspace
        .registered_source_path(document_id)
        .map_err(|error| error.to_string())?;
    open_host_path(&path)
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let url = validate_external_url(&url)?;
    open_host_url(url.as_str())
}

#[tauri::command]
fn open_current_release_pdf(edit_root: String, document_id: Uuid) -> Result<(), String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let path = workspace
        .current_release_pdf_path(document_id)
        .map_err(|error| error.to_string())?;
    open_host_path(&path)
}

#[tauri::command]
fn open_release_pdf(edit_root: String, document_id: Uuid, release_id: Uuid) -> Result<(), String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let path = workspace
        .release_pdf_path(document_id, release_id)
        .map_err(|error| error.to_string())?;
    open_host_path(&path)
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
fn load_audit_reports(edit_root: String) -> Result<AuditReportSnapshot, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    audit_report_snapshot(&workspace)
}

#[tauri::command]
fn generate_audit_report(
    edit_root: String,
    request: AuditReportRequest,
) -> Result<AuditReportSnapshot, String> {
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .generate_audit_report(request)
        .map_err(|error| error.to_string())?;
    audit_report_snapshot(&workspace)
}

#[tauri::command]
fn verify_audit_report(edit_root: String, event_id: Uuid) -> Result<AuditReportSnapshot, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .verify_report(event_id)
        .map_err(|error| error.to_string())?;
    audit_report_snapshot(&workspace)
}

#[tauri::command]
fn open_audit_report_folder(edit_root: String, event_id: Uuid) -> Result<(), String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    open_host_path(&audit_report_folder(&workspace, event_id)?)
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

fn complete_periodic_review_with<G: GraphClient + ?Sized>(
    edit_root: &str,
    document_id: Uuid,
    review_id: Uuid,
    result: PeriodicReviewResult,
    comment: &str,
    confirmed: bool,
    graph: &mut G,
) -> Result<PeriodicReview, String> {
    if !confirmed {
        return Err("periodic-review result requires explicit confirmation".to_owned());
    }
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    workspace
        .complete_periodic_review(document_id, review_id, result, comment, graph)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn complete_periodic_review(
    edit_root: String,
    document_id: Uuid,
    review_id: Uuid,
    result: PeriodicReviewResult,
    comment: String,
    confirmed: bool,
    integrations: tauri::State<'_, DesktopIntegrations>,
) -> Result<PeriodicReview, String> {
    let mut graph = integrations
        .graph
        .lock()
        .map_err(|_| "Microsoft Graph integration lock is poisoned".to_owned())?;
    complete_periodic_review_with(
        &edit_root,
        document_id,
        review_id,
        result,
        &comment,
        confirmed,
        &mut *graph,
    )
}

#[tauri::command]
fn cancel_periodic_review(
    edit_root: String,
    document_id: Uuid,
    review_id: Uuid,
    comment: String,
    confirmed: bool,
) -> Result<PeriodicReview, String> {
    if !confirmed {
        return Err("periodic-review cancellation requires explicit confirmation".to_owned());
    }
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .cancel_periodic_review(document_id, review_id, &comment)
        .map_err(|error| error.to_string())
}

fn remind_periodic_review_with<N: NotificationClient + ?Sized>(
    edit_root: &str,
    document_id: Uuid,
    review_id: Uuid,
    confirmed: bool,
    notifier: &mut N,
) -> Result<DeliveryAttempt, String> {
    if !confirmed {
        return Err("periodic-review reminder requires explicit confirmation".to_owned());
    }
    let mut workspace = Workspace::open(Path::new(edit_root)).map_err(|error| error.to_string())?;
    workspace
        .remind_periodic_review(document_id, review_id, notifier)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn remind_periodic_review(
    edit_root: String,
    document_id: Uuid,
    review_id: Uuid,
    confirmed: bool,
) -> Result<DeliveryAttempt, String> {
    let mut notifier = production_notifier(&edit_root, false)?;
    remind_periodic_review_with(&edit_root, document_id, review_id, confirmed, &mut notifier)
}

#[tauri::command]
fn backup_workspace(edit_root: String, archive_path: String) -> Result<BackupOutcome, String> {
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .backup_workspace(Path::new(&archive_path))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn workspace_lock_status(edit_root: String) -> Result<WorkspaceLockStatus, String> {
    Workspace::open(Path::new(&edit_root))
        .and_then(|workspace| workspace.lock_status())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn acquire_workspace_lock(
    edit_root: String,
    take_over_stale: bool,
    override_existing: bool,
) -> Result<WorkspaceLockStatus, String> {
    Workspace::open(Path::new(&edit_root))
        .and_then(|workspace| {
            if override_existing {
                workspace.override_lock()
            } else {
                workspace.acquire_lock(take_over_stale)
            }
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn release_workspace_lock(
    edit_root: String,
    owner: WorkspaceLock,
    confirmed: bool,
) -> Result<(), String> {
    if !confirmed {
        return Err("workspace lock release requires explicit confirmation".to_owned());
    }
    dms_core::release_workspace_lock_owned(Path::new(&edit_root), &owner)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn configure_workspace_lock_staleness(
    edit_root: String,
    hours: u32,
    confirmed: bool,
) -> Result<WorkspaceLockStatus, String> {
    if !confirmed {
        return Err("workspace lock-staleness change requires explicit confirmation".to_owned());
    }
    let mut workspace =
        Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    workspace
        .configure_lock_staleness(hours)
        .map_err(|error| error.to_string())?;
    workspace.lock_status().map_err(|error| error.to_string())
}

#[tauri::command]
fn restore_workspace_backup(
    archive_path: String,
    edit_root: String,
    publish_root: String,
    replace_existing: bool,
    take_over_stale_lock: bool,
    confirmed: bool,
) -> Result<RestoreOutcome, String> {
    dms_core::restore_workspace_backup(RestoreRequest {
        archive_path: Path::new(&archive_path),
        edit_root: Path::new(&edit_root),
        publish_root: Path::new(&publish_root),
        replace_existing,
        take_over_stale_lock,
        confirmed,
    })
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
    selected_excerpt_lines: Option<Vec<usize>>,
) -> Result<ClaudeAssistancePreview, String> {
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
        .preview_claude_assistance(document_id, selected_excerpt_lines.as_deref())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn launch_claude_assistance(
    edit_root: String,
    document_id: Uuid,
    payload_digest: String,
    selected_excerpt_lines: Option<Vec<usize>>,
    confirmed: bool,
) -> Result<(), String> {
    if !confirmed {
        return Err(
            "explicit confirmation is required before handing data to Claude Desktop".to_owned(),
        );
    }
    let workspace = Workspace::open(Path::new(&edit_root)).map_err(|error| error.to_string())?;
    let payload = workspace
        .prepare_claude_assistance(document_id, selected_excerpt_lines.as_deref())
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
    let preferences = serde_json::from_str(&content)
        .map_err(|error| format!("preferences at {} are invalid: {error}", path.display()))?;
    Ok(normalize_preferences(preferences))
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
    let content = serde_json::to_vec_pretty(&normalize_preferences(preferences.clone()))
        .map_err(|error| format!("cannot encode preferences: {error}"))?;
    fs::write(path, content)
        .map_err(|error| format!("cannot write preferences at {}: {error}", path.display()))
}

fn global_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|error| format!("cannot resolve the app-config directory: {error}"))
        .map(|directory| directory.join(GLOBAL_SETTINGS_FILENAME))
}

fn load_global_settings_at(path: &Path) -> Result<GlobalSettings, String> {
    if !path.exists() {
        return Ok(GlobalSettings::default());
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("cannot read global settings at {}: {error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("global settings at {} are invalid: {error}", path.display()))
}

fn save_global_settings_at(path: &Path, settings: &GlobalSettings) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("global settings path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "cannot create app-config directory {}: {error}",
            parent.display()
        )
    })?;
    let content = serde_json::to_vec_pretty(settings)
        .map_err(|error| format!("cannot encode global settings: {error}"))?;
    fs::write(path, content).map_err(|error| {
        format!(
            "cannot write global settings at {}: {error}",
            path.display()
        )
    })
}

fn nonempty_environment(name: &str) -> Result<Option<String>, String> {
    match env::var(name) {
        Ok(value) if value.trim().is_empty() => Ok(None),
        Ok(value) => Ok(Some(value.trim().to_owned())),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(format!("cannot read {name}: {error}")),
    }
}

fn effective_global_entra_configuration(
    settings: &GlobalSettings,
) -> Result<GlobalEntraConfiguration, String> {
    effective_global_entra_configuration_with_overrides(
        settings,
        nonempty_environment("DMS_ENTRA_CLIENT_ID")?,
        nonempty_environment("DMS_ENTRA_TENANT_ID")?,
    )
}

fn effective_global_entra_configuration_with_overrides(
    settings: &GlobalSettings,
    client_override: Option<String>,
    tenant_override: Option<String>,
) -> Result<GlobalEntraConfiguration, String> {
    let client_id = client_override
        .clone()
        .unwrap_or_else(|| settings.entra_client_id.trim().to_owned());
    let tenant_id = tenant_override
        .clone()
        .unwrap_or_else(|| settings.entra_tenant_id.trim().to_owned());
    if let Some(tenant_id) = tenant_override.as_deref() {
        Uuid::parse_str(tenant_id).map_err(|_| {
            "DMS_ENTRA_TENANT_ID must be a Microsoft Entra directory UUID".to_owned()
        })?;
    }
    Ok(GlobalEntraConfiguration {
        client_id,
        tenant_id,
        client_id_environment_managed: client_override.is_some(),
        tenant_id_environment_managed: tenant_override.is_some(),
    })
}

fn runtime_entra_configuration(
    effective: &GlobalEntraConfiguration,
) -> Result<Option<graph::RuntimeEntraConfiguration>, String> {
    if effective.client_id.is_empty() && effective.tenant_id.is_empty() {
        return Ok(None);
    }
    if effective.client_id.is_empty() || effective.tenant_id.is_empty() {
        return Err(
            "both Microsoft Entra public-client ID and tenant ID must be configured".to_owned(),
        );
    }
    Uuid::parse_str(&effective.client_id)
        .map_err(|_| "Microsoft Entra public-client ID must be an application UUID".to_owned())?;
    Ok(Some(graph::RuntimeEntraConfiguration {
        client_id: effective.client_id.clone(),
        tenant_id: Uuid::parse_str(&effective.tenant_id)
            .map_err(|_| "Microsoft Entra tenant ID must be a directory UUID".to_owned())?,
    }))
}

fn normalize_preferences(mut preferences: Preferences) -> Preferences {
    let mut seen = BTreeSet::new();
    preferences.recent_libraries = preferences
        .recent_libraries
        .into_iter()
        .map(|path| path.trim().to_owned())
        .filter(|path| !path.is_empty() && seen.insert(path.clone()))
        .take(RECENT_LIBRARIES_LIMIT)
        .collect();
    preferences
}

fn resolve_registered_permalink_from(
    preferences: &Preferences,
    uri: &str,
) -> Result<DesktopPermalinkResolution, String> {
    for edit_root in &preferences.recent_libraries {
        let Ok(workspace) = Workspace::open(Path::new(edit_root)) else {
            continue;
        };
        let resolved = match workspace.resolve_permalink(uri) {
            Ok(resolved) => resolved,
            Err(DmsError::PermalinkWorkspaceMismatch(_)) => continue,
            Err(error) => return Err(error.to_string()),
        };
        let document = workspace
            .document(resolved.document_id)
            .map_err(|error| error.to_string())?;
        let folder = document
            .relative_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(path_text)
            .unwrap_or_else(|| ".".to_owned());
        let target = match resolved.target {
            PermalinkTarget::Document => "document",
            PermalinkTarget::Review => "review",
            PermalinkTarget::Notes => "notes",
        };
        return Ok(DesktopPermalinkResolution {
            workspace: workspace_summary_from(&workspace),
            document_id: resolved.document_id,
            title: document.control.title.clone(),
            document_number: document.control.document_number.clone(),
            folder,
            target: target.to_owned(),
            review_id: resolved.review_id,
        });
    }
    Err("permalink workspace is not registered or accessible".to_owned())
}

fn configuration_role_update(value: &str) -> Result<RoleUpdate, String> {
    match value.trim() {
        "__inherit" => Ok(RoleUpdate::Clear),
        "__unchanged" => Ok(RoleUpdate::Unchanged),
        value => Uuid::parse_str(value)
            .map(RoleUpdate::replace)
            .map_err(|_| format!("workflow role must be a Microsoft Entra object ID: {value}")),
    }
}

fn workspace_summary(edit_root: &Path) -> Result<WorkspaceSummary, String> {
    let workspace = Workspace::open(edit_root).map_err(|error| error.to_string())?;
    Ok(workspace_summary_from(&workspace))
}

fn workspace_summary_from(workspace: &Workspace) -> WorkspaceSummary {
    WorkspaceSummary {
        workspace_id: workspace.workspace_id.to_string(),
        edit_root: workspace.edit_root.to_string_lossy().into_owned(),
        publish_root: workspace.publish_root.to_string_lossy().into_owned(),
        document_count: workspace.documents().len(),
    }
}

fn workspace_configuration(
    app: &AppHandle,
    edit_root: &Path,
) -> Result<WorkspaceConfiguration, String> {
    let workspace = Workspace::open(edit_root).map_err(|error| error.to_string())?;
    let settings = load_global_settings_at(&global_settings_path(app)?)?;
    workspace_configuration_from_with_global(
        &workspace,
        effective_global_entra_configuration(&settings)?,
    )
}

fn workspace_configuration_from(workspace: &Workspace) -> Result<WorkspaceConfiguration, String> {
    workspace_configuration_from_with_global(
        workspace,
        effective_global_entra_configuration(&GlobalSettings::default())?,
    )
}

fn workspace_configuration_from_with_global(
    workspace: &Workspace,
    global_entra_configuration: GlobalEntraConfiguration,
) -> Result<WorkspaceConfiguration, String> {
    workspace_configuration_from_with_global_and_credentials(
        workspace,
        global_entra_configuration,
        &notify::OsCredentialStore,
    )
}

fn workspace_configuration_from_with_global_and_credentials<C: notify::CredentialStore>(
    workspace: &Workspace,
    global_entra_configuration: GlobalEntraConfiguration,
    credentials: &C,
) -> Result<WorkspaceConfiguration, String> {
    Ok(WorkspaceConfiguration {
        workspace: workspace_summary_from(workspace),
        markdown_template: workspace.markdown_template().cloned(),
        markdown_template_validation: workspace.markdown_template_validation(),
        default_review_interval_months: workspace.default_review_interval_months(),
        document_types: workspace.document_types().into_iter().cloned().collect(),
        confidentiality_types: workspace
            .confidentiality_types()
            .into_iter()
            .cloned()
            .collect(),
        confidentiality_policies: workspace
            .confidentiality_policies()
            .into_iter()
            .cloned()
            .collect(),
        policy_folders: workspace
            .policy_folders()
            .map_err(|error| error.to_string())?,
        identity_source: workspace.identity_source().cloned(),
        eligible_people: workspace.eligible_people().into_iter().cloned().collect(),
        workflow_policies: workspace.workflow_policies(),
        notification_settings: workspace.notification_settings().cloned(),
        global_entra_configuration,
        smtp_credential_configured: credentials.smtp_password_exists(workspace.workspace_id)?,
    })
}

fn mutate_workspace_configuration(
    edit_root: &Path,
    mutation: impl FnOnce(&mut Workspace) -> dms_core::Result<()>,
) -> Result<WorkspaceConfiguration, String> {
    let mut workspace = Workspace::open(edit_root).map_err(|error| error.to_string())?;
    mutation(&mut workspace).map_err(|error| error.to_string())?;
    workspace.save().map_err(|error| error.to_string())?;
    workspace_configuration_from(&workspace)
}

fn library_snapshot(edit_root: &Path, folder: &Path) -> Result<LibrarySnapshot, String> {
    let mut workspace = Workspace::open(edit_root).map_err(|error| error.to_string())?;
    if workspace
        .sync_all_registered_lifecycles()
        .map_err(|error| error.to_string())?
        > 0
    {
        workspace.save().map_err(|error| error.to_string())?;
    }
    let (tree, folder) = workspace
        .library_snapshot(folder)
        .map_err(|error| error.to_string())?;
    Ok(LibrarySnapshot { tree, folder })
}

fn document_selection(edit_root: &Path, document_id: Uuid) -> Result<DocumentSelection, String> {
    let mut workspace = Workspace::open(edit_root).map_err(|error| error.to_string())?;
    if workspace
        .sync_lifecycle_from_source(document_id)
        .map_err(|error| error.to_string())?
    {
        workspace.save().map_err(|error| error.to_string())?;
    }
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
    let current_release = workspace
        .current_release(document_id)
        .map_err(|error| error.to_string())?
        .map(|release| CurrentReleaseSelection {
            release_id: release.id,
            version: release.version.to_string(),
            relative_pdf_path: path_text(&release.relative_pdf_path),
            pdf_exists: workspace.release_pdf_path(document_id, release.id).is_ok(),
            effective_date: release.effective_date,
            profile: release
                .control
                .as_ref()
                .map(|control| ReleaseProfileSelection {
                    title: control.title.clone(),
                    document_number: control.document_number.clone(),
                    document_type: control.document_type.clone(),
                    owner: release.owner.clone(),
                }),
        });
    let active_candidate = workspace
        .active_candidate(document_id)
        .map(|candidate| candidate.cloned())
        .map_err(|error| error.to_string())?;
    let retryable_decision_candidate = workspace
        .candidates(document_id)
        .map_err(|error| error.to_string())?
        .into_iter()
        .rev()
        .find(|candidate| {
            matches!(
                candidate.status,
                dms_core::CandidateStatus::Approved
                    | dms_core::CandidateStatus::Rejected
                    | dms_core::CandidateStatus::ChangesRequested
            ) && candidate.delivery_attempts.last().is_some_and(|attempt| {
                attempt.kind == dms_core::NotificationKind::DecisionOutcome
                    && !matches!(
                        attempt.status,
                        dms_core::DeliveryStatus::Accepted | dms_core::DeliveryStatus::Confirmed
                    )
            })
        })
        .cloned();
    let retryable_minor_publication = workspace
        .candidates(document_id)
        .map_err(|error| error.to_string())?
        .into_iter()
        .rev()
        .find(|candidate| {
            candidate.status == dms_core::CandidateStatus::Released
                && !candidate.approval_required
                && candidate.delivery_attempts.last().is_some_and(|attempt| {
                    attempt.kind == dms_core::NotificationKind::MinorPublication
                        && !matches!(
                            attempt.status,
                            dms_core::DeliveryStatus::Accepted
                                | dms_core::DeliveryStatus::Confirmed
                        )
                })
        })
        .cloned();
    let lifecycle_actions = workspace
        .local_lifecycle_actions(document_id)
        .map_err(|error| error.to_string())?;
    let workflow_events = workspace
        .workflow_history(document_id)
        .map_err(|error| error.to_string())?
        .into_iter()
        .cloned()
        .collect();
    let workflow_verification = workspace
        .verify_workflow(document_id)
        .map_err(|error| error.to_string())?;
    let eligible_people = workspace
        .eligible_people()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let empty_identity_success = workspace
        .identity_source()
        .is_some_and(|source| source.last_refreshed_at.is_some())
        && eligible_people.is_empty();
    let current_owner = if let Some(reference) = document.control.owner {
        eligible_people
            .iter()
            .find(|person| person.object_id == reference.object_id)
            .map(|person| serde_json::to_value(person).expect("owner snapshot serializes"))
            .unwrap_or_else(|| {
                serde_json::json!({
                    "kind": "entra",
                    "binding_id": reference.binding_id,
                    "object_id": reference.object_id,
                })
            })
    } else if let Some(label) = document.control.legacy_owner_label.as_ref() {
        serde_json::json!({ "kind": "legacy", "label": label })
    } else if empty_identity_success {
        serde_json::json!({ "kind": "placeholder", "label": "<owner>" })
    } else {
        serde_json::Value::Null
    };
    let effective_workflow_roles = workspace.effective_workflow_roles(document_id).ok();
    let requires_identity_handover = empty_identity_success
        && (document.control.owner.is_none()
            || effective_workflow_roles
                .as_ref()
                .and_then(|roles| roles.editor.as_ref())
                .is_none());
    let persisted_review_schedule = workspace
        .document_review_schedule(document_id)
        .map_err(|error| error.to_string())?;
    Ok(DocumentSelection {
        document_id,
        source_name,
        relative_path: path_text(&document.relative_path),
        folder,
        source_exists: workspace.edit_root.join(&document.relative_path).is_file(),
        source_state: document.source_state,
        lifecycle: document.lifecycle,
        control: document.control.clone(),
        current_owner,
        requires_identity_handover,
        review_schedule: DocumentReviewSchedule {
            workspace_interval_months: persisted_review_schedule.workspace_interval_months,
            interval_months: persisted_review_schedule.interval_months,
            exemption_reason: persisted_review_schedule.exemption_reason,
            next_due_date: persisted_review_schedule.next_due_date,
        },
        document_types: workspace.document_types().into_iter().cloned().collect(),
        confidentiality_types: workspace
            .confidentiality_types()
            .into_iter()
            .cloned()
            .collect(),
        confidentiality_override: workspace
            .document_confidentiality_override(document_id)
            .map_err(|error| error.to_string())?
            .map(str::to_owned),
        effective_confidentiality: workspace.effective_confidentiality(document_id).ok(),
        effective_workflow_roles,
        current_release,
        active_candidate,
        retryable_decision_candidate,
        retryable_minor_publication,
        eligible_people,
        lifecycle_actions,
        workflow_events,
        workflow_verification,
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
                document_title: release.control.as_ref().map_or_else(
                    || "<legacy title unrecorded>".to_owned(),
                    |control| control.title.clone(),
                ),
                release_id: release.id,
                version: release.version.to_string(),
                relative_pdf_path: path_text(&release.relative_pdf_path),
                pdf_digest: release.pdf_digest.clone(),
                confidentiality_id: release.confidentiality.type_id.clone(),
                confidentiality_label: release.confidentiality.label.clone(),
                effective_date: release.effective_date,
                profile: release
                    .control
                    .as_ref()
                    .map(|control| ReleaseProfileSelection {
                        title: control.title.clone(),
                        document_number: control.document_number.clone(),
                        document_type: control.document_type.clone(),
                        owner: release.owner.clone(),
                    }),
                workflow_chain_head: release.workflow_chain_head.clone(),
                approval_chain_head: release.approval_chain_head.clone(),
                released_at: release.released_at.to_rfc3339(),
                withdrawn: release.withdrawn,
                orphaned: document.source_state != SourceState::Registered,
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

fn audit_report_snapshot(workspace: &Workspace) -> Result<AuditReportSnapshot, String> {
    let reports = workspace.recent_reports();
    let verifications = workspace
        .verify_reports()
        .map_err(|error| error.to_string())?;
    if reports.len() != verifications.len() {
        return Err("audit report evidence and verification counts differ".to_owned());
    }
    let rows = reports
        .into_iter()
        .zip(verifications)
        .map(|(report, verification)| {
            if report.event_id != verification.event_id {
                return Err("audit report evidence and verification order differ".to_owned());
            }
            Ok(AuditReportRow {
                report,
                verification: verification.status,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(AuditReportSnapshot {
        rows,
        evidence_chain: workspace.verify_report_chain(),
    })
}

fn audit_report_folder(workspace: &Workspace, event_id: Uuid) -> Result<PathBuf, String> {
    let report = workspace
        .recent_reports()
        .into_iter()
        .find(|report| report.event_id == event_id)
        .ok_or_else(|| format!("audit report event {event_id} was not found"))?;
    let parent = workspace
        .edit_root
        .join(report.relative_path())
        .parent()
        .ok_or_else(|| "audit report path has no containing folder".to_owned())?
        .to_path_buf();
    let canonical_root = fs::canonicalize(&workspace.edit_root)
        .map_err(|error| format!("cannot resolve the edit root: {error}"))?;
    let canonical_parent = fs::canonicalize(&parent)
        .map_err(|error| format!("cannot resolve the report folder: {error}"))?;
    if !canonical_parent.starts_with(canonical_root) {
        return Err("the report folder is outside the edit root".to_owned());
    }
    Ok(canonical_parent)
}

fn open_host_path(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("explorer.exe");
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");

    command
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("cannot open {}: {error}", path.display()))
}

fn validate_external_url(value: &str) -> Result<url::Url, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("external URL is required".to_owned());
    }
    let url = url::Url::parse(value).map_err(|error| format!("invalid external URL: {error}"))?;
    let host = url
        .host_str()
        .ok_or_else(|| "external URL must include a host".to_owned())?;
    match url.scheme() {
        "https" => Ok(url),
        "http" if matches!(host, "localhost" | "127.0.0.1") => Ok(url),
        scheme => Err(format!("external URL scheme {scheme:?} is not allowed")),
    }
}

fn open_host_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("explorer.exe");
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");

    command
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("cannot open external URL: {error}"))
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

fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_single_instance::init(
        |app, _arguments, _working_directory| {
            focus_main_window(app);
        },
    ));
    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DesktopIntegrations::default())
        .setup(|app| {
            let handle = app.handle().clone();
            let settings = load_global_settings_at(&global_settings_path(&handle)?)?;
            let runtime =
                runtime_entra_configuration(&effective_global_entra_configuration(&settings)?)?;
            *app.state::<DesktopIntegrations>()
                .graph
                .lock()
                .map_err(|_| "Microsoft Graph integration state is unavailable")? =
                graph::MicrosoftGraphClient::production(runtime);
            let handle = app.handle().clone();
            app.deep_link()
                .on_open_url(move |_event| focus_main_window(&handle));
            if std::env::var_os("DMS_DESKTOP_SMOKE").is_some() {
                app.handle().exit(0);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_preferences,
            save_preferences,
            select_directory,
            initialize_workspace,
            open_workspace,
            resolve_registered_permalink,
            load_workspace_configuration,
            choose_markdown_template,
            remove_markdown_template,
            configure_global_entra,
            open_external_url,
            configure_default_review_interval,
            configure_document_type,
            configure_confidentiality_type,
            set_confidentiality_policy,
            remove_confidentiality_policy,
            set_workflow_policy,
            begin_identity_source_sign_in,
            complete_identity_source_sign_in,
            begin_approver_sign_in,
            complete_approver_sign_in,
            apply_identity_source,
            refresh_identity_source,
            configure_notifications,
            test_smtp_notification,
            load_library,
            search_library,
            add_library_documents,
            unregister_library_documents,
            choose_reassociate_source,
            reassociate_library_document,
            load_document_selection,
            update_document_control,
            update_document_review_schedule,
            set_document_confidentiality,
            cancel_document_review,
            mark_document_obsolete,
            submit_document_candidate,
            retry_review_notification,
            decide_document_review,
            release_document_candidate,
            retry_decision_notification,
            retry_minor_publication_notification,
            open_document_source,
            open_current_release_pdf,
            load_document_notes,
            load_releases,
            withdraw_release,
            open_release_pdf,
            verify_release,
            verify_all_releases,
            load_audit_reports,
            generate_audit_report,
            verify_audit_report,
            open_audit_report_folder,
            load_periodic_reviews,
            start_periodic_review,
            complete_periodic_review,
            cancel_periodic_review,
            remind_periodic_review,
            backup_workspace,
            workspace_lock_status,
            acquire_workspace_lock,
            release_workspace_lock,
            configure_workspace_lock_staleness,
            restore_workspace_backup,
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

    struct TestGraph {
        tenant_id: Uuid,
        people: Vec<EntraPerson>,
    }

    impl GraphClient for TestGraph {
        fn tenant_id(&self) -> std::result::Result<Uuid, String> {
            Ok(self.tenant_id)
        }

        fn direct_user_members(
            &mut self,
            _source: &EntraIdentitySource,
        ) -> std::result::Result<Vec<EntraPerson>, String> {
            Ok(self.people.clone())
        }

        fn authenticated_actor(
            &mut self,
            _source: &EntraIdentitySource,
        ) -> std::result::Result<AuthenticatedActor, String> {
            Err("interactive sign-in is not configured for this test".to_owned())
        }
    }

    #[test]
    fn missing_preferences_use_expanded_sidebar_and_no_saved_views() {
        let directory = tempfile::tempdir().unwrap();
        let preferences = load_preferences_at(&directory.path().join("missing.json")).unwrap();

        assert_eq!(preferences, Preferences::default());
    }

    #[test]
    fn desktop_bundle_declares_the_dms_scheme_and_guest_permission() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(
            config["plugins"]["deep-link"]["desktop"]["schemes"],
            serde_json::json!(["dms"])
        );
        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
        assert!(capability["permissions"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("deep-link:default")));
    }

    #[test]
    fn external_url_validation_allows_only_browser_safe_urls() {
        assert!(validate_external_url("https://example.com/sign-in").is_ok());
        assert!(validate_external_url("http://localhost:1234/callback").is_ok());
        assert!(validate_external_url("http://127.0.0.1:3000/callback").is_ok());
        assert!(validate_external_url("file:///etc/passwd").is_err());
        assert!(validate_external_url("javascript:alert(1)").is_err());
        assert!(validate_external_url("").is_err());
        assert!(validate_external_url("http://example.com").is_err());
    }

    #[test]
    fn first_identity_source_setup_requires_and_persists_root_roles_atomically() {
        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        let editor_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();
        let people = vec![
            EntraPerson::eligible(editor_id, "Eva Editor", "editor@example.test"),
            EntraPerson::eligible(approver_id, "Ada Approver", "approver@example.test"),
        ];

        assert!(matches!(
            apply_identity_source_to_workspace(
                &mut workspace,
                Uuid::new_v4(),
                "DMS workflow",
                people.clone(),
                None,
                None,
            ),
            Err(DmsError::RequiredRootWorkflowPolicy)
        ));
        assert!(workspace.identity_source().is_none());

        apply_identity_source_to_workspace(
            &mut workspace,
            Uuid::new_v4(),
            "DMS workflow",
            people,
            Some(editor_id),
            Some(approver_id),
        )
        .unwrap();
        workspace.save().unwrap();

        let root = workspace
            .workflow_policies()
            .into_iter()
            .find(|policy| policy.folder == ".")
            .unwrap();
        assert_eq!(root.editor.unwrap().object_id, editor_id);
        assert_eq!(root.approver.unwrap().object_id, approver_id);
    }

    #[test]
    fn first_identity_source_setup_accepts_a_successful_empty_group_without_fake_roles() {
        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();

        apply_identity_source_to_workspace(
            &mut workspace,
            Uuid::new_v4(),
            "Empty DMS workflow group",
            Vec::new(),
            None,
            None,
        )
        .unwrap();

        assert!(workspace.identity_source().is_some());
        assert!(workspace.eligible_people().is_empty());
        assert!(workspace.workflow_policies().is_empty());
    }

    #[test]
    fn replacement_identity_source_does_not_remap_existing_root_roles() {
        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        let editor_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();
        apply_identity_source_to_workspace(
            &mut workspace,
            Uuid::new_v4(),
            "Original group",
            vec![
                EntraPerson::eligible(editor_id, "Eva Editor", "editor@example.test"),
                EntraPerson::eligible(approver_id, "Ada Approver", "approver@example.test"),
            ],
            Some(editor_id),
            Some(approver_id),
        )
        .unwrap();
        let original_root = workspace.workflow_policies().remove(0);

        let replacement_editor_id = Uuid::new_v4();
        let replacement_approver_id = Uuid::new_v4();
        apply_identity_source_to_workspace(
            &mut workspace,
            Uuid::new_v4(),
            "Replacement group",
            vec![
                EntraPerson::eligible(replacement_editor_id, "Rita Editor", "rita@example.test"),
                EntraPerson::eligible(
                    replacement_approver_id,
                    "Arno Approver",
                    "arno@example.test",
                ),
            ],
            Some(replacement_editor_id),
            Some(replacement_approver_id),
        )
        .unwrap();
        workspace.save().unwrap();

        let replacement_root = workspace.workflow_policies().remove(0);
        assert_eq!(replacement_root.editor, original_root.editor);
        assert_eq!(replacement_root.approver, original_root.approver);
        assert_ne!(
            replacement_root.editor.unwrap().binding_id,
            workspace.identity_source().unwrap().binding_id
        );
    }

    #[test]
    fn preferences_round_trip_outside_workspace_metadata() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config").join(PREFERENCES_FILENAME);
        let preferences = Preferences {
            sidebar_expanded: false,
            recent_libraries: vec!["/Users/name/DMS/Edit".into()],
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
    fn preferences_keep_at_most_ten_unique_recent_libraries() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config").join(PREFERENCES_FILENAME);
        let mut recent_libraries = vec!["/library/0".to_owned(), "/library/0".to_owned()];
        recent_libraries.extend((1..=11).map(|index| format!("/library/{index}")));
        let preferences = Preferences {
            sidebar_expanded: true,
            saved_views: Vec::new(),
            recent_libraries,
        };

        save_preferences_at(&path, &preferences).unwrap();
        let loaded = load_preferences_at(&path).unwrap();

        assert_eq!(loaded.recent_libraries.len(), 10);
        assert_eq!(loaded.recent_libraries[0], "/library/0");
        assert_eq!(loaded.recent_libraries[9], "/library/9");
    }

    #[test]
    fn preferences_without_recent_libraries_remain_readable() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        fs::write(&path, r#"{"sidebar_expanded":false,"saved_views":[]}"#).unwrap();

        let loaded = load_preferences_at(&path).unwrap();

        assert!(!loaded.sidebar_expanded);
        assert!(loaded.recent_libraries.is_empty());
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
    fn desktop_configuration_commands_persist_workspace_and_document_defaults() {
        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(edit_root.path().join("Policies/HR")).unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        workspace
            .configure_confidentiality_type("internal", "Internal", true)
            .unwrap();
        workspace
            .configure_confidentiality_type("restricted", "Restricted", true)
            .unwrap();
        workspace
            .set_confidentiality_policy(".", "internal")
            .unwrap();
        workspace.save().unwrap();
        let root = edit_root.path().to_string_lossy().into_owned();

        let initial =
            workspace_configuration_from(&Workspace::open(edit_root.path()).unwrap()).unwrap();
        assert!(initial.default_review_interval_months > 0);
        assert!(initial
            .policy_folders
            .iter()
            .any(|folder| folder.relative_path == "Policies/HR"));

        let template_path = edit_root.path().join("Word template.docx");
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../dms-core/tests/fixtures/markdown-template.docx"),
            &template_path,
        )
        .unwrap();
        let template =
            import_markdown_template_from_path(edit_root.path(), &template_path).unwrap();
        assert_eq!(
            template.markdown_template.as_ref().unwrap().relative_path,
            PathBuf::from("Word template.docx")
        );
        assert_eq!(
            template.markdown_template_validation.unwrap().state,
            dms_core::MarkdownTemplateValidationState::Valid
        );
        let hidden = library_snapshot(edit_root.path(), Path::new(".")).unwrap();
        assert!(!hidden
            .folder
            .entries
            .iter()
            .any(|entry| entry.relative_path == Path::new("Word template.docx")));
        assert!(remove_markdown_template(root.clone(), false).is_err());
        let removed = remove_markdown_template(root.clone(), true).unwrap();
        assert!(removed.markdown_template.is_none());
        let visible = library_snapshot(edit_root.path(), Path::new(".")).unwrap();
        assert!(visible
            .folder
            .entries
            .iter()
            .any(|entry| entry.relative_path == Path::new("Word template.docx")));

        let interval = configure_default_review_interval(root.clone(), 6).unwrap();
        assert_eq!(interval.default_review_interval_months, 6);

        let catalogue =
            configure_document_type(root.clone(), "procedure".into(), "Procedure".into(), true)
                .unwrap();
        assert_eq!(catalogue.document_types[0].id, "procedure");

        let policy =
            set_confidentiality_policy(root.clone(), "Policies/HR".into(), "restricted".into())
                .unwrap();
        assert!(policy
            .confidentiality_policies
            .iter()
            .any(|entry| entry.folder == "Policies/HR" && entry.type_id == "restricted"));

        let removed = remove_confidentiality_policy(root, "Policies/HR".into()).unwrap();
        assert!(!removed
            .confidentiality_policies
            .iter()
            .any(|entry| entry.folder == "Policies/HR"));
    }

    #[test]
    fn desktop_configuration_commands_persist_workflow_and_notifications() {
        struct RefreshedGraph {
            people: Vec<EntraPerson>,
        }

        impl GraphClient for RefreshedGraph {
            fn tenant_id(&self) -> Result<Uuid, String> {
                Ok(Uuid::nil())
            }

            fn direct_user_members(
                &mut self,
                _source: &EntraIdentitySource,
            ) -> Result<Vec<EntraPerson>, String> {
                Ok(self.people.clone())
            }

            fn authenticated_actor(
                &mut self,
                _source: &EntraIdentitySource,
            ) -> Result<dms_core::AuthenticatedActor, String> {
                Err("not used by configuration".to_owned())
            }
        }

        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(edit_root.path().join("Policies/HR")).unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        let editor_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();
        workspace
            .replace_identity_source(
                Uuid::new_v4(),
                "DMS workflow",
                vec![
                    EntraPerson::eligible(editor_id, "Lukas Roth", "lukas@example.test"),
                    EntraPerson::eligible(approver_id, "Anna Berg", "anna@example.test"),
                ],
            )
            .unwrap();
        workspace
            .update_workflow_policy(
                ".",
                RoleUpdate::replace(editor_id),
                RoleUpdate::replace(approver_id),
            )
            .unwrap();
        workspace.save().unwrap();
        let root = edit_root.path().to_string_lossy().into_owned();

        let mut graph = RefreshedGraph {
            people: vec![
                EntraPerson::eligible(editor_id, "Lukas Roth", "lukas@example.test"),
                EntraPerson::eligible(approver_id, "Anna Berg", "anna@example.test"),
            ],
        };
        let workflow = set_workflow_policy_with(
            edit_root.path(),
            "Policies/HR",
            RoleUpdate::replace(editor_id),
            RoleUpdate::Unchanged,
            &mut graph,
        )
        .unwrap();
        assert_eq!(
            workflow.identity_source.unwrap().group_label,
            "DMS workflow"
        );
        assert_eq!(workflow.eligible_people.len(), 2);
        assert!(workflow.workflow_policies.iter().any(|policy| {
            policy.folder == "Policies/HR" && policy.editor.is_some() && policy.approver.is_none()
        }));

        let catalogue = configure_confidentiality_type(
            root,
            "restricted".into(),
            "Restricted".into(),
            true,
            true,
        )
        .unwrap();
        assert_eq!(catalogue.confidentiality_types[0].id, "restricted");
    }

    #[test]
    fn global_entra_settings_use_complete_environment_overrides_or_fail_closed() {
        let settings = GlobalSettings {
            entra_client_id: Uuid::new_v4().to_string(),
            entra_tenant_id: Uuid::new_v4().to_string(),
        };
        let tenant_id = Uuid::new_v4();
        let client_id = Uuid::new_v4();

        let effective = effective_global_entra_configuration_with_overrides(
            &settings,
            Some(client_id.to_string()),
            Some(tenant_id.to_string()),
        )
        .unwrap();
        assert_eq!(effective.client_id, client_id.to_string());
        assert_eq!(effective.tenant_id, tenant_id.to_string());
        assert!(effective.client_id_environment_managed);
        assert!(effective.tenant_id_environment_managed);
        assert_eq!(
            runtime_entra_configuration(&effective)
                .unwrap()
                .unwrap()
                .tenant_id,
            tenant_id
        );

        assert!(effective_global_entra_configuration_with_overrides(
            &settings,
            None,
            Some("not-a-tenant-id".to_owned()),
        )
        .unwrap_err()
        .contains("DMS_ENTRA_TENANT_ID"));
        assert!(runtime_entra_configuration(
            &effective_global_entra_configuration_with_overrides(
                &settings,
                Some(client_id.to_string()),
                None,
            )
            .unwrap(),
        )
        .is_ok());
        assert!(runtime_entra_configuration(
            &effective_global_entra_configuration_with_overrides(
                &settings,
                Some("not-a-client-id".to_owned()),
                None,
            )
            .unwrap(),
        )
        .unwrap_err()
        .contains("public-client ID"));
    }

    #[test]
    fn global_entra_settings_round_trip_outside_workspace_metadata() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory
            .path()
            .join("config")
            .join(GLOBAL_SETTINGS_FILENAME);
        let settings = GlobalSettings {
            entra_client_id: Uuid::new_v4().to_string(),
            entra_tenant_id: Uuid::new_v4().to_string(),
        };

        save_global_settings_at(&path, &settings).unwrap();

        assert_eq!(load_global_settings_at(&path).unwrap(), settings);
        assert!(!path.to_string_lossy().contains(".dms"));
    }

    #[test]
    fn smtp_configuration_requires_a_stored_app_password_and_never_persists_it() {
        #[derive(Default)]
        struct MemoryCredentials {
            password: std::sync::Mutex<Option<String>>,
        }

        impl notify::CredentialStore for MemoryCredentials {
            fn smtp_password(&self, _workspace_id: Uuid) -> Result<String, String> {
                self.password
                    .lock()
                    .map_err(|_| "credential test store is unavailable".to_owned())?
                    .clone()
                    .ok_or_else(|| "SMTP app password is missing".to_owned())
            }

            fn set_smtp_password(&self, _workspace_id: Uuid, password: &str) -> Result<(), String> {
                *self
                    .password
                    .lock()
                    .map_err(|_| "credential test store is unavailable".to_owned())? =
                    Some(password.to_owned());
                Ok(())
            }

            fn delete_smtp_password(&self, _workspace_id: Uuid) -> Result<(), String> {
                *self
                    .password
                    .lock()
                    .map_err(|_| "credential test store is unavailable".to_owned())? = None;
                Ok(())
            }

            fn smtp_password_exists(&self, _workspace_id: Uuid) -> Result<bool, String> {
                Ok(self
                    .password
                    .lock()
                    .map_err(|_| "credential test store is unavailable".to_owned())?
                    .is_some())
            }
        }

        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        let credentials = MemoryCredentials::default();
        let root = edit_root.path().to_string_lossy().into_owned();
        let smtp_input = |smtp_app_password: &str| NotificationConfigurationInput {
            transport: "smtp".to_owned(),
            relay_host: "smtp.example.test".to_owned(),
            relay_port: 587,
            login_user: "smtp-login@example.test".to_owned(),
            from_mailbox: "\"Doc Mgmt\" <dms@example.test>".to_owned(),
            smtp_app_password: smtp_app_password.to_owned(),
        };

        let error = configure_notifications_with_credentials(&root, smtp_input(""), &credentials)
            .unwrap_err();

        assert!(error.contains("requires a Microsoft 365 app password"));
        assert!(Workspace::open(edit_root.path())
            .unwrap()
            .notification_settings()
            .is_none());

        let configured = configure_notifications_with_credentials(
            &root,
            smtp_input("one-way-secret"),
            &credentials,
        )
        .unwrap();
        assert!(configured.smtp_credential_configured);
        assert!(!serde_json::to_string(&configured)
            .unwrap()
            .contains("one-way-secret"));
        assert!(
            !fs::read_to_string(edit_root.path().join(".dms/workspace.json"))
                .unwrap()
                .contains("one-way-secret")
        );

        let retained =
            configure_notifications_with_credentials(&root, smtp_input(""), &credentials).unwrap();
        assert!(retained.smtp_credential_configured);

        #[derive(Default)]
        struct RecordingNotifier {
            recipient: Option<String>,
            subject: Option<String>,
        }

        impl NotificationClient for RecordingNotifier {
            fn send(
                &mut self,
                _settings: &NotificationSettings,
                message: &NotificationMessage,
            ) -> std::result::Result<dms_core::DeliveryReceipt, String> {
                self.recipient = Some(message.recipient.clone());
                self.subject = Some(message.subject.clone());
                Ok(dms_core::DeliveryReceipt::accepted(250, "fake accepted"))
            }
        }

        let workspace = Workspace::open(edit_root.path()).unwrap();
        let mut notifier = RecordingNotifier::default();
        let result = test_smtp_notification_with(&workspace, &credentials, &mut notifier).unwrap();
        assert_eq!(result.recipient, "dms@example.test");
        assert_eq!(result.response_code, Some(250));
        assert_eq!(notifier.recipient.as_deref(), Some("dms@example.test"));
        assert_eq!(
            notifier.subject.as_deref(),
            Some("DMS SMTP configuration test")
        );

        struct FailingNotifier;

        impl NotificationClient for FailingNotifier {
            fn send(
                &mut self,
                _settings: &NotificationSettings,
                _message: &NotificationMessage,
            ) -> std::result::Result<dms_core::DeliveryReceipt, String> {
                Err("relay details that must not cross IPC".to_owned())
            }
        }

        let error = test_smtp_notification_with(&workspace, &credentials, &mut FailingNotifier)
            .unwrap_err();
        assert_eq!(
            error,
            "SMTP test delivery failed. Verify the saved relay, identity, From mailbox, and app password."
        );
        assert!(!error.contains("relay details"));

        let mailto = configure_notifications_with_credentials(
            &root,
            NotificationConfigurationInput {
                transport: "mailto".to_owned(),
                relay_host: String::new(),
                relay_port: 587,
                login_user: String::new(),
                from_mailbox: String::new(),
                smtp_app_password: String::new(),
            },
            &credentials,
        )
        .unwrap();
        assert!(!mailto.smtp_credential_configured);
        assert_eq!(
            mailto.notification_settings.unwrap().transport,
            NotificationTransport::Mailto
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
        workspace
            .configure_document_type("procedure", "Procedure", true)
            .unwrap();
        workspace
            .configure_confidentiality_type("internal", "Internal", true)
            .unwrap();
        workspace
            .configure_confidentiality_type("restricted", "Restricted", true)
            .unwrap();
        workspace
            .set_confidentiality_policy(".", "internal")
            .unwrap();
        let tenant_id = Uuid::new_v4();
        let owner_object_id = Uuid::new_v4();
        let owner = EntraPerson::eligible(owner_object_id, "People team", "people@example.test");
        workspace
            .replace_identity_source(Uuid::new_v4(), "DMS owners", vec![owner.clone()])
            .unwrap();
        workspace
            .update_workflow_policy(
                ".",
                RoleUpdate::replace(owner_object_id),
                RoleUpdate::replace(owner_object_id),
            )
            .unwrap();
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
        assert_eq!(selection.document_types[0].id, "procedure");
        assert_eq!(selection.confidentiality_types.len(), 2);
        assert!(!selection.lifecycle_actions.cancel_review.available);
        assert!(selection.lifecycle_actions.mark_obsolete.available);
        assert_eq!(
            selection.workflow_verification,
            WorkflowVerification::Missing
        );
        assert_eq!(
            serde_json::to_value(WorkflowVerification::TamperedAt(document.id)).unwrap(),
            serde_json::json!({ "tampered_at": document.id })
        );
        assert!(selection.permalink.ends_with(&document.id.to_string()));
        let preferences = Preferences {
            recent_libraries: vec![
                "/unavailable/workspace".to_owned(),
                edit_root.path().to_string_lossy().into_owned(),
            ],
            ..Preferences::default()
        };
        let notes = resolve_registered_permalink_from(
            &preferences,
            &format!("{}&target=notes", selection.permalink),
        )
        .unwrap();
        assert_eq!(
            notes.workspace.workspace_id,
            workspace.workspace_id.to_string()
        );
        assert_eq!(notes.document_id, document.id);
        assert_eq!(notes.folder, "Policies");
        assert_eq!(notes.target, "notes");
        assert_eq!(notes.review_id, None);
        assert!(
            resolve_registered_permalink_from(&Preferences::default(), &selection.permalink,)
                .unwrap_err()
                .contains("not registered or accessible")
        );

        let root = edit_root.path().to_string_lossy().into_owned();
        let mut graph = TestGraph {
            tenant_id,
            people: vec![owner],
        };
        let updated = update_document_control_with(
            &root,
            document.id,
            "Employee handbook".into(),
            "HR-001".into(),
            "procedure".into(),
            owner_object_id,
            &mut graph,
        )
        .unwrap();
        assert_eq!(updated.control.title, "Employee handbook");
        assert_eq!(updated.control.document_number.as_deref(), Some("HR-001"));
        assert_eq!(updated.control.document_type.as_deref(), Some("procedure"));
        assert_eq!(
            updated.control.owner.map(|owner| owner.object_id),
            Some(owner_object_id)
        );
        assert_eq!(updated.workflow_verification, WorkflowVerification::Valid);
        assert_eq!(
            updated.workflow_events[0].body.event_type,
            dms_core::WorkflowEventType::DocumentControlDataChanged
        );

        let overridden =
            set_document_confidentiality(root.clone(), document.id, "restricted".into()).unwrap();
        assert_eq!(
            overridden.confidentiality_override.as_deref(),
            Some("restricted")
        );
        assert!(
            overridden
                .effective_confidentiality
                .unwrap()
                .document_override
        );
        let inherited =
            set_document_confidentiality(root.clone(), document.id, String::new()).unwrap();
        assert_eq!(inherited.confidentiality_override, None);
        assert_eq!(
            inherited.effective_confidentiality.unwrap().type_id,
            "internal"
        );

        assert!(cancel_document_review(
            root.clone(),
            document.id,
            "Requirements changed".into(),
            false,
        )
        .unwrap_err()
        .contains("explicit confirmation"));
        assert!(
            mark_document_obsolete(root.clone(), document.id, "Superseded".into(), false,)
                .unwrap_err()
                .contains("explicit confirmation")
        );
        let obsolete = mark_document_obsolete(
            root,
            document.id,
            "Superseded by global policy".into(),
            true,
        )
        .unwrap();
        assert_eq!(obsolete.lifecycle, Lifecycle::Obsolete);
        assert!(!obsolete.lifecycle_actions.mark_obsolete.available);
        assert_eq!(
            obsolete.workflow_events[0].body.event_type,
            dms_core::WorkflowEventType::DocumentObsoleted
        );
        assert_eq!(obsolete.workflow_verification, WorkflowVerification::Valid);
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
            None,
            false,
        )
        .unwrap_err();
        assert!(error.contains("explicit confirmation"));
    }

    #[test]
    fn periodic_review_closure_commands_refuse_unconfirmed_actions_before_workspace_access() {
        let mut graph = UnavailableGraphClient;
        let mut notifier = UnavailableNotificationClient;
        let completion = complete_periodic_review_with(
            "missing",
            Uuid::nil(),
            Uuid::nil(),
            PeriodicReviewResult::ConfirmedCurrent,
            "Current",
            false,
            &mut graph,
        )
        .unwrap_err();
        let cancellation = cancel_periodic_review(
            "missing".to_owned(),
            Uuid::nil(),
            Uuid::nil(),
            "Cancelled".to_owned(),
            false,
        )
        .unwrap_err();
        let reminder =
            remind_periodic_review_with("missing", Uuid::nil(), Uuid::nil(), false, &mut notifier)
                .unwrap_err();

        assert!(completion.contains("explicit confirmation"));
        assert!(cancellation.contains("explicit confirmation"));
        assert!(reminder.contains("explicit confirmation"));
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
        assert!(withdraw_release(
            root.clone(),
            Uuid::nil(),
            Uuid::nil(),
            "Correction".to_owned(),
            false,
        )
        .unwrap_err()
        .contains("explicit confirmation"));
        assert!(load_periodic_reviews(root.clone()).unwrap().is_empty());
        let archive = directory.path().join("workspace.zip");
        let outcome = backup_workspace(root, archive.to_string_lossy().into_owned()).unwrap();
        assert_eq!(outcome.entry_count, 1);
        assert_eq!(outcome.manifest_digest.len(), 64);
        assert!(archive.is_file());
    }

    #[test]
    fn desktop_integrity_commands_require_confirmed_lock_release_and_restore() {
        let directory = tempfile::tempdir().unwrap();
        let edit_root = directory.path().join("edit");
        let publish_root = directory.path().join("publish");
        fs::create_dir(&edit_root).unwrap();
        fs::create_dir(&publish_root).unwrap();
        Workspace::init(&edit_root, &publish_root).unwrap();
        let root = edit_root.to_string_lossy().into_owned();

        assert_eq!(
            workspace_lock_status(root.clone()).unwrap().state,
            dms_core::WorkspaceLockState::Unlocked
        );
        let acquired = acquire_workspace_lock(root.clone(), false, false).unwrap();
        assert_eq!(acquired.state, dms_core::WorkspaceLockState::Current);
        let owner = acquired.lock.unwrap();
        assert!(acquire_workspace_lock(root.clone(), false, false).is_err());
        assert!(acquire_workspace_lock(root.clone(), true, false).is_err());
        let overridden = acquire_workspace_lock(root.clone(), false, true).unwrap();
        let replacement_owner = overridden.lock.unwrap();
        assert_ne!(replacement_owner, owner);
        assert!(release_workspace_lock(root.clone(), owner, true).is_err());
        assert!(release_workspace_lock(root.clone(), replacement_owner.clone(), false).is_err());
        assert!(edit_root.join(".dms/lock").is_file());
        release_workspace_lock(root.clone(), replacement_owner, true).unwrap();
        assert_eq!(
            configure_workspace_lock_staleness(root.clone(), 12, true)
                .unwrap()
                .stale_after_hours,
            12
        );

        let archive = directory.path().join("workspace-restore.zip");
        backup_workspace(root, archive.to_string_lossy().into_owned()).unwrap();
        let replacement_edit = directory.path().join("replacement-edit");
        let replacement_publish = directory.path().join("replacement-publish");
        fs::create_dir(&replacement_edit).unwrap();
        fs::create_dir(&replacement_publish).unwrap();
        let restore = |confirmed| {
            restore_workspace_backup(
                archive.to_string_lossy().into_owned(),
                replacement_edit.to_string_lossy().into_owned(),
                replacement_publish.to_string_lossy().into_owned(),
                false,
                false,
                confirmed,
            )
        };
        assert!(restore(false).is_err());
        assert!(!replacement_edit.join(".dms/workspace.json").exists());
        let restored = restore(true).unwrap();
        assert_eq!(
            restored.edit_root,
            fs::canonicalize(replacement_edit).unwrap()
        );
        assert_eq!(
            restored.publish_root,
            fs::canonicalize(replacement_publish).unwrap()
        );
    }

    #[test]
    fn desktop_audit_report_commands_generate_list_verify_and_resolve_the_folder() {
        let directory = tempfile::tempdir().unwrap();
        let edit_root = directory.path().join("edit");
        let publish_root = directory.path().join("publish");
        fs::create_dir_all(&edit_root).unwrap();
        fs::create_dir_all(&publish_root).unwrap();
        let mut workspace = Workspace::init(&edit_root, &publish_root).unwrap();
        workspace
            .configure_confidentiality_type("internal", "Internal", true)
            .unwrap();
        workspace
            .set_confidentiality_policy(".", "internal")
            .unwrap();
        let source = edit_root.join("Policy.md");
        fs::write(&source, "# Policy\n\nSOURCE BYTES").unwrap();
        let document = workspace.add_document(&source).unwrap();
        workspace.save().unwrap();
        let root = edit_root.to_string_lossy().into_owned();

        let snapshot = generate_audit_report(
            root.clone(),
            dms_core::AuditReportRequest {
                format: dms_core::AuditReportFormat::Csv,
                relative_path: Some(PathBuf::from(".dms/exports/policy.csv")),
                filter: dms_core::AuditReportFilter {
                    document_ids: vec![document.id],
                    confidentiality_type_ids: vec!["internal".to_owned()],
                    ..Default::default()
                },
            },
        )
        .unwrap();

        assert_eq!(snapshot.rows.len(), 1);
        assert_eq!(
            snapshot.rows[0].verification,
            AuditReportVerificationStatus::Match
        );
        let event_id = snapshot.rows[0].report.event_id;
        assert_eq!(load_audit_reports(root.clone()).unwrap(), snapshot);
        assert_eq!(
            verify_audit_report(root, event_id).unwrap().rows[0].verification,
            AuditReportVerificationStatus::Match
        );
        assert_eq!(
            audit_report_folder(&Workspace::open(&edit_root).unwrap(), event_id).unwrap(),
            fs::canonicalize(edit_root.join(".dms/exports")).unwrap()
        );
    }

    #[test]
    fn production_lifecycle_helpers_compose_graph_delivery_and_pdf_export_with_fakes() {
        struct TestGraph {
            tenant_id: Uuid,
            people: Vec<EntraPerson>,
        }

        impl GraphClient for TestGraph {
            fn tenant_id(&self) -> std::result::Result<Uuid, String> {
                Ok(self.tenant_id)
            }

            fn direct_user_members(
                &mut self,
                _source: &EntraIdentitySource,
            ) -> std::result::Result<Vec<EntraPerson>, String> {
                Ok(self.people.clone())
            }

            fn authenticated_actor(
                &mut self,
                _source: &EntraIdentitySource,
            ) -> std::result::Result<AuthenticatedActor, String> {
                Err("the signed-in desktop wrapper supplies the actor".to_owned())
            }
        }

        struct TestNotifier;

        impl NotificationClient for TestNotifier {
            fn send(
                &mut self,
                _settings: &NotificationSettings,
                _message: &dms_core::NotificationMessage,
            ) -> std::result::Result<dms_core::DeliveryReceipt, String> {
                Ok(dms_core::DeliveryReceipt::accepted(250, "accepted"))
            }
        }

        struct TestExporter;

        impl PdfExporter for TestExporter {
            fn export(
                &mut self,
                request: &dms_core::ExportRequest,
            ) -> std::result::Result<(), String> {
                fs::write(&request.temporary_pdf_path, b"%PDF-1.7\nfake export")
                    .map_err(|error| error.to_string())
            }
        }

        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(edit_root.path().join("Policies")).unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        let template_path = edit_root.path().join("Markdown-template.docx");
        fs::write(
            &template_path,
            include_bytes!("../../dms-core/tests/fixtures/markdown-template.docx"),
        )
        .unwrap();
        workspace.import_markdown_template(&template_path).unwrap();
        workspace
            .configure_document_type("procedure", "Procedure", true)
            .unwrap();
        workspace
            .configure_confidentiality_type("internal", "Internal", true)
            .unwrap();
        workspace
            .set_confidentiality_policy(".", "internal")
            .unwrap();
        let tenant_id = Uuid::new_v4();
        let editor_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();
        let people = vec![
            EntraPerson::eligible(editor_id, "Eva Editor", "editor@example.test"),
            EntraPerson::eligible(approver_id, "Ada Approver", "approver@example.test"),
            EntraPerson::eligible(requester_id, "Rita Requester", "requester@example.test"),
        ];
        workspace
            .replace_identity_source(Uuid::new_v4(), "DMS workflow", people.clone())
            .unwrap();
        workspace
            .update_workflow_policy(
                ".",
                RoleUpdate::replace(editor_id),
                RoleUpdate::replace(approver_id),
            )
            .unwrap();
        workspace
            .configure_notifications(
                NotificationTransport::Smtp,
                Some(SmtpSettings {
                    relay_host: "smtp.example.test".to_owned(),
                    relay_port: 587,
                    login_user: "dms@example.test".to_owned(),
                    from_mailbox: "dms@example.test".to_owned(),
                }),
            )
            .unwrap();
        let source = edit_root.path().join("Policies/Handbook.md");
        fs::write(
            &source,
            "---\nversion: 1.0\nconfidentiality: internal\n---\n# Handbook\n",
        )
        .unwrap();
        let document = workspace.add_document(&source).unwrap();
        let binding_id = workspace.identity_source().unwrap().binding_id;
        workspace
            .update_control(
                document.id,
                ControlUpdate {
                    title: Some("Employee handbook".to_owned()),
                    document_type: Some(Some("procedure".to_owned())),
                    owner: Some(Some(OwnerReference {
                        binding_id,
                        object_id: editor_id,
                    })),
                    ..ControlUpdate::default()
                },
            )
            .unwrap();
        workspace.save().unwrap();
        let root = edit_root.path().to_string_lossy().into_owned();
        let mut graph = TestGraph { tenant_id, people };
        let mut notifier = TestNotifier;

        let submitted = submit_document_candidate_with(
            &root,
            CandidateSubmissionInput {
                document_id: document.id,
                target_mode: "next_major".to_owned(),
                manual_major: None,
                manual_minor: None,
                changelog: "Clarify escalation path".to_owned(),
                effective_date: "2026-08-15".to_owned(),
                requester_object_id: requester_id,
                staged_owner_object_id: None,
                staged_editor_object_id: None,
                review_override_reason: String::new(),
                mailto_confirmed: false,
            },
            &mut graph,
            &mut notifier,
        )
        .unwrap();
        assert_eq!(
            submitted.active_candidate.as_ref().unwrap().status,
            dms_core::CandidateStatus::InReview
        );

        let approved = decide_document_review_with(
            &root,
            document.id,
            ReviewDecision::Approved,
            "Ready for release".to_owned(),
            AuthenticatedActor {
                tenant_id,
                object_id: approver_id,
            },
            &mut graph,
            &mut notifier,
        )
        .unwrap();
        assert_eq!(
            approved.active_candidate.as_ref().unwrap().status,
            dms_core::CandidateStatus::Approved
        );

        let released = release_document_candidate_with(
            &root,
            document.id,
            String::new(),
            &mut graph,
            &mut notifier,
            &mut TestExporter,
        )
        .unwrap();
        assert!(released.current_release.unwrap().pdf_exists);
        let maintenance = load_releases(root).unwrap();
        let release = &maintenance.rows[0];
        assert_eq!(release.document_title, "Employee handbook");
        assert_eq!(release.effective_date, NaiveDate::from_ymd_opt(2026, 8, 15));
        let profile = release.profile.as_ref().unwrap();
        assert_eq!(profile.title, "Employee handbook");
        assert_eq!(profile.document_type.as_deref(), Some("procedure"));
        assert_eq!(profile.owner.as_ref().unwrap().object_id, editor_id);
    }

    #[cfg(target_os = "windows")]
    #[test]
    #[ignore = "requires installed Microsoft Word and explicit operator input paths"]
    fn windows_installed_word_releases_operator_markdown_template() {
        struct TestNotifier;

        impl NotificationClient for TestNotifier {
            fn send(
                &mut self,
                _settings: &NotificationSettings,
                _message: &dms_core::NotificationMessage,
            ) -> std::result::Result<dms_core::DeliveryReceipt, String> {
                Ok(dms_core::DeliveryReceipt::accepted(250, "accepted"))
            }
        }

        let required_path = |name: &str| {
            PathBuf::from(
                env::var_os(name)
                    .unwrap_or_else(|| panic!("{name} must name an explicit Windows path")),
            )
        };
        let source_input = required_path("DMS_WINDOWS_MARKDOWN_SMOKE_SOURCE");
        let template_input = required_path("DMS_WINDOWS_MARKDOWN_SMOKE_TEMPLATE");
        let smoke_root = required_path("DMS_WINDOWS_MARKDOWN_SMOKE_ROOT");
        assert!(source_input.is_file(), "Markdown smoke source is missing");
        assert!(template_input.is_file(), "Word smoke template is missing");
        assert!(
            !smoke_root.exists(),
            "retained smoke workspace already exists"
        );

        let source_before = fs::read(&source_input).unwrap();
        let template_before = fs::read(&template_input).unwrap();
        let edit_root = smoke_root.join("edit");
        let publish_root = smoke_root.join("publish");
        fs::create_dir_all(edit_root.join("81_ISO 27001.2024")).unwrap();
        fs::create_dir(&publish_root).unwrap();
        let source = edit_root
            .join("81_ISO 27001.2024")
            .join(source_input.file_name().unwrap());
        let template = edit_root.join("Vorlage.docx");
        fs::copy(&source_input, &source).unwrap();
        fs::copy(&template_input, &template).unwrap();

        let assembled = smoke_root.join("assembled.docx");
        let filled = smoke_root.join("release.docx");
        let preflight_pdf = smoke_root.join("preflight.pdf");
        dms_core::assemble_markdown_docx(
            &template,
            &fs::read_to_string(&source).unwrap(),
            &assembled,
        )
        .unwrap();
        export::fill_office_placeholders(
            &assembled,
            &filled,
            &dms_core::ExportChrome {
                version_label: "V1.0".to_owned(),
                confidentiality: dms_core::ConfidentialitySnapshot {
                    type_id: "internal-allied".to_owned(),
                    label: "Intern + Unternehmensverbund".to_owned(),
                },
                title: "A.8.29 Sicherheitsprüfung bei Entwicklung und Abnahme".to_owned(),
                document_number: None,
            },
        )
        .unwrap();
        export::OfficeAutomation::export_pdf(
            &mut export::InstalledOfficeAutomation,
            &filled,
            &preflight_pdf,
        )
        .unwrap();

        let mut workspace = Workspace::init(&edit_root, &publish_root).unwrap();
        workspace.import_markdown_template(&template).unwrap();
        workspace
            .configure_document_type("procedure", "Verfahrensanweisung", true)
            .unwrap();
        workspace
            .configure_confidentiality_type("internal-allied", "Intern + Unternehmensverbund", true)
            .unwrap();
        workspace
            .set_confidentiality_policy(".", "internal-allied")
            .unwrap();
        let tenant_id = Uuid::new_v4();
        let editor_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();
        let people = vec![
            EntraPerson::eligible(editor_id, "Windows Smoke Editor", "editor@example.test"),
            EntraPerson::eligible(
                approver_id,
                "Windows Smoke Approver",
                "approver@example.test",
            ),
            EntraPerson::eligible(
                requester_id,
                "Windows Smoke Requester",
                "requester@example.test",
            ),
        ];
        workspace
            .replace_identity_source(Uuid::new_v4(), "Windows smoke", people.clone())
            .unwrap();
        workspace
            .update_workflow_policy(
                ".",
                RoleUpdate::replace(editor_id),
                RoleUpdate::replace(approver_id),
            )
            .unwrap();
        workspace
            .configure_notifications(
                NotificationTransport::Smtp,
                Some(SmtpSettings {
                    relay_host: "smtp.example.test".to_owned(),
                    relay_port: 587,
                    login_user: "dms@example.test".to_owned(),
                    from_mailbox: "dms@example.test".to_owned(),
                }),
            )
            .unwrap();
        let document = workspace.add_document(&source).unwrap();
        let binding_id = workspace.identity_source().unwrap().binding_id;
        workspace
            .update_control(
                document.id,
                ControlUpdate {
                    title: Some("A.8.29 Sicherheitsprüfung bei Entwicklung und Abnahme".to_owned()),
                    document_type: Some(Some("procedure".to_owned())),
                    owner: Some(Some(OwnerReference {
                        binding_id,
                        object_id: editor_id,
                    })),
                    ..ControlUpdate::default()
                },
            )
            .unwrap();
        workspace.save().unwrap();

        let root = edit_root.to_string_lossy().into_owned();
        let mut graph = TestGraph { tenant_id, people };
        let mut notifier = TestNotifier;
        let submitted = submit_document_candidate_with(
            &root,
            CandidateSubmissionInput {
                document_id: document.id,
                target_mode: "next_minor".to_owned(),
                manual_major: None,
                manual_minor: None,
                changelog: "Validate the Word-template Markdown release path".to_owned(),
                effective_date: "2026-08-15".to_owned(),
                requester_object_id: requester_id,
                staged_owner_object_id: None,
                staged_editor_object_id: None,
                review_override_reason: String::new(),
                mailto_confirmed: false,
            },
            &mut graph,
            &mut notifier,
        )
        .unwrap();
        assert_eq!(
            submitted.active_candidate.as_ref().unwrap().status,
            dms_core::CandidateStatus::InReview
        );
        decide_document_review_with(
            &root,
            document.id,
            ReviewDecision::Approved,
            "Installed-Word smoke approved".to_owned(),
            AuthenticatedActor {
                tenant_id,
                object_id: approver_id,
            },
            &mut graph,
            &mut notifier,
        )
        .unwrap();
        let mut exporter = export::LocalPdfExporter::new(export::InstalledOfficeAutomation);
        release_document_candidate_with(
            &root,
            document.id,
            String::new(),
            &mut graph,
            &mut notifier,
            &mut exporter,
        )
        .unwrap();

        let reopened = Workspace::open(&edit_root).unwrap();
        let release = reopened.current_release(document.id).unwrap().unwrap();
        let pdf = reopened.publish_root.join(&release.relative_pdf_path);
        assert!(pdf.is_file());
        assert_eq!(fs::read(&source_input).unwrap(), source_before);
        assert_eq!(fs::read(&template_input).unwrap(), template_before);
        println!("DMS_SMOKE_PDF={}", pdf.display());
        println!("DMS_SMOKE_SHA256={}", release.pdf_digest);
        println!("DMS_SMOKE_VERSION={}", release.version);
    }

    fn lost_source_workspace() -> (tempfile::TempDir, tempfile::TempDir, Uuid, String) {
        let edit_root = tempfile::tempdir().unwrap();
        let publish_root = tempfile::tempdir().unwrap();
        let mut workspace = Workspace::init(edit_root.path(), publish_root.path()).unwrap();
        fs::create_dir_all(edit_root.path().join("Policies")).unwrap();
        let original = edit_root.path().join("Policies/Handbook.md");
        fs::write(&original, "# Handbook").unwrap();
        let document = workspace.add_document(&original).unwrap();
        fs::remove_file(&original).unwrap();
        workspace.save().unwrap();
        let root = edit_root.path().to_string_lossy().into_owned();
        (edit_root, publish_root, document.id, root)
    }

    #[test]
    fn reassociate_source_picker_offers_only_supported_drafts() {
        assert_eq!(
            REASSOCIATE_SOURCE_FILTER_EXTENSIONS,
            &["md", "docx", "xlsx", "pptx"]
        );
        assert!(!REASSOCIATE_SOURCE_FILTER_EXTENSIONS
            .iter()
            .any(|extension| *extension == "*" || extension.contains('*')));
        assert_eq!(REASSOCIATE_SOURCE_FILTER_NAME, "Supported drafts");
    }

    #[test]
    fn reassociate_picker_starts_at_last_known_folder_or_edit_root() {
        let edit_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(edit_root.path().join("Policies")).unwrap();
        let policies =
            reassociate_source_picker_start_dir(edit_root.path(), "Policies/Handbook.md");
        assert_eq!(
            policies,
            edit_root.path().join("Policies").canonicalize().unwrap()
        );
        fs::remove_dir_all(edit_root.path().join("Policies")).unwrap();
        let fallback =
            reassociate_source_picker_start_dir(edit_root.path(), "Policies/Handbook.md");
        assert_eq!(fallback, edit_root.path());
    }

    #[test]
    fn desktop_reassociate_refuses_outside_registered_and_unsupported_paths() {
        let (edit_root, _publish_root, document_id, root) = lost_source_workspace();
        fs::write(edit_root.path().join("Policies/notes.txt"), "plain").unwrap();
        fs::write(edit_root.path().join("Policies/Occupied.md"), "# Occupied").unwrap();
        let occupied = add_library_documents(
            root.clone(),
            vec![edit_root
                .path()
                .join("Policies/Occupied.md")
                .to_string_lossy()
                .into_owned()],
        )
        .unwrap();
        assert_eq!(occupied.len(), 1);

        let outside = tempfile::NamedTempFile::with_suffix(".md").unwrap();
        fs::write(outside.path(), "# Outside").unwrap();
        let outside_error = reassociate_library_document(
            root.clone(),
            document_id,
            outside.path().to_string_lossy().into_owned(),
        )
        .expect_err("outside path");
        assert!(outside_error.contains(DESKTOP_REASSOCIATE_RULE_LOCATION));
        assert!(outside_error.contains(
            "The selected file must be a supported unregistered source file inside the edit root."
        ));

        let unsupported = reassociate_library_document(
            root.clone(),
            document_id,
            "Policies/notes.txt".to_owned(),
        )
        .expect_err("unsupported path");
        assert!(unsupported.contains(DESKTOP_REASSOCIATE_RULE_FORMAT));

        let registered = reassociate_library_document(
            root.clone(),
            document_id,
            "Policies/Occupied.md".to_owned(),
        )
        .expect_err("registered path");
        assert!(registered.contains(DESKTOP_REASSOCIATE_RULE_UNREGISTERED));

        let workspace = Workspace::open(edit_root.path()).unwrap();
        assert_eq!(workspace.documents().len(), 2);
        assert_eq!(
            workspace.document(document_id).unwrap().relative_path,
            Path::new("Policies/Handbook.md")
        );
        assert_eq!(
            workspace.document(occupied[0].id).unwrap().relative_path,
            Path::new("Policies/Occupied.md")
        );
        assert_eq!(
            workspace.document(occupied[0].id).unwrap().source_state,
            SourceState::Registered
        );
    }

    #[test]
    fn desktop_reassociate_accepts_an_unregistered_in_root_file() {
        let (edit_root, _publish_root, document_id, root) = lost_source_workspace();
        fs::write(
            edit_root.path().join("Policies/Relocated.md"),
            "# Relocated",
        )
        .unwrap();
        let restored =
            reassociate_library_document(root, document_id, "Policies/Relocated.md".to_owned())
                .unwrap();
        assert_eq!(restored.relative_path, Path::new("Policies/Relocated.md"));
        assert_eq!(restored.source_state, SourceState::Registered);
        let workspace = Workspace::open(edit_root.path()).unwrap();
        assert_eq!(workspace.documents().len(), 1);
        assert_eq!(
            workspace.document(document_id).unwrap().relative_path,
            Path::new("Policies/Relocated.md")
        );
    }
}
