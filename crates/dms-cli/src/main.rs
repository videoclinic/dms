use std::{error::Error, fs, io, path::PathBuf, process};

use clap::{Parser, Subcommand};
use dms_core::{ControlUpdate, Document, EntraPerson, Note, RoleUpdate, Workspace};
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
