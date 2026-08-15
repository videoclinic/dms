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
| `ui/app.mjs` | Workspace setup, static shell, session activities, saved-view persistence, and IPC orchestration |
| `ui/library.mjs` | Folder-first explorer state, markup, selection rules, search, and sorting |
| `ui/notes.mjs` | Per-document note activity state, create/edit/delete markup, and confirmation flow |
| `ui/maintenance.mjs` | Releases, periodic-review, advisory-lock status/configuration, full backup, and confirmed restore panes |
| `ui/reports.mjs` | Audit-report generation filters, recent-report history, verification state, paging, and host actions |
| `ui/assistance.mjs` | Workspace policy form plus document-scoped payload preview, consent, handoff, and editable response draft |
| `ui/configuration.mjs` | Routed workspace/default/workflow/notification state, secondary catalogue/identity surfaces, and mutation requests |
| `ui/print/` | Shipped app-local Markdown print shell, stylesheet, and logo |
| `ui/*.test.mjs` | Framework-free shell and Library interaction tests |
| `capabilities/` | Tauri window permissions |
| `icons/` | SVG source and derived PNG/Windows application icons |
| `tauri.conf.json` | Desktop window, local frontend, security, and bundle configuration |
| `tests/` | Desktop integration coverage when separate fixtures are needed |

## Local Contracts

- Call `dms-core` for workspace domain behaviour; do not duplicate its rules.
- Before a workspace exists, expose only Set up workspace. Opening requires an
  existing edit root; initialization requires explicit edit + publish roots and
  confirmation before the adapter may create `.dms` or the publish root.
- Opening or switching workspace sessions acquires the destination advisory
  lock before activation, offers explicit stale takeover and a separately
  warned override-any-lock option from setup, releases the previous lock only
  after acquisition succeeds, and removes the active lock on a clean window
  close only when its recorded owner still matches.
- Store sidebar, saved-view, and recent-library preferences in the OS user
  app-config directory, never under `<edit-root>/.dms`. Recent libraries are at
  most ten unique edit roots in most-recent-first order; removing one never
  touches workspace metadata or files. A failed recent-library open reports the
  error beside that list and retains the edit root in the explicit open form.
- Register the configured `dms://` scheme in Windows and macOS bundles. The
  single-instance plugin remains the first Tauri plugin so an activation focuses
  the existing main window and reaches the deep-link listener.
- Resolve inbound permalinks only through `dms-core` against accessible edit
  roots in the recent-library registry. Switch through the normal advisory-lock
  boundary, key activities by stable IDs, and load retained document details by
  ID even when no current filesystem row can be selected.
- Change the persisted sidebar preference or current-session unfolded overlay
  only through an explicit sidebar control. Destination, saved-view, open-pane,
  and in-surface actions must preserve the current sidebar presentation.
- Contain the shell to the window viewport. Keep the sidebar and activity header
  outside activity scrolling; ordinary activities scroll in main content. In the
  Library, keep the path toolbar fixed and let the folder tree, file table, and
  selection details scroll independently.
- Give every directory-selection field a native **Browse…** action through the
  desktop adapter. Start its picker at the OS user's home directory and leave
  the field unchanged when selection is cancelled.
- Keep open activities in frontend session state only.
- Saved document targets use workspace ID + document ID, never source paths.
- Library navigation keeps one session activity while saved Library views retain
  edit-root-relative folder, sort, and at most one stable document target.
- Render the Library folder surface as a nested tree with independent branch
  toggles. Navigating expands the current folder's ancestor chain without
  resetting unrelated session branch state.
- The Library selection pane owns add/unregister/reassociate/permalink controls,
  document-control editing, the document confidentiality override, candidate
  submission, review decision, release, local lifecycle actions, and canonical
  workflow evidence; file rows remain selection-only and preserve exact source
  names. The adapter supplies lifecycle availability and precondition explanations
  rather than duplicating core transitions in the frontend. `mailto:` delivery
  opens the host handler first and advances only through its explicit confirmation
  retry; review decisions require a fresh interactive Entra sign-in. Cancel review
  and mark obsolete retain failed reason drafts and require explicit confirmation.
  Validation failures stay in that selected-document context. Its notes action
  opens a document-scoped activity keyed by stable document ID. Document Notes
  returns to the singleton Library activity with the same stable document
  selected, preserves an unchanged Library view without reloading, and retains
  Notes drafts and confirmation state when restoration succeeds or fails.
- Document-profile editing submits an eligible owner object ID and never edits an
  effective date. Candidate submission owns the required effective date and may
  stage eligible owner/editor replacements that apply only with a successful
  release. Current-release and history views use immutable release snapshots;
  review schedule remains a separate mutable surface.
- Literal `<owner>` / `<editor>` placeholders appear only after a successful
  direct-member refresh with zero enabled users. They are display-only unresolved
  state, never fabricated identities; every missing-binding, tenant, Graph,
  inaccessible-group, and disabled-only result fails closed.
- Note mutations call `dms-core`, save the workspace metadata, retain failed
  form drafts for retry, and clear the composer only after a successful save.
- The Releases pane lists every recorded release with its immutable title,
  profile/owner snapshot, effective date, and verification status; legacy
  omissions are visibly unrecorded rather than replaced by current metadata. It
  filters by captured title and exposes per-release and workspace-wide
  verification actions. It never edits, repairs, or replaces release bytes.
- The Audit & Reports pane generates filtered CSV/PDF reports, lists report
  evidence with filter-before-pagination, and exposes read-only verification and
  host-mediated Open folder actions. It also shows periodic-review markers and
  request, result, comment-required cancellation, and reminder actions. Result,
  cancellation, and reminder require explicit confirmation; delivery and
  integrity failures are surfaced in place.
- The Maintenance pane shows advisory-lock status and configures its positive
  staleness threshold. It writes a workspace backup archive to a user-supplied
  path without overwrite and restores only after explicit roots, replacement
  policy, lock takeover, and confirmation are supplied to `dms-core`.
- Configuration remains one session activity across Workspace, Document
  defaults, Workflow, and Notifications routes. Confidentiality catalogue and
  identity-source management are in-place secondary surfaces. Folder role
  pickers refresh then use the core-owned eligible-person cache. The identity
  surface accepts administrator-supplied app-global public-client/tenant IDs and
  a library-bound group ID, drives delegated device authorization, previews and
  explicitly applies a binding, and refreshes direct enabled user members
  through Microsoft Graph. First setup selects and persists the required
  edit-root editor and approver atomically with that binding; replacement never
  remaps existing role references.
  The current-source overview shows the effective app-global public-client and
  tenant IDs alongside the library-bound group; its Group ID control opens the
  encoded Microsoft My Account group page through the host-browser boundary.
  Desktop-only delegated tokens remain in the OS credential store; credentials
  never cross the frontend IPC boundary or enter `.dms`.
- Claude Desktop assistance remains unavailable unless workspace policy,
  effective confidentiality, a verified current release, and a supported local
  installation all allow it. An oversized preview shows its measured size and
  selectable exact excerpts; the adapter re-previews only the selected subset.
  Every handoff requires fresh consent and recomputes that subset's digest.
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