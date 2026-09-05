# CHG-0049 — Visible selection-pane action feedback

A selected document's action outcome must stay visible even when the section that formerly hosted the error is folded. One alert appears above the foldable topics and the initiating section opens.

**Plan ID:** CHG-0049-visible-selection-pane-action-feedback
**Created:** 2026-08-31
**Depends on:** CHG-0048 candidate submission outcome feedback
**Context sources:** `AGENTS.md`; `docs/AGENTS.md`; `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; CAP-0002 Outcome 4; CAP-0006 Outcome 6; CAP-0015 Outcomes 11 and 14; `crates/dms-desktop/ui/{app,library}.mjs`; `crates/dms-desktop/ui/{app,library}.test.mjs`.
**Produces:** Visible selected-document feedback that opens the initiating section and never hides a lifecycle validation failure behind folded Document control data.
**Status:** done — visible feedback, CAP-linked wireframes, and workspace gate passed

| Field | Value |
| --- | --- |
| ID | CHG-0049 |
| Status | done — visible feedback, CAP-linked wireframes, and workspace gate passed |
| External request | Direct operator request: I've retried again. The point is, that there was an error message "configuration field document type cannot be empty" for the document but hidden behind the "Document control data" section which was folded. This was the reason for a missing respond, because the pane scrolled up -- it look like -- but the section "Document control data" was not unfolded so the user cound not see the error message. This is a UI concept issue |
| Affected CAPs | CAP-0002, CAP-0006, CAP-0015 |
| Decision records | none |

## Root cause

`lifecycleFailureLibraryState` stores the failed candidate request in `library.detail_error`. `selectionMarkup` rendered that property only inside the folded **Document control data** form. It neither recorded the action that failed nor opened **Revision cycle**. The operator saw a selection-pane refresh but could not see the failure or determine the next action.

## Design rule

Every selected-document mutation renders one visible alert above the foldable topics and opens the section that initiated it. A later selection, folder snapshot, or successful action clears stale feedback.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Carry visible feedback and open its initiating section | done (`node --test crates/dms-desktop/ui/*.test.mjs` — 135 passed) | Focused UI tests prove a failed V1.0 candidate opens Revision cycle with its exact error, while document-control, schedule, confidentiality, and reassociation failures open their own sections. |
| 2 | Publish contracts and review screens | done (`node docs/product/wireframes/generate.mjs`; 1600×1600 Chrome PNGs for CAP-0002, CAP-0006, and CAP-0015) | Regenerate CAP-0002, CAP-0006, and CAP-0015 HTML/PNG review artifacts. |
| 3 | Run workspace gate and close | done (`cargo fmt`, workspace clippy/test, 135 desktop UI tests, and `git diff --check`) | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0. |

## Phase 1 — Carry feedback with its initiating section

Replace the hidden lifecycle error with session-only selected-document feedback. Render one alert above the foldable topics, force the initiating section open after a failure, and retain form drafts for retry. Successful candidate notices remain in Revision cycle. Do not change core validation, candidate persistence, advisory locks, or approval routing.

## Phase 2 — Publish contracts and review screens

Amend CAP-0002, CAP-0006, CAP-0015, and the desktop DOX contract. Show an inline Revision cycle error state in matching wireframes. Regenerate generated artifacts; do not hand-edit them.

## Phase 3 — Workspace gate and close

Run the workspace gate, archive this record, and refresh the changes index only after every gate passes.
