# dms-desktop

## Purpose

Provide the Tauri 2 desktop adapter and local WebView shell for Windows and
macOS.

## Ownership

| Path | Owns |
| --- | --- |
| `src/` | Tauri startup, IPC commands, `dms-core` adapter, OS user preferences |
| `src/assistance.rs` | Supported-location detection and process launch for Claude Desktop on Windows and macOS |
| `src/export.rs` | `.docx`/Markdown PDF adapters, Office automation, native WebView PDF capture, and format-specific tests |
| `ui/app.mjs` | Static shell, session activities, saved-view persistence, and IPC orchestration |
| `ui/library.mjs` | Folder-first explorer state, markup, selection rules, search, and sorting |
| `ui/notes.mjs` | Per-document note activity state, create/edit/delete markup, and confirmation flow |
| `ui/maintenance.mjs` | Releases, periodic-review, and full-backup panes with read-only verification, paging, and confirmation copy |
| `ui/assistance.mjs` | Workspace policy form plus document-scoped payload preview, consent, handoff, and editable response draft |
| `ui/print/` | Shipped app-local Markdown print shell, stylesheet, and logo |
| `ui/*.test.mjs` | Framework-free shell and Library interaction tests |
| `capabilities/` | Tauri window permissions |
| `icons/` | SVG source and derived PNG/Windows application icons |
| `tauri.conf.json` | Desktop window, local frontend, security, and bundle configuration |
| `tests/` | Desktop integration coverage when separate fixtures are needed |

## Local Contracts

- Call `dms-core` for workspace domain behaviour; do not duplicate its rules.
- Store sidebar and saved-view preferences in the OS user app-config directory,
  never under `<edit-root>/.dms`.
- Keep open activities in frontend session state only.
- Saved document targets use workspace ID + document ID, never source paths.
- Library navigation keeps one session activity while saved Library views retain
  edit-root-relative folder, sort, and at most one stable document target.
- The Library selection pane owns add/unregister/reassociate/permalink controls;
  file rows remain selection-only and preserve exact source names. Its notes
  action opens a document-scoped activity keyed by stable document ID.
- Note mutations call `dms-core`, save the workspace metadata, retain failed
  form drafts for retry, and clear the composer only after a successful save.
- The Releases pane lists every recorded release with its verification status,
  filters by document title, and exposes per-release and workspace-wide
  verification actions. It never edits, repairs, or replaces release bytes.
- The Audit & Reports pane shows periodic-review markers and lets a designated
  approver request a new review; integrity-gated request failures are surfaced
  in place.
- The Maintenance pane writes a workspace backup archive to a user-supplied
  path and refuses to overwrite an existing archive.
- Claude Desktop assistance remains unavailable unless workspace policy,
  effective confidentiality, a verified current release, and a supported local
  installation all allow it. Every handoff previews the exact payload and
  requires fresh consent bound to its digest.
- Claude responses remain untrusted session state. Copying one into an editable
  changelog draft cannot select a target, approve, release, or mutate lifecycle
  metadata.
- Load only app-local frontend assets; do not add remote runtime dependencies.
- PDF adapters write only to the temporary path supplied by `dms-core`; core
  owns the classified final path, digest, atomic rename, and release evidence.
- Office placeholder fill always operates on a temporary OOXML copy. Native
  Markdown export uses WebView2 `PrintToPdf` or WKWebView `createPDF`, never an
  interactive print dialog.

## Work Guidance

- Keep the shell usable without a frontend package manager until a compiled UI
  framework provides a concrete benefit.
- Preserve accessible names for icon-only controls and elided activity labels.

## Verification

- `cargo test -p dms-desktop`
- `node --test crates/dms-desktop/ui/*.test.mjs`
- `DMS_DESKTOP_SMOKE=1 cargo run -p dms-desktop`
- `DMS_DESKTOP_EXPORT_SMOKE=1 cargo run -p dms-desktop` on Windows and macOS
- `.github/workflows/desktop-platform-smoke.yml` on Windows and macOS

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.