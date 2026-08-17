# CHG-0016 — Actions footer full height

**Plan ID:** CHG-0016-actions-footer-full-height
**Execution slot:** P0101
**Created:** 2026-08-17
**Depends on:** CHG-0015-fixed-foldable-actions-footer
**Entry checkpoint:** CHG-0015 is archived as done; `77182ec` includes its runtime and wireframe baseline.
**Context sources:** `docs/product/capabilities/CAP-0006-library-explorer.md#outcomes` (outcomes 2 and 6), `docs/product/capabilities/CAP-0015-document-control-data.md#outcomes` (outcome 11), `crates/dms-desktop/AGENTS.md#local-contracts`, `crates/dms-desktop/ui/styles.css` (`.selection-actions-footer`), `crates/dms-desktop/ui/app.test.mjs` (`shell and Library panes contain scrolling without moving navigation`), `docs/product/wireframes/generate.mjs` (`.selection-actions-footer`)
**Produces:** A bottom-docked Actions footer whose summary stays fully visible and whose expanded body shows every action, with matching CAPs, tests, DOX, and CAP-0006/CAP-0015 wireframes.
**Status:** done — heading-visible, show-all-actions footer shipped; archived after workspace gate

Keep the CHG-0015 bottom-docked foldable Actions placement. Remove the half-pane height cap that clips the heading and hides actions.

| Field | Value |
| --- | --- |
| ID | CHG-0016 |
| Status | done |
| External request | Direct operator request: Positioning of "Actions" was right, but the element is too small in height to see all elements. The height should at least show the "Actions" heading in full height. Unfolding "Actions" should show all elements. |
| Affected CAPs | CAP-0006, CAP-0015 |
| Decision records | none |

## Current state

- Actions remains a bottom-docked foldable sibling of `.selection-scroll`.
- Runtime CSS sizes the footer to its content with a `2.75rem` heading min-height, `flex: 0 0 auto` when open, and `max-height: 100%` only as a last resort. The main details scroller shrinks first.
- CAP-0006 outcomes 2 and 6, CAP-0015 outcome 11, and `crates/dms-desktop/AGENTS.md` require the heading-visible, show-all-actions contract.
- Focused tests: `node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs` 53/53.
- Wireframes: CAP-0006 Lost-source and CAP-0015 healthy-document panes show the full Actions heading and every expanded action. Native 1600×1600 Chrome exports are non-empty.
- Workspace gate: `cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `node --test crates/dms-desktop/ui/*.test.mjs` 94/94; `check-md-links.py --format summary .` 0 issues.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Contract, CSS, and focused tests | done (`node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs` 53/53) | `node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs` exits 0; CSS tests require a heading-sized min-height, no half-pane or 18rem cap, a non-shrinking open footer, and last-resort body scroll only |
| 2 | CAP-linked wireframes and visual audit | done (`node generate.mjs` + 1600×1600 CAP-0006/CAP-0015 Chrome exports) | Wireframe generation and 1600×1600 CAP-0006/CAP-0015 Chrome exports exit 0; both non-empty PNGs show the full Actions heading and every expanded action |
| 3 | Workspace gate and change-record closeout | done (`cargo test --workspace`; `clippy -D warnings`; frontend 94/94; link check 0) | Full Rust/frontend, link, and diff checks exit 0; CHG-0016 is archived as done and the change index has no active row for it |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, `pending` otherwise.

## Phase 1 — Contract, CSS, and focused tests

**Goal:** The selection pane keeps Actions docked at the bottom, always shows the Actions heading at full height, and shows every action when unfolded.

Steps:

1. Update the CSS contract test so `.selection-actions-footer` has a heading-sized `min-height`, no `max-height: 50%`, no `[open]` 18rem cap, a non-shrinking summary and open footer, and a body that scrolls only as a last resort when the pane is shorter than the heading plus those actions.
2. In `crates/dms-desktop/ui/styles.css`, drop the half-pane cap and `min-height: 0`. Size the footer to its content. Keep the summary fully visible when collapsed or expanded. Let the main details scroller shrink first. Keep last-resort body overflow only when the pane cannot fit the heading plus every action.
3. Amend CAP-0006 outcomes 2 and 6 and CAP-0015 outcome 11 to require the heading-visible, show-all-actions height contract. Remove the superseded claim that an expanded Actions body is capped so it cannot cover the main details.
4. Amend `crates/dms-desktop/AGENTS.md` with the same height contract. Leave parent AGENTS files unchanged.

**Verification gate:** `node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs` exits 0 with the CSS assertions listed above.

## Phase 2 — CAP-linked wireframes and visual audit

**Goal:** The CAP-0006 Lost-source and CAP-0015 healthy-document wireframes show the same heading-visible, unclipped Actions footer as runtime.

Steps:

1. In `docs/product/wireframes/generate.mjs`, drop the `50%` and `18rem` caps. Give the footer a heading-sized min-height and let the open body show every action.
2. Run `node generate.mjs` from `docs/product/wireframes/`, then render fresh CAP-0006 and CAP-0015 PNGs at 1600×1600 with repository-local `google-chrome`.
3. Inspect both native-size exports for a fully visible Actions heading and every expanded action without a half-pane clip.

**Verification gate:** `(cd docs/product/wireframes && node generate.mjs && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0006-library-explorer.png "file://$PWD/html/CAP-0006-library-explorer.html" && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0015-document-control-data.png "file://$PWD/html/CAP-0015-document-control-data.html" && test -s exports/CAP-0006-library-explorer.png && test -s exports/CAP-0015-document-control-data.png)` exits 0.

## Phase 3 — Workspace gate and change-record closeout

**Goal:** Repository-wide checks pass and the completed behavior is recorded only in current CAPs plus an archived CHG receipt.

Steps:

1. Re-run the focused frontend tests, then run all frontend and Rust workspace gates.
2. Run strict Markdown link validation from the repository root and `git diff --check`.
3. Perform the DOX closeout against every changed path.
4. Record exact phase evidence, set the CHG and every phase to done, move this file to `docs/changes/archive/`, and move its `docs/changes/README.md` row from Active to Archive in the same change.
5. Hand the completed diff to the operator without committing or pushing unless explicitly requested.

**Verification gate:** `cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/*.test.mjs && python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary . && git diff --check` exits 0; `test ! -e docs/changes/active/CHG-0016-actions-footer-full-height.md && test -s docs/changes/archive/CHG-0016-actions-footer-full-height.md` exits 0 after closeout.

## Out of scope

- Changing Actions placement, foldability, default-open state, or session-only fold persistence.
- Changing which actions exist, their order, command mapping, authority, confirmation, or enabled/disabled rules.
- Changing Lost-source reassociate applicability, Browse behavior, supported formats, submit-time validation, or error copy.
- Docking batch, folder, unsupported-file, or not-in-library selection actions.
- Changing `dms-core`, the CLI, workspace metadata, or any schema.
