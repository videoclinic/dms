# CHG-0041 — Library batch execution activity

DMS Desktop will let an operator execute the already-applicable homogeneous Library multi-selection actions through one session-only **Batch execution** activity, which records each operation-level success, error, or safety warning and lets the operator filter and delete the currently visible log entries without changing workspace metadata or workflow evidence.

**Plan ID:** CHG-0041-library-batch-execution-activity
**Execution slot:** P0820
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** CAP-0006 and the desktop currently support modifier-assisted file multi-selection plus atomic batch add/unregister IPC commands, but surface their outcome only as a refresh or one global error.
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `docs/product/capabilities/CAP-0005-desktop-shell.md` (open-activity identity and session-only state); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcomes 2, 3, 6–10); `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-desktop/ui/app.mjs` (`activityKey`, `openActivity`, `activityMarkup`, Library batch click handlers); `crates/dms-desktop/ui/library.mjs` (`toggleLibrarySelection`, `selectedEntries`, `selectionMarkup`); `crates/dms-desktop/src/lib.rs` (`add_library_documents`, `unregister_library_documents`); `crates/dms-core/src/library.rs` (`Workspace::add_documents`, `Workspace::unregister_documents`); `crates/dms-desktop/ui/app.test.mjs`; `crates/dms-desktop/ui/library.test.mjs`; `docs/product/wireframes/generate.mjs` (`batchSelectionPane`).
**Produces:** An explicit, reusable session-only Batch execution activity for the existing all-or-nothing Add-to-library and Unregister multi-selection operations; an accessible success/warning/error log with type filtering and filtered deletion; focused frontend evidence; and matching CAP/wireframe contracts.
**Status:** done — all three phases verified; the complete Phase 3 gate passes after the user-authorized repair of two pre-existing CHG-0040 Markdown links.

| Field | Value |
| --- | --- |
| ID | CHG-0041 |
| Status | done |
| External request | Direct operator request: "Add the ability to multiselect files and execution action on the multiselection. If it's not an overkill this is a batch execution where in a new pane the actions are logged with success/error/warning messages. Because it's a new pane the should also be a filter for the type of success/error/warning and an ability to delete the logged messages, based on the selected filter." |
| Affected CAPs | CAP-0005, CAP-0006 |
| Decision records | No new ADR. This is a session-only desktop activity for existing Library operations; it neither creates durable audit evidence nor changes the core metadata transaction boundary. |

## Current state

- File rows already support Ctrl/Cmd-assisted selection, and `toggleLibrarySelection` clears document detail while retaining the selected relative paths (`crates/dms-desktop/ui/library.mjs:244-258`).
- The selection pane already offers **Add N documents to library** only for homogeneous not-in-library source files and **Unregister N documents** only for homogeneous registered/lost-source documents; a mixed selection has no common action (`crates/dms-desktop/ui/library.mjs:1080-1096`).
- Those buttons invoke one existing batch IPC call, refresh after success, and put any failure into the app-wide error field (`crates/dms-desktop/ui/app.mjs:1389-1415`).
- `Workspace::add_documents` and `Workspace::unregister_documents` clone first and replace the workspace only when every selected target succeeds, so both commands are all-or-nothing; splitting them into per-file calls would weaken that invariant just to manufacture per-file status rows (`crates/dms-core/src/library.rs:208-238`).
- The shell already opens/focuses session activities by workspace/task key and does not persist activity state or tabs (`crates/dms-desktop/ui/app.mjs:179-210`; `docs/product/capabilities/CAP-0005-desktop-shell.md`).
- The CAP-0006 batch contract and wireframe mention bulk verify, but the runtime exposes only batch add and unregister. This change must correct that mismatch rather than invent an unrelated integrity batch API.

## Risk call-out

**Unregister** is a destructive membership operation, although it preserves source files, stable IDs, document-control data, workflow/release history, and checksums. The Batch execution activity must record that safety warning before dispatching unregister and must not make a batch look partially complete when the core transaction rejected it.

The existing core commands are deliberately atomic. The implementation must issue each command once for its captured homogeneous selection, then record one operation-level outcome containing the action and target identities. Do not loop over individual targets, retry failures, or add a “continue on error” mode: those would create partial workspace mutations with no recovery or rollback contract.

The activity log is session-only presentation state. It must never enter `<edit-root>/.dms`, workflow evidence, audit reports, saved views, OS-user preferences, or notifications. Closing its tab or switching/closing the workspace discards it.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Implement the reusable Batch execution activity and atomic operation logging | done (Phase 1 node gate: 110 passed, 0 failed; `git diff --check` exit 0) | `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/batch-execution.test.mjs` exits 0 with homogeneous action eligibility, activity reuse, atomic dispatch, success/error/warning entries, filter visibility, filtered deletion, and session-only cleanup coverage. |
| 2 | Publish Library/shell contracts and matching wireframe states | done (four review corrections verified; exact generation/render and syntax gates exit 0; both 1600×1600 PNGs visually inspected; focused Node suite 110 passed; `git diff --check` exit 0) | `node docs/product/wireframes/generate.mjs` and a 1600×1600 headless-Chrome render of `CAP-0005-desktop-shell.html` and `CAP-0006-library-explorer.html` exit 0 with both generated PNGs non-empty and visibly showing the Batch execution and typed-log controls. |
| 3 | Run the workspace gate and close the record | done (all six gates exit 0 after authorized link repair; Rust 188 passed; Node 160 passed; Markdown links 0 issues; archived and index updated) | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, the repository Markdown-link check, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree, and CHG-0041 is archived. |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Batch execution activity and atomic operation logging

**Goal:** A homogeneous multi-selection can explicitly execute its existing batch action through one reusable activity tab that retains a session-only operation log and never changes the atomicity or scope of the underlying DMS mutation.

Steps:

1. Keep the current single-click and Ctrl/Cmd-assisted file-selection model. Preserve the existing rule that only homogeneous selections receive an action: supported not-in-library files may add; registered or lost-source documents may unregister; folders, unsupported files, and mixed sets receive no executable batch action.
2. Add a small `ui/batch-execution.mjs` module with pure state helpers and markup for an activity-local log. Its entry shape includes a stable frontend ID, `success`/`warning`/`error` type, action label, captured short source/document identity list, and safe display message. It has no timestamp persistence requirement and does not model a job queue, retries, cancellation, progress polling, or background work.
3. Extend `createInitialState`, workspace-session cleanup, activity routing, and `activityMarkup` in `ui/app.mjs` so **Batch execution** is one reusable workspace-scoped activity. Opening a new batch focuses the existing tab and preserves prior entries while that workspace session remains open. Closing the tab removes its in-memory log; opening/switching/closing a workspace clears the log. Do not add it to saved views or preferences.
4. Replace the direct multi-selection handlers with an explicit **Execute: Add N documents to library** or **Execute: Unregister N documents** path. Capture the action and target identities before invoking the existing one-shot IPC command, open/focus Batch execution, and retain the Library selection until the result is known. Do not expose a generic command picker or allow a user to run an action not applicable to every selected item.
5. For unregister, append a visible warning before dispatch: it removes library membership but does not delete the source file, stable ID, document control data, workflow/release history, or checksums. On a fulfilled IPC call append one success entry and refresh the Library snapshot. On a rejected IPC call append one error entry, preserve the pre-operation selection, and do not claim that any target changed. Keep the existing core/adapter batch commands unchanged and call each exactly once.
6. Render **All**, **Success**, **Warning**, and **Error** filter controls with accessible pressed state. The default is All. Filter changes only visibility. **Delete shown messages** deletes exactly entries matching the active filter; when All is selected it deletes all entries. It never calls IPC or changes any DMS state. Render an empty-state message after deletion and retain unfiltered entries.
7. Add focused unit and interaction tests. Cover modifier multi-selection and no-action mixed selections; correct action labels; one task-keyed tab reused for later batches; captured target identity; one IPC dispatch; no dispatch of an inapplicable action; add success refresh; unregister warning then success; error without false success or refresh; filter semantics; delete-only-filtered semantics; and activity/workspace close cleanup. Assert that source content, credentials, and absolute edit-root paths are absent from log markup.

**Verification gate:** `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/batch-execution.test.mjs` exits 0 with homogeneous action eligibility, activity reuse, atomic dispatch, success/error/warning entries, filter visibility, filtered deletion, and session-only cleanup coverage.

### Phase 1 evidence and bounded decisions

- The exact Phase 1 node gate exits 0: 110 tests passed, none failed, skipped, or cancelled. `git diff --check` exits 0. Coverage includes late mutation/refresh completions after activity close, workspace activation, and window close.
- Each eligible execution captures source names/document IDs and dispatches one existing batch command. Single-document actions and the Rust core/adapter are unchanged. New executions select All so prior filtering cannot hide unregister's pre-dispatch preservation warning.
- Log messages use fixed safe copy, not raw backend errors or response payloads. Mutation rejection retains selection without asserting success or rollback; a post-success refresh failure is explicitly a refresh error, not a rejected mutation.
- Session identity checks discard late log entries after tab close and prevent old results from restoring a replaced workspace. Closing the log does not cancel an already-dispatched operation.
- DOX pass: parent ownership/indexes are unchanged. Desktop AGENTS, CAP-0005/CAP-0006, and wireframes remain untouched because their coordinated updates belong to pending Phase 2. No unrelated CHG is changed. The independent read-only review remains a handoff gate, not claimed here.

## Phase 2 — Library/shell contracts and wireframe evidence

**Goal:** The product contracts and visual references describe the concrete existing multi-actions and their session-only execution result pane without claiming unsupported bulk workflows.

Steps:

1. Amend CAP-0006 Outcomes 3, 7, and 8 to name the two implemented multi-selection operations, their homogeneous-selection boundary, their explicit Batch execution handoff, and their single operation-level result entries. Remove the stale “bulk verify where defined” claim; do not add batch lifecycle, release, review, reassociation, permalink, or integrity actions.
2. Amend CAP-0005’s activity outcome to define **Batch execution** as one reusable workspace-scoped, session-only activity. Its log may be filtered by success/warning/error and may delete entries matching the active filter. It is not a saved view, audit record, notification history, or workspace metadata.
3. Amend `crates/dms-desktop/AGENTS.md` because it owns the durable Desktop activity and Library selection contracts: batch add/unregister retain core atomicity, show an activity-local operation log, and never persist that log. Leave parent AGENTS files unchanged unless the DOX pass finds an actual ownership/index change.
4. Update `docs/product/wireframes/generate.mjs` only. The CAP-0006 state must show a homogeneous multi-selection with an explicit execution action and disclose that mixed selections have no action. The CAP-0005 state must show the **Batch execution** tab/pane, success/warning/error type filters, representative synthetic entries, and **Delete shown messages**. Regenerate HTML, `index.html`, and `manifest.json`; do not hand-edit generated output.
5. Render the two generated HTML pages to their existing PNG paths and visually inspect normal, filtered, and empty-log hierarchy without overlap or clipping. Keep sample names synthetic and avoid paths, credentials, or document content.

**Verification gate:** `(cd docs/product/wireframes && node generate.mjs && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0005-desktop-shell.png "file://$PWD/html/CAP-0005-desktop-shell.html" && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0006-library-explorer.png "file://$PWD/html/CAP-0006-library-explorer.html" && test -s exports/CAP-0005-desktop-shell.png && test -s exports/CAP-0006-library-explorer.png)` exits 0, and the rendered PNGs visibly show the Batch execution and typed-log controls without overlap or clipping.

### Phase 2 evidence and bounded decisions

- The four review corrections are verified: a visible single-document pane
  remains alongside the homogeneous batch states; Batch execution has no bookmark
  control; batch Source files contain exact names only, with the preservation
  warning confined to the execution log; deletion sits outside each filter group.
- The exact Phase 2 verification command above exits 0. Final exports:
  [`CAP-0005-desktop-shell.png`](../../product/wireframes/exports/CAP-0005-desktop-shell.png)
  is 234684 bytes; [`CAP-0006-library-explorer.png`](../../product/wireframes/exports/CAP-0006-library-explorer.png)
  is 328500 bytes. Both PNG headers confirm 1600×1600 dimensions. SHA-256 checks
  before and after final visual inspection confirm the inspected files stayed unchanged.
- Visual inspection of those final PNGs passes: CAP-0005 shows the matching
  active tab/header, typed operation entries, All/Success/Warning/Error controls,
  Success-only filtering, Error-filter deletion with other types retained, and
  All-empty deletion; both empty states disable deletion. CAP-0006 shows two
  selected registered rows matching the right pane, explicit unregister/add
  execution, and a mixed-selection no-action state. Its single-document alternative
  shows title, number, Source file, document-control topic, action feedback, and
  all six bottom-docked actions. Required controls and state copy are legible
  without overlap or accidental clipping.
- A local headless-CDP layout probe confirms both selected Library rows fit
  inside the table viewport, the alternative-state panels fit inside 1600×1600,
  and the table and action buttons have no horizontal content overflow. It also
  confirms exact batch names, no batch warning, no header bookmark, four filter-only
  role groups with sibling deletion controls, and both disabled empty-state deletions.
  Unselected lower rows and exhaustive document topics retain their normal scrollers.
  The CAP-0006-only
  layout uses the full render width; CAP-0005 retains other shell examples in a
  collapsed disclosure so the batch states remain visible together.
- `node --check docs/product/wireframes/generate.mjs` and `git diff --check`
  exit 0. A second generator run reproduces all 23 HTML/index/manifest outputs
  byte-for-byte; inventory checks confirm 21 unique screens with matching
  non-empty HTML/PNG pairs and index links. Only the two affected CAP pages,
  their PNGs, and the generated index/manifest differ.
- The focused regression command
  `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/batch-execution.test.mjs`
  exits 0: 110 passed, 0 failed, skipped, or cancelled. Runtime code and tests
  are unchanged. Phase 1 evidence is preserved verbatim.
- CAP-0005 retains its bounded `not implemented` status. Its saved-view outcome
  explicitly excludes Batch execution to avoid contradicting the activity
  contract. CAP-0006 removes bulk verification and describes only the existing
  atomic homogeneous add/unregister commands, with no filesystem-rollback claim.
  No ADR or new persistence boundary is needed.
- DOX pass: `crates/dms-desktop/AGENTS.md` owns the updated activity, selection,
  and module responsibilities; that existing Phase 2 edit and both CAP edits are
  unchanged by the review corrections. Root, parent, product, change, and wireframe
  AGENTS files and CAP/CHG indexes remain unchanged because ownership, record
  lifecycle, and child boundaries are unchanged. Generated outputs were not
  hand-edited. No commit, push, deployment, secret access, or external-state
  mutation was performed. Independent read-only review remains the next handoff
  gate; the Phase 3 workspace gate and record closure remain pending.

## Phase 3 — Workspace gate and record close

**Goal:** The runtime, tests, product records, generated visual evidence, and active-change lifecycle agree before the progress record is archived.

Steps:

1. Run the focused frontend gate and then the workspace format, Clippy, Rust, frontend, link, and diff gates. Fix only regressions caused by this change; retain unrelated working-tree changes.
2. Run `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .` and confirm CAP/CHG links, wireframe references, and generated inventory remain valid.
3. After every gate passes, record phase evidence, mark phases done, set the CHG status done, move this file to `docs/changes/archive/`, and update `docs/changes/README.md` so CHG-0041 appears only in Archive. Do not change unrelated active records.

**Verification gate:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .`, and `git diff --check` exit 0; `docs/changes/README.md` lists CHG-0041 only in Archive and `docs/changes/active/CHG-0041-library-batch-execution-activity.md` no longer exists.

### Phase 3 evidence and authorized blocker repair

Initial gate in the designated `dms-chg-0041-phase2` worktree after marking
Phase 3 `in-progress`, before authorization to repair the unrelated links:

| Command | Outcome |
| --- | --- |
| `cargo fmt --all -- --check` | Exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0. |
| `cargo test --workspace` | Exit 0; 188 passed, 0 failed, 0 ignored. |
| `node --test crates/dms-desktop/ui/*.test.mjs` | Exit 0; 160 passed, 0 failed, skipped, or cancelled. |
| `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .` | Exit 1; two `file_missing` issues in unrelated CHG-0040 records. |
| `git diff --check` | Exit 0. |

- Link blockers: `docs/changes/active/handoff-CHG-0040-phase-2.md:8` and
  `docs/changes/active/task-packet-CHG-0040-phase-2.md:5` link to the absent
  `active/CHG-0040-document-type-id-migration.md`. At HEAD
  `0850913a22775f1ee3478543305413e89c11e605`, both links already exist and the
  target is under `archive/`; all three files were unchanged from HEAD at the
  initial gate.
  No CHG-0041 link issue was reported.
- The initial gate left those unrelated links untouched and Phase 3
  `in-progress`. The user then explicitly authorized changing only the two
  link targets to `../archive/CHG-0040-document-type-id-migration.md` and
  rerunning the complete gate before closure. Both target-only repairs are
  applied; link labels and all other CHG-0040 record content are unchanged.

Complete gate rerun after the authorized repair:

| Command | Outcome |
| --- | --- |
| `cargo fmt --all -- --check` | Exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0. |
| `cargo test --workspace` | Exit 0; 188 passed, 0 failed, 0 ignored. |
| `node --test crates/dms-desktop/ui/*.test.mjs` | Exit 0; 160 passed, 0 failed, skipped, or cancelled. |
| `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .` | Exit 0; strict mode, 0 issues, 0 files with issues. |
| `git diff --check` | Exit 0. |

- Phase 3 and the CHG status are `done`. The record is at
  `docs/changes/archive/CHG-0041-library-batch-execution-activity.md`, its active
  path is absent, and `docs/changes/README.md` lists CHG-0041 only in Archive.
- DOX closure check: this gate evidence changes no runtime contract, ownership,
  or child boundary. Root, documentation, and change AGENTS files and their
  child indexes need no edits. Existing Phase 2 CAP, desktop DOX, and wireframe
  edits are preserved; the declined review nit is not reopened. No runtime
  changes, commit, push, deployment, or secret access were performed.

## Out of scope

- Per-file best-effort execution, retries, cancellation, progress polling, queues, or a background worker; add and unregister retain their existing one-command atomic transactions.
- Bulk integrity verification, lifecycle transitions, release, review, reassociation, document-control edits, notes, permalinks, or any action other than the currently implemented batch add/unregister operations.
- Persisting or exporting batch log entries to `.dms`, OS-user preferences, saved views, audit reports, workflow evidence, notifications, or another workspace/session.
- Changing document-selection behavior, adding folder multi-selection, per-row menus, or a separate file-manager workflow.

## Risks & open questions

- The term “pane” is implemented as an activity tab/pane, not a fourth persistent Library column. This reuses the shell’s session-only activity model and avoids shrinking the already bounded Library selection pane.
- Warning entries are deliberately limited to concrete non-fatal safety information (currently unregister’s preservation boundary). A rejected command produces an error, never a warning or a frontend success claim.
- Existing backend limitation, outside this frontend phase: `Workspace::add_documents` validates the batch before replacing its candidate state, but then projects Markdown frontmatter; the adapter saves afterward. Projection/save errors do not establish filesystem rollback. Phase 1 preserves these commands and reports only that success was not confirmed when IPC rejects.
