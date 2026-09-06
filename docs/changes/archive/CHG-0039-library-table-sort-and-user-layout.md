# CHG-0039 — Library table sort direction and user layout

DMS Desktop will let an operator reverse the Library table’s existing sort order, retain every resized Library table column width in that OS user’s app preferences across libraries and relaunches, and reset only those personal table-layout widths through Configuration without modifying a workspace’s `.dms` metadata.

**Plan ID:** CHG-0039-library-table-sort-and-user-layout
**Execution slot:** P0800
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** CHG-0022 is archived with session-only table resizing evidence.
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `docs/product/capabilities/CAP-0005-desktop-shell.md` (Outcomes 13, 17); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcomes 2, 9–10, 14); `docs/changes/archive/CHG-0022-library-table-workflow-columns.md`; `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-desktop/src/lib.rs` (`Preferences`, `load_preferences_at`, `save_preferences_at`, `normalize_preferences`); `crates/dms-desktop/ui/app.mjs` (`defaultPreferences`, `createInitialState`, `persistPreferences`, `updateLibraryActivity`, Library sort and column-resize handlers); `crates/dms-desktop/ui/library.mjs` (`createLibraryState`, `sortLibraryEntries`, `LIBRARY_COLUMNS`, `setColumnWidth`, `libraryMarkup`); `crates/dms-desktop/ui/configuration.mjs` (`workspaceMarkup`); `crates/dms-desktop/ui/*.test.mjs`; `docs/product/wireframes/generate.mjs`
**Produces:** Every Library table supports ascending and descending order for its existing sort keys while folders remain before files; all eight table-column widths are one OS-user preference shared by every library and restored after relaunch; Configuration can reset those widths to defaults without touching `.dms`, saved views, sidebar preferences, pane widths, or session state.
**Status:** done — Phase 1 committed as `0f8d439`; Phase 2 gates passed with browser-harness wireframe rendering because this WSL image has no runnable native Chrome.

| Field | Value |
| --- | --- |
| ID | CHG-0039 |
| Status | done |
| External request | Direct operator request: "The table view of the directory content, the documents and folders, should allow switching sort order. If the user changes the width of an column, this width should not be changed and kept fix. this confiugration is user specific not library specific. within the configuration a ability to reset the \"layout\" could be implemented" |
| Affected CAPs | CAP-0005, CAP-0006 |
| Decision records | No new ADR. This extends the established OS-user preference boundary and keeps controlled workspace metadata unchanged. |

## Current state

- The current Library toolbar selects only a sort key (`Name`, `Title`, `Document number`, or `Lifecycle`); `sortLibraryEntries` always orders ascending and always keeps folders before files (`crates/dms-desktop/ui/library.mjs:752-763, 1182-1183`).
- The Library’s eight column headers already expose pointer-resize grips and write their final widths to session-only `library.column_widths`; pointer completion does not persist preferences (`crates/dms-desktop/ui/library.mjs:855-878`; `crates/dms-desktop/ui/app.mjs:2630-2707`).
- Desktop preferences already live in the OS user app-config `preferences.json`, separately from `.dms`, and are loaded once at startup and saved through the existing Tauri commands (`crates/dms-desktop/src/lib.rs:103-120, 2183-2209`; `crates/dms-desktop/ui/app.mjs:422-426, 2772-2778`).
- CAP-0006 and archived CHG-0022 explicitly describe table widths as session-only and name persistence and sorting as out of scope (`docs/product/capabilities/CAP-0006-library-explorer.md:87-92`; `docs/changes/archive/CHG-0022-library-table-workflow-columns.md:28-35`).
- Configuration → Workspace already distinguishes `.dms` workspace settings from other user-scoped application settings, making it the existing configuration surface for an explicitly labelled personal-layout reset (`crates/dms-desktop/ui/configuration.mjs:132-143`).
- Surprise: CAP-0006 says search results can sort by latest release and next-review due date, but the runtime exposes only the four keys above. This change must make the current implemented sort surface truthful; it must not quietly add unrelated sort keys.

## Risk call-out

Resetting a layout is intentionally destructive to one OS user’s chosen column widths. The control must be named **Reset Library table layout**, explain that it restores only the eight column widths to defaults for every library opened by that OS user, and apply immediately only after the operator invokes it. It must not reset `.dms`, saved views, recent libraries, sidebar preference, folder/detail pane widths, selected documents, or any workflow data.

The persisted preference file is user-controlled and may come from an earlier version or manual repair. Missing layout data must default cleanly; unknown column keys and unusable widths must be ignored rather than breaking Library rendering. Existing preference fields and saved views must remain readable.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Implement direction switching and user-scoped table-width persistence | done (`cargo test -p dms-desktop` — 91 passed; `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/configuration.test.mjs` — 117 passed; `cargo fmt --all -- --check` + `git diff --check` exit 0) | `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/configuration.test.mjs` exit 0 with ascending/descending ordering, folders-first ordering, preference round-trip, relaunch hydration, cross-library width reuse, and reset coverage. |
| 2 | Publish Library and shell contracts, wireframes, and completion evidence | done (`node docs/product/wireframes/generate.mjs`; browser-harness inspection confirmed the CAP-0005 personal-layout card and CAP-0006 sort/direction controls without clipping; `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs` — 140 passed, repository-wide Markdown-link check, and `git diff --check` all exit 0. Native Chrome is unavailable: downloaded Chrome lacks `libnspr4.so` and dependency installation needs interactive sudo.) | `node docs/product/wireframes/generate.mjs`, the CAP-0005 and CAP-0006 PNG render commands, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree. |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Implement direction switching and user-scoped table-width persistence

**Goal:** The Library table reverses its selected sort order on demand, keeps folder rows ahead of files in either direction, and applies one validated OS-user column-width map across all Library sessions without touching workspace metadata.

Steps:

1. Define an explicit ascending/descending direction in Library state and normalize it to ascending when absent or invalid. Extend `sortLibraryEntries` so direction reverses comparisons only within the folder group and the file group; folders must remain first, and equal selected-key values must have a deterministic name tie-breaker. Keep the existing four sort keys; do not add column reordering, hiding, or unimplemented latest-release/next-review sort keys in this change.
2. Render an accessible direction control beside the existing **Sort** key selector. Changing key or direction resets pagination, updates the singleton Library activity’s route state, and preserves the existing saved-view behavior. Extend saved-view route state/identity so an explicitly saved descending Library view restores the same key and direction without changing the preference scope of column widths.
3. Add a `library_table_column_widths` preference field with a backward-compatible default to the Rust `Preferences` model and its JavaScript defaults. Normalize the stored map to the known eight `LIBRARY_COLUMNS` keys and each column’s existing minimum width; discard unknown or unusable entries. Hydrate every new Library state from that one preference map, so opening another library and relaunching DMS uses the same widths rather than a workspace-specific value.
4. Make a completed header-drag update both active `library.column_widths` and `preferences.library_table_column_widths`, then save through the existing `save_preferences` IPC path. Do not persist on pointer movement, and keep Escape/cancel semantics unchanged. The persistence path must retain sidebar, saved-view, and recent-library values unchanged.
5. Add an explicitly scoped **Personal Library table layout** card to Configuration → Workspace. Its **Reset Library table layout** action clears only the width preference map, immediately restores default widths in the active Library when present, persists the preference, and presents a truthful success/failure message. It must not dispatch a workspace mutation command or require/alter `.dms` data.
6. Add focused Rust and frontend tests: legacy preferences without the new field still load; the width map round-trips outside the workspace; invalid/unknown saved entries cannot corrupt headers; a resized width hydrates into fresh Library state for another library; direction switches ascending/descending values while folders remain first; a saved descending view restores its direction; and the Configuration card/reset action clears only personal widths while preserving other preference and session state.

Verification gate: `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/configuration.test.mjs` exit 0 with ascending/descending ordering, folders-first ordering, preference round-trip, relaunch hydration, cross-library width reuse, and reset coverage.

## Phase 2 — Publish Library and shell contracts, wireframes, and completion evidence

**Goal:** Current-state records and review screens distinguish user-scoped table layout from workspace metadata and show the implemented sort-direction and reset interactions.

Steps:

1. Amend CAP-0006 to replace session-only column widths with the user-scoped cross-library preference, state the direction toggle and folders-first invariant, and make the implemented sort-key list truthful. Keep saved-view sort behavior distinct from column-width preferences.
2. Amend CAP-0005 to state that the personal Library table layout is stored in the OS-user app-config store and can be reset through Configuration without touching `.dms`, saved views, or other application layout preferences.
3. Amend `crates/dms-desktop/AGENTS.md` with the durable ownership boundary: Library table column widths are one OS-user setting shared across libraries; the Configuration reset clears only those widths. Leave parent AGENTS files unchanged unless the DOX pass finds an ownership or index change.
4. Update the CAP-0006 wireframe definition with a visible sort-key/direction affordance and resizable table headers. Update the CAP-0005 wireframe definition with the explicitly user-scoped **Reset Library table layout** card. Regenerate HTML, `index.html`, `manifest.json`, and both PNGs; visually inspect them. Do not add CAP IDs or hand-edit generated outputs.
5. Update the CAP and CHG indexes, record passing evidence, mark each phase done, archive this CHG, and refresh `docs/changes/README.md` only at completion. Do not change unrelated active records.

Verification gate: `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0005-desktop-shell.png "file://$PWD/html/CAP-0005-desktop-shell.html" && test -s exports/CAP-0005-desktop-shell.png)`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0006-library-explorer.png "file://$PWD/html/CAP-0006-library-explorer.html" && test -s exports/CAP-0006-library-explorer.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree.

## Out of scope

- Table column reordering, hiding, new columns, or a generic reset that also changes sidebar, pane, activity, saved-view, or workspace configuration.
- Persisting Library folder tree width, selection-pane width, folded state, expanded branches, search, selection, history, or open activities; their existing session-only contracts remain unchanged.
- New latest-release or next-review sort keys; CAP-0006 is corrected to the current implemented key set rather than widening this request.
- Writing a user’s table layout into `.dms`, workflow evidence, document metadata, audit reports, or another user’s OS app-config directory.
