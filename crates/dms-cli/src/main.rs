use std::{error::Error, fs, io, path::PathBuf, process};

use clap::{Parser, Subcommand, ValueEnum};
use dms_core::{
    AuditReportFilter, AuditReportFormat, AuditReportRequest, AuthenticatedActor, ControlUpdate,
    DeliveryReceipt, Document, EntraIdentitySource, EntraPerson, GraphClient, Note,
    NotificationClient, NotificationMessage, NotificationSettings, PeriodicReviewResult,
    RestoreRequest, RoleUpdate, Workspace,
};
use serde::Serialize;
use uuid::Uuid;

type CliResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(name = "dms", version, about = "Headless local DMS core operations")]
struct Cli {
    #[arg(long, global = true, help = "Emit structured JSON results")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },
    Document {
        #[command(subcommand)]
        command: DocumentCommand,
    },
    Note {
        #[command(subcommand)]
        command: NoteCommand,
    },
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },
    Library {
        #[command(subcommand)]
        command: LibraryCommand,
    },
    PeriodicReview {
        #[command(subcommand)]
        command: PeriodicReviewCommand,
    },
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
}

#[derive(Debug, Subcommand)]
enum WorkspaceCommand {
    Init {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        publish_root: PathBuf,
        #[arg(long, help = "Confirm creation of <edit-root>/.dms")]
        confirm: bool,
    },
    Status {
        #[arg(long)]
        edit_root: PathBuf,
    },
    Verify {
        #[arg(long)]
        edit_root: PathBuf,
    },
    LockStatus {
        #[arg(long)]
        edit_root: PathBuf,
    },
    LockAcquire {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long, help = "Explicitly replace a stale advisory lock")]
        take_over_stale: bool,
    },
    LockRelease {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long, help = "Confirm removal of the advisory lock")]
        confirm: bool,
    },
    ConfigureLockStaleness {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        hours: u32,
        #[arg(long, help = "Confirm the workspace lock-staleness setting")]
        confirm: bool,
    },
    Backup {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        archive: PathBuf,
    },
    Restore {
        #[arg(long)]
        archive: PathBuf,
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        publish_root: PathBuf,
        #[arg(long, help = "Replace manifest-listed destination files")]
        replace_existing: bool,
        #[arg(long, help = "Remove a stale destination lock before restore")]
        take_over_stale_lock: bool,
        #[arg(long, help = "Confirm restore into the selected roots")]
        confirm: bool,
    },
}

#[derive(Debug, Subcommand)]
enum DocumentCommand {
    Add {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        path: PathBuf,
    },
    List {
        #[arg(long)]
        edit_root: PathBuf,
    },
    Show {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
    },
    UpdateControl {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        document_number: Option<String>,
        #[arg(long)]
        document_type: Option<String>,
        #[arg(long)]
        owner: Option<String>,
    },
    Unregister {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
    },
    Reassociate {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
        #[arg(long)]
        path: PathBuf,
    },
    Permalink {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
    },
}

#[derive(Debug, Subcommand)]
enum LibraryCommand {
    Tree {
        #[arg(long)]
        edit_root: PathBuf,
    },
    List {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long, default_value = ".")]
        folder: PathBuf,
    },
    Search {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long, default_value = ".")]
        folder: PathBuf,
        #[arg(long)]
        query: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum PeriodicReviewResultArg {
    ConfirmedCurrent,
    ChangesRequired,
    Obsolete,
}

impl From<PeriodicReviewResultArg> for PeriodicReviewResult {
    fn from(value: PeriodicReviewResultArg) -> Self {
        match value {
            PeriodicReviewResultArg::ConfirmedCurrent => Self::ConfirmedCurrent,
            PeriodicReviewResultArg::ChangesRequired => Self::ChangesRequired,
            PeriodicReviewResultArg::Obsolete => Self::Obsolete,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ReportFormatArg {
    Csv,
    Pdf,
}

impl From<ReportFormatArg> for AuditReportFormat {
    fn from(value: ReportFormatArg) -> Self {
        match value {
            ReportFormatArg::Csv => Self::Csv,
            ReportFormatArg::Pdf => Self::Pdf,
        }
    }
}

#[derive(Debug, Subcommand)]
enum ReportCommand {
    Generate {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long, value_enum)]
        format: ReportFormatArg,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        document: Vec<Uuid>,
        #[arg(long)]
        approver: Vec<Uuid>,
        #[arg(long)]
        confidentiality: Vec<String>,
        #[arg(long, value_name = "RFC3339")]
        from: Option<String>,
        #[arg(long, value_name = "RFC3339")]
        through: Option<String>,
    },
    List {
        #[arg(long)]
        edit_root: PathBuf,
    },
    Verify {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        event: Uuid,
    },
}

#[derive(Debug, Subcommand)]
enum PeriodicReviewCommand {
    List {
        #[arg(long)]
        edit_root: PathBuf,
    },
    Start {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
    },
    Result {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
        #[arg(long)]
        review: Uuid,
        #[arg(long)]
        result: PeriodicReviewResultArg,
        #[arg(long)]
        comment: String,
        #[arg(long, help = "Confirm recording the periodic-review result")]
        confirm: bool,
    },
    Cancel {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
        #[arg(long)]
        review: Uuid,
        #[arg(long)]
        comment: String,
        #[arg(
            long,
            help = "Confirm cancellation without changing the review schedule"
        )]
        confirm: bool,
    },
    Reminder {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
        #[arg(long)]
        review: Uuid,
        #[arg(long, help = "Confirm sending a reminder to the snapshotted approver")]
        confirm: bool,
    },
}

#[derive(Debug, Subcommand)]
enum NoteCommand {
    Add {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
        #[arg(long)]
        body: String,
        #[arg(long)]
        author: Option<String>,
    },
    List {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
    },
    Edit {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
        #[arg(long)]
        note: Uuid,
        #[arg(long)]
        body: String,
    },
    Remove {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        document: Uuid,
        #[arg(long)]
        note: Uuid,
    },
}

#[derive(Debug, Subcommand)]
enum PolicyCommand {
    Folders {
        #[arg(long)]
        edit_root: PathBuf,
    },
    ConfigureConfidentialityType {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        label: String,
        #[arg(long, default_value_t = true)]
        enabled: bool,
        #[arg(long, help = "Set this type as the required edit-root policy")]
        root: bool,
    },
    ConfigureDocumentType {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        label: String,
        #[arg(long, default_value_t = true)]
        enabled: bool,
    },
    SetConfidentiality {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        folder: String,
        #[arg(long)]
        type_id: String,
    },
    RemoveConfidentiality {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        folder: String,
    },
    ReplaceIdentitySource {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        tenant_id: Uuid,
        #[arg(long)]
        tenant_display: String,
        #[arg(long)]
        group_id: Uuid,
        #[arg(long)]
        group_label: String,
        #[arg(long, value_name = "@file:PATH")]
        eligible_people: String,
        #[arg(long)]
        root_editor: Uuid,
        #[arg(long)]
        root_approver: Uuid,
    },
    SetWorkflowRoles {
        #[arg(long)]
        edit_root: PathBuf,
        #[arg(long)]
        folder: String,
        #[arg(long)]
        editor: Option<Uuid>,
        #[arg(long)]
        approver: Option<Uuid>,
        #[arg(long, conflicts_with = "editor")]
        clear_editor: bool,
        #[arg(long, conflicts_with = "approver")]
        clear_approver: bool,
    },
}

#[derive(Serialize)]
struct VerificationResult {
    workspace_id: Uuid,
    schema_version: u32,
    document_count: usize,
    result: &'static str,
}

#[derive(Serialize)]
struct RemovalResult {
    document_id: Uuid,
    note_id: Uuid,
    result: &'static str,
}

struct UnavailableGraphClient;

impl GraphClient for UnavailableGraphClient {
    fn direct_user_members(
        &mut self,
        _source: &EntraIdentitySource,
    ) -> std::result::Result<Vec<EntraPerson>, String> {
        Err("live Microsoft Graph integration is not configured".to_owned())
    }

    fn authenticated_actor(
        &mut self,
        _source: &EntraIdentitySource,
    ) -> std::result::Result<AuthenticatedActor, String> {
        Err("live interactive Microsoft Entra sign-in is not configured".to_owned())
    }
}

struct UnavailableNotificationClient;

impl NotificationClient for UnavailableNotificationClient {
    fn send(
        &mut self,
        _settings: &NotificationSettings,
        _message: &NotificationMessage,
    ) -> std::result::Result<DeliveryReceipt, String> {
        Err("live notification delivery is not configured".to_owned())
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(cli) {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> CliResult<()> {
    match cli.command {
        Command::Workspace { command } => run_workspace(command, cli.json),
        Command::Document { command } => run_document(command, cli.json),
        Command::Note { command } => run_note(command, cli.json),
        Command::Policy { command } => run_policy(command, cli.json),
        Command::Library { command } => run_library(command, cli.json),
        Command::PeriodicReview { command } => run_periodic_review(command, cli.json),
        Command::Report { command } => run_report(command, cli.json),
    }
}

fn run_report(command: ReportCommand, json: bool) -> CliResult<()> {
    match command {
        ReportCommand::Generate {
            edit_root,
            format,
            output,
            document,
            approver,
            confidentiality,
            from,
            through,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let report = workspace.generate_audit_report(AuditReportRequest {
                format: format.into(),
                relative_path: output,
                filter: AuditReportFilter {
                    document_ids: document,
                    approver_object_ids: approver,
                    confidentiality_type_ids: confidentiality,
                    from: parse_report_time(from.as_deref())?,
                    through: parse_report_time(through.as_deref())?,
                },
            })?;
            let message = format!("generated audit report {}", report.relative_path);
            print_value(&report, json, message)
        }
        ReportCommand::List { edit_root } => {
            let workspace = Workspace::open(&edit_root)?;
            let reports = workspace.recent_reports();
            let count = reports.len();
            print_value(&reports, json, format!("{count} audit reports"))
        }
        ReportCommand::Verify { edit_root, event } => {
            let workspace = Workspace::open(&edit_root)?;
            let verification = workspace.verify_report(event)?;
            let message = format!("audit report verification: {:?}", verification.status);
            print_value(&verification, json, message)
        }
    }
}

fn parse_report_time(value: Option<&str>) -> CliResult<Option<chrono::DateTime<chrono::Utc>>> {
    value
        .map(|value| {
            chrono::DateTime::parse_from_rfc3339(value)
                .map(|timestamp| timestamp.with_timezone(&chrono::Utc))
                .map_err(|error| input_error(&format!("invalid RFC3339 report timestamp: {error}")))
        })
        .transpose()
}

fn run_periodic_review(command: PeriodicReviewCommand, json: bool) -> CliResult<()> {
    match command {
        PeriodicReviewCommand::List { edit_root } => {
            let workspace = Workspace::open(&edit_root)?;
            let markers = workspace.periodic_review_markers(chrono::Utc::now().date_naive())?;
            let count = markers.len();
            print_value(&markers, json, format!("{count} periodic-review markers"))
        }
        PeriodicReviewCommand::Start {
            edit_root,
            document,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let review = workspace.start_periodic_review(document)?;
            print_value(
                &review,
                json,
                format!("started periodic review {}", review.id),
            )
        }
        PeriodicReviewCommand::Result {
            edit_root,
            document,
            review,
            result,
            comment,
            confirm,
        } => {
            if !confirm {
                return Err(input_error("periodic-review result requires --confirm"));
            }
            let mut workspace = Workspace::open(&edit_root)?;
            let mut graph = UnavailableGraphClient;
            let completed = workspace.complete_periodic_review(
                document,
                review,
                result.into(),
                &comment,
                &mut graph,
            )?;
            print_value(
                &completed,
                json,
                format!("recorded periodic-review result for {}", completed.id),
            )
        }
        PeriodicReviewCommand::Cancel {
            edit_root,
            document,
            review,
            comment,
            confirm,
        } => {
            if !confirm {
                return Err(input_error(
                    "periodic-review cancellation requires --confirm",
                ));
            }
            let mut workspace = Workspace::open(&edit_root)?;
            let cancelled = workspace.cancel_periodic_review(document, review, &comment)?;
            print_value(
                &cancelled,
                json,
                format!("cancelled periodic review {}", cancelled.id),
            )
        }
        PeriodicReviewCommand::Reminder {
            edit_root,
            document,
            review,
            confirm,
        } => {
            if !confirm {
                return Err(input_error("periodic-review reminder requires --confirm"));
            }
            let mut workspace = Workspace::open(&edit_root)?;
            let mut notifier = UnavailableNotificationClient;
            let attempt = workspace.remind_periodic_review(document, review, &mut notifier)?;
            print_value(
                &attempt,
                json,
                format!("periodic-review reminder status: {:?}", attempt.status),
            )
        }
    }
}

fn run_library(command: LibraryCommand, json: bool) -> CliResult<()> {
    match command {
        LibraryCommand::Tree { edit_root } => {
            let workspace = Workspace::open(&edit_root)?;
            let tree = workspace.library_tree()?;
            print_value(&tree, json, format!("{} library folders", tree.len()))
        }
        LibraryCommand::List { edit_root, folder } => {
            let workspace = Workspace::open(&edit_root)?;
            let listing = workspace.library_folder(&folder)?;
            let count = listing.entries.len();
            print_value(&listing, json, format!("{count} entries"))
        }
        LibraryCommand::Search {
            edit_root,
            folder,
            query,
        } => {
            let workspace = Workspace::open(&edit_root)?;
            let results = workspace.search_library(&folder, &query)?;
            let count = results.len();
            print_value(&results, json, format!("{count} matching files"))
        }
    }
}

fn run_workspace(command: WorkspaceCommand, json: bool) -> CliResult<()> {
    match command {
        WorkspaceCommand::Init {
            edit_root,
            publish_root,
            confirm,
        } => {
            if !confirm {
                return Err(input_error("workspace initialization requires --confirm"));
            }
            let workspace = Workspace::init(&edit_root, &publish_root)?;
            print_value(
                &workspace,
                json,
                format!(
                    "initialized workspace {} at {}",
                    workspace.workspace_id,
                    workspace.edit_root.display()
                ),
            )
        }
        WorkspaceCommand::Status { edit_root } => {
            let workspace = Workspace::open(&edit_root)?;
            print_value(
                &workspace,
                json,
                format!(
                    "workspace {}: {} documents; edit root {}; publish root {}",
                    workspace.workspace_id,
                    workspace.documents().len(),
                    workspace.edit_root.display(),
                    workspace.publish_root.display()
                ),
            )
        }
        WorkspaceCommand::Verify { edit_root } => {
            let workspace = Workspace::open(&edit_root)?;
            workspace.validate()?;
            let result = VerificationResult {
                workspace_id: workspace.workspace_id,
                schema_version: workspace.schema_version,
                document_count: workspace.documents().len(),
                result: "valid",
            };
            print_value(
                &result,
                json,
                format!(
                    "workspace {} is valid ({} documents)",
                    result.workspace_id, result.document_count
                ),
            )
        }
        WorkspaceCommand::LockStatus { edit_root } => {
            let workspace = Workspace::open(&edit_root)?;
            let status = workspace.lock_status()?;
            print_value(&status, json, format!("workspace lock: {:?}", status.state))
        }
        WorkspaceCommand::LockAcquire {
            edit_root,
            take_over_stale,
        } => {
            let workspace = Workspace::open(&edit_root)?;
            let status = workspace.acquire_lock(take_over_stale)?;
            print_value(&status, json, "workspace advisory lock acquired".to_owned())
        }
        WorkspaceCommand::LockRelease { edit_root, confirm } => {
            if !confirm {
                return Err(input_error("workspace lock release requires --confirm"));
            }
            dms_core::release_workspace_lock(&edit_root)?;
            print_value(
                &serde_json::json!({ "result": "released" }),
                json,
                "workspace advisory lock released".to_owned(),
            )
        }
        WorkspaceCommand::ConfigureLockStaleness {
            edit_root,
            hours,
            confirm,
        } => {
            if !confirm {
                return Err(input_error(
                    "workspace lock-staleness configuration requires --confirm",
                ));
            }
            let mut workspace = Workspace::open(&edit_root)?;
            workspace.configure_lock_staleness(hours)?;
            print_value(
                &serde_json::json!({ "stale_after_hours": hours }),
                json,
                format!("workspace locks become stale after {hours} hours"),
            )
        }
        WorkspaceCommand::Backup { edit_root, archive } => {
            let workspace = Workspace::open(&edit_root)?;
            let outcome = workspace.backup_workspace(&archive)?;
            print_value(
                &outcome,
                json,
                format!("created backup {}", archive.display()),
            )
        }
        WorkspaceCommand::Restore {
            archive,
            edit_root,
            publish_root,
            replace_existing,
            take_over_stale_lock,
            confirm,
        } => {
            let outcome = dms_core::restore_workspace_backup(RestoreRequest {
                archive_path: &archive,
                edit_root: &edit_root,
                publish_root: &publish_root,
                replace_existing,
                take_over_stale_lock,
                confirmed: confirm,
            })?;
            print_value(
                &outcome,
                json,
                format!("restored workspace {}", outcome.workspace_id),
            )
        }
    }
}

fn run_policy(command: PolicyCommand, json: bool) -> CliResult<()> {
    match command {
        PolicyCommand::Folders { edit_root } => {
            let workspace = Workspace::open(&edit_root)?;
            let folders = workspace.policy_folders()?;
            print_value(&folders, json, format!("{} policy folders", folders.len()))
        }
        PolicyCommand::ConfigureConfidentialityType {
            edit_root,
            id,
            label,
            enabled,
            root,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let configured = workspace.configure_confidentiality_type(&id, &label, enabled)?;
            if root {
                workspace.set_confidentiality_policy(".", &id)?;
            }
            workspace.save()?;
            print_value(
                &configured,
                json,
                format!("configured confidentiality type {id}"),
            )
        }
        PolicyCommand::ConfigureDocumentType {
            edit_root,
            id,
            label,
            enabled,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let configured = workspace.configure_document_type(&id, &label, enabled)?;
            workspace.save()?;
            print_value(&configured, json, format!("configured document type {id}"))
        }
        PolicyCommand::SetConfidentiality {
            edit_root,
            folder,
            type_id,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let policy = workspace.set_confidentiality_policy(&folder, &type_id)?;
            workspace.save()?;
            print_value(
                &policy,
                json,
                format!("set confidentiality policy for {folder}"),
            )
        }
        PolicyCommand::RemoveConfidentiality { edit_root, folder } => {
            let mut workspace = Workspace::open(&edit_root)?;
            workspace.remove_confidentiality_policy(&folder)?;
            workspace.save()?;
            print_value(
                &folder,
                json,
                format!("removed confidentiality policy for {folder}"),
            )
        }
        PolicyCommand::ReplaceIdentitySource {
            edit_root,
            tenant_id,
            tenant_display,
            group_id,
            group_label,
            eligible_people,
            root_editor,
            root_approver,
        } => {
            let people: Vec<EntraPerson> = read_marked_json(&eligible_people)?;
            let mut workspace = Workspace::open(&edit_root)?;
            let source = workspace.replace_identity_source(
                tenant_id,
                &tenant_display,
                group_id,
                &group_label,
                people,
            )?;
            workspace.update_workflow_policy(
                ".",
                RoleUpdate::replace(root_editor),
                RoleUpdate::replace(root_approver),
            )?;
            workspace.save()?;
            print_value(&source, json, "replaced identity source".to_owned())
        }
        PolicyCommand::SetWorkflowRoles {
            edit_root,
            folder,
            editor,
            approver,
            clear_editor,
            clear_approver,
        } => {
            let editor = role_update(editor, clear_editor);
            let approver = role_update(approver, clear_approver);
            if editor == RoleUpdate::Unchanged && approver == RoleUpdate::Unchanged {
                return Err(input_error("set-workflow-roles requires a role update"));
            }
            let mut workspace = Workspace::open(&edit_root)?;
            let policy = workspace.update_workflow_policy(&folder, editor, approver)?;
            workspace.save()?;
            print_value(
                &policy,
                json,
                format!("updated workflow roles for {folder}"),
            )
        }
    }
}

fn run_document(command: DocumentCommand, json: bool) -> CliResult<()> {
    match command {
        DocumentCommand::Add { edit_root, path } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let document = workspace.add_document(&path)?;
            workspace.save()?;
            print_document(&document, json, "registered")
        }
        DocumentCommand::List { edit_root } => {
            let workspace = Workspace::open(&edit_root)?;
            let documents = workspace.documents();
            let text = if documents.is_empty() {
                "no registered documents".to_owned()
            } else {
                documents
                    .iter()
                    .map(|document| {
                        format!(
                            "{}  {}  {}",
                            document.id,
                            document.relative_path.display(),
                            document.control.title
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            print_value(&documents, json, text)
        }
        DocumentCommand::Show {
            edit_root,
            document,
        } => {
            let workspace = Workspace::open(&edit_root)?;
            let document = workspace.document(document)?;
            print_document(document, json, "document")
        }
        DocumentCommand::UpdateControl {
            edit_root,
            document,
            title,
            document_number,
            document_type,
            owner,
        } => {
            if title.is_none()
                && document_number.is_none()
                && document_type.is_none()
                && owner.is_none()
            {
                return Err(input_error(
                    "update-control requires at least one control-data option",
                ));
            }
            let mut workspace = Workspace::open(&edit_root)?;
            let updated = workspace.update_control(
                document,
                ControlUpdate {
                    title,
                    document_number: document_number.map(Some),
                    document_type: document_type.map(Some),
                    owner: owner.map(Some),
                },
            )?;
            workspace.save()?;
            print_document(&updated, json, "updated")
        }
        DocumentCommand::Unregister {
            edit_root,
            document,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let unregistered = workspace.unregister_document(document)?;
            workspace.save()?;
            print_document(&unregistered, json, "unregistered")
        }
        DocumentCommand::Reassociate {
            edit_root,
            document,
            path,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let reassociated = workspace.reassociate_document(document, &path)?;
            workspace.save()?;
            print_document(&reassociated, json, "reassociated")
        }
        DocumentCommand::Permalink {
            edit_root,
            document,
        } => {
            let workspace = Workspace::open(&edit_root)?;
            let permalink = workspace.document_permalink(document)?;
            print_value(&permalink, json, permalink.clone())
        }
    }
}

fn run_note(command: NoteCommand, json: bool) -> CliResult<()> {
    match command {
        NoteCommand::Add {
            edit_root,
            document,
            body,
            author,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let note = workspace.add_note(document, &body, author.as_deref())?;
            workspace.save()?;
            print_note(&note, json, "added")
        }
        NoteCommand::List {
            edit_root,
            document,
        } => {
            let workspace = Workspace::open(&edit_root)?;
            let notes = workspace.notes(document)?;
            let text = if notes.is_empty() {
                "no document notes".to_owned()
            } else {
                notes
                    .iter()
                    .map(|note| format!("{}  {}  {}", note.id, note.created_at, note.author))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            print_value(&notes, json, text)
        }
        NoteCommand::Edit {
            edit_root,
            document,
            note,
            body,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            let updated = workspace.edit_note(document, note, &body)?;
            workspace.save()?;
            print_note(&updated, json, "updated")
        }
        NoteCommand::Remove {
            edit_root,
            document,
            note,
        } => {
            let mut workspace = Workspace::open(&edit_root)?;
            workspace.remove_note(document, note)?;
            workspace.save()?;
            let result = RemovalResult {
                document_id: document,
                note_id: note,
                result: "removed",
            };
            print_value(&result, json, format!("removed note {note}"))
        }
    }
}

fn role_update(person: Option<Uuid>, clear: bool) -> RoleUpdate {
    if let Some(person) = person {
        RoleUpdate::replace(person)
    } else if clear {
        RoleUpdate::Clear
    } else {
        RoleUpdate::Unchanged
    }
}

fn read_marked_json<T: serde::de::DeserializeOwned>(marker: &str) -> CliResult<T> {
    let path = marker
        .strip_prefix("@file:")
        .ok_or_else(|| input_error("eligible people must use @file:PATH"))?;
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn print_document(document: &Document, json: bool, verb: &str) -> CliResult<()> {
    print_value(
        document,
        json,
        format!(
            "{verb} document {}: {} ({})",
            document.id,
            document.control.title,
            document.relative_path.display()
        ),
    )
}

fn print_note(note: &Note, json: bool, verb: &str) -> CliResult<()> {
    print_value(
        note,
        json,
        format!("{verb} note {} for {}", note.id, note.author),
    )
}

fn print_value<T: Serialize>(value: &T, json: bool, text: String) -> CliResult<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{text}");
    }
    Ok(())
}

fn input_error(message: &str) -> Box<dyn Error> {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}
