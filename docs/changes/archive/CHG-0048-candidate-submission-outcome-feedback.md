# CHG-0048 — Candidate submission outcome feedback

Candidate submission must visibly state the resulting workflow state instead of merely replacing the Revision cycle form with refreshed document data.

**Plan ID:** CHG-0048-candidate-submission-outcome-feedback
**Created:** 2026-08-31
**Depends on:** CHG-0047 candidate form validation and approval-route clarity
**Context sources:** `AGENTS.md`; `docs/AGENTS.md`; `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; CAP-0002 Outcome 4; CAP-0006 Outcome 6; CAP-0015 Outcome 14; `crates/dms-desktop/ui/app.mjs`; `crates/dms-desktop/ui/library.mjs`; `crates/dms-desktop/ui/{app,library}.test.mjs`.
**Produces:** A selection-pane Revision cycle success notice that names the created candidate version and its workflow outcome.
**Status:** done — runtime feedback, CAP-linked wireframes, and workspace gate passed

| Field | Value |
| --- | --- |
| ID | CHG-0048 |
| Status | done — runtime feedback, CAP-linked wireframes, and workspace gate passed |
| External request | Direct operator request: The library is now advisory locked. I've clicked on "Create release candidate" but did not get any feedback but the pane reloaded |
| Affected CAPs | CAP-0002, CAP-0006, CAP-0015 |
| Decision records | none |

## Current state

- A successful `submit_document_candidate` refreshes `DocumentSelection`; the form disappears because an active candidate now exists and version history opens.
- The refreshed selection had no explicit success feedback, so a valid V1.0 review request looked like an inert pane reload.
- A failed lifecycle request stores `detail_error`, while success had no equivalent selection-scoped state.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Render outcome-specific candidate success feedback | done (`node --test crates/dms-desktop/ui/*.test.mjs` — 133 passed; `cargo test -p dms-desktop --lib` — 89 passed; clippy clean) | Candidate success covers approval requested, delivery failed, and approval-optional draft outcomes; selection/folder/lifecycle transitions clear stale feedback. |
| 2 | Publish contracts and matching review screens | done (`node docs/product/wireframes/generate.mjs`; 1600×1600 Chrome PNGs for CAP-0002, CAP-0006, and CAP-0015) | Regenerate CAP-0002, CAP-0006, and CAP-0015 HTML/PNG review artifacts. |
| 3 | Run workspace gate and close | done (`cargo fmt`, workspace clippy/test, 133 desktop UI tests, and `git diff --check`) | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0. |

## Phase 1 — Render outcome-specific candidate success feedback

The frontend adds a session-only `lifecycle_notice` to the Library state. A successful candidate submission writes one of these explicit outcomes:

- **Pending approval:** V1.0 or another approval-required candidate was submitted and the document entered Pending approval.
- **Delivery failed:** the candidate was recorded but approval delivery did not advance the workflow; the operator must confirm or retry delivery.
- **Direct release:** an approval-optional candidate was created and remains Draft until explicit export and release.

The notice is rendered at the top of Revision cycle, has `role="status"`, and clears on document selection, folder snapshot, failure, or a later lifecycle action. It does not alter candidate persistence, notification delivery, approval routing, or lifecycle transitions.

## Phase 2 — Publish contracts and matching review screens

Amend CAP-0002, CAP-0006, CAP-0015, and the desktop DOX contract. Update CAP-0002, CAP-0006, and CAP-0015 wireframe generator definitions to show post-submission feedback. Regenerate HTML, manifest/index, and PNGs. Do not hand-edit generated artifacts.

## Phase 3 — Workspace gate and close

Run the workspace gate, archive this record, and refresh the change index only after every gate passes.
