# CHG-0014 — Pinned Actions, Lost-source reassociate, native file pick

| Field | Value |
| --- | --- |
| ID | CHG-0014 |
| Status | done |
| External request | Direct operator request: The "Reassociate source" feature in the document control pane should only be visible, as a own topic, only if it's applicable. Follow-up: "Actions" should always be visible below the document control but within the pane so the user can always have access to actions. The "Reassociate source" action should also be kept within this section in that cases where we recover a lost document. Follow-up: The edit field should be extended by a standard operating system file/directory search dialog so the user does not have to enter the path and name manually. If the user selects a file outside of the edit directory, the selection is skipped and an appropriate error message shows the rules that the selected file have to be in the edit directory; the selected file can also not be registered in the library and further rules, if they apply. Follow-up: this check and warning message appear only after the user presses the "Reassociate source" button, not earlier. Follow-up: Also only supported file formats can be selected. |
| Affected CAPs | CAP-0006, CAP-0013, CAP-0015 |
| Decision records | none |

**Plan ID:** `CHG-0014-reassociate-source-topic-visibility`
**Execution slot:** P0100
**Created:** 2026-08-16
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `docs/product/capabilities/CAP-0006-library-explorer.md` (selection-pane actions and Lost source), `docs/product/capabilities/CAP-0013-library-maintenance.md` (reassociate outcomes), `docs/product/capabilities/CAP-0015-document-control-data.md` (foldable topics), `crates/dms-desktop/ui/library.mjs` (`selectionMarkup`, `DEFAULT_SELECTION_OPEN`), `crates/dms-desktop/ui/library.test.mjs`, `crates/dms-desktop/ui/app.mjs` (`library-reassociate-form`), `crates/dms-desktop/src/lib.rs` (`select_directory`, `choose_markdown_template`, `reassociate_library_document`), `docs/product/wireframes/generate.mjs` (`documentControlDataSelectionPane`)
**Produces:** Pinned non-foldable **Actions** under **Document control data**; Lost-source-only reassociate form with a native **Browse…** limited to supported drafts; submit-time fail-closed rule error; no picker-time warning.
**Status:** done — pinned Actions, Lost-source Browse, submit-time rules, and wireframes shipped

The Library selection pane pins **Actions** under **Document control data**. **Reassociate source** lives in that block only for a single Lost source document. The path field has a native OS file picker. Validation and the rule error run only when the operator presses **Reassociate source**.

## Current state

- `selectionMarkup` order is foldable `control` → pinned Actions → foldable `schedule` → `revision` → `releases`. Actions is not a `<details>` topic and is dropped from `DEFAULT_SELECTION_OPEN`.
- `#library-reassociate-form` is a typed path plus **Browse…** only for a single Lost source document. Browse calls `choose_reassociate_source` (supported-draft filter, no All files). Submit runs desktop standing rules before `reassociate_library_document` and keeps failures in `library.detail_error`.
- Native pickers: `select_directory` starts at home; `choose_markdown_template` starts at the edit root and picks a `.docx`; `choose_reassociate_source` starts at the lost document's last known folder under the edit root, else the edit root.
- Core still absorbs a registered target. Desktop Actions refuse that target and do not mutate the workspace.
- Operator lock: Lost source only; Reassociate stays in Actions; Actions always visible; Browse extends the field and can select only supported drafts (`md` / `docx` / `xlsx` / `pptx` from `is_supported_source`); checks run on **Reassociate source** only.

## Pane layout (locked)

For exactly one registered document:

1. Identity header, Lost source banner (when applicable), Source file.
2. **Document control data** — foldable (`control`).
3. **Actions** — not a `<details>` topic. Always rendered immediately after Document control data. Drop `actions` from `DEFAULT_SELECTION_OPEN`.
4. **Document review schedule**, **Revision cycle**, **Releases** — still foldable.

Actions stays in normal pane flow (not a sticky overlay).

Inside Actions, show the reassociate help + field + **Browse…** + **Reassociate source** only when Lost source.

## Applicability predicate (Reassociate)

Show the form iff all of:

1. Exactly one file is selected.
2. That file is a registered library document.
3. `detail.source_state === "registered"` and `detail.source_exists === false`.

## Picker and submit rules (locked)

- **Browse…** opens the host file picker (same `tauri_plugin_dialog` path as `choose_markdown_template`). Start at the lost document's last known folder under the edit root, else the edit root. The dialog offers **only** supported draft extensions from `is_supported_source`: `.md`, `.docx`, `.xlsx`, `.pptx`. Do not add an All files / * filter. Cancel leaves the field unchanged. A chosen local path is written into the field as an edit-root-relative path when it can be relativized; otherwise the absolute path is stored so submit can explain it. Browse itself does not validate location/registration, warn, or mutate the document.
- Manual typing stays allowed; an unsupported typed path is still refused on **Reassociate source**.
- **Reassociate source** is the only moment that checks the path. On failure: do not call `reassociate_document` / do not save; keep the field; show one pane error that lists every failed standing rule that applies to this path. On success: existing reassociate mutation.
- Standing desktop submit rules (all must hold):
  1. Regular file under the workspace edit root (not outside, not a directory, not under `.dms`).
  2. Supported draft format; not an Office lock/temp sidecar; not the workspace Word-template asset.
  3. Not already another **registered** library document.
- Rule 3 is a desktop-pane close of CAP-0013 absorb: this control refuses a registered target instead of merging audit history. `Workspace::reassociate_document` and the CLI keep absorb. Do not show absorb-merge copy in this form.

## Phases

| # | Phase | Status | Verification gate |
|---|---|---|---|
| 1 | CAP wording + selection-pane layout | done (`node --test crates/dms-desktop/ui/library.test.mjs` 25/25) | `node --test crates/dms-desktop/ui/library.test.mjs` exits 0; healthy markup has no `#library-reassociate-form` and no `data-library-section="actions"`; Lost source markup has the form after `data-library-section="control"` and before `data-library-section="schedule"` |
| 2 | Native Browse + submit-time rule error | done (`node --test crates/dms-desktop/ui/library.test.mjs` 27/27; `cargo test -p dms-desktop --lib reassociate` 4/4) | Desktop/frontend tests: picker filters are only `md`/`docx`/`xlsx`/`pptx` with no All-files filter; Browse cancel leaves the field; Browse does not emit the rule error; submit of an outside-edit-root path, an already-registered path, and an unsupported typed path each refuse with a rule-list error and leave workspace documents unchanged; a valid unregistered in-root file still reassociates |
| 3 | Wireframes + desktop contract + workspace gate | done (`cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`) | Regenerated CAP-0006/0013/0015 HTML+PNG; `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` all exit 0 |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, `pending` otherwise.

## Phase 1 — CAP wording + selection-pane layout

**Goal:** Runtime pane and CAP-0006 / CAP-0013 / CAP-0015 agree on pinned Actions and Lost-source-only reassociate.

Steps:

1. Amend CAP-0006 / CAP-0015: foldable topics are **Document control data**, **Document review schedule**, **Revision cycle**, **Releases**. **Actions** is a non-foldable block immediately under Document control data.
2. Amend CAP-0006 outcome 8: **Reassociate source** remains an Actions control, shown only when the single selection is **Lost source**. Drop "or when an in-root rename/repath is allowed".
3. Amend CAP-0013 outcome 3: selection-pane reassociate is Lost-source-only, lives in Actions, uses a path field plus native **Browse…** that can select only supported drafts (`.md`, `.docx`, `.xlsx`, `.pptx`), and validates location/registration only on **Reassociate source**.
4. Amend CAP-0013 outcome 4: desktop Actions refuse an already-registered target (no absorb from this control). Keep core/CLI absorb.
5. In `selectionMarkup`, render Actions after `control` and before `schedule`; gate the reassociate form on `sourceLost`; drop `actions` from `DEFAULT_SELECTION_OPEN`.
6. Update `library.test.mjs` for the layout and Lost-source-only form. Phase 2 owns Browse/submit assertions.
7. Amend `crates/dms-desktop/AGENTS.md` for pinned Actions and Lost-source-only reassociate. Browse and submit-time rules stay Phase 3.

**Verification gate:** `node --test crates/dms-desktop/ui/library.test.mjs` exits 0 with the layout assertions in the phase table.

## Phase 2 — Native Browse + submit-time rule error

**Goal:** The operator picks a file with the OS dialog; the rule error appears only after **Reassociate source**.

Steps:

1. Add a Tauri command (reuse `DialogExt` like `choose_markdown_template`) that only picks a file and returns `Option<String>`. Start directory = parent of the stored locator if it is still under the edit root, else the edit root. Register one filter for supported drafts: `md`, `docx`, `xlsx`, `pptx`. Do not register All files. Cancel → `Ok(None)`. Do not validate location or registration here.
2. Wire **Browse…** beside the path input (`directory-field` pattern in `app.mjs:434`). On pick, write the path into the input. Do not set `appState.error`.
3. On **Reassociate source**, run the standing rules before `reassociate_document`. If any fail, skip the mutation and show one error that names every failed rule plus the standing constraints (must be a supported unregistered source file inside the edit root). Keep the error in the selected-document context (`library.detail_error` or the existing pane error), not a picker dialog.
4. Tests: adapter or UI coverage for cancel, no warning on Browse, submit refuses outside-root / registered / unsupported without saving, happy-path still reassociates.

**Verification gate:** the phase-2 commands in the table exit 0 and prove no workspace mutation on each refused submit.

## Phase 3 — Wireframes + desktop contract + workspace gate

**Goal:** Visual and adapter contracts match the pane; workspace still builds.

Steps:

1. Healthy CAP-0015 pane: Actions non-foldable under Document control data; no Reassociate control.
2. Lost source CAP-0006 sample: Actions shows path + **Browse…** + **Reassociate source**.
3. CAP-0013 may keep a maintenance/rescan Reassociate hint; do not show absorb-from-this-pane copy on the document Actions form.
4. Regenerate HTML then PNG per `docs/product/wireframes/AGENTS.md`.
5. Update `crates/dms-desktop/AGENTS.md`: pinned Actions; Lost-source-only reassociate; Browse; submit-time rules.
6. Run workspace fmt / test / clippy.

**Verification gate:** regenerated CAP-0006/0013/0015 HTML+PNG exist; `cargo fmt --all -- --check`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` exit 0.

## Out of scope

- In-app rename/move (CAP-0013 outcome 1).
- Changing core/CLI absorb for `Workspace::reassociate_document`.
- Batch reassociate.
- Rescan suggestions (CAP-0013 outcome 5).
- Sticky/overlay Actions that stay in the viewport while the pane scrolls.
- Validating or warning when Browse closes or when the path field blurs.
