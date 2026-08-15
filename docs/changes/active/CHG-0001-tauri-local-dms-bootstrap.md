# CHG-0001 — Bootstrap Tauri local DMS for ISO 27001 document control

| Field | Value |
| --- | --- |
| ID | CHG-0001 |
| Status | in-progress |
| External request | Direct operator request: develop a https://tauri.app/ application primarily for Windows for document version control to comply with ISO 27001 requirements. Drafts are original Microsoft Office tools; final approved and released documents are PDFs. No database; operator maintains release and approval process. Document details stored in hidden local directories per folder (e.g. `.dms`). Git-based VCS not mandatory. User can make notes. Released PDFs are checksummed. Refinements: (1) file explorer of controlled library with add/remove; library members are versioned as `*_vMAJOR.MINOR.pdf` on release while Office draft remains editable; (2) persist edit root and publish root; reconstruct mirrored directory tree under publish root on release; (3) PDF export and file versioning are performed by the application using preinstalled Microsoft Office; (4) macOS support is required in addition to Windows; (5) approval notification email contains a URI that opens the installed app to the requested document; (6) responsible editor and approver are assignable at root, folder, subfolder, or individual-document level with inheritance; (7) the person who requested approval receives notification of the recorded approval outcome; (8) distinguish automatic, current-session activity panes from explicit operator bookmarks and make their placement, creation, persistence, restoration, and removal clear in the CAPs and wireframes; (9) "working with folders would be the more dominent part of using DMS Desktop so the folder structure should become more present in the ui. also navigating through the folders in the library view should be more windows explorer like for convinience of the user"; (10) show the exact Office source filename separately from DMS-managed document control data, do not source that data from Office properties, and do not change it when the source file is renamed; (11) name activity panes with their task and referenced folder/document; document labels include title and document number; navigating to an already-open task and document reuses that pane. |
| Follow-up request | Direct operator request: extend the supported file format with markdown files including PDF export |
| Follow-up request | Direct operator request: CAP-0016 should enable to open the files. the view should also allow to select how many changes are seen in the bale before pagination beginns. also a search for "Doc" should be possible (as filter). The "Doc" title should be adjusted to the corresponding masterdata file name "Title" (if this is the sense); also in other wireframes the naming of the column missmatch the masterdata naming convention; check and fix this too |
| Follow-up request | Direct operator request: All the tables where the number of rows over time increses need a pagination and filter function like in CAP-0016 |
| Follow-up request | Direct operator request: For the CAP-0005 hamburger/collapsed view add also the "Saved views" and "Open Panes" as icons but then with a folding context menu so the user do not have to expand the whole left pane |
| Follow-up request | Direct operator request: the "Effective approver" and "Effective editor" users are defined how this users are managed; the capability to maintain these users is missing; a synchronisation with Entra ID and/or SharePoint/OneDrive members; make a suggestion how to manage the users without creating a new user management in a Microsoft 365 setup. Proceed as recommended. |
| Follow-up request | Direct operator request: before every later approval nomination, the editor records a changelog and selects a minor, major, or manual target version; only an approved and successfully released target is occupied, while failed review evidence remains auditable; approvers are prompted for an optional non-approval comment. |
| Follow-up request | Direct operator request: show CAP-0011 workflow-chain entries newest first. |
| Follow-up request | Direct operator request: reconcile published/release terminology. `publish root` names the storage destination only; release is the sole workflow action and lifecycle transition that creates a released PDF. |
| Follow-up request | Direct operator request: define the approval-task email’s canonical subject and body; include the requester, DMS-managed title, and target version. |
| Follow-up request | Direct operator request: only major version changes need approval; after a minor version is published, notify the document's approver about the new minor publication. |
| Follow-up request | Direct operator request: clarify how an operator selects a folder for confidentiality policy and exactly which documents that policy classifies. Proceed as recommended. |
| Follow-up request | Direct operator request: CAP-0019 and CAP-0008 look different but have similar goals: definition of defaults. redesign the ui in same way. do not let the configuration of the confidentiality policies and entra id setup so mutch space because these settings are changes less frequent |
| Follow-up request | Direct operator request: There are so many configuration pages just now, and I do not see/know how sould the user navigate through all of them |
| Follow-up request | Direct operator request: the Library folder element must be a real hierarchical tree view, and an unfolded left menu must retain its state when an action or destination is selected |
| Follow-up request | Direct operator request: on the initial Library/setup page, retain the last 10 opened libraries with per-entry removal; every directory-selection field must offer native directory browsing starting at the OS user's home directory |
| Follow-up request | Direct operator request: The UI concept should be adapted for scrolling, so the areas where navigation happens are not affected by scrolling in areas like document control data that can be very exchaustive and long |
| Follow-up request | Direct operator request: Clicking "Recent libraries" is without effect |
| Follow-up request | Direct operator request: Add an option to override the lock in the app |
| Follow-up request | Direct operator request: make the Microsoft Entra public-client ID and tenant ID runtime-configurable for each company; `DMS_ENTRA_CLIENT_ID` and a tenant environment variable supplied before DMS starts must be read-only in the UI, while absent values remain editable; store the public-client ID and tenant ID globally, keep the group ID in library metadata, add Microsoft 365 SMTP app-password setup, and defer final macOS testing because no macOS computer is currently available. |
| Follow-up request | Direct operator request: Add also the "Tenant ID", "Public client ID" in the "Microsoft Entra identity source" overview. Add also a link to the personal Microsoft 365 group page `https://myaccount.microsoft.com/groups/<Group ID>` behind the Group ID. |
| Follow-up request | Direct operator request: replace the free-text document owner with a picker over eligible people; persist immutable Microsoft Entra object IDs rather than owner/editor/approver names or email addresses; and show recursive folder counters `~<drafts>`, `+<applicable files not in the library>`, and `!<unsupported files>` beside each folder in both the tree and list views. |
| Follow-up request | Direct operator request: add three on/off controls above the directory table for “Draft documents”, “Available to add”, and “Unsupported files”; apply them across every folder rather than per folder, leave folder counters unchanged, and paginate the filtered rows. |
| Follow-up request | Direct operator request: when a successful Microsoft Graph import has no eligible direct user members, use the literal dummy values `<owner>` and `<editor>`; show `<editor>` as Requesting editor. Once real eligible people are assigned to the library, the operator can select real Owner and Editor values that take effect with the document's next successful release. |
| Follow-up request | Direct operator request: “if I open notes for an document, there is no "back" button to the previous view with the selected document view in the library” |
| Affected CAPs | CAP-0001 … CAP-0022 |
| Decision records | ADR-0001 … ADR-0023 in `docs/design-decisions.md`; Phase 9k.1 adds ADR-0024 and Phase 9k.3 adds ADR-0025 |

## Scope

Deliver the first vertical slice of a Tauri 2 desktop app for **Windows and
macOS** that:

- Ships a separate, Tauri-independent `dms` CLI over a shared Rust core for
  explicit local workspace initialization, library membership, DMS-managed
  document control data, and document notes; later desktop work consumes that
  core rather than a CLI sidecar
- Configures edit root + publish root and stores metadata under `<edit-root>/.dms`
- Provides a foldable left menu (hamburger when collapsed), session-only
  open-activity panes/tabs as quicklinks in the left chrome, task-and-target
  pane labels with same-task-and-document reuse, and explicit per-user saved
  views; its collapsed rail exposes saved-view and open-pane icons with
  group-specific flyouts that do not expand the full left menu
- Shows a folder-dominant, Windows Explorer-like Library workspace with a
  persistent edit-root-relative tree, Back/Forward/Up + clickable breadcrumbs,
  current-folder child folders and files annotated by library membership,
  recursive per-folder `~` draft / `+` applicable-not-controlled / `!`
  unsupported-file counters shown consistently in tree and list views, three
  session-wide file-visibility toggles above the directory table whose filtered
  rows determine pagination without changing those counters, and a
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
- Gives every Document Notes activity an explicit **Back to Library** action that
  restores the same stable document in the Library selection view without
  closing the Notes activity or discarding its session-only draft/edit state
- Applies inherited confidentiality types and workflow roles as matching
  defaults-first policies: Configuration shows compact root-default and
  people-source summaries, then an edit-root-relative tree and selected-folder
  editor for direct exceptions; confidentiality catalogue administration and
  Entra source setup open only as explicit secondary surfaces. The root policy
  is required, non-root policies can be assigned, replaced, or removed, and a
  single selected library document can carry an override; unregistered files have
  no DMS classification and release-time snapshots remain immutable
- Selects each document Owner from the library's eligible Entra people and keys
  Owner, editor, and approver authority by tenant-scoped object ID under the
  current group binding; display names and email addresses remain mutable
  presentation/notification data rather than identity keys
- Provides one Configuration workspace with persistent **Workspace**,
  **Document defaults**, **Workflow**, and **Notifications** routes. It retains
  one Configuration activity; catalogue administration and Entra identity-source
  setup are contextual secondary surfaces that return to their parent route.
  - Before a workspace is initialized, only **Set up workspace** is available.
  - The setup page keeps up to ten most recently opened edit roots in per-user app
    preferences, lets the operator reopen or remove each one, and gives every
    directory field a native folder picker rooted initially at the OS user's home
    directory.
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
- Gives every growing data table an appropriate case-insensitive text filter and
  selected rows-per-page pagination, with filters applied before pagination
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

## Risk call-out — Phase 9k.5

Phase 9k.5 replaces the persisted SMTP `sender` field, which currently serves
both as authentication user and message `From` mailbox. Its schema migration
therefore changes settings that can cause real external email delivery. The
phase also adds an intentional test delivery.

- Migrate the completed Phase 9k.3 schema-v12 settings to schema v13 by copying
  the legacy `sender` value into both `login_user` and `from_mailbox`, validating
  the result before atomically writing it, and retaining
  `.dms/workspace.v12.json.bak`. If migration or validation fails, leave the
  original metadata in place; recover from the retained backup before retrying.
  Do not infer a different login user or alter the OS-stored password.
- The test button sends one actual SMTP message to the parsed mailbox part of
  the already saved `From` value. Its explicit label names that target; it is
  unavailable unless SMTP settings and the stored credential are present. A
  relay-accepted message cannot be recalled, so automated tests use fakes only.
  Neither the app-password value nor a relay diagnostic may enter `.dms`, an
  IPC response, a wireframe, a test fixture, or the CHG.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 0 | Product records and architecture bootstrap | done (docs tree; Markdown source draft plus format-specific local PDF export; release version policy, approval, notification, confidentiality, workflow-role routing, maintenance, periodic review, optional Claude handoff, foldable shell chrome, task-and-target session activity panes with duplicate reuse, saved views, folder-dominant Explorer-like Library navigation with foldable single-document sections and a same-pane batch state, stable document permalinks, Win+macOS recorded) | CAP/CHG/ADR files exist; indexes list CAP-0001…0020; no CAP claims implemented runtime |
| 0a | Microsoft Entra workflow identity records and wireframes | done (CAP-0021, CAP-0019, ADR-0019, ADR-0021, architecture, privacy, and static screens updated) | CAP-0021 + ADR-0021 replace the application roster with a group binding; CAP-0019 selects only eligible Entra users; static screens show a read-only source, folder-role selection, and no user CRUD; all CAP/CHG/ADR indexes and links validate; no CAP claims implemented runtime |
| 0b | Target-version review, major-only approval, and failure-evidence records + wireframes | done (CAP-0002/0011/0012, CAP-0010, ADR-0007/0013, architecture, and affected static screens updated) | CAP-0002/0011/0012 define the required changelog, minor/major/manual candidate selection, no-reservation rule, `V1.0`/major-only approval, direct minor release with effective-approver publication notification, explicit release actions, optional non-approval comment, and newest-first workflow history; CAP-0010 defines the canonical major-review and minor-publication messages; CHG/ADR/architecture align; affected static screens show direct minor release, major review, and retained unsuccessful evidence; generated exports, links, and Markdown checks pass; no CAP claims implemented runtime |
| 0c | Defaults-first confidentiality and routing wireframes | done (CAP-0008, CAP-0019, CAP-0021, and static screens updated) | CAP-0008 and CAP-0019 use matching compact default summaries, edit-root-relative trees, selected-folder editors, and direct-exception lists; confidentiality catalogue and Entra identity setup are explicit secondary surfaces; generated exports, links, and Markdown checks pass; no CAP claims implemented runtime |
| 0d | Configuration information architecture and wireframes | done (generated HTML/PNG and records checks) | CAP-0005, CAP-0001, CAP-0008, CAP-0010, CAP-0019, and CAP-0021 define one Configuration workspace with four visible task routes, explicit no-workspace setup, and contextual catalogue/identity flows; ADR-0022, architecture, generated screens, exports, links, and Markdown checks align; no CAP claims implemented runtime |
| 0e | Growing-table navigation records and wireframes | done (CAP/CHG contracts plus generated HTML/PNG) | Every unbounded table declares an appropriate case-insensitive filter and CAP-0005's 10/25/50/100 rows-per-page behaviour; generated screens show the shared controls; fixed handler registry remains unpaginated; no CAP claims implemented runtime |
| 1 | Shared Rust core + `dms` CLI foundation, then Tauri 2 app skeleton (Windows + macOS) + DOX for source tree | done (Rust 1.88 local workspace, shell-model, and Xvfb launch gates pass; [Windows and macOS platform smoke](https://github.com/videoclinic/dms/actions/runs/31336690342) passes) | The workspace MSRV meets the selected Tauri 2 release; `cargo test --workspace` covers explicit confirmed workspace initialization, stable IDs, in-root supported-source registration, platform-independent relative-path JSON, independent document-control data, unique document numbers, note CRUD, and desktop shell state; the desktop app launches on Windows and macOS and proves the phase's shell, activity, and saved-view requirements |
| 2 | `.dms` store + dual-root open/configure + confidentiality and document-type catalogues, Entra identity binding, and workflow-role policies | done (schema-v2 migration, core policy tests, CLI policy adapter, local gates, and [Windows/macOS smoke](https://github.com/videoclinic/dms/actions/runs/31357042872) pass) | Tests: migrate schema v1 to v2 with a retained pre-migration backup, reject unknown fields or newer schemas without rewriting metadata, and persist/reload edit+publish roots, stable workspace ID, document-type catalogue, and non-secret Entra tenant/group binding; Configuration's edit-root-relative policy tree includes the root, empty accessible folders, and no `.dms`, and permits only its selected existing node as a policy target; create/replace a direct folder policy, remove a non-root policy, refuse root-policy removal, resolve the nearest inherited class for library documents only, and preserve a document override; inherit each workflow role independently; replacing the identity source marks live role policies unresolved without changing historical evidence; init `.dms` only on confirm |
| 3 | Folder-first Library explorer + add/unregister/reassociate + selection pane | done (schema-v3 migration; core, CLI, desktop-adapter, and UI tests; local gates and [Windows/macOS smoke](https://github.com/videoclinic/dms/actions/runs/31359360786) pass) | Tests: folder pane is visible by default and includes empty edit-root folders while hiding `.dms`; Back/Forward/Up, breadcrumb, tree, and immediate-child contents stay synchronized; every file row keeps its exact filesystem name while an in-library row shows DMS-managed document data separately; one or more unregistered supported source files (including `.md`) can be selected and added from the right pane, while mixed/unsupported selections expose no incompatible batch action; folder navigation reuses one Library activity and updates its folder label; add under edit root; reject outside path; unregister preserves history; rename/reassociate updates only the source locator and does not change document control data or history; ambiguous move is never auto-linked; current-folder and Entire-library search scopes return matching files with paths and clear back to the complete folder listing; single controlled-document selection shows an always-visible Source file identity plus CAP-0015 Document control data and actions in the right pane; the data is loaded from `.dms`, not Office properties or Markdown front matter; its data, action, revision, and release sections fold independently while retaining document and source-file identity; navigating to an already-open task+document focuses the existing pane, while different tasks for that document may remain open; multi-select of controlled documents shows only multi-applicable actions in the same pane; a saved library view restores folder/sort and a single-document stable ID but never batch selection; copy permalink uses workspace+document IDs only and never changes saved views |
| 4 | Lifecycle + Entra-verified major approval + notification + target-version release + tree mirror | done (schema-v4 migration, core lifecycle/port tests, local gates, and [Windows/macOS smoke](https://github.com/videoclinic/dms/actions/runs/31364454130) pass) | Tests: schema v3 migrates to schema v4 with empty lifecycle evidence while preserving prior records and a retained backup; with a Graph client fake, every release requires non-empty changelog, requester, one target mode (next minor, next major, or validated manual greater-than-current unused target), and release-time validation; `V1.0`, major, and a manual target with a higher major component require a current eligible effective approver plus transport success, while a minor candidate creates no review request and releases directly after validation; a failed group refresh or unresolved identity blocks an approval-required request and minor-publication recipient snapshot; SMTP and `mailto:` render the canonical major-review subject/body with DMS-managed title, filesystem-derived relative source path, requester display name, candidate target version, confidentiality label, action, and CAP-0020 review permalink only, and render the committed-minor publication notification to the effective approver; decision requires the snapshotted Entra tenant/object ID and current eligibility; rejected/changes-requested decisions prompt for an optional reason; the workflow history retains changelog, candidate, mode, major decision evidence when applicable, and minor publication delivery attempts newest first while retaining each predecessor hash; a rejected, changes-requested, cancelled, invalidated, or failed-export candidate remains available; CAP-0020 deep link resolves only an accessible registered workspace to the intended review request and still resolves after rename/version bump; each decision notifies the snapshotted requester, while minor publication delivery failure is retryable and never reverses its committed release; approver-policy change invalidates an open review; DOCX body/header/footer and Markdown rendered-body version and confidentiality markers must equal the candidate release and effective type before an approval-required review and again before every release; formats without implemented visible-content scanning fail closed; missing, mismatched, and conflicting markers block by default; an explicit, reasoned false-positive override is revision-bound, visible to the approver, and recorded in the event chain; major approval preserves its accepted target but only atomic export commits it; metadata change invalidates approval; first version V1.0; refuse overwrite |
| 5 | Format-specific local PDF export on release (Win + macOS adapters) | done (release chrome map, `.docx` temp-copy fill, CommonMark print shell, valid-PDF gate, local/cross-target gates, and [Windows/macOS native WebView PDF smoke](https://github.com/videoclinic/dms/actions/runs/31367938246) pass) | Tests/integration: export Office drafts through installed Office (or a test double), replacing `{CONFIDENTIALITY}`/`{VERSION}` on a temp copy from the release chrome map; export Markdown through CommonMark + shipped print shell (logo, `Vertraulichkeitsstufe:` / `Version:` footers from the same map, front matter stripped) + native WebView PDF API to the versioned, classified path on each OS; PDF chrome values match the release snapshot; a failed export does not commit or occupy the approved target; WebView2 and WKWebView smoke cover multi-page footer chrome |
| 6 | Notes on documents | done (core-backed Tauri CRUD, stable-ID Notes activity, draft-preserving errors, explicit delete confirmation, local gates, visual QA, and [Windows/macOS smoke](https://github.com/videoclinic/dms/actions/runs/31373187917) pass) | Tests: note CRUD persistence across restart; list is newest-first; New note compose field is above the latest note and remains there after save |
| 7 | Release checksum + periodic review + verify | done (implementation `8fe38bb`, Windows portability fix `e4f40dc`, schema v5 migration, local gates, visual QA, and [Windows/macOS smoke](https://github.com/videoclinic/dms/actions/runs/31381510835) pass) | Tests: exported PDF → expected SHA-256 `match`; tamper → `mismatch`; missing file → `missing_file`; release snapshots draft digest, target-version mode/label, changelog, and chain; a case-insensitive Title filter scopes publish-tree releases; selected rows-per-page controls pagination of the filtered result; each non-missing release exposes a verify-this-release action while missing files surface a "Missing PDF" badge; periodic confirm keeps version and schedules the next review; changes-required begins revision; obsolete marks the document; full backup manifest covers both roots and refuses to overwrite an existing archive |
| 8 | Optional Claude Desktop handoff | done (implementation `99a26b8`, schema v6 migration, local gates, and [Windows/macOS smoke](https://github.com/videoclinic/dms/actions/runs/31385257274) pass) | Tests: disabled/missing app never blocks; policy and consent gate payload; accepted suggestion remains editable and cannot mutate lifecycle |
| 9 | Packaging smoke | done ([Windows/macOS run 31385996931](https://github.com/videoclinic/dms/actions/runs/31385996931) produced NSIS/DMG artifacts and passed workspace, launch, and native Markdown PDF smoke gates) | Windows and macOS jobs build the workspace, launch the desktop, exercise native Markdown PDF export, and produce installable NSIS/DMG artifacts |
| 9a | Desktop workspace setup | done (explicit dual-root initialization command and setup UI; unconfirmed requests are side-effect-free; focused Rust/frontend tests, full local gates, native form validation, and visual QA pass) | Before a workspace exists, the desktop exposes only Set up workspace; it can open an existing edit root or initialize edit + publish roots only after explicit confirmation; Rust adapter and frontend tests cover refusal, initialization, open, and setup markup; local workspace gates pass |
| 9b | Periodic-review closure | done (`cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `node --test ui/*.test.mjs`; focused periodic-review lifecycle tests) | Core tests cover completion, comment-required cancellation with no schedule shift, and reminder attempts with no duplicate request or lifecycle transition; CLI/Tauri commands and desktop controls expose Result, Cancel, and Reminder with explicit confirmation; fake-backed Rust/frontend gates pass |
| 9b.1 | Library tree and sidebar-state correction | done (`node --test ui/app.test.mjs ui/library.test.mjs`; `node --test ui/*.test.mjs`; full Rust format/test/clippy gates; browser visual QA) | Frontend tests prove nested folder branches with explicit expand/collapse controls, current-folder ancestor expansion, and stable unfolded-sidebar state across destination, saved-view, and open-pane actions; visual QA confirms the folder tree reads as a hierarchy |
| 9b.2 | Recent libraries and native directory browsing | done (`node --test ui/*.test.mjs`; full Rust format/test/clippy gates; Linux desktop launch smoke; browser visual and interaction QA) | Preferences retain at most 10 unique edit roots in most-recent-first order; setup UI can reopen or remove each entry; every directory field exposes a native folder picker that starts at the OS user's home directory; focused Rust/frontend tests, full local gates, and visual QA pass |
| 9c | Audit export | done (schema v7 migration; focused core/CLI/desktop tests; `cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; 36 frontend tests; Linux launch smoke; browser visual QA) | Deterministic document/approver/confidentiality/date-filtered CSV/PDF reports contain classification, workflow, release, periodic-review, target-mode, delivery, hash, and verification evidence without source/release bytes; paths stay inside the edit root and never overwrite; generation appends canonical `report_generated` evidence; CLI and desktop expose generation/list/verify/open-folder operations; Recent reports filters before pagination and surfaces chain, file, and missing/tampered states |
| 9d | Workspace integrity and recovery | done (schema v8 migration; core/CLI/Tauri/frontend lock and restore operations; focused failure-path tests; full Rust format/test/Clippy gates; 39 frontend tests; Linux launch smoke) | Advisory lock status/acquire/owner-matched release/stale takeover and manifest-verified backup restore refuse unsafe or cross-platform-colliding paths, symlinks, fresh-lock overwrite, and unconfirmed replacement; restore holds an advisory lock while writing; core, CLI, desktop, and failure-path tests pass |
| 9e | Release and library maintenance | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; 43 frontend tests) | Release withdrawal requires confirmation and a reason, appends hash-chained evidence, preserves exact history, and cannot drift from canonical evidence; one current-release resolver skips withdrawn records while later target allocation advances beyond all committed versions; missing/orphan releases remain explicit; Library and Releases actions open only validated existing source/PDF paths through host commands |
| 9f.1 | Document control and confidentiality actions | done (schema-v9 migration; canonical before/after control-change evidence; narrow Tauri commands and Library forms; full Rust format/test/Clippy gates; 45 frontend tests; records/link gates) | The Library selection pane edits title, number, type, owner, and effective date through narrow desktop commands; it sets or clears the document confidentiality override from configured types; validation failures remain in the selected document context; adapter/frontend tests pass |
| 9f.2 | Local lifecycle and evidence actions | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; 46 frontend tests; records/link gates) | The Library selection pane invokes Begin revision, Cancel review with a required reason and confirmation, Mark obsolete with a required reason and confirmation, and opens canonical workflow evidence; unavailable transitions explain their preconditions; adapter/frontend tests pass |
| 9f.3 | Workspace and Document defaults configuration | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; 51 frontend tests; release build; Linux launch smoke; records/link and browser visual QA) | One routed Configuration activity provides Workspace and Document defaults routes, persists supported local core settings and catalogue operations, and keeps workspace setup as the only pre-open route; adapter/frontend tests pass |
| 9f.4 | Workflow and Notifications configuration | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; 54 frontend tests; release build; Linux launch smoke; records/link gates; browser visual QA) | The same routed Configuration activity provides Workflow and Notifications routes plus explicit identity-source and confidentiality-catalogue secondary surfaces without creating duplicate activities; adapter/frontend tests pass |
| 9f.5 | Viewport-scoped shell and independent workspace scrolling | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; release build; 55 frontend tests; regenerated wireframes; browser computed-layout and visual QA; `git diff --check`) | The sidebar and main activity header remain available while ordinary activity content scrolls inside the viewport; the Library path toolbar remains fixed while its folder tree, current-folder table, and exhaustive selection details scroll independently; frontend layout tests pass |
| 9f.5.1 | Recent-library open feedback | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; 56 frontend tests; strict records links; Markdown tables; browser success and advisory-lock failure interaction QA; `git diff --check`) | A recent-library click visibly opens an unlocked workspace; an open or advisory-lock failure stays beside the recent list and retains the selected root in the explicit open form; focused frontend tests and browser interaction QA pass |
| 9f.5.2 | Explicit advisory-lock override | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; 56 frontend tests; strict records links; Markdown tables; browser current-lock refusal and explicit-override interaction QA; `git diff --check`) | Setup exposes a clearly warned override-any-lock option; the core rewrites a current or stale lock only after that explicit choice while preserving ordinary refusal and stale-only takeover; focused core, adapter, frontend, and browser gates plus full workspace checks pass |
| 9g | Permalink OS integration | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; release build; 58 frontend tests; strict records links; Markdown tables; Linux launch smoke; [Windows/macOS launch, native PDF export, and package smoke](https://github.com/videoclinic/dms/actions/runs/31503850761)) | Windows and macOS register `dms://`; inbound document/review/note links resolve workspace + stable document identity, focus or create the correct activity, survive rename/version changes, and fail closed for unavailable targets; platform-specific tests remain warnings-clean and the platform smoke passes |
| 9h | Operator-selected Claude excerpts | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; 59 frontend tests; Linux launch smoke) | Oversized assistance payloads show their size and selectable excerpts; preview retries only with the operator-selected subset, never silently truncates, and still requires digest-bound consent; core, adapter, and frontend tests pass |
| 9i | Canonical notification templates and desktop delivery adapters | done (`39db45b`; local Rust format/test/Clippy and 59 frontend tests pass) | Core emits CAP-0010's literal review-request and minor-publication templates; desktop SMTP and host-mail adapters resolve credentials only from OS storage, preserve explicit `mailto:` confirmation, and pass fake-backed delivery tests and operator setup checks |
| 9j | Live Entra setup, refresh, and approver sign-in | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; 59 frontend tests; Linux launch smoke; release build) | Schema v10 persists each identity-cache refresh time; Configuration previews and explicitly applies administrator-supplied tenant/group bindings through interactive delegated Graph access; OS credential storage holds the delegated-token cache; direct-user filtering, refresh-before-role-assignment, policy rerouting, and approver actor verification pass fake-backed tests |
| 9k | External lifecycle commands and Office export | done (`cargo fmt --all -- --check`; `cargo test --workspace`; Clippy with warnings denied; release desktop build; 60 frontend tests; strict repository links; Markdown table structure; `git diff --check`) | Production submit/review/decision/release commands and Library operator surfaces compose the 9i delivery and 9j Graph adapters with installed Office automation on Windows/macOS; retryable mailto confirmations cover review, decision-outcome, and minor-publication delivery; integration fakes and operator smoke instructions pass |
| 9k.1 | Runtime Entra configuration and SMTP app-password setup | done (`cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; 60 desktop UI tests; generated CAP-0010/CAP-0021 HTML and PNG checks; capability-link and Markdown-table checks; `git diff --check`) | Schema v11 removes workspace tenant ID/display while preserving the library group binding, cache, roles, and historical evidence; desktop runtime Graph construction obtains the public-client/tenant configuration from app-global settings and fails closed for invalid overrides; Configuration shows read-only environment-managed Entra fields and group-only library binding; SMTP app passwords are write-only OS credentials, retained on blank input, deleted for `mailto:`, and absent from workspace metadata, snapshots, IPC responses, errors, and wireframes |
| 9k.1.1 | Document Notes return navigation | done (`cargo fmt --all -- --check`; workspace tests; Clippy with warnings denied; 73 frontend tests; regenerated CAP-0003 HTML and 1440×1100 PNG; exact-return, stable-ID fallback, failure-retention, and visual browser QA; strict repository links; 54 Markdown tables; DOX pass; `git diff --check`) | **Back to Library** restores the same stable document without duplicating or reloading the Library activity, falls back through stable-ID resolution when the Library changed, leaves Notes open, and preserves Notes compose/edit/delete state on success and failure |
| 9k.2 | Entra identity-source overview | done (workspace format/test/Clippy gate; 14 focused configuration UI tests; encoded host-browser URL coverage; regenerated and visually inspected 1440×1100 CAP-0021 HTML/PNG; 21-entry manifest; strict links; 41 Markdown tables; DOX pass; `git diff --check`) | `cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/configuration.test.mjs` exits 0; CAP-0021 and its generated HTML/PNG show public-client ID, tenant ID, and a host-browser group-page control; `git diff --check` exits 0 |
| 9k.3 | Release-bound control data and immutable owner identity | pending | Schema-v11 fixture migrates without inventing historical release or owner identity; a successful empty Graph result produces only the literal `<owner>` / `<editor>` placeholders while a Graph failure still fails closed; a later successful real-person refresh permits an operator-selected release-bound Owner/Editor replacement without rewriting past evidence; new candidate/release tests prove immutable control/effective-date snapshots and effective-date-based scheduling; owner/editor/approver references remain object-ID based across name/email changes; `cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/library.test.mjs` exits 0; regenerated CAP-0002/CAP-0015/CAP-0017 wireframes agree with their manifest entries; CAP-0019/CAP-0021 identity links pass; `git diff --check` exits 0 |
| 9k.4 | Library filtering, recursive folder counters, interaction, width, and shipped icons | pending | Core and UI tests prove identical recursive `~` draft, `+` applicable-not-controlled, and `!` unsupported counts beside each folder in tree and list views; all-on session-wide **Draft documents** / **Available to add** / **Unsupported files** toggles filter every folder and search result before pagination without changing counters; candidate form selects `Next minor` by default; a table-folder click opens that folder without a double-click; the centre/details splitter is pointer- and keyboard-usable with bounded session-only width; runtime and wireframes use app-local SVGs rather than a font dependency; `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs` and full Rust/frontend gates exit 0; regenerated CAP-0002/CAP-0006/CAP-0015 wireframes match their manifest entries; `git diff --check` exits 0 |
| 9k.5 | Workflow tree and SMTP identity/test configuration | pending | Workflow renders an accessible folder tree with all direct role assignments visibly marked; schema-v12 SMTP settings migrate to separate login user and RFC 5322 `From` mailbox without exposing the password; a fake-backed SMTP test addresses that `From` mailbox and a saved credential renders only `***`; Rust/frontend gates and regenerated CAP-0010/CAP-0019 wireframes pass; `git diff --check` exits 0 |
| 9l | Windows external operator smokes + CAP promotion | pending | Licensed Office release smoke passes on Windows; configured Entra group + interactive decision and notification smokes pass; full Rust/frontend/records/link gates pass without deprecated Node-runtime action annotations; CAP statuses distinguish Windows-host evidence from existing CI coverage and make no untested macOS-installed-Office claim; CHG status is done and the record is archived |

**Current phase:** 9k.2 complete. Phase 9k.3 is the next dependency-ready phase
and remains pending. It must pass before Phase 9k.4 and Phase 9k.5, before Phase
9l can start. macOS remains a supported runtime target with existing CI coverage,
but an external macOS operator smoke is not a Phase 9l gate.

Mark a phase `in-progress` only while it is being executed, `done (<evidence>)`
only after its gate passes, and `pending` otherwise.

## Risk call-out — Phase 9k.1

The change moves a currently workspace-persisted Entra tenant binding into an
app-global configuration and starts accepting an SMTP app password. A partial
migration could make an existing group source unusable. The password may exist
only transiently in the write-only form and its single local command input; it
must never appear in `.dms`, persisted frontend state, an IPC response, test
fixture, log, or the CHG.

- Keep the working tree clean before the schema migration. The v10 fixture and
  migration backup are the recovery path: if the v11 migration or save fails,
  restore the retained v10 `workspace.json` backup and do not retry against the
  partially written store.
- Treat a non-empty but invalid `DMS_ENTRA_CLIENT_ID` or `DMS_ENTRA_TENANT_ID`
  as an explicit configuration error. Do not silently fall back to a stored
  value, and do not give the user an editable field that appears to override it.
- Validate SMTP host/port/sender and the selected transport before writing a new
  app password to OS storage. A blank password means “keep the stored password”;
  an SMTP setup without a stored password must fail clearly before it can be
  used. Switching to `mailto:` removes the workspace-scoped SMTP credential.
- The local test suite and CI fakes prove the adapters only. They cannot prove a
  Microsoft 365 SMTP app password, delegated Graph consent, or installed Office
  on a configured Windows host; those remain Phase 9l evidence. Existing macOS
  CI coverage is retained, but no external macOS operator smoke is scheduled.

## Phase 9k.1 — Runtime Entra configuration and SMTP app-password setup

**Goal:** Replace compile-time `DMS_ENTRA_CLIENT_ID` with app-global runtime
configuration, add app-global `DMS_ENTRA_TENANT_ID` support, retain only each
library’s Entra group ID in `.dms`, and let the Notifications route write a
Microsoft 365 SMTP app password directly to OS credential storage without
persisting or returning it.

**Plan ID:** `CHG-0001#phase-9k.1`
**Execution slot:** P0100 (phase-local; the active CHG keeps its immutable
`CHG-0001-…` filename under the change-record lifecycle)
**Created:** 2026-08-13
**Depends on:** `CHG-0001#phase-9k` (`7b7402b`)
**Plan family:** `CHG-0001-tauri-local-dms-bootstrap`
**Status:** done — local implementation and verification complete; Phase 9l
retains the external operator-smoke and CAP-promotion evidence.

### Phase 9k.1 fresh-session context

- **Entry checkpoint:** Phase 9k checkpoint
  [`7b7402b`](https://github.com/videoclinic/dms/commit/7b7402b); no uncommitted
  changes unless they belong to this phase.
- **Context sources:** this phase and the phase table in this CHG;
  `docs/architecture.md` (**Runtime shape**, **Trust and control boundary**),
  `docs/privacy.md` (**Data classes**, **Processing principles**),
  `docs/design-decisions.md` (ADR-0009, ADR-0012, ADR-0021, ADR-0022),
  `docs/product/capabilities/CAP-0001-local-folder-dms.md`,
  `docs/product/capabilities/CAP-0010-notification-transport.md`,
  `docs/product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md`,
  `crates/dms-core/AGENTS.md`, and `crates/dms-desktop/AGENTS.md`.
- **Atomicity rationale:** This is one vertical slice: v11 metadata migration,
  app-global settings, Graph construction, secure SMTP credential mutation, and
  the Configuration forms must agree on the same ownership boundary. Splitting
  them would either leave tenant IDs in workspace metadata or expose a password
  input with no safe persistence path. This CHG exceeds the normal 40 KiB plan
  warning because it is the repository’s single active historical progress
  authority; a new session need load only this subsection and its listed sources.
- **Produces:** schema-v11 migration/test fixtures; a global runtime Entra
  configuration boundary; group-only library bindings; app-password setup;
  refreshed CAP/ADR/architecture/privacy/wireframe contracts; and a Phase 9l
  Windows-host external verification condition.

**Steps:**

1. Amend the current-state contracts before code: add ADR-0024 for app-global
   Entra client/tenant configuration and its environment precedence; update
   CAP-0001, CAP-0002, CAP-0010, CAP-0021, `architecture.md`, and `privacy.md`.
   Regenerate the CAP-0010 and CAP-0021 wireframes from
   `docs/product/wireframes/generate.mjs` and render their PNGs. The contracts
   must distinguish app-global configuration from per-library metadata and from
   OS credentials.
2. Add an app-global, per-OS-user configuration file beside
   `preferences.json` in Tauri’s `app_config_dir`, named
   `global-settings.json`. It holds only the non-secret Entra public-client ID
   and tenant ID. It is shared by all local DMS libraries for that OS user and
   is never written below `<edit-root>/.dms/`.
3. Define the effective runtime configuration precisely:
   - `DMS_ENTRA_CLIENT_ID` is read through `std::env::var` when DMS starts; it
     replaces the compile-time `option_env!` path.
   - `DMS_ENTRA_TENANT_ID` is read through `std::env::var` when DMS starts.
   - Each non-empty environment value wins over the corresponding stored global
     value for that process. The UI labels that field as environment-managed and
     makes it read-only. An absent variable leaves the stored global field
     editable. An invalid non-empty environment value is a blocking error, not
     a fallback trigger.
   - The Configuration → Workflow → Manage identity source surface has a
     distinct **Application Entra configuration** card for the global public
     client ID and tenant ID, and a **Library Entra group** card for the current
     workspace’s group ID. Saving the global card does not rewrite any library.
4. Migrate `.dms` from schema v10 to v11. Remove the tenant ID/display from
   `EntraIdentitySource`; retain the binding ID, group ID/label, last refresh,
   cached people, role references, and historical workflow evidence. The group
   ID is the only current Entra configuration identifier owned by library
   metadata. Runtime Graph operations obtain the tenant/client from effective
   app-global settings. Existing historical decision events keep their recorded
   tenant/object IDs. A global tenant change does not rewrite library metadata;
   the next required refresh revalidates the group and fails closed on mismatch
   or inaccessible membership.
5. Retain the existing core Graph port, but keep app-config, environment,
   desktop Graph transport, and credential-store logic out of `dms-core`.
   Inject the effective global Entra configuration into the desktop Graph
   client. Token entries remain tenant-scoped in the OS credential store;
   `.dms` contains no token, client ID, tenant ID, or client secret after v11.
6. Extend **Configuration → Notifications** for SMTP app-password setup. For
   SMTP it shows a non-secret credential status and a write-only **Microsoft 365
   app password** field; it never pre-fills, serializes, or returns the value.
   The value exists only until its one-way local command submission completes.
   An empty field retains an existing credential, while a missing credential is
   rejected before an SMTP workflow operation. Write/replace/delete through a
   narrow desktop credential-store boundary keyed by workspace ID, after core
   notification settings validate. Selecting `mailto:` deletes the
   workspace-scoped SMTP credential with the relay settings. The sender remains
   the SMTP authentication user unless a later product request adds a distinct
   username.
7. Add focused migration, core-port, desktop adapter, credential-store fake, and
   frontend tests. Cover runtime environment precedence and invalid overrides;
   read-only environment-managed fields; editable stored fields; group-only v11
   metadata; legacy v10 migration/backup; global tenant-change fail-closed
   refresh; password write/replace/delete; blank-password retention; no password
   in Configuration snapshots, IPC results, `.dms`, or errors; and the existing
   SMTP/`mailto:` delivery paths. Use synthetic values only.
8. Run the phase gate, check generated wireframe/index/manifest links and
   Markdown tables, then record the exact command output in the phase status
   before starting Phase 9l. Do not attempt or promote external Microsoft 365,
   Office, or Windows observations as local test evidence.

**Verification gate:**
`cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/configuration.test.mjs` exits 0; v10→v11 fixtures prove tenant/client IDs are absent from `.dms` while group binding and evidence survive; frontend tests prove environment-provided client/tenant values are read-only; credential-store fakes prove the app password is write-only and absent from all returned/persisted workspace data; regenerated CAP-0010/CAP-0021 wireframes match the manifest; repository link and Markdown-table checks pass.

**Completion evidence (2026-08-13):** `cargo fmt --all -- --check`,
`cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`
exit 0; `node --test crates/dms-desktop/ui/*.test.mjs` passes all 60 tests.
The focused migration and desktop configuration tests cover group-only v11
metadata, retained v10 backup, fail-closed runtime Entra configuration, and
write-only SMTP credential retention/replacement/deletion. `node generate.mjs`
regenerated the wireframes; headless Chrome rendered CAP-0010 and CAP-0021 PNGs,
the manifest contains 21 non-empty HTML/PNG pairs, generated contract text is
present, capability Markdown links resolve, Markdown table structure validates,
and `git diff --check` exits 0. No Microsoft 365, licensed Office, Windows, or
macOS external observation was attempted or claimed.

### Phase 9k.1 out of scope

- Running against a live Microsoft 365 tenant, SMTP relay, or licensed Windows
  Office host; those observations are Phase 9l-only evidence. An external macOS
  operator smoke is deferred and is not a gate for this CHG.
- Promoting affected CAPs to implemented or archiving this CHG before Phase 9l
  passes its Windows-host gate. Existing macOS CI evidence remains required for
  the supported-target claim, but no external macOS Office observation gates
  this CHG.

## Phase 9k.1.1 — Document Notes return navigation

**Goal:** Add a visible **Back to Library** control to every Document Notes
activity; activating it restores the Library with that same stable document in
the selection view and leaves the Notes activity plus its unsaved session state
intact.

**Plan ID:** `CHG-0001#phase-9k.1.1`
**Execution slot:** P0140 (phase-local; the active CHG keeps its immutable
`CHG-0001-…` filename under the change-record lifecycle)
**Created:** 2026-08-14
**Depends on:** `CHG-0001#phase-9k.1`
**External request:** Direct operator request: “if I open notes for an document,
there is no "back" button to the previous view with the selected document view
in the library”
**Plan family:** `CHG-0001-tauri-local-dms-bootstrap`
**Entry checkpoint:** Phase 9k.1 marked `done (<gate evidence>)` in this CHG;
the checkout is clean apart from planned CHG updates.
**Atomicity rationale:** The CAP wording, generated reference, runtime control,
activity transition, failure retention, and focused tests are one small vertical
slice; splitting them would leave a visible button without a proven return
contract or a contract without a usable control. This CHG exceeds the normal 40
KiB warning because it is the repository's single active historical progress
authority; a fresh session need load only this section, its exact context
sources, and Phase 9k.1 completion evidence.
**Context sources:** this phase and the phase table;
`docs/product/capabilities/CAP-0003-document-notes.md` (**Outcomes** and
**Non-goals**); `docs/product/capabilities/CAP-0005-desktop-shell.md` (**Open
activities**); `docs/product/capabilities/CAP-0006-library-explorer.md`
(**Outcomes** 6, 8–9, 12, and 15);
`docs/product/wireframes/AGENTS.md` and
`docs/product/wireframes/generate.mjs` (CAP-0003 screen); `crates/AGENTS.md`;
`crates/dms-desktop/AGENTS.md`; `crates/dms-desktop/ui/notes.mjs`
(`documentNotesMarkup`); `crates/dms-desktop/ui/app.mjs` (`openActivity`,
`permalinkActivity`, `applyPermalinkDocumentSelection`, `openDocumentNotes`,
`loadLibraryFolder`, and `handleNotesClick`);
`crates/dms-desktop/ui/styles.css`;
`crates/dms-desktop/ui/notes.test.mjs`; and
`crates/dms-desktop/ui/app.test.mjs`.
**Produces:** Current CAP-0003/CAP-0005/CAP-0006 navigation contracts and a
matching regenerated CAP-0003 wireframe; a tested Notes control that focuses or
recreates the singleton Library activity with the same stable document selected;
and failure behaviour that preserves the current Notes draft/edit state.
**Status:** done (`cargo fmt --all -- --check`; workspace tests; Clippy with
warnings denied; 73 frontend tests; regenerated CAP-0003 HTML and 1440×1100
PNG; exact-return, stable-ID fallback, failure-retention, and visual browser QA;
strict repository links; 54 Markdown tables; DOX pass; `git diff --check`).

### Current state

- CAP-0003 defines note CRUD, persistence, ordering, and composer placement but
  no exit or return path; its generated screen exposes only **Add note**
  (`docs/product/capabilities/CAP-0003-document-notes.md:9-27`,
  `docs/product/wireframes/generate.mjs:129-152`).
- `documentNotesMarkup` replaces the main activity body with a Notes heading,
  composer, and list, but renders no Library-return control or click hook
  (`crates/dms-desktop/ui/notes.mjs:70-80`).
- `openDocumentNotes` opens a stable-ID Notes activity while retaining the
  process-wide Library state, but records an empty `route_state`; Notes
  permalinks do the same even though their resolution already supplies the
  current document folder (`crates/dms-desktop/ui/app.mjs:182-210`, `835-858`).
- The shell already gives Notes and Library separate stable activity keys and
  focuses an existing key instead of duplicating it; the Library owns one
  session activity (`crates/dms-desktop/ui/app.mjs:148-179`,
  `docs/product/capabilities/CAP-0005-desktop-shell.md:68-99`).
- The document-permalink path already loads a Library folder by stable document
  ID and applies retained document detail even when no filesystem row exists
  (`crates/dms-desktop/ui/app.mjs:213-223`, `573-599`); `DocumentSelection`
  supplies the current relative parent folder (`crates/dms-desktop/src/lib.rs:179-203`,
  `1902-1918`).
- `handleNotesClick` handles edit/delete controls only, and the focused Notes and
  app tests contain no return-navigation assertion
  (`crates/dms-desktop/ui/app.mjs:1241-1297`,
  `crates/dms-desktop/ui/notes.test.mjs`, `app.test.mjs:235-279`).

**Steps:**

1. Amend CAP-0003, CAP-0005, and CAP-0006 before runtime code. A Document Notes
   activity exposes a prominent **Back to Library** control that targets the
   same workspace and stable document ID. If the Library still holds that exact
   document selection, focus its singleton activity without reloading so folder,
   search, sort, history, and selection state remain the previous view. If the
   Library activity was closed, changed to another document, or Notes came from
   a permalink/saved view, resolve the stable ID and reveal its current folder
   and selection pane. A missing source creates no fabricated row but retains
   the registered document detail and missing-source state. Returning focuses or
   recreates Library; it does not close Notes, clear its composer/edit/delete
   state, mutate notes, or create a second Library activity. Resolution failure
   leaves Notes current, shows a document-scoped error, and preserves all draft
   state. This is a targeted activity transition, not global browser history.
2. Update CAP-0003 in `generate.mjs` so **Back to Library** appears before the
   note composer and clearly names the destination while the same document title
   and stable ID remain visible. Regenerate HTML/index/manifest, render
   `exports/CAP-0003-document-notes.png` with the documented headless-Chrome
   procedure, and inspect action hierarchy, keyboard discoverability, clipping,
   and the entry/return relationship to the selected-document Library view.
3. In `documentNotesMarkup`, render one ordinary button with visible **Back to
   Library** text, an accessible name that includes “selected document”, and a
   narrow `data-note-return-library` hook ahead of the Notes title/composer. Add
   only the local layout styling needed to keep the control visible at narrow
   widths; do not add a second navigation bar or overload the Library's folder
   Back/Forward controls.
4. Record the source Library folder in Notes `route_state.folder` when opening
   Notes from a selection, and retain `resolution.folder` for a Notes permalink;
   the stable document ID remains authoritative if the source is later renamed.
   Add one shared Library-document restore path used by the new Notes action and,
   where it removes duplication, the existing document-permalink path. On an
   exact in-memory match, focus or recreate the `task: "Library"` activity and
   preserve the current Library state. Otherwise load `DocumentSelection` and
   the current folder while Notes remains current, then atomically apply the
   folder snapshot and `applyPermalinkDocumentSelection` before focusing the
   Library. Ensure folder loading updates the Library activity, never rewrites
   the Notes activity merely because both use the Library destination.
5. Handle `data-note-return-library` at the start of `handleNotesClick`, before
   note edit/delete dispatch. Consume the click once, disable repeated dispatch
   while fallback loading is active, and map any lookup/load failure into that
   document's existing Notes error state without resetting composer, editor, or
   delete-confirmation values. The success path leaves the Notes activity and
   `note_documents[document_id]` state available for an immediate return.
6. Extend `notes.test.mjs` and `app.test.mjs`. Prove the visible control/hook is
   before the composer; Notes opened from Library and from a permalink retain a
   folder hint; an exact previous Library view is focused without reload or
   duplication; a changed/closed Library is restored by stable ID to the current
   folder and selected detail; a missing filesystem row keeps detail with an
   empty row selection; failure keeps Notes current; and compose, edit, and
   delete-confirmation state survive both success and failure. Include repeated
   activation and escaped title/ID coverage. Run browser interaction QA by
   opening Notes from a selected Library document, clicking **Back to Library**,
   and verifying that the same row/detail is selected and the Notes pane remains
   available with its draft intact.
7. Run the phase gate, regenerate and inspect CAP-0003 HTML/PNG/manifest output,
   complete the DOX pass, and record exact evidence in this phase before moving
   Phase 9k.2 to `in-progress`.

**Verification gate:**
`cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/*.test.mjs` exits 0; focused frontend tests prove the Notes control, exact previous-view fast path, stable-ID fallback, missing-row detail, failure retention, singleton Library activity, and unchanged Notes draft/edit/delete state; `(cd docs/product/wireframes && node generate.mjs && google-chrome --headless=new --hide-scrollbars --window-size=1440,1100 --screenshot=exports/CAP-0003-document-notes.png "file://$PWD/html/CAP-0003-document-notes.html" && test -s exports/CAP-0003-document-notes.png)` exits 0; CAP-0003 HTML, PNG, and manifest agree; browser interaction QA returns to the same selected Library document while leaving Notes open; strict repository links and Markdown tables pass; the DOX pass is complete; and `git diff --check` exits 0.

### Phase 9k.1.1 completion evidence

- The full Rust/frontend gate exits 0, including 73 frontend tests. Focused
  tests cover control placement and escaping, exact unchanged-Library reuse,
  stable-ID restoration after a changed or closed Library, missing-row retained
  detail, singleton activities, duplicate-return suppression, and retained
  compose/edit/delete state.
- Browser interaction QA opens Notes from a selected fixture document. The
  exact return performs no additional IPC and restores the selected row/detail;
  changed Library state invokes stable-ID resolution; failed resolution leaves
  Notes current, shows the document error, keeps the unsaved body and author,
  and re-enables retry. The Notes pane remains open throughout.
- Wireframe regeneration and its 1440×1100 Chrome export exit 0. The 21-entry
  manifest resolves every matching HTML/PNG pair, and visual inspection finds
  no clipping, overlap, broken glyph, or action-hierarchy defect.
- Strict repository links report zero issues; structural validation passes for
  54 Markdown tables and 386 data rows; `git diff --check` exits 0.
- The DOX pass updates `crates/dms-desktop/AGENTS.md` for the durable return and
  state-retention contract. Parent/product/wireframe ownership and child indexes
  are unchanged.

## Out of scope — Phase 9k.1.1

- A global cross-activity history stack, changing the Library folder
  Back/Forward toolbar, or adding return controls to every document task.
- Closing Notes automatically, persisting open Notes tabs across sessions, or
  persisting unsaved note drafts to `.dms` or OS preferences.
- Changing note storage, CRUD semantics, the metadata schema, Tauri/core note
  commands, stable document identity, or the `dms://` permalink format.
- Implementing CAP-0006's remaining Library outcomes or the Phase 9k.4 Library
  filtering, counters, splitter, and icon changes.

## Phase 9k.2 — Entra identity-source overview

**Goal:** The Microsoft Entra identity-source overview shows the effective
non-secret Public client ID and Tenant ID with its bound group, and its displayed
Group ID opens `https://myaccount.microsoft.com/groups/<encoded-group-id>` in the
host browser.

**Plan ID:** `CHG-0001#phase-9k.2`
**Execution slot:** P0150 (phase-local; the active CHG keeps its immutable
`CHG-0001-…` filename under the change-record lifecycle)
**Created:** 2026-08-14
**Depends on:** `CHG-0001#phase-9k.1.1`
**Plan family:** `CHG-0001-tauri-local-dms-bootstrap`
**Entry checkpoint:** Phase 9k.1.1 marked `done (<gate evidence>)` in this CHG;
the checkout is clean apart from this planned CHG update.
**Context sources:** this phase and the phase table; CAP-0021 (**Implemented
subset**, **Full capability contract** item 2);
`docs/product/wireframes/AGENTS.md`; `docs/product/wireframes/generate.mjs`
(CAP-0021 screen); `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`;
`crates/dms-desktop/ui/configuration.mjs` (`identitySourceMarkup`);
`crates/dms-desktop/ui/configuration.test.mjs` (workflow/identity-source markup
tests); `crates/dms-desktop/ui/app.mjs` (`handleClick`); and
`crates/dms-desktop/src/lib.rs` (`validate_external_url`).
**Produces:** CAP-0021's current summary contract and matching regenerated
wireframe; a tested desktop identity-source overview that displays the effective
global IDs and opens the bound group page through the existing host-browser
boundary.
**Status:** done — local implementation and verification complete.

**Steps:**

1. Amend CAP-0021's implemented subset and full-contract current-source summary
   to distinguish the app-global effective Public client ID and Tenant ID from
   the library-bound group data. Specify that the overview renders the Group ID
   as a host-browser control for
   `https://myaccount.microsoft.com/groups/<encoded-group-id>`; retain the
   existing environment precedence and no-secret guarantees.
2. Update the CAP-0021 screen definition in
   `docs/product/wireframes/generate.mjs` with synthetic public-client, tenant,
   and group values plus a visible Group ID page control. Run `node generate.mjs`
   from `docs/product/wireframes/`, render the generated CAP-0021 HTML with the
   repository's headless-Chrome procedure into its PNG, and confirm the manifest
   still inventories matching HTML/PNG outputs.
3. In `identitySourceMarkup`, render the effective app-global Public client ID
   and Tenant ID in the existing Microsoft Entra identity-source overview even
   though they are not library metadata. Build the group-page URL from the fixed
   `https://myaccount.microsoft.com/groups/` origin plus an encoded displayed
   group ID. Render that value as an accessible Group ID control using the
   existing `data-open-external` host-browser path; do not introduce direct
   WebView navigation, a new IPC command, or a broader URL policy. The existing
   `validate_external_url` accepts HTTPS URLs and the shared click handler maps
   the control to `open_external_url`.
4. Extend `configuration.test.mjs` from its current populated identity-source
   fixture. Keep that fixture schema-v11 accurate by retaining only group data
   in `identity_source`, with the effective client/tenant IDs supplied by
   `global_entra_configuration`. Assert the overview exposes the Public client
   ID and Tenant ID, emits exactly the expected encoded My Account group URL
   through `data-open-external`, and does not add a `target="_blank"` WebView
   link. Keep the no-source state free of a fabricated group-page control.
5. Run the phase gate and inspect the CAP/HTML/PNG output before marking the
   phase done. Record the exact passing evidence in this CHG, then leave Phase
   9l pending for its Windows-only external pre-checks.

**Verification gate:**
`cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/configuration.test.mjs` exits 0; the populated UI fixture proves both global IDs and the exact encoded `data-open-external` group URL without `target="_blank"`; `node generate.mjs` exits 0 in `docs/product/wireframes/`; the regenerated CAP-0021 HTML, PNG, and manifest agree; and `git diff --check` exits 0.

### Phase 9k.2 completion evidence

- The Rust format, workspace test, and warnings-denied Clippy gate exits 0; all
  14 focused Configuration UI tests pass.
- The schema-v11 frontend fixture keeps effective client/tenant IDs in
  `global_entra_configuration` and only group data in `identity_source`. The
  populated overview test proves the exact encoded Microsoft My Account URL,
  host-browser data hook, accessible Group ID label, and absence of WebView
  `target="_blank"`; the no-source state renders no group-page control.
- `node generate.mjs` exits 0. CAP-0021's HTML and 1440×1100 PNG visibly show
  the Public client ID, Tenant ID, library group, Group ID action, and last
  refresh without clipping or overlap. The manifest inventories 21 non-empty
  HTML/PNG pairs.
- Strict repository links report zero issues; 41 Markdown tables pass structural
  validation; the DOX pass updates the desktop adapter contract; and
  `git diff --check` exits 0. No live Microsoft 365 or Windows-host evidence was
  attempted or claimed.

## Out of scope — Phase 9k.2

- Changing global Entra persistence, environment precedence, delegated tokens,
  or the library group-binding schema.
- Entra or Microsoft 365 group administration, membership refresh semantics, or
  a new external-URL permission policy.
- Windows-host Microsoft 365, notification, or Office evidence; those remain
  Phase 9l gates.

## Risk call-out — Phase 9k.3

Schema v11 stores `effective_date` as mutable document-control data even though
the contracts call it the current release's date. Release records do not retain
that date or a document-control snapshot, and review scheduling instead starts
from `released_at`. A naïve migration would either erase a real date or invent
historical release data; both are unacceptable for document-control evidence.

Schema v11 also stores `DocumentControl.owner` as arbitrary text. Names and
email addresses are mutable and may be duplicated, so that text cannot be
safely converted into a Microsoft Entra identity. Inferring a person during
migration would silently assign authority to an unverified account.

A direct-member response with zero eligible users is distinct from a Graph,
tenant, group, or authorization failure. Conflating those states would let an
outage masquerade as an unassigned library; inventing a UUID for `<owner>` or
`<editor>` would let a display fallback become workflow authority.

- Keep a clean tree before opening a v11 workspace. The existing migration
  backup is the recovery path: if v11→v12 validation or save fails, restore the
  retained v11 `workspace.json` backup and do not retry against partially
  migrated metadata.
- Migrate a stored v11 effective date into the current non-withdrawn release
  only when one exists. Preserve `None` for earlier releases that never captured
  a date, retain their existing due dates, and display that absence as legacy
  evidence rather than substituting `released_at` or today's date.
- The migration must remove the mutable field only after its mapped release
  record and the complete workspace validate. It must not recompute an existing
  next-review-due date, change a version, release PDF, candidate, workflow hash,
  or source locator.
- Move each non-empty legacy owner string to a display-only
  `legacy_owner_label`, leave the new owner reference unset, and require an
  explicit eligible-person selection before a new candidate or release. Never
  match a legacy label to a person by display name or email address. Clear the
  legacy label only after a validated object-ID assignment succeeds.
- Carry the successful-empty-import state explicitly from Graph refresh through
  core and desktop presentation; do not derive it from a generic empty cache.
  A refresh failure remains an error and must leave the last known state intact.
  `<owner>` and `<editor>` never receive an object ID, never become approver or
  mail-recipient data, and cannot pass candidate/release validation. Once a
  later refresh provides real eligible people, validate the operator's selected
  IDs again at the successful release commit; retain the prior placeholder
  snapshots and restore the pre-release routing state if export or save fails.

## Phase 9k.3 — Release-bound control data and immutable owner identity

**Goal:** Replace the mutable document-level effective date with a required
candidate effective date and immutable release control snapshot; replace the
free-text owner with an eligible-person picker backed by a Microsoft Entra object
ID; and preserve literal `<owner>` / `<editor>` placeholders only while a
successful Graph import has no eligible direct users, so releases remain
attributable across later name or email changes.

**Plan ID:** `CHG-0001#phase-9k.3`
**External request:** Direct operator request: “Fix the design of the library
document control data.”
**Follow-up request:** Direct operator request: the Owner in **Edit document
control data** should be selected from **Eligible people**, and owner, editor,
and approver identity must use an object ID that survives name or email changes.
**Follow-up request:** Default the candidate **Target version** control to
**Next minor**.
**Follow-up request:** When a successful Graph import contains no eligible
direct users, use dummy `<owner>` and `<editor>` values, including `<editor>` as
Requesting editor. After a later successful real-person import, the operator may
select real Owner and Editor values to apply on the document's next successful
release.
**Execution slot:** P0175 (phase-local; the active CHG keeps its immutable
`CHG-0001-…` filename under the change-record lifecycle)
**Created:** 2026-08-14
**Depends on:** `CHG-0001#phase-9k.2`
**Plan family:** `CHG-0001-tauri-local-dms-bootstrap`
**Entry checkpoint:** Phase 9k.2 marked `done (<gate evidence>)` in this CHG;
the checkout is clean apart from this planned CHG update.
**Atomicity rationale:** The schema migration, owner-reference conversion,
candidate/release invariants, schedule anchor, adapter contract, and
selection-pane labels must land as one vertical slice. Splitting them would
either permit a mutable effective date with no historic snapshot, leave owner
authority keyed by mutable text, or persist a new snapshot the operator cannot
supply or inspect. This CHG exceeds the normal 40 KiB warning because it is the
single active historical progress authority; a fresh session need load only
this section, its listed sources, and Phase 9k.2 completion evidence.
**Context sources:** this phase and the phase table; `docs/architecture.md`
(**Runtime shape**, **Trust and control boundary**, **Dual-root path model**);
`docs/privacy.md` (**Data classes**, **Processing principles**);
`docs/design-decisions.md` (ADR-0013, ADR-0015, ADR-0016, ADR-0017, ADR-0021);
CAP-0002 outcomes 4, 9, and 11; CAP-0006 outcomes 5, 6, and 8;
CAP-0012 (**Outcomes**); CAP-0015 (**Outcomes** 1, 2, 10–12);
CAP-0017 outcomes 1–2; CAP-0019 outcomes 4–8; CAP-0021 outcomes 1–8;
`docs/product/wireframes/AGENTS.md`;
`docs/product/wireframes/generate.mjs` (CAP-0002, CAP-0015, CAP-0017);
`crates/AGENTS.md`; `crates/dms-core/AGENTS.md`;
`crates/dms-cli/AGENTS.md`; `crates/dms-desktop/AGENTS.md`;
`crates/dms-core/src/lib.rs` (`DocumentControl`, `ControlUpdate`, migration);
`crates/dms-core/src/policies.rs` (`EntraPerson`, `WorkflowRoleRef`, resolution);
`crates/dms-core/src/lifecycle.rs` (`CandidateMetadataSnapshot`,
`ReleaseCandidate`, `ReleaseRecord`, release commit);
`crates/dms-core/src/maintenance.rs` (`schedule_next_review`);
`crates/dms-core/src/audit.rs` (`audit_rows`);
`crates/dms-cli/src/main.rs` (`DocumentCommand`, lifecycle command input);
`crates/dms-desktop/src/graph.rs` (eligible-person refresh);
`crates/dms-desktop/src/lib.rs` (`DocumentSelection`, owner/candidate commands);
and `crates/dms-desktop/ui/library.mjs` / `library.test.mjs`.
**Produces:** schema-v12 metadata that distinguishes mutable document profile,
candidate/release snapshots, and review schedule; an explicit effective-date
input during candidate creation; object-ID-based owner/editor/approver identity;
stable historical release/audit views; updated CAPs and regenerated wireframes;
and a Phase 9l checkpoint based on that model.
**Status:** pending — queued behind the independent Entra overview phase.

### Current state

- `DocumentControl` combines title, number, free-text `owner`, and mutable
  `effective_date` (`crates/dms-core/src/lib.rs:329-355`), even though
  CAP-0015 calls that date a property of the current released version
  (`docs/product/capabilities/CAP-0015-document-control-data.md:14-27`).
- The desktop renders Owner as a free text input and sends a string through the
  Tauri adapter and CLI (`crates/dms-desktop/ui/library.mjs:168-186`,
  `crates/dms-desktop/src/lib.rs:763-804`, `crates/dms-cli/src/main.rs:140-153`).
- Editor and approver already persist `binding_id` plus immutable Entra
  `object_id`; their display names and email addresses are cache/snapshot data,
  not role keys (`crates/dms-core/src/policies.rs:55-102`,
  `crates/dms-core/src/lifecycle.rs:51-85`). The missing identity conversion is
  Owner, plus explicit regression coverage that mutable display data never
  becomes identity.
- Every control edit appends before/after evidence and invalidates candidates,
  but `update_control` accepts an effective-date update in every lifecycle state
  (`crates/dms-core/src/lib.rs:618-650`).
- Candidate metadata snapshots the full mutable control object, while
  `ReleaseRecord` retains no control snapshot or effective date
  (`crates/dms-core/src/lifecycle.rs:79-85`, `253-272`).
- Release schedules the next review from `released_at`, not an effective date
  (`crates/dms-core/src/lifecycle.rs:948-991`); existing `next_review_due` is
  document schedule state, not a `DocumentControl` field
  (`crates/dms-core/src/lib.rs:307-314`).
- The Library's Edit document control data form exposes the date, while the
  candidate form has no effective-date input
  (`crates/dms-desktop/ui/library.mjs:168-186`, `441-466`).
- Audit release rows use the current document title, so a later profile rename
  can relabel historic releases (`crates/dms-core/src/audit.rs:467-498`).
- `replace_identity_source` and `refresh_eligible_people` accept an empty cache,
  but identity-source setup and all role/requester pickers disable themselves
  when it is empty (`crates/dms-core/src/policies.rs:284-305`,
  `crates/dms-core/src/lifecycle.rs:507-534`, and
  `crates/dms-desktop/ui/configuration.mjs:172-207`); the current candidate
  form therefore has no requester value in that state
  (`crates/dms-desktop/ui/library.mjs:441-457`).
- `PersonSnapshot` and `WorkflowRoleRef` require UUID-backed Entra identity
  (`crates/dms-core/src/lifecycle.rs:51-57` and
  `crates/dms-core/src/policies.rs:75-102`), so the literal placeholders cannot
  be fabricated as Graph object IDs or silently become role assignments.

**Steps:**

1. Amend the current-state contracts before code. Add ADR-0025 for the three
   durable data domains: mutable document profile (title, number, type, and an
   owner reference), immutable candidate/release snapshot (profile including
   the owner's tenant/object ID plus display snapshot, effective date,
   confidentiality, and workflow people), and mutable review schedule
   (interval/exemption/next due). Amend ADR-0021, CAP-0015, CAP-0019, CAP-0021,
   `architecture.md`, and `privacy.md`: owner/editor/approver authority is keyed
   only by tenant-scoped Entra object ID under the current group binding; names
   and email addresses are refreshable display/notification data. Update
   CAP-0002, CAP-0006, CAP-0012, and CAP-0017 so effective date is a required
   candidate input captured only by a successful release, never ordinary
   profile data. State that new releases schedule from the snapshot's effective
   date and historic v11 omissions remain visibly unrecorded rather than
   fabricated. Specify the empty-successful-Graph fallback exactly: only a
   successful direct-member response containing zero eligible users exposes the
   literal UI placeholders `<owner>` and `<editor>`; a missing binding, tenant
   mismatch, refresh error, disabled-only response, or inaccessible group still
   fails closed. Placeholders are neither Entra identities nor approvers and
   cannot authorize review, decision, notification, or release. They remain
   visible as unresolved local document/routing state until an operator selects
   real eligible people after a later nonempty refresh. No automatic name/email
   matching or historical rewrite is allowed. Complete the DOX pass for changed
   core/desktop contracts.
2. Update the CAP-0002, CAP-0015, and CAP-0017 generator definitions with
   synthetic data. The selection pane must separate **Document profile**,
   **Current release**, and **Review schedule**; the candidate form owns the
   effective-date input; **Owner** is a required select over eligible people with
   option value `object_id` and visible synthetic display name/email; unresolved
   legacy/current references are explicit and not selectable aliases. Include a
   no-eligible-people state showing literal `<owner>` and `<editor>` placeholders
   (including **Requesting editor**), plus the later real-person state that lets
   an operator stage Owner and Editor replacement for the next successful
   release. The release list displays each release's captured date and profile
   snapshot.
   Regenerate HTML/index/manifest and render only the affected PNGs through the
   documented wireframe procedure.
3. In `dms-core`, make `DocumentControl` the mutable profile only. Persist Owner
   as the current Entra group-binding ID plus object ID using the same immutable
   reference semantics as editor and approver; keep display name/email out of
   identity equality. Model a placeholder as an explicit non-Entra state rather
   than a synthetic UUID, and surface its literal label only for an empty
   successful direct-member response. Add owner resolution for presentation and
   require a resolved eligible owner before candidate submission or release.
   When a nonempty refresh follows placeholder use, let a candidate carry an
   explicit real Owner reference and Editor reference to be applied atomically
   only after export succeeds; do not mutate the current profile or workflow
   override merely by opening a form or submitting a failed candidate. Add a
   required effective date and owner `PersonSnapshot` to candidate metadata,
   persist the accepted profile snapshot and an optional legacy-aware effective
   date on `ReleaseRecord`, and make all new candidate/release paths populate
   them. Compare candidate staleness by profile fields and object-ID references,
   so a cache-only name/email refresh neither changes authority nor invalidates
   an open candidate. Remove effective date from `ControlUpdate`; preserve
   before/after profile-change evidence and candidate invalidation when the owner
   object ID actually changes.
4. Raise the metadata schema to v12 with a retained v11 backup and a focused
   migration fixture. For each v11 document, move a stored effective date to its
   current non-withdrawn release when available; keep a release's effective date
   unset when no authoritative v11 value exists. Move each non-empty text owner
   to `legacy_owner_label`, leave the owner reference unset, and never infer an
   object ID from that text, display name, or email. Apply the same lossless
   conversion to retained candidate snapshots, then preserve existing schedule
   values, release bytes, and evidence and validate before the atomic save. New
   releases must have a date and resolved owner; legacy records render an
   explicit unrecorded date and unresolved legacy owner.
5. Thread the new candidate input and release snapshot through the explicit CLI
   and desktop adapter boundaries. Replace CLI `--owner <TEXT>` with explicit
   `--owner-object-id <UUID>` without an alias; refresh eligible people before a
   desktop owner assignment and fail closed if the selected ID is no longer an
   enabled direct group member. Expose resolved current owner plus immutable
   current-release effective date/profile data in `DocumentSelection` and
   release-maintenance rows, while keeping domain validation in `dms-core`. In
   the narrow release path, apply a staged real Owner to document control data
   and a staged real Editor as a document-level workflow override only inside the
   successful release transaction; preserve folder policies, source filenames,
   Office properties, and Markdown front matter. The Requesting editor control
   shows `<editor>` only in the empty-successful-import state and becomes a real
   object-ID selector after a nonempty refresh.
   Update audit release rows to use the stored release profile and effective date
   when present; label pre-v12 missing snapshot data as unrecorded rather than
   falling back silently to mutable profile or current cache values.
6. Replace the Library effective-date field in **Edit document control data**
   with profile-only inputs and replace the Owner text box with a required
   **Eligible people** select. Display `name · email` for usability but submit
   only `object_id`; show unresolved ID/legacy-label state explicitly and require
   reselection. When Graph successfully returns zero eligible users, render the
   non-editable literal `<owner>` and `<editor>` placeholders and explain that a
   real Graph person is required before a release can be submitted; never show a
   fabricated UUID picker option. Once a later refresh contains real users,
   present explicit real Owner and Editor selectors for a document still using
   placeholders and label them **Apply with successful release**. Add the date
   to **Submit release candidate**, make **Next minor**
   its selected Target version default (while retaining explicit major and
   manual options), keep candidate inputs immutable while in review or approved,
   and show captured owner/date in current-release/release-history views beside
   the separately calculated next review due date. Add frontend tests for owner
   options/request shape, mutable display-data changes, required calendar dates,
   the `next_minor` default, no profile date field, current-release rendering,
   legacy unrecorded rendering, and XSS-safe values.
7. Add focused core migration, lifecycle, maintenance, audit, CLI, adapter, and
   frontend tests. Cover: assignment rejects IDs absent after refresh; owner,
   editor, and approver authority survives name/email changes because comparisons
   use object IDs; identity-source replacement makes live references unresolved;
   legacy owner text is preserved but never mapped; profile changes retain event
   evidence; a new candidate snapshots profile/owner/date; later object-ID or
   profile edits invalidate it without changing its snapshot; cache-only display
   edits do not; release stores the approved snapshot; due dates derive from the
   stored effective date; and post-release changes cannot alter historic
   release/audit labels. Also cover a successful empty Graph list producing only
   the literal placeholders, Graph/tenant/group failures never producing them,
   blocked candidate/release transitions while placeholders remain, a later
   real-person refresh exposing only object-ID options, and an atomic
   successful-release handover that updates the current owner/document editor
   while retaining the placeholder release and all earlier event snapshots
   unchanged. Run the phase gate and record exact evidence here.

**Verification gate:**
`cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/library.test.mjs` exits 0; the v11→v12 fixture retains the migration backup, maps only authoritative effective dates, preserves legacy owner text without assigning an object ID, preserves existing due dates and chain verification, and invents no release snapshot; a successful empty Graph response displays only literal `<owner>` / `<editor>` placeholders while failed Graph states fail closed; focused policy/lifecycle/maintenance/audit/CLI tests prove owner/editor/approver authority is object-ID based across name/email changes, a real-person handover applies only in the successful release transaction without rewriting historic evidence, new releases snapshot profile/owner/effective date, and review schedules start from that date; the owner form submits only an eligible object ID and the candidate form selects `next_minor` by default; `node generate.mjs` exits 0 in `docs/product/wireframes/`; the three regenerated HTML/PNG pairs remain in `manifest.json`; CAP/ADR/privacy/architecture links resolve; and `git diff --check` exits 0.

## Out of scope — Phase 9k.3

- Creating a future-effective lifecycle state, delayed publication, or calendar
  reminders while the application is closed.
- Retrospectively deriving effective dates or release profiles from PDF content,
  filenames, timestamps, or current mutable metadata.
- Matching a legacy owner label to a Microsoft Entra person, using display name
  or email as an identity key, or adding application-managed person/group CRUD.
- Treating a Graph error, disabled-only directory response, missing binding, or
  inaccessible group as an empty successful import; fabricating an Entra UUID;
  allowing a placeholder to approve, receive workflow mail, or release a
  document; or rewriting earlier candidates, releases, or workflow events after
  real people become available.
- Writing Owner or Editor values into source directory names, source filenames,
  Office properties, or Markdown front matter. The release-time handover updates
  DMS control/routing metadata and its immutable release snapshot only.
- Changing confidentiality inheritance or PDF export chrome.
- Windows-host Office, Entra, or notification smoke evidence; those remain Phase
  9l gates after this model is implemented and verified locally.

## Phase 9k.4 — Library filtering, recursive folder counters, interaction, width, and shipped icons

**Goal:** Give every Library folder one core-computed recursive `~`/`+`/`!`
summary rendered beside its name in both the tree and list views, add three
session-wide file-visibility toggles whose result set drives table pagination
without changing those summaries, make the candidate form start at **Next
minor**, open a folder immediately on row click, support bounded detail-pane
resizing, and use app-local SVG navigation assets.

**Plan ID:** `CHG-0001#phase-9k.4`
**External request:** Direct operator request: “Set also for Target version to
default of Next minor; make the directory view resizeable in width; open a
directory immediately when it is selected in the table; assess a built-in Nerd
Font for document/file, directory, and navigation icons.”
**Follow-up request:** Direct operator request: show beside each folder name
`~<number of documents as draft>`, `+<applicable files missing from the
library>`, and `!<files not applicable because of wrong format>`, and reuse the
same numbering for that folder in the list view.
**Follow-up request:** Direct operator request: add three filters above the file
directory view as on/off controls for draft files, unsupported files, and files
not in the library; apply them to all folders at once, keep the counters
independent, and make pagination reflect the filters. Runtime labels are **Draft
documents**, **Available to add**, and **Unsupported files**.
**Execution slot:** P0180 (phase-local; the active CHG keeps its immutable
`CHG-0001-…` filename under the change-record lifecycle)
**Created:** 2026-08-14
**Depends on:** `CHG-0001#phase-9k.3`
**Plan family:** `CHG-0001-tauri-local-dms-bootstrap`
**Entry checkpoint:** Phase 9k.3 marked `done (<gate evidence>)` in this CHG;
the checkout is clean apart from planned CHG updates.
**Atomicity rationale:** Recursive folder-summary semantics, global file
visibility, filter-before-pagination ordering, click contract, pane geometry,
icons, accessible labels, wireframes, and focused tests together define one
Library interaction. Splitting them would leave the core snapshot, product
contract, generated reference, and runtime disagreeing about a folder's state,
which rows or pages are visible, how it opens, or where its document details fit.
This CHG exceeds the normal 40 KiB warning because it is the single active
historical progress authority; a fresh session need load only this section, its
listed sources, and Phase 9k.3 completion evidence.
**Context sources:** this phase and the phase table; CAP-0002 outcome 4;
`CAP-0005-desktop-shell.md` (**Open activities**); CAP-0006 outcome 2 (folder-navigation and
three-pane rules), outcomes 5–6 (table/selection-pane rules), and non-goals;
CAP-0015 outcomes 2 and 11; `docs/product/wireframes/AGENTS.md`;
`docs/product/wireframes/generate.mjs` (CAP-0002, CAP-0006, CAP-0015);
`crates/AGENTS.md`; `crates/dms-core/AGENTS.md`;
`crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/library.rs`
(`LibraryFolderNode`, `LibraryEntry`, `library_tree`, `library_folder`);
`crates/dms-core/tests/library.rs`;
`crates/dms-desktop/ui/styles.css` (Library pane layout);
`crates/dms-desktop/ui/library.mjs` (`createLibraryState`, tree, row filtering,
sorting, pagination, candidate markup); `crates/dms-desktop/ui/app.mjs`
(`handleClick`, `handleChange`, `handleKeyDown`, Library navigation); and
`crates/dms-desktop/ui/app.test.mjs` /
`library.test.mjs`. The Mona Sans repository is a variable text-font source
under SIL OFL 1.1, not a Nerd Font or a file/navigation icon set.
**Produces:** CAP-0002/CAP-0006/CAP-0015 contracts and regenerated wireframes
for the revised interaction; recursive folder summaries shared by tree and list
rows; session-wide file-visibility toggles with filter-before-pagination
semantics; bounded session-only centre/details resizing; single-click folder
navigation; accessible app-local SVG icons; and tested candidate default/Library
navigation without a font dependency.
**Status:** pending — queued after the release-bound data-model vertical slice.

### Current state

- The candidate `<select>` renders **Next major** as its first selected option,
  although the request builder already accepts `next_minor`, `next_major`, and
  `manual` (`crates/dms-desktop/ui/library.mjs:208-232`, `441-466`).
- The folder tree uses native horizontal resize, but the centre directory view
  grows with flex while the document-details pane has a fixed 315px width;
  there is no centre/details divider to give control data more width
  (`crates/dms-desktop/ui/styles.css:228-276`).
- A table folder row currently selects first; only double-click or Enter opens
  it (`crates/dms-desktop/ui/app.mjs:1202-1215`, `2131-2142`).
- `LibraryFolderNode` carries only name/path, and folder `LibraryEntry` carries
  neither document membership nor a subtree summary; the tree and row renderers
  therefore have no shared count source (`crates/dms-core/src/library.rs:46-70`,
  `73-150`, `crates/dms-desktop/ui/library.mjs:378-405`).
- Core already classifies each visible file as `InLibrary`, `NotInLibrary`, or
  `Unsupported` and hides `.dms` plus Office `~$` files, so the counters can use
  the existing classification rather than a second frontend format heuristic
  (`crates/dms-core/src/library.rs:93-150`, `372-387`).
- Library state has sort and page controls but no category-visibility state;
  `rowsMarkup` currently sorts every folder/search entry and passes the complete
  result directly to `paginateLibraryEntries` (`crates/dms-desktop/ui/library.mjs:6-25`,
  `340-410`). CAP-0006 outcome 13 explicitly rules out the filter control now
  requested (`docs/product/capabilities/CAP-0006-library-explorer.md:161-163`).
- Pagination already supports 10/25/50/100 rows, clamps an out-of-range page,
  and resets to page zero after sort or page-size changes; the new visibility
  operation must join that same ordering before pagination rather than hide rows
  after a page is sliced (`crates/dms-desktop/ui/library.mjs:349-359`,
  `crates/dms-desktop/ui/app.mjs:2117-2127`).
- Runtime Library icons are Unicode stand-ins (`▰`, `▸`, `□`) rather than
  shipped application assets (`crates/dms-desktop/ui/library.mjs:378-405`).
- Mona Sans describes itself as a GitHub variable font and ships font files; it
  is not a Nerd Font-patched icon glyph collection. Shipping it would not solve
  the icon requirement and would add a font-loading surface.

**Steps:**

1. Update CAP-0002 outcome 4 to specify that a later-release candidate defaults
   to **Next minor**, while first-release `V1.0`, major, and manual choices keep
   their existing rules. Update CAP-0006 so a centre-table folder row opens on
   primary click (and Enter for keyboard users) rather than being selectable;
   controlled and uncontrolled file rows retain selection and multi-select
   semantics. Define one recursive summary for every root/child folder: `~N`
   counts visible controlled files whose lifecycle is exactly `draft`; `+N`
   counts visible supported Office/Markdown source files not under control; `!N`
   counts visible unsupported regular files. Descendant files contribute to
   every ancestor folder, a file contributes to at most one counter, zero
   counters are omitted, and `.dms`, Office `~$` files, directories, and
   controlled non-draft files do not contribute. State that the same summary is
   shown immediately to the right of the folder name in tree and list views.
   Replace CAP-0006 outcome 13's blanket no-filter rule with three independent
   session-wide file-visibility controls above the directory table, all enabled
   by default: **Draft documents** shows/hides controlled rows whose lifecycle is
   exactly `draft`; **Available to add** shows/hides supported source files with
   `NotInLibrary` membership; **Unsupported files** shows/hides unsupported
   regular files. Folder rows and controlled non-draft rows remain visible.
   Toggle state follows the Library activity across every folder and active
   search result, is never stored per folder, and never changes the recursive
   counters. Filtering precedes sorting and pagination; page count, visible
   total, and page contents derive from the filtered rows, and a toggle change
   returns to page zero. This is a file-visibility aid, not CAP-0012 metadata
   reporting. Also state that the centre/details divider is resizable within
   explicit bounds for the active session, while independently scrolling panes
   and the fixed toolbar remain unchanged. Update CAP-0015 only for details-pane
   width/independent-scroll presentation; no metadata field rule changes here.
2. Amend the CAP-0002, CAP-0006, and CAP-0015 screen definitions. Show **Next
   minor** selected, one visible centre/details resize handle with its accessible
   label, an enlarged Document control data state, and direct folder navigation
   on table activation. In CAP-0006, add an accessible **Show in folder** toggle
   group immediately above the table with visibly active **Draft documents**,
   **Available to add**, and **Unsupported files** controls; keep search/sort in
   the toolbar and pagination below the table. Render nonzero `~`, `+`, and `!`
   badges in that order beside folder names in both the tree and current-folder
   list; give each symbol/count an accessible text equivalent and make the two
   occurrences of a folder use identical unfiltered values. Use only synthetic
   content. Regenerate HTML/index/manifest and render the three PNGs through the
   documented procedure; inspect the native-size CAP-0006 render for control
   discoverability, active-state legibility, hierarchy, wrapping, and clipping.
3. Make `next_minor` selected in the candidate form without changing the core
   target-mode validation, candidate version calculation, or explicit manual/
   major choices. Keep the first release's existing version behaviour; the UI
   default is not a server-side override.
4. Add a serializable `FolderCounters` value in `dms-core` and compute it from
   the same filtered filesystem inventory and registered-document index used by
   Library membership. Attach the recursive value to every `LibraryFolderNode`
   and folder `LibraryEntry`; file entries have no folder counters. Compute each
   folder once per snapshot, with root including the complete visible edit tree,
   and expose the same values unchanged through CLI/Tauri snapshots. Do not
   persist or frontend-recalculate counters. Add core tests with nested folders,
   draft and non-draft controlled files, supported unregistered files,
   unsupported files, `.dms`, and `~$` sidecars to prove exact ancestor totals
   and mutually exclusive buckets.
5. Add all-on `show_draft_documents`, `show_available_to_add`, and
   `show_unsupported_files` state to `createLibraryState`. Add one pure
   `filterLibraryEntries` classifier using core membership and lifecycle values:
   folders always pass; controlled draft, not-in-library, and unsupported file
   rows obey their corresponding flags; controlled non-draft rows and any
   unclassified rows remain visible rather than being silently hidden. Apply
   that result to both current-folder entries and search results
   before `sortLibraryEntries` and `paginateLibraryEntries`, so the pagination
   total and page count describe rows that can actually be shown. Render three
   `aria-pressed` toggle buttons in a labelled group above the table, retain their
   state across folder navigation/search within the Library activity, and reset
   page to zero when any toggle changes. Prune selections hidden by the new state
   and clear a now-hidden detail pane; do not leave invisible rows actionable.
   Show a filter-specific empty state when filesystem entries exist but none pass.
6. Replace the centre/details fixed-width relationship with an explicit,
   session-only splitter in `library.mjs`, `app.mjs`, and `styles.css`. Pointer
   drag and keyboard adjustment use one bounded detail width (minimum 280px,
   maximum 640px while leaving the centre at least 360px); pressing Escape during
   a drag restores its starting width. Do not write this width to `.dms`, saved
   views, or OS preferences. Preserve the existing independently scrolling
   folder tree, table, and detail pane.
7. On a primary click of a folder table row, call `loadLibraryFolder` directly,
   record ordinary Back/Forward history, and do not first select it or invoke
   document-detail loading. Retain Enter activation for focused folder rows,
   remove the obsolete double-click-only route, and leave modifier-key file
   multi-select unaffected.
8. Add a small app-local inline-SVG icon helper for the folder, file, chevron,
   Back, Forward, Up, and Refresh affordances. Icons are decorative where a
   visible label exists and have an accessible name where a control is icon-only.
   Do not add `@font-face`, WOFF/TTF assets, a Nerd Font, Mona Sans, a remote
   dependency, or MIME-specific file-type artwork. The visual goal is consistent
   DMS navigation/iconography, not a product-wide typeface change.
9. Render counters beside folder names from the supplied core values only,
   suppress zero badges, retain the `~`, `+`, `!` visual order, and provide
   explicit accessible labels such as “3 draft documents”. Add focused frontend
   tests proving tree/list equality, zero suppression, escaping, and exact label
   text. Cover all filter combinations, all-on defaults, state retained across
   folder/search navigation, unchanged counter markup, folder/non-draft
   visibility, filter-before-sort-and-pagination totals, page-zero reset,
   hidden-selection pruning, the filter-specific empty state, and safe label
   escaping. Also cover the selected `next_minor` option/request shape, direct
   click/Enter folder navigation with no transient selection/detail load,
   bounded pointer/keyboard splitter behaviour and Escape rollback, session
   reset, SVG markup/labels, and file multi-select preservation. Run the phase
   gate, inspect generated HTML/PNGs, record evidence, and leave Phase 9k.5 as
   the next pending phase.

**Verification gate:**
`cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs` exits 0; focused core/UI tests prove exact recursive and mutually exclusive folder totals, identical nonzero `~`/`+`/`!` values in tree and list views, and accessible labels; all three visibility controls default on, remain Library-session-wide across folders/search, preserve unfiltered counters, prune hidden selections, and filter before sorting/pagination so page totals and contents match visible rows; tests also prove `next_minor` is the selected candidate default, table-folder primary click/Enter navigates immediately without selection/detail fetch, the splitter enforces its bounds and Escape rollback, and no font asset or `@font-face` is introduced; `node generate.mjs` exits 0 in `docs/product/wireframes/`; CAP-0002/CAP-0006/CAP-0015 HTML, PNG, and manifest entries agree; the CAP-0006 native-size render shows the filter group above the table without clipping; the DOX pass is complete; and `git diff --check` exits 0.

## Out of scope — Phase 9k.4

- Persisting pane widths across sessions, devices, saved views, or workspaces.
- Persisting file-visibility toggles in `.dms`, OS preferences, saved views, or
  per-folder state; they are all-on defaults scoped to the active Library session.
- Changing the folder-tree resize behaviour, creating a four-pane Library, or
  weakening the 360px minimum directory-table width.
- Persisting or caching folder counters in `.dms`, counting missing registered
  sources that have no visible file, or treating unsupported formats as
  applicable DMS source types.
- Per-extension/MIME artwork, a new general design-system icon package, or a
  product-wide typography change.
- Installing a host font or bundling Mona Sans/Nerd Font binaries solely for
  icons; application UI must be self-contained through the local SVG helper.

## Phase 9k.5 — Workflow tree and SMTP identity/test configuration

**Goal:** Replace Workflow's flat folder list with a marked role-routing tree,
and make saved SMTP configuration distinguish its login user from a formatted
`From` mailbox while exposing only `***` for a stored password and a deliberate
test message to that mailbox.

**Plan ID:** `CHG-0001#phase-9k.5`
**External request:** Direct operator request: “(1) the
Configuration→Wofkflow→Choose default or exception is not a tree view but only
a list. Create a tree view where all elements where a role assigment is applied
are visible and marked that there are roles assigned (2) In Review and release
email place *** if a password is set and the configuration is applied. Offer a
email send test button to the sender (3) There should be a difference between
the login user and the From address the email is sent including a formating like
\"Doc Mgmt\" <name.surname@domain.local>.”
**Execution slot:** P0190 (phase-local; the active CHG keeps its immutable
`CHG-0001-…` filename under the change-record lifecycle)
**Created:** 2026-08-14
**Depends on:** `CHG-0001#phase-9k.4`
**Plan family:** `CHG-0001-tauri-local-dms-bootstrap`
**Entry checkpoint:** Phase 9k.4 marked `done (<gate evidence>)` in this CHG;
the checkout is clean apart from planned CHG updates.
**Atomicity rationale:** The role-tree hierarchy, direct-assignment markers,
SMTP schema migration, credential indicator, delivery adapter, and their two
CAP wireframes are one Configuration contract. Splitting them would leave a
persisted transport model or a primary operator surface inconsistent with the
behaviour it claims.
**Context sources:** this phase and the phase table; CAP-0010 outcomes 1–9 and
non-goals; CAP-0019 outcomes 2–4 and 11; CAP-0005 outcome 10;
ADR-0024; `docs/product/wireframes/AGENTS.md` and
`docs/product/wireframes/generate.mjs` (CAP-0010 and CAP-0019);
`crates/AGENTS.md`; `crates/dms-core/AGENTS.md`;
`crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/lib.rs` (schema migration);
`crates/dms-core/src/lifecycle.rs` (`SmtpSettings` and notification validation);
`crates/dms-desktop/src/notify.rs` (credential store and SMTP adapter);
`crates/dms-desktop/src/lib.rs` (configuration snapshot and Tauri commands);
`crates/dms-desktop/ui/configuration.mjs`, `app.mjs`, `styles.css`, and their
focused Rust/frontend tests.
**Produces:** A keyboard-accessible Workflow role tree with visible direct-role
markers; schema-v13 SMTP settings with separately persisted non-secret login
user and RFC 5322 `From` mailbox; a non-secret `***` credential indicator; and
a fake-backed, explicitly targeted SMTP test-delivery command and UI.
**Status:** pending — queued after the Library interaction vertical slice.

### Current state

- `workflowMarkup` reuses `folderTreeMarkup`, which maps every policy folder
  into one depth-indented button rather than a parent/child tree, and shows no
  direct editor/approver assignment marker (`crates/dms-desktop/ui/configuration.mjs:101-110`,
  `196-209`).
- The configuration snapshot already supplies all existing policy folders and
  direct workflow policies, so the frontend can construct the tree without a
  new directory or identity query (`crates/dms-desktop/src/lib.rs:143-155`,
  `1849-1877`).
- Persisted `SmtpSettings.sender` conflates the SMTP authentication user and
  message sender (`crates/dms-core/src/lifecycle.rs:148-160`); the desktop
  adapter uses it for both `From` and `Credentials::new` (`crates/dms-desktop/src/notify.rs:155-179`).
- Notifications currently render a single email-only Sender input and a blank
  password input; a stored credential appears only as prose, not `***`
  (`crates/dms-desktop/ui/configuration.mjs:234-239`).
- The app already keeps the app password in the workspace-scoped OS credential
  store and exposes only a boolean configuration status in the snapshot
  (`crates/dms-desktop/src/notify.rs:15-77`, `crates/dms-desktop/src/lib.rs:1853-1877`).
- The current store is schema v11. This phase starts only after Phase 9k.3 has
  produced schema v12, so it must migrate v12 settings to schema v13 rather than
  silently accepting an unknown persisted field shape (`crates/dms-core/src/lib.rs:31`,
  `400-463`).

**Steps:**

1. Amend CAP-0019: **Choose default or exception** is a semantic nested folder
   tree, not an indented list or exception table. It includes the edit root and
   every existing eligible edit-root directory, hides `.dms`, exposes only one
   selected policy target, and visibly marks each directory with a direct Editor
   assignment, direct Approver assignment, or both. A folder with neither direct
   assignment remains visible without a marker; inherited values remain in the
   selected-folder editor. Remove the competing folder-exceptions-table outcome.
2. Amend CAP-0010 and ADR-0024 where necessary: SMTP saves non-secret relay
   host/port, `login_user`, and `from_mailbox` separately. `login_user` is used
   only for SMTP authentication; `from_mailbox` is parsed as an RFC 5322 mailbox
   and may use a display name such as `\"Doc Mgmt\" <name.surname@domain.local>`.
   The app password stays write-only in OS credential storage. When SMTP is
   saved and that credential exists, Notifications shows the literal `***` as
   its configured-password indicator while the empty password input still means
   retain. A saved SMTP configuration exposes **Send test email to
   <from-mailbox>**; it sends no document content, workflow action, audit event,
   or arbitrary recipient, and `mailto:` exposes no SMTP test control.
3. Update CAP-0010 and CAP-0019 screen definitions in `generate.mjs`. The
   workflow screen shows a true expandable tree, direct Editor/Approver badges,
   a selected inherited folder, and an unresolved direct assignment. The
   notification screen shows distinct **SMTP login user** and **From address**
   fields, a synthetic `***` configured-password indicator, and the exact test
   target. Regenerate HTML/index/manifest and render both PNGs; do not put real
   mailbox or credential data in a wireframe.
4. In `configuration.mjs`, create a Workflow-only tree model from
   `policy_folders` and `workflow_policies`; retain the Document-defaults
   picker unchanged. Keep the root and every direct-assignment ancestor expanded
   on initial render so every configured role location is visible. Add explicit
   branch toggles with `role=tree`, `treeitem`, `aria-level`, and `aria-expanded`;
   retain the current folder selection and keyboard activation. Render distinct
   direct Editor and Approver markers from the policy's actual fields, never from
   inherited effective values. Extend session-only configuration state and the
   `app.mjs` click/key path for branch toggles without adding an OS preference.
5. Bump `dms-core` to schema v13 and replace `SmtpSettings.sender` with
   `login_user` and `from_mailbox`. Migrate a v12 `sender` into both fields,
   remove the legacy key before strict deserialization, validate the migrated
   workspace, retain the v12 backup, and atomically write only after validation.
   Validate nonblank host/login and a valid `Mailbox` for `from_mailbox`; retain
   the existing `mailto:` clearing behaviour and never persist the password.
6. Change the desktop configuration command, snapshot, form request, and
   notification adapter to use the split fields. The UI uses a normal username
   control for login and a text `From address` control so a display-name mailbox
   remains valid. Construct message `From` from `from_mailbox` and SMTP
   `Credentials` from `login_user`; failures name the invalid field but never
   return the password.
7. Add a narrow `test_smtp_notification` Tauri command that reads the already
   saved workspace settings and credential, refuses `mailto`, missing
   configuration, invalid mailbox, or absent credential, and sends one fixed
   plain-text configuration test to the parsed address in `from_mailbox`.
   Reuse the SMTP transport construction but return only a sanitized success or
   failure plus optional numeric SMTP response code. The UI button is disabled
   until the saved snapshot is SMTP with a configured credential, names its
   recipient, and displays the result in the existing Configuration notice/error
   area. It does not submit unsaved form values.
8. Add focused migration, configuration-command, and notification-adapter tests
   for legacy `sender` preservation, separate auth/From use, display-name
   mailbox formatting, blank-password retention, `***` rendering, unavailable
   test button states, and fake-backed test delivery to the `From` address.
   Add frontend tree tests for nested paths, all direct-role badges, keyboard
   branch control, and a selected unassigned/inherited folder. Run the gate,
   inspect regenerated artifacts, record evidence, then move Phase 9l to
   `in-progress` only after this phase is `done`.

**Verification gate:**
`cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/app.test.mjs` exits 0; focused tests prove v12→v13 migration retains a backup and maps legacy sender to both fields, SMTP authentication uses only `login_user`, `From` accepts a formatted mailbox, no password leaves the credential store or is rendered beyond literal `***`, and fake test delivery goes only to the saved `From` mailbox; `node generate.mjs` exits 0 in `docs/product/wireframes/`; CAP-0010/CAP-0019 HTML, PNG, and manifest entries agree; and `git diff --check` exits 0.

## Out of scope — Phase 9k.5

- Separate credentials per SMTP login, OAuth SMTP, an alternate envelope sender,
  arbitrary test recipients, attachments, or testing the host `mailto:` handler.
- A role-assignment inventory table, role inheritance redesign, user/group CRUD,
  or any workflow-policy target outside the existing edit-root folder tree.
- Recording configuration test mail as document workflow/audit evidence or using
  it to advance, approve, release, or retry a document lifecycle transition.

## Implementation notes

- Prefer a minimal frontend unless a UI kit is required for the library
  directory navigator.
- `crates/dms-core` owns Tauri-independent domain behaviour. `crates/dms-cli`
  exposes only implemented core operations and has no Tauri, WebView, Office,
  Entra, or mail dependency. `crates/dms-desktop` calls the same core through
  narrow Tauri commands.
- The Cargo workspace MSRV must build the complete pinned desktop dependency
  graph, not only meet the top-level Tauri and tauri-build crate declarations.
- OS user app config holds sidebar preference and saved-view targets. Open
  activity tabs are session-only; neither belongs in `.dms` workflow evidence.
- OS user app config also holds at most ten most-recently-opened edit roots.
  Removing a history entry does not remove or modify the workspace.
- A document activity key is workspace ID + task + stable document ID; its label
  is task + current DMS title + optional document number. Folder activity labels
  use edit-root-relative paths; the Library updates its one session pane in
  place while navigating folders.
- `.dms` format: inspectable JSON (or similar); schema beside the store module
  in phase 2 — must include `edit_root`, `publish_root`, stable workspace ID,
  library entries with
  relative paths and stable IDs, confidentiality and document-type catalogues,
  Microsoft Entra group binding plus a read-only display cache (the global
  client/tenant configuration is outside `.dms`),
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
  indicators on WebView2 and WKWebView. The installed-Office adapter exists but
  is not yet constructed by a production desktop lifecycle command; phase 9i
  owns that wiring and its licensed-host evidence.

## Phase 9 audit findings

- [Windows/macOS run 31385996931](https://github.com/videoclinic/dms/actions/runs/31385996931)
  produced `windows-x64-nsis` and `darwin-aarch64-dmg` workflow artifacts and
  passed the workspace, launch, and native Markdown PDF smoke gates.
- CAP-0003, CAP-0012, and the existing CAP-0022 have complete executable
  evidence. CAP-0021 now records the verified live identity-source subset. The
  remaining capability outcomes retain their bounded statuses until their
  operator surfaces or transitions land, including full lifecycle composition,
  complete corruption recovery and backup-history surfaces, and external
  Entra/Office smoke evidence.
- The Windows Office and administrator-configured Microsoft 365 Entra smokes
  require a licensed Windows host application, a configured tenant/group, and an
  interactive operator identity. CI fakes and unit tests do not satisfy those
  external gates. The CI macOS package and native Markdown PDF smoke remain the
  current macOS evidence; external macOS Office observation is deferred.

## Phase 9l — Windows external operator smokes + CAP promotion

**Goal:** On a suitable Windows host, prove the configured Entra,
SMTP-or-host-mail, and Office release paths end to end, then update CAP status
only for evidence that host establishes. macOS remains supported through existing
CI coverage; this phase does not assert external macOS Office evidence.

**Plan ID:** `CHG-0001#phase-9l`
**Execution slot:** P0200 (phase-local; the active CHG keeps its immutable
`CHG-0001-…` filename under the change-record lifecycle)
**Created:** 2026-08-13
**Depends on:** `CHG-0001#phase-9k.5`
**Plan family:** `CHG-0001-tauri-local-dms-bootstrap`
**Status:** pending — awaits Phase 9k.5 and then a suitable Windows host meeting the entry pre-checks

### Phase 9l fresh-session context

- **Entry checkpoint:** Phase 9k.5 marked `done (<gate evidence>)` in this CHG.
- **Context sources:** this Phase 9l section, the phase table, and Phase 9k.5's
  completion evidence; CAP-0001, CAP-0002, CAP-0006, CAP-0010, CAP-0015,
  CAP-0017, CAP-0019, CAP-0021; `docs/architecture.md`; `docs/privacy.md`; and
  the Windows/macOS desktop-smoke workflow for retained CI evidence. Earlier
  phase bodies are not fresh-session prerequisites.
- **Produces:** non-secret Windows operator-smoke evidence, current CAP outcomes
  where that evidence warrants them, and an archived CHG only after all active
  change gates pass. No CAP outcome may claim unobserved macOS-installed-Office
  behaviour.

**Steps:**

1. Run every Phase 9l entry pre-check before changing this phase to
   `in-progress`; leave it pending if the Windows host, licensed Office, or
   controlled identity/notification access is unavailable.
2. On the Windows host, run the controlled Entra identity, major-review, decision,
   notification, and Office/Markdown release scenarios below. Preserve only
   non-secret paths, checksums, test output, and workflow evidence.
3. Run the repository record/link/table and workspace gates on the checkout that
   supplies the observed build. Update CAPs only with present-tense behaviour
   proven by the evidence, then record exact gates before closing the CHG.

**Verification gate:**
All five entry pre-checks pass; Windows produces non-secret evidence for the
configured identity, notification, and release flows; required Rust/frontend/
records/link gates exit 0; CAP statuses make no untested macOS-installed-Office
claim; and CHG closure matches the recorded evidence.

### Phase 9l operator smoke instructions

1. In Configuration, enter the global public-client/tenant configuration, apply
   the library group binding, refresh its eligible people, and configure either
   SMTP with its OS-stored credential or `mailto:` delivery.
2. Add or open a controlled draft, select its Owner and requesting editor from
   eligible people, submit a major candidate, and confirm that a `mailto:` draft
   does not advance review until the operator confirms it was sent.
3. Use the Library review action to complete interactive approver sign-in, record
   a decision as an eligible approver, and confirm any `mailto:` decision notice.
4. Release the approved candidate on the licensed Windows Office host for `.docx`
   and release a Markdown candidate through the native WebView print path; verify
   the versioned PDFs and workflow evidence. Exercise a direct minor release and
   confirm its publication notice the same way.
5. Use the evidence panel to verify each delivery attempt and retry control. The
   resulting configured Entra, delivery, and licensed-Office observations are
   Phase 9l acceptance evidence; never place tenant credentials in the record.

### Phase 9l entry pre-checks

Run and record these checks before changing Phase 9l from pending to in-progress.
If any check fails, leave Phase 9l pending; do not promote a CAP or run a partial
external smoke as phase evidence.

1. A Windows host is available outside WSL with the Phase 9k.1 checkpoint or a
   later `main` checkout and an activated, licensed Office installation that can
   export a controlled `.docx` to PDF.
2. The Windows host has a disposable DMS workspace with explicit edit and publish roots,
   a controlled `.docx` draft, and permission to inspect and remove its generated
   PDF evidence.
3. An Entra administrator can apply the intended tenant/group binding, and two
   direct enabled group members can complete delegated sign-in: one requester and
   one eligible approver. The interactive actor must be able to record a decision.
4. The chosen notification transport is usable end-to-end: either an approved
   SMTP relay with its credential already in OS credential storage, or a registered
   host mail handler that can compose and manually confirm delivery. Use only
   controlled test recipients.
5. The operator can retain non-secret Windows-host evidence: exported PDF path
   and checksum, workflow-history delivery outcomes, Entra actor verification, and
   command/test output. Never record tokens, relay credentials, or tenant secrets.
- Phase 9 was expected to be promotion-only, but the CAP audit found unresolved
  runtime and operator prerequisites across distinct subsystems. Phases 9a–9j
  make those prerequisites independently gated instead of promoting partial
  capabilities or treating one packaging run as product completion.
- Phase 9a closes desktop workspace setup without promoting CAP-0001: the
  remaining live Entra/configuration and release-path outcomes still depend on
  phases 9f–9j. CAP-0001 now links the partial executable setup evidence.
- Phase 9c closes CAP-0012 with core-owned deterministic report semantics,
  schema-v7 report evidence, CLI/desktop adapters, and the Audit & Reports
  operator surface. External installed-Office and Entra smokes are unrelated to
  this local export capability.
- CI exercises the real native WebView Markdown PDF path, desktop startup, and
  NSIS/DMG packaging. It does not exercise installed Office, a configured
  Microsoft Graph tenant/group, interactive Entra sign-in, SMTP, or host-mail
  delivery; production code has live delegated Graph and notification adapters,
  while external service and installed-Office smokes remain phase 9l gates.
- CAP-0005 and CAP-0006 have substantial partial executable evidence but remain
  `not implemented`: host-mediated source/release opening is present, while the
  full Configuration and lifecycle IPC surfaces remain absent. Their evidence
  fields must describe the proven subset without promoting the whole contract.
- Phase 9b closes the local periodic-review result, cancellation, and reminder
  paths. Core evidence refreshes current approver eligibility, preserves the
  due date on cancellation, and records every reminder delivery attempt without
  duplicating the request or changing lifecycle state. CLI/Tauri/frontend
  commands require explicit confirmation; live Entra and notification delivery
  remain phase 9i prerequisites, so CAP-0017 is not promoted yet.
- Phase 9b.1 corrects two phase-3/shell gaps reported during phase-9 use. The
  Library now renders nested branches with independent expand/collapse controls
  and keeps current-folder ancestors expanded. Navigation clears only a narrow
  flyout; it no longer folds an already-unfolded left menu. Focused and full
  frontend gates, workspace Rust gates, and browser visual QA pass.
- Phase 9d adds schema-v8 lock-staleness configuration, atomic advisory-lock
  acquisition and owner-matched desktop release, and manifest-verified restore
  into confirmed replacement roots. Restore validates all archive entries and
  destinations before file writes, rejects symlinks, traversal, portable-name
  collisions, fresh-lock replacement, and unconfirmed overwrite, and holds a
  destination lock while writing. CAP-0014 remains `not implemented` until its
  corruption recovery, missing-root reassignment, and backup-history outcomes
  are delivered.
- Phase 9e makes withdrawal a confirmed, reasoned, hash-chained state change
  without deleting release records or PDFs. Active-release consumers share one
  resolver, version allocation still advances over withdrawn history, missing
  and orphaned releases remain visible, and validated source/release paths open
  through narrow host commands.
- Phase 9f is split at restart-safe operator boundaries so each local desktop
  surface remains independently testable. Submit, review, decision, and release
  stay with phase 9i because their production commands require the live Office,
  Entra, and notification adapters owned by that phase.
- Phase 9f.1 introduces schema v9 because the pre-phase document-control model
  had no effective-date field; schema v8 workspaces migrate that field to unset
  before desktop editing is exposed. Actual control-data changes append
  hash-chained `document_control_data_changed` evidence with structured before
  and after values, then invalidate stale candidates. The Library keeps command
  validation failures beside the selected document and offers only enabled
  document/confidentiality types, including explicit override clearing.
- Phase 9f.2 keeps lifecycle preconditions in `dms-core`; the desktop adapter
  reports action availability and exact unavailable reasons, then invokes the
  same core transitions through narrow commands. Cancel review and Mark obsolete
  require a reason and confirmation, retain failed drafts in the selected-document
  context, and append canonical evidence. The pane lists newest-first workflow
  events with hashes and predecessor hashes and exposes the chain verification
  result without rewriting evidence.
- Phase 9f.3 replaces the prior single-purpose Configuration page with one
  stable routed activity. Workspace exposes roots, identity, the default review
  interval, and the existing optional Claude policy; Document defaults exposes
  the edit-root-relative confidentiality policy tree and document-type catalogue.
  The expected Configuration routing and local mutation IPC did not exist—the
  page rendered only the Claude policy—so the new core queries and narrow Tauri
  commands are required phase work. Workflow, Notifications, Microsoft Entra
  identity setup, and confidentiality catalogue administration remain phase
  9f.4 scope.
- Phase 9f.5 corrects viewport containment rather than adding another navigation
  model. The pane-level overflow rules existed, but the outer shell could still
  grow with exhaustive document details and move the sidebar or activity header.
  The shell now owns the window viewport, ordinary activities own the main
  content scroll, and Library navigation, contents, and details keep separate
  scroll regions. Browser QA at a 720 px viewport measured no document-level
  overflow, a 520 px selection viewport over 1,890 px of detail content, and
  unchanged sidebar/header positions after scrolling the detail pane 500 px.
- Phase 9i starts with an existing contract mismatch: the core generated
  abbreviated review-request and minor-publication templates while CAP-0010
  specifies literal subjects, labels, and field order. This is required current
  scope because live SMTP and `mailto:` must send the canonical contract, not
  merely deliver an outdated payload. The phase also wires the existing installed
  Office adapter, which was implemented but not reachable from a production
  desktop release command.
- Phase 9j found that the cached Entra people had no persisted refresh time,
  despite CAP-0021 requiring the Workflow summary to show it. Schema v10 adds
  the optional timestamp with a migration instead of fabricating recency from
  cached display data. The same review of Microsoft Graph permissions found that
  `/me` requires delegated `User.Read` in addition to the group-membership scope;
  the live adapter requests it only to identify the interactive decision actor.
- Phase 9j is one vertical identity slice. Its 836-line `graph.rs` module keeps
  device authorization, token persistence, Graph transport, and fake-backed
  protocol tests inside the desktop adapter boundary; splitting that protocol
  across phase-local files would not make the checkpoint independently usable or
  easier to verify.
- Phase 9k found that core records a decision-outcome delivery attempt but did
  not expose a retry path or append a retry attempt to canonical history, even
  though CAP-0010 requires failed or unconfirmed outcome notifications to be
  retryable and recorded. This belongs in the current lifecycle composition
  slice: add the narrow retry operation, canonical attempt evidence, and desktop
  confirmation surface alongside review and minor-publication delivery rather
  than leaving a decision state that cannot complete its notification evidence.

## Resume checklist

1. Read this CHG and affected CAP files (including CAP-0005, CAP-0006, CAP-0007).
2. Confirm phase statuses against the working tree.
3. Continue the single `in-progress` phase; do not open parallel progress plans.
4. Update CAP outcomes to present-tense implemented language only when tests prove them.
