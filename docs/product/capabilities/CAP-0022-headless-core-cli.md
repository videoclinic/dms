# CAP-0022 — Headless DMS core CLI

| Field | Value |
| --- | --- |
| ID | CAP-0022 |
| Status | implemented |
| Executable | `dms` |
| Platforms | Windows and macOS |
| Tests | `crates/dms-core/tests/workspace.rs`; `crates/dms-cli/tests/cli.rs` |

## Outcomes

The CLI provides explicit, local operations over the shared `dms-core` library:

1. `dms workspace init` requires `--confirm`, creates `.dms/workspace.json`,
   records canonical edit and publish roots plus a stable workspace ID, and
   refuses to overwrite an existing workspace.
2. `dms workspace status` and `dms workspace verify` reopen and validate the
   stored schema and metadata without a Tauri or WebView runtime.
3. `dms document add` accepts only regular `.md`, `.docx`, `.xlsx`, and `.pptx`
   files within the edit root; it rejects external paths, `.dms` contents,
   Office temporary files, unsupported formats, and duplicate registrations.
4. Registered documents retain a stable ID and edit-root-relative source path.
   Their DMS-managed control data has an initial title derived once from the
   source stem and remains distinct from the source name and path.
5. `dms document list`, `show`, and `update-control` expose and change only the
   stored DMS-managed title, document number, document type, and owner. A title
   cannot be empty; document numbers are case-insensitively unique when set.
6. `dms note add`, `list`, `edit`, and `remove` maintain document-scoped
   plain-text notes with stable note IDs, author, and timestamps. Listing is
   newest-first.
7. All mutating commands persist validated JSON through the workspace store.
   `--json` emits structured results suitable for automation; normal output is
   concise, human-readable command feedback.

## Non-goals

- Tauri windows, WebView IPC, or desktop activity state
- PDF export, release lifecycle, checksum verification, approval, Entra,
  notification, confidentiality, or audit-report workflows
- A Tauri plugin, sidecar, or a command that drives a running desktop app
- Source-file mutation, automatic source discovery, or automatic library add

## Links

- Architecture and ADR-0023: [`../../architecture.md`](../../architecture.md)
- Workspace: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- Library membership: [`CAP-0006-library-explorer.md`](CAP-0006-library-explorer.md)
- Notes: [`CAP-0003-document-notes.md`](CAP-0003-document-notes.md)
- Document control data: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
