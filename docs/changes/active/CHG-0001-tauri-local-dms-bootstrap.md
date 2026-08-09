# CHG-0001 — Bootstrap Tauri local DMS for ISO 27001 document control

| Field | Value |
| --- | --- |
| ID | CHG-0001 |
| Status | in-progress |
| External request | Direct operator request: develop a https://tauri.app/ application primarily for Windows for document version control to comply with ISO 27001 requirements. Drafts are original Microsoft Office tools; final approved and released documents are PDFs. No database; operator maintains release and approval process. Document details stored in hidden local directories per folder (e.g. `.dms`). Git-based VCS not mandatory. User can make notes. Released PDFs are checksummed. Refinements: (1) file explorer of controlled library with add/remove; library members are versioned as `*_vMAJOR.MINOR.pdf` on release while Office draft remains editable; (2) persist edit root and publish root; reconstruct mirrored directory tree under publish root on release; (3) PDF export and file versioning are performed by the application using preinstalled Microsoft Office; (4) macOS support is required in addition to Windows; (5) approval notification email contains a URI that opens the installed app to the requested document; (6) responsible editor and approver are assignable at root, folder, subfolder, or individual-document level with inheritance; (7) the person who requested approval receives notification of the recorded approval outcome; (8) distinguish automatic, current-session activity panes from explicit operator bookmarks and make their placement, creation, persistence, restoration, and removal clear in the CAPs and wireframes; (9) "working with folders would be the more dominent part of using DMS Desktop so the folder structure should become more present in the ui. also navigating through the folders in the library view should be more windows explorer like for convinience of the user"; (10) show the exact Office source filename separately from DMS-managed document control data, do not source that data from Office properties, and do not change it when the source file is renamed; (11) name activity panes with their task and referenced folder/document; document labels i... [truncated]
| Follow-up request | Direct operator request: extend the supported file format with markdown files including PDF export |
| Affected CAPs | CAP-0001 … CAP-0020 |
| Decision records | ADR-0001 … ADR-0020 in `docs/design-decisions.md` |

## Scope

Deliver the first vertical slice of a Tauri 2 desktop app for **Windows and
macOS** that:

- Configures edit root + publish root and stores metadata under `<edit-root>/.dms`
- Provides a foldable left menu (hamburger when collapsed), session-only
  open-activity panes/tabs as quicklinks in the left chrome, task-and-target
  pane labels with same-task-and-document reuse, and explicit per-user saved
  views
- Shows a folder-dominant, Windows Explorer-like Library workspace with a
  persistent edit-root-relative tree, Back/Forward/Up + clickable breadcrumbs,
  current-folder child folders and files annotated by library membership, and a
  right selection pane with an always-visible filesystem-derived Source file
  identity plus independently foldable CAP-0015 Document control data, action,
  revision, and release sections for a single document, including an action to
  open the current released PDF, or a batch summary and multi-applicable
  actions for a multi-selection; add/remove documents under control
- Runs operator-driven approval and release
- Sends approver review-request email and requester decision-outcome email
  (SMTP or host mail handler), and records revision-bound approval comments,
  notification delivery attempts, and event-chain hashes in `.dms`; each
  notification links the installed local app to the addressed review request
  via a CAP-0020 permalink
- Registers stable document permalinks (workspace ID + document ID; optional
  review/notes target) that survive draft rename and version bumps; selection
  pane can copy the permalink
- Applies inherited confidentiality types from direct folder policies: the root
  policy is required, non-root policies can be assigned, replaced, or removed,
  with per-document overrides and release-time snapshots
- Before review submission and release, checks the current source draft for
  canonical version and confidentiality markers. A missing, mismatched, or
  ambiguous marker blocks the transition by default; an operator can continue
  after recording a reasoned false-positive override in the workflow evidence
- Routes one responsible editor and one approver from inherited folder policies,
  with independent document overrides and immutable workflow snapshots
- Opens the registered Office application for Office drafts or the host default
  text editor for Markdown without embedding an editor and keeps a stable
  document ID across renames inside the edit root
- Performs advisory locking and atomic `.dms` writes, supports backup and
  restore, and exposes a workflow verify routine and operator-triggered audit
  export
- Maintains DMS-managed document control data (title, number, type, owner,
  review due) independently of source-file names, Office properties, and
  Markdown front matter; begin-revision after release, cancel-review,
  obsolescence, and publish-tree history / orphan handling
- Runs due-date periodic reviews against the current released PDF without
  creating a new version when unchanged content is confirmed
- Optionally hands a previewed local text comparison to installed Claude
  Desktop for advisory major/minor classification and changelog wording
- On release: app snapshots effective confidentiality type, assigns version,
  mirrors tree under publish root, builds export chrome from `.dms`, exports
  Office drafts via preinstalled Office (temp-copy token fill) or Markdown
  drafts through a CommonMark HTML print shell plus native WebView PDF APIs to
  `<stem>_VMAJOR.MINOR_<confidentiality-type-id>.pdf`, checksums it
- Keeps editable source drafts in place after release
- Attaches notes; verifies released PDF checksums
- Ships core workflows on both Windows and macOS

## Non-scope

- SharePoint/Graph synchronization
- Bundled Office runtime or cloud conversion services
- Multi-user server backend
- Git-based version control
- Linux as a required supported platform
- Auto-library-add of every file under the edit root
- Automatic invocation of Claude Desktop or AI-gated lifecycle decisions

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 0 | Product records and architecture bootstrap | done (docs tree; Markdown source draft plus format-specific local PDF export; release version policy, approval, notification, confidentiality, workflow-role routing, maintenance, periodic review, optional Claude handoff, foldable shell chrome, task-and-target session activity panes with duplicate reuse, saved views, folder-dominant Explorer-like Library navigation with foldable single-document sections and a same-pane batch state, stable document permalinks, Win+macOS recorded) | CAP/CHG/ADR files exist; indexes list CAP-0001…0020; no CAP claims implemented runtime |
| 1 | Tauri 2 app skeleton (Windows + macOS) + DOX for source tree | pending | Dev app launches on Windows and macOS; README run steps for both; foldable left menu + hamburger + session-only open-activity tabs show task-and-target labels and focus an existing matching task+document pane; explicit saved-view bookmark persists in OS user config and restores as a fresh activity |
| 2 | `.dms` store + dual-root open/configure + confidentiality and workflow-role policies | pending | Tests: persist/reload edit+publish roots, stable workspace ID, and schema version; safe older-schema migration/newer-schema read-only; create/replace a direct folder policy, remove a non-root policy, refuse root-policy removal, and recompute nearest inherited class; inherit each workflow role independently; init `.dms` only on confirm |
| 3 | Folder-first Library explorer + add/unregister/reassociate + selection pane | pending | Tests: folder pane is visible by default and includes empty edit-root folders while hiding `.dms`; Back/Forward/Up, breadcrumb, tree, and immediate-child contents stay synchronized; every file row keeps its exact filesystem name while an in-library row shows DMS-managed document data separately; one or more unregistered supported source files (including `.md`) can be selected and added from the right pane, while mixed/unsupported selections expose no incompatible batch action; folder navigation reuses one Library activity and updates its folder label; add under edit root; reject outside path; unregister preserves history; rename/reassociate updates only the source locator and does not change document control data or history; ambiguous move is never auto-linked; current-folder and Entire-library search scopes return matching files with paths and clear back to the complete folder listing; single controlled-document selection shows an always-visible Source file identity plus CAP-0015 Document control data and actions in the right pane; the data is loaded from `.dms`, not Office properties or Markdown front matter; its data, action, revision, and release sections fold independently while retaining document and source-file identity; navigating to an already-open task+document focuses the existing pane, while different tasks for that document may remain open; multi-select of controlled documents shows only multi-applicable actions in the same pane; a saved library view restores folder/sort and a single-document stable ID but never batch selection; copy permalink uses workspace+document IDs only and never changes saved views |
| 4 | Lifecycle + approval notification + version assign + tree mirror | pending | Tests: request requires summary, requester, change class, effective approver, and transport success; CAP-0020 deep link resolves only an accessible registered workspace to the intended review request and still resolves after rename/version bump; each decision notifies the snapshotted requester, with failure retryable and non-reverting; approver-policy change invalidates an open review; cosmetic→minor, substantive/uncertain→major; DOCX body/header/footer and Markdown rendered-body version and confidentiality markers must equal the candidate release and effective type before review and again before release; missing, mismatched, and conflicting markers block by default; an explicit, reasoned false-positive override is revision-bound, visible to the approver, and recorded in the event chain; comments/event hash persist; metadata change invalidates approval; first version V1.0; refuse overwrite |
| 5 | Format-specific local PDF export on release (Win + macOS adapters) | pending | Tests/integration: export Office drafts through installed Office (or a test double), replacing `{CONFIDENTIALITY}`/`{VERSION}` on a temp copy from the release chrome map; export Markdown through CommonMark + shipped print shell (logo, `Vertraulichkeitsstufe:` / `Version:` footers from the same map, front matter stripped) + native WebView PDF API to the versioned, classified path on each OS; PDF chrome values match the release snapshot; failure rolls back version success; WebView2 and WKWebView smoke cover multi-page footer chrome |
| 6 | Notes on documents | pending | Tests: note CRUD persistence across restart |
| 7 | Release checksum + periodic review + verify | pending | Tests: exported PDF → expected SHA-256; tamper → mismatch; release snapshots draft digest/class/chain; periodic confirm keeps version; changes-required begins revision; full backup manifest covers both roots |
| 8 | Optional Claude Desktop handoff | pending | Tests: disabled/missing app never blocks; policy and consent gate payload; accepted suggestion remains editable and cannot mutate lifecycle |
| 9 | Packaging smoke + CAP promotion | pending | Windows and macOS smoke covers Office and Markdown PDF export; CAP statuses updated only with test links; records check if present |

**Current phase:** 0 complete. Set phase 1 to `in-progress` when skeleton work starts; keep only one phase in-progress.

## Implementation notes

- Prefer a minimal frontend unless a UI kit is required for the library
  directory navigator.
- OS user app config holds sidebar preference and saved-view targets. Open
  activity tabs are session-only; neither belongs in `.dms` workflow evidence.
- A document activity key is workspace ID + task + stable document ID; its label
  is task + current DMS title + optional document number. Folder activity labels
  use edit-root-relative paths; the Library updates its one session pane in
  place while navigating folders.
- `.dms` format: inspectable JSON (or similar); schema beside the store module
  in phase 2 — must include `edit_root`, `publish_root`, stable workspace ID,
  library entries with
  relative paths and stable IDs, confidentiality and workflow-person catalogues,
  direct relative folder policies (including the required root policy),
  document control data, per-doc overrides, version counters,
  release history, approval event chain, review-request IDs, requester identity,
  notification delivery attempts, notes, checksums.
- Version pattern: `<stem>_V<major>.<minor>_<confidentiality-type-id>.pdf`;
  cosmetic changes increment minor and substantive or uncertain changes
  increment major (ADR-0007).
- Path mapping: `publish_abs = publish_root / relative_parent / versioned_name`
  (ADR-0006).
- PDF export: one format dispatcher with Office adapters (Windows COM / macOS
  automation) and Markdown print-shell + native WebView PDF adapters (ADR-0008
  Option A). Shared export chrome comes only from `.dms` release context.
  Ship default `shell.html` / `print.css` / logo derived from the corporate
  Vorlage; do not route Markdown through Word. CI uses fakes when platform
  export is unavailable; phase 5 must still spike fixed header/footer + page
  indicators on WebView2 and WKWebView.

## Resume checklist

1. Read this CHG and affected CAP files (including CAP-0005, CAP-0006, CAP-0007).
2. Confirm phase statuses against the working tree.
3. Continue the single `in-progress` phase; do not open parallel progress plans.
4. Update CAP outcomes to present-tense implemented language only when tests prove them.
