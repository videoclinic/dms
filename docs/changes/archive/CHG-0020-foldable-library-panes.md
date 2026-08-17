# CHG-0020 — Foldable Library side panes

**Plan ID:** CHG-0020-foldable-library-panes
**Created:** 2026-08-17
**Depends on:** none
**Context sources:** `docs/product/capabilities/CAP-0006-library-explorer.md` (Library layout), `crates/dms-desktop/ui/library.mjs` (`libraryMarkup`, `createLibraryState`), `crates/dms-desktop/ui/styles.css` (`.library-grid`, `.folder-tree`, `.selection-pane`, `.library-splitter`)
**Produces:** The Library exposes Fold left (folder tree) and Fold right (selection pane) controls in the path toolbar; folding collapses the matching pane, hides its splitter, and lets the centre folder-contents surface fill the freed space while a one-click re-open restores the prior session width. Folding a pane with a focused child keeps it focused so re-opening does not lose state.
**Status:** done — Fold left/right icon controls live in the Library path toolbar; both side panes (folder tree and selection details) hide their matching drag splitter and the centre folder-contents column fills the freed space; archived after workspace gate

| Field | Value |
| --- | --- |
| ID | CHG-0020 |
| Status | done |
| External request | Direct operator request: The left and right panes of the library view should be able to be folded left/right so the directory view stays in the middle |
| Affected CAPs | CAP-0006 |
| Decision records | none |

## Current state

- The library surface renders three horizontal regions in `library-grid`: `.folder-tree` (left), `.folder-contents` (centre, `flex: 1 1 480px`), and `.selection-pane` (right), separated by two 7 px drag splitters.
- Each side pane has its own session width (`tree_width`, `detail_width`) but no way to fully hide it. The minimum widths (`170 px` tree, `280 px` detail) keep the splitters from collapsing the panes past the bounds.
- A user wanting the directory view alone must drag each splitter to its minimum and still tolerates the remaining chrome.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Fold state, toggle helpers, and markup | done (`node --test crates/dms-desktop/ui/library.test.mjs`: 30 passed) | `node --test crates/dms-desktop/ui/library.test.mjs` exits 0 |
| 2 | Click wiring and layout adaptation | done (`node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs`: 57 passed) | `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs` exits 0 |
| 3 | Workspace gate and record closeout | done (`cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; frontend 98/98; link check 0; `git diff --check`; CHG archived as done) | Rust/frontend/link/diff checks exit 0; CHG archived as done |

## Phase 1 — Fold state, toggle helpers, and markup

**Goal:** Library state gains `tree_folded` and `detail_folded` flags; `libraryMarkup` renders Fold left / Fold right controls in the path toolbar, hides the matching aside + splitter when folded, and emits accessible labels that reflect the action.

Steps:

1. Add `tree_folded: false, detail_folded: false` to `createLibraryState`.
2. Add `toggleLibraryPaneFold(library, side)` and `isLibraryPaneFolded(library, side)` helpers that flip the flag and return an updated library.
3. In the library toolbar, render two new icon buttons `data-library-fold="tree"` and `data-library-fold="detail"` after the refresh control, with `aria-pressed` reflecting the folded state and the same `Expand/Collapse …` accessible names used elsewhere.
4. When a pane is folded, omit its `<aside>` and its splitter from the markup; otherwise render normally. The centre `.folder-contents` already uses `flex: 1 1 auto` so it should grow into the freed space.
5. Add focused unit tests covering: defaults, toggle flip, `aria-pressed` on the fold buttons, and that the folded pane + splitter are absent from the markup.

**Verification gate:** `node --test crates/dms-desktop/ui/library.test.mjs` exits 0.

## Phase 2 — Click wiring and layout adaptation

**Goal:** A click on each fold button toggles the matching session flag and re-renders; re-opening a folded pane keeps its last `tree_width` / `detail_width` and the splitter drag still works when the pane is open.

Steps:

1. Add `data-library-fold` click handling in `handleLibraryClick` that invokes the helper and renders.
2. Update the pointer-drag handlers so the tree / detail splitters never start a drag while the matching pane is folded.
3. Update `libraryDetailMaximum` / `libraryTreeMaximum` (or the new layout path) so the centre column sees the full grid width minus only the open panes plus open splitters.
4. Add focused tests covering click dispatch through the rendered markup and the absence of drag-affordance attributes while folded.

**Verification gate:** `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs` exits 0.

## Phase 3 — Workspace gate and record closeout

**Goal:** Rust, frontend, link, and diff checks all pass and the change becomes an archived receipt describing the proven behaviour.

Steps:

1. Run the full verification gate.
2. Update `docs/product/capabilities/CAP-0006-library-explorer.md` outcomes (and any wireframe that shows the library surface) to describe foldable side panes with one-click re-open.
3. Update `crates/dms-desktop/AGENTS.md` "Local Contracts" with the new fold rules.
4. Move this CHG to `docs/changes/archive/` and update `docs/changes/README.md`.

**Verification gate:** the full workspace gate command exits 0; CHG moves to `archive/`.

## Out of scope

- Animating the fold transition.
- Persisting the folded state across sessions (per session only, like the existing splitter widths).
- Reordering the centre column to occupy the full window (toolbar/back/forward/refresh stay in the path toolbar).
- Hiding the back/forward/refresh controls; folding the centre column is not requested.
