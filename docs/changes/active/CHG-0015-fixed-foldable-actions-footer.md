# CHG-0015 — Fixed foldable Actions footer

**Plan ID:** CHG-0015-fixed-foldable-actions-footer
**Execution slot:** P0100
**Created:** 2026-08-16
**Depends on:** CHG-0014-reassociate-source-topic-visibility
**Entry checkpoint:** CHG-0014 is archived as done; `92efa51` includes its runtime and wireframe baseline.
**Context sources:** `docs/product/capabilities/CAP-0006-library-explorer.md#outcomes` (outcomes 2, 6, and 8), `docs/product/capabilities/CAP-0015-document-control-data.md#outcomes` (outcome 11), `docs/changes/archive/CHG-0014-reassociate-source-topic-visibility.md#pane-layout-locked`, `crates/dms-desktop/AGENTS.md#local-contracts`, `crates/dms-desktop/ui/library.mjs` (`DEFAULT_SELECTION_OPEN`, `selectionMarkup`, `libraryMarkup`), `crates/dms-desktop/ui/styles.css` (`.selection-pane`, `.selection-actions-block`), `crates/dms-desktop/ui/library.test.mjs` (single-document markup and fold-state tests), `crates/dms-desktop/ui/app.test.mjs` (`shell and Library panes contain scrolling without moving navigation`), `docs/product/wireframes/generate.mjs` (`lostSourceSelectionPane`, `documentControlDataSelectionPane`)
**Produces:** A single-document Library selection pane whose main details scroll above a bottom-docked, independently foldable Actions footer, with matching CAPs, tests, DOX, and CAP-0006/CAP-0015 wireframes.
**Status:** in-progress — Phase 2 gate passed; Phase 3 not started

Restructure the single-document Library selection pane so document details scroll independently above a bottom-docked Actions disclosure whose header never leaves the pane viewport.

| Field | Value |
| --- | --- |
| ID | CHG-0015 |
| Status | in-progress |
| External request | Direct operator request: Redesign the position of the "Acionts" section in the "document control data" pane: fix the "Actions" on the bottom on the "document control data" pane so that scrolling within the "document control data" pane does not move the "Actions"; Actions should be foldable to save space if neede |
| Affected CAPs | CAP-0006, CAP-0015 |
| Decision records | none |

## Current state

- `crates/dms-desktop/ui/library.mjs` defaults Actions open in `DEFAULT_SELECTION_OPEN` and emits one `.selection-scroll` plus a sibling `<details class="selection-actions-footer" data-library-section="actions">` for a single registered document. Empty, loading, folder, unsupported, not-in-library, and multi-selection states stay in the scroller without an Actions footer.
- `crates/dms-desktop/ui/styles.css` makes `.selection-pane` a non-scrolling flex column; `.selection-scroll` is the growing overflow region; the Actions footer is a non-overlay bottom sibling capped at half pane height with an independently scrollable body.
- CAP-0006 outcomes 2 and 6, CAP-0015 outcome 11, and `crates/dms-desktop/AGENTS.md` require that layout and session-only Actions fold state.
- Focused tests: `node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs` 53/53.
- Wireframes: CAP-0006 Lost-source and CAP-0015 healthy-document panes use one `.selection-scroll` plus an open `.selection-actions-footer`. Native 1600×1600 Chrome exports are non-empty.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Contract, runtime layout, and focused tests | done (`node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs` 53/53) | `node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs` exits 0; tests prove Actions is the foldable sibling after the sole details scroller and its state survives document switches |
| 2 | CAP-linked wireframes and visual audit | done (`node generate.mjs` + 1600×1600 CAP-0006/CAP-0015 Chrome exports) | Wireframe generation and 1600×1600 CAP-0006/CAP-0015 Chrome exports exit 0; both non-empty PNGs show the Actions summary docked at the pane bottom without overlap or clipping |
| 3 | Workspace gate and change-record closeout | pending | Full Rust/frontend, link, and diff checks exit 0; CHG-0015 is archived as done and the change index has no active row for it |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, `pending` otherwise.

## Phase 1 — Contract, runtime layout, and focused tests

**Goal:** The implemented selection pane and CAP contracts agree that only the main document details scroll while a foldable Actions footer remains docked at the bottom.

Steps:

1. Add failing coverage before changing markup or CSS:
   - restore `actions: true` in `DEFAULT_SELECTION_OPEN` and prove Actions is expanded by default;
   - prove single-document markup contains exactly one `.selection-scroll` region followed by one `<details class="selection-actions-footer" data-library-section="actions">` sibling;
   - prove Document control data, Document review schedule, Revision cycle, and Releases are inside `.selection-scroll`, while every document action and the Lost-source reassociate form is inside the Actions footer;
   - extend the document-switch fold-state test so an operator-collapsed Actions footer remains collapsed across document switches for the open Library activity;
   - update the CSS contract test to require `.selection-pane` as a non-scrolling flex column, `.selection-scroll` as the growing `overflow: auto` region, and the Actions footer as the non-overlay bottom sibling with an independently scrollable expanded body.
2. Refactor `selectionMarkup` / `libraryMarkup` in `crates/dms-desktop/ui/library.mjs` so the single registered-document state emits two direct children of `.selection-pane`: the main `.selection-scroll` content and the Actions `<details>` footer. Keep empty, loading, folder, unsupported, not-in-library, and multi-selection states in the ordinary selection scroller without inventing an empty document Actions footer.
3. Reuse the existing disclosure chevron, Expand/Collapse cue, `selectionSectionOpen`, and `setSelectionSectionOpen` behavior for the Actions summary. Default Actions to open and keep its state session-only alongside the other section states; do not persist it in `.dms`, OS preferences, or saved views.
4. In `crates/dms-desktop/ui/styles.css`, make `.selection-pane` a bounded flex column with `overflow: hidden`; make `.selection-scroll` the only main-detail vertical scroller; render the Actions footer as a normal flex sibling at the bottom rather than `position: sticky`, `fixed`, or an overlay. Cap the expanded footer at half the pane height and let its body scroll so the footer cannot starve or cover the main details in a short window. Keep the summary visible when the footer is collapsed or its body scrolls.
5. Preserve every existing action, order, enabled/disabled predicate, Lost-source reassociate rule, inline error, and command mapping. Moving and folding the container must not change authority semantics.
6. Amend CAP-0006 outcomes 2 and 6 and CAP-0015 outcome 11: the right pane contains a main document-detail scroll region plus a bottom-docked Actions disclosure; Actions is expanded by default, independently foldable, and uses the existing session-only state across document switches. Remove the superseded claims that the whole selection pane scrolls and that Actions is non-foldable immediately after Document control data.
7. Amend `crates/dms-desktop/AGENTS.md` with the same durable layout and session-state contract. Leave root, `crates/AGENTS.md`, and docs AGENTS files unchanged because ownership boundaries and parent contracts do not change.

**Verification gate:** `node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs` exits 0 with the structural, CSS, default-open, state-retention, healthy-document, and Lost-source assertions listed above.

## Phase 2 — CAP-linked wireframes and visual audit

**Goal:** The current CAP-0006 Lost-source and CAP-0015 healthy-document wireframes visibly demonstrate the same bottom-docked foldable Actions layout as runtime.

Steps:

1. Update shared wireframe styles in `docs/product/wireframes/generate.mjs` so each single-document detail pane is a bounded flex column with one scrolling detail body and a non-overlay foldable Actions footer. Use the same chevron and Expand/Collapse affordance as the other topics.
2. Restructure `lostSourceSelectionPane` so its Lost-source banner, source identity, and document topics scroll above the footer while Browse and Reassociate source remain inside Actions.
3. Restructure `documentControlDataSelectionPane` so Document control data, Document review schedule, Revision cycle, and Releases scroll above the footer. Keep Actions open in the primary screenshot and show enough of the details region to make the separate scroll boundary legible.
4. Run `node generate.mjs` from `docs/product/wireframes/`, then render fresh CAP-0006 and CAP-0015 PNGs at 1600×1600 with repository-local `google-chrome`.
5. Inspect both native-size exports for direct parentage, bottom docking, disclosure visibility, content occlusion, nested-scroll usability, overlap, clipping, broken glyphs, destructive-action distinction, and the Lost-source Browse/Reassociate path. Reject a result where Actions only appears at the bottom of normal document flow.

**Verification gate:** `(cd docs/product/wireframes && node generate.mjs && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0006-library-explorer.png "file://$PWD/html/CAP-0006-library-explorer.html" && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0015-document-control-data.png "file://$PWD/html/CAP-0015-document-control-data.html" && test -s exports/CAP-0006-library-explorer.png && test -s exports/CAP-0015-document-control-data.png)` exits 0; native-size visual inspection confirms the predicates in step 5 and the generated HTML keeps the Actions footer outside the sole main-detail scroller.

## Phase 3 — Workspace gate and change-record closeout

**Goal:** Repository-wide checks pass and the completed behavior is recorded only in current CAPs plus an archived CHG receipt.

Steps:

1. Re-run the focused frontend tests, then run all frontend and Rust workspace gates.
2. Run strict Markdown link validation from the repository root, inspect every changed Markdown table, and run `git diff --check`. The checker must scan `.` so links from `docs/` to `crates/` remain inside its resolution scope.
3. Perform the DOX closeout against every changed path. Confirm `crates/dms-desktop/AGENTS.md` owns the runtime contract, CAP-0006/CAP-0015 own current behavior, wireframe ownership/indexes remain unchanged, and no parent AGENTS update is needed.
4. Record exact phase evidence, set the CHG and every phase to done, move this file to `docs/changes/archive/`, and move its `docs/changes/README.md` row from Active to Archive in the same change.
5. Hand the completed diff to the operator without committing or pushing unless explicitly requested.

**Verification gate:** `cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/*.test.mjs && python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary . && git diff --check` exits 0; `test ! -e docs/changes/active/CHG-0015-fixed-foldable-actions-footer.md && test -s docs/changes/archive/CHG-0015-fixed-foldable-actions-footer.md` exits 0 after closeout; the archive row is the only CHG-0015 index entry.

## Out of scope

- Changing which actions exist, their order, command mapping, authority, confirmation, or enabled/disabled rules.
- Changing Lost-source reassociate applicability, Browse behavior, supported formats, submit-time validation, or error copy.
- Docking batch, folder, unsupported-file, or not-in-library selection actions; those non-document-control states remain in their ordinary selection scroller.
- Persisting Actions fold state outside the current Library activity.
- Using a sticky/fixed overlay that can cover document details; the footer is a flex sibling below the main scroller.
- Changing `dms-core`, the CLI, workspace metadata, or any schema.
