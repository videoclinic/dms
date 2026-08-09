# CHG-0001 — Bootstrap Tauri local DMS for ISO 27001 document control

| Field | Value |
| --- | --- |
| ID | CHG-0001 |
| Status | in-progress |
| External request | Direct operator request: develop a https://tauri.app/ application primarily for Windows for document version control to comply with ISO 27001 requirements. Drafts are original Microsoft Office tools; final approved and released documents are PDFs. No database; operator maintains release and approval process. Document details stored in hidden local directories per folder (e.g. `.dms`). Git-based VCS not mandatory. User can make notes. Released PDFs are checksummed. Refinements: (1) file explorer of controlled library with add/remove; library members are versioned as `*_vMAJOR.MINOR.pdf` on release while Office draft remains editable; (2) persist edit root and publish root; reconstruct mirrored directory tree under publish root on release; (3) PDF export and file versioning are performed by the application using preinstalled Microsoft Office; (4) macOS support is required in addition to Windows; (5) approval notification email contains a URI that opens the installed app to the requested document; (6) responsible editor and approver are assignable at root, folder, subfolder, or individual-document level with inheritance; (7) the person who requested approval receives notification of the recorded approval outcome; (8) distinguish automatic, current-session activity panes from explicit operator bookmarks and make their placement, creation, persistence, restoration, and removal clear in the CAPs and wireframes; (9) "working with folders would be the more dominent part of using DMS Desktop so the folder structure should become more present in the ui. also navigating through the folders in the library view should be more windows explorer like for convinience of the user"; (10) show the exact Office source filename separately from DMS-managed document control data, do not source that data from Office properties, and do not change it when the source file is renamed; (11) name activity panes with their task and referenced folder/document; document labels i... [truncated]
| Follow-up request | Direct operator request: extend the supported file format with markdown files including PDF export |
| Follow-up request | Direct operator request: CAP-0016 should enable to open the files. the view should also allow to select how many changes are seen in the bale before pagination beginns. also a search for "Doc" should be possible (as filter). The "Doc" title should be adjusted to the corresponding masterdata file name "Title" (if this is the sense); also in other wireframes the naming of the column missmatch the masterdata naming convention; check and fix this too |
| Follow-up request | Direct operator request: For the CAP-0005 hamburger/collapsed view add also the "Saved views" and "Open Panes" as icons but then with a folding context menu so the user do not have to expand the whole left pane |
| Follow-up request | Direct operator request: the "Effective approver" and "Effective editor" users are defined how this users are managed; the capability to maintain these users is missing; a synchronisation with Entra ID and/or SharePoint/OneDrive members; make a suggestion how to manage the users without creating a new user management in a Microsoft 365 setup. Proceed as recommended. |
| Follow-up request | Direct operator request: before every later approval nomination, the editor records a changelog and selects a minor, major, or manual target version; only an approved and successfully released target is occupied, while failed review evidence remains auditable; approvers are prompted for an optional non-approval comment. |
| Follow-up request | Direct operator request: show CAP-0011 workflow-chain entries newest first. |
| Follow-up request | Direct operator request: reconcile published/release terminology. `publish root` names the storage destination only; release is the sole workflow action and lifecycle transition that creates a released PDF. |
| Follow-up request | Direct operator request: define the approval-task email’s canonical subject and body; include the requester, DMS-managed title, and target version. |
| Follow-up request | Direct operator request: only major version changes need approval; after a minor version is published, notify the document's approver about the new minor publication. |
| Affected CAPs | CAP-0001 … CAP-0021 |
| Decision records | ADR-0001 … ADR-0021 in `docs/design-decisions.md` |

## Scope

Deliver the first vertical slice of a Tauri 2 desktop app for **Windows and
macOS** that:

- Configures edit root + publish root and stores metadata under `<edit-root>/.dms`
- Provides a foldable left menu (hamburger when collapsed), session-only
  open-activity panes/tabs as quicklinks in the left chrome, task-and-target
  pane labels with same-task-and-document reuse, and explicit per-user saved
  views; its collapsed rail exposes saved-view and open-pane icons with
  group-specific flyouts that do not expand the full left menu
- Shows a folder-dominant, Windows Explorer-like Library workspace with a
  persistent edit-root-relative tree, Back/Forward/Up + clickable breadcrumbs,
  current-folder child folders and files annotated by library membership, and a
  right selection pane with an always-visible filesystem-derived Source file
  identity plus independently foldable CAP-0015 Document control data, action,
  revision, and release sections for a single document, including an action to
  open the current released PDF, or a batch summary and multi-applicable
  actions for a multi-selection; add/remove documents under control
- Requires Entra-verified approval only for `V1.0` and candidates that increase
  the major component; minor candidates release directly after validation
- Requires the editor to record a changelog and select a minor, major, or
  validated manual target version before every later release; retains every
  unapproved major-review attempt without occupying its candidate version; asks
  approvers for an optional reason when approval is not granted
- Shows CAP-0011 workflow-chain entries newest first while retaining each event's
  predecessor hash
- Sends approver major-review-request email, requester decision-outcome email,
  and post-release minor-publication email to the effective approver
  (SMTP or host mail handler), and records revision-bound approval comments,
  notification delivery attempts, and event-chain hashes in `.dms`; the
  canonical review-request subject/body contains the DMS-managed title,
  filesystem-derived relative source path, requester display name, candidate
  target version, confidentiality label, action, and CAP-0020 review permalink
  without document content or a source/PDF URL
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
  with independent document overrides and immutable workflow snapshots; binds
  the workspace to a Microsoft Entra group as the read-only source of eligible
  people and verifies the approver through interactive Entra sign-in
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
  creating a new version when unchanged content is confirmed; exposes a
  publish-tree Title filter, operator-selected release rows per page, and an
  Open PDF action for each recorded non-missing release
- Optionally hands a previewed local text comparison to installed Claude
  Desktop for advisory target-version mode and changelog wording
- On release: app snapshots effective confidentiality type, commits the approved
  target version only after successful atomic export,
  mirrors tree under publish root, builds export chrome from `.dms`, exports
  Office drafts via preinstalled Office (temp-copy token fill) or Markdown
  drafts through a CommonMark HTML print shell plus native WebView PDF APIs to
  `<stem>_VMAJOR.MINOR_<confidentiality-type-id>.pdf`, checksums it
- Keeps editable source drafts in place after release
- Attaches notes; verifies released PDF checksums
- Ships core workflows on both Windows and macOS

## Non-scope

- SharePoint/OneDrive document-content synchronization or using file permissions
  as a workflow-person directory
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
| 0a | Microsoft Entra workflow identity records and wireframes | done (CAP-0021, CAP-0019, ADR-0019, ADR-0021, architecture, privacy, and static screens updated) | CAP-0021 + ADR-0021 replace the application roster with a group binding; CAP-0019 selects only eligible Entra users; static screens show a read-only source, folder-role selection, and no user CRUD; all CAP/CHG/ADR indexes and links validate; no CAP claims implemented runtime |
| 0b | Target-version review, major-only approval, and failure-evidence records + wireframes | done (CAP-0002/0011/0012, CAP-0010, ADR-0007/0013, architecture, and affected static screens updated) | CAP-0002/0011/0012 define the required changelog, minor/major/manual candidate selection, no-reservation rule, `V1.0`/major-only approval, direct minor release with effective-approver publication notification, explicit release actions, optional non-approval comment, and newest-first workflow history; CAP-0010 defines the canonical major-review and minor-publication messages; CHG/ADR/architecture align; affected static screens show direct minor release, major review, and retained unsuccessful evidence; generated exports, links, and Markdown checks pass; no CAP claims implemented runtime |
| 1 | Tauri 2 app skeleton (Windows + macOS) + DOX for source tree | pending | Dev app launches on Windows and macOS; README run steps for both; foldable left menu + hamburger + session-only open-activity tabs show task-and-target labels and focus an existing matching task+document pane; collapsed rail icons open saved-view and open-pane flyouts without expanding the whole menu, with open/remove and focus/close actions respectively; explicit saved-view bookmark persists in OS user config and restores as a fresh activity |
| 2 | `.dms` store + dual-root open/configure + confidentiality, Entra identity binding, and workflow-role policies | pending | Tests: persist/reload edit+publish roots, stable workspace ID, schema version, and non-secret Entra tenant/group binding; safe older-schema migration/newer-schema read-only; create/replace a direct folder policy, remove a non-root policy, refuse root-policy removal, and recompute nearest inherited class; inherit each workflow role independently; replacing the identity source marks live role policies unresolved without changing historical evidence; init `.dms` only on confirm |
| 3 | Folder-first Library explorer + add/unregister/reassociate + selection pane | pending | Tests: folder pane is visible by default and includes empty edit-root folders while hiding `.dms`; Back/Forward/Up, breadcrumb, tree, and immediate-child contents stay synchronized; every file row keeps its exact filesystem name while an in-library row shows DMS-managed document data separately; one or more unregistered supported source files (including `.md`) can be selected and added from the right pane, while mixed/unsupported selections expose no incompatible batch action; folder navigation reuses one Library activity and updates its folder label; add under edit root; reject outside path; unregister preserves history; rename/reassociate updates only the source locator and does not change document control data or history; ambiguous move is never auto-linked; current-folder and Entire-library search scopes return matching files with paths and clear back to the complete folder listing; single controlled-document selection shows an always-visible Source file identity plus CAP-0015 Document control data and actions in the right pane; the data is loaded from `.dms`, not Office properties or Markdown front matter; its data, action, revision, and release sections fold independently while retaining document and source-file identity; navigating to an already-open task+document focuses the existing pane, while different tasks for that document may remain open; multi-select of controlled documents shows only multi-applicable actions in the same pane; a saved library view restores folder/sort and a single-document stable ID but never batch selection; copy permalink uses workspace+document IDs only and never changes saved views |
| 4 | Lifecycle + Entra-verified major approval + notification + target-version release + tree mirror | pending | Tests with a Graph client fake: every release requires non-empty changelog, requester, one target mode (next minor, next major, or validated manual greater-than-current unused target), and release-time validation; `V1.0`, major, and a manual target with a higher major component require a current eligible effective approver plus transport success, while a minor candidate creates no review request and releases directly after validation; a failed group refresh or unresolved identity blocks an approval-required request and minor-publication recipient snapshot; SMTP and `mailto:` render the canonical major-review subject/body with DMS-managed title, filesystem-derived relative source path, requester display name, candidate target version, confidentiality label, action, and CAP-0020 review permalink only, and render the committed-minor publication notification to the effective approver; decision requires the snapshotted Entra tenant/object ID and current eligibility; rejected/changes-requested decisions prompt for an optional reason; the workflow history retains changelog, candidate, mode, major decision evidence when applicable, and minor publication delivery attempts newest first while retaining each predecessor hash; a rejected, changes-requested, cancelled, invalidated, or failed-export candidate remains available; CAP-0020 deep link resolves only an accessible registered workspace to the intended review request and still resolves after rename/version bump; each decision notifies the snapshotted requester, while minor publication delivery failure is retryable and never reverses its committed release; approver-policy change invalidates an open review; DOCX body/header/footer and Markdown rendered-body version and confidentiality markers must equal the candidate release and effective type before an approval-required review and again before every release; missing, mismatched, and conflicting markers block by default; an explicit, reasoned false-positive override is revision-bound, visible to the approver, and recorded in the event chain; major approval preserves its accepted target but only atomic export commits it; metadata change invalidates approval; first version V1.0; refuse overwrite |
| 5 | Format-specific local PDF export on release (Win + macOS adapters) | pending | Tests/integration: export Office drafts through installed Office (or a test double), replacing `{CONFIDENTIALITY}`/`{VERSION}` on a temp copy from the release chrome map; export Markdown through CommonMark + shipped print shell (logo, `Vertraulichkeitsstufe:` / `Version:` footers from the same map, front matter stripped) + native WebView PDF API to the versioned, classified path on each OS; PDF chrome values match the release snapshot; a failed export does not commit or occupy the approved target; WebView2 and WKWebView smoke cover multi-page footer chrome |
| 6 | Notes on documents | pending | Tests: note CRUD persistence across restart; list is newest-first; New note compose field is above the latest note and remains there after save |
| 7 | Release checksum + periodic review + verify | pending | Tests: exported PDF → expected SHA-256; tamper → mismatch; release snapshots draft digest, target-version mode/label, changelog, and chain; a case-insensitive Title filter scopes publish-tree releases; selected rows-per-page controls pagination of the filtered result; each non-missing release opens only its recorded PDF while missing files have no open action; periodic confirm keeps version; changes-required begins revision; full backup manifest covers both roots |
| 8 | Optional Claude Desktop handoff | pending | Tests: disabled/missing app never blocks; policy and consent gate payload; accepted suggestion remains editable and cannot mutate lifecycle |
| 9 | Packaging smoke + CAP promotion | pending | Windows and macOS smoke covers Office and Markdown PDF export; a Microsoft 365 administrator-configured Entra group smoke covers source preview, direct-member role selection, ineligible-role blocking, and an interactive decision sign-in; CAP statuses updated only with test links; records check if present |

**Current phase:** 0b is complete; phase 1 is pending. Set phase 1 to `in-progress` before implementation and keep only one phase in-progress.

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
  relative paths and stable IDs, confidentiality and document-type catalogues,
  Microsoft Entra tenant/group binding plus a read-only display cache,
  direct relative folder policies (including the required root policy),
  document control data, per-doc overrides, version counters,
  release history, approval event chain, review-request IDs, requester identity,
  notification delivery attempts, notes, checksums.
- Version pattern: `<stem>_V<major>.<minor>_<confidentiality-type-id>.pdf`;
  the editor proposes next-minor, next-major, or a validated manual target for
  each later release. `V1.0` and major-component increases require approval;
  direct minor publication notifies the effective approver. Unapproved candidates
  remain audit evidence and never occupy a version (ADR-0007).
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
