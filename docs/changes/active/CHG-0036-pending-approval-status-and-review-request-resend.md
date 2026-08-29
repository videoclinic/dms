# CHG-0036 — Pending-approval status and review-request resend

Show an approval-required document as **Pending approval** throughout the Library, and let an operator resend the existing review request to its snapshotted approver without creating a new candidate, changing its review identity, or changing the document lifecycle.

**Plan ID:** CHG-0036-pending-approval-status-and-review-request-resend
**Execution slot:** P0500
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `docs/product/capabilities/CAP-0002-document-lifecycle.md` (Outcomes 3–6); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcomes); `docs/product/capabilities/CAP-0010-notification-transport.md` (Outcomes 2, 4, 8); `docs/product/capabilities/CAP-0011-approval-evidence.md` (Outcomes 1, 4, 8); `docs/design-decisions.md` (ADR-0004, ADR-0013); `crates/AGENTS.md`; `crates/dms-core/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/lifecycle.rs` (`Workspace::submit_candidate`, `Workspace::retry_review_notification`, `Workspace::resend_review_notification`, `WorkflowEventType`, `CandidateStatus`); `crates/dms-core/src/audit.rs` (`event_type_text`, `Workspace::audit_rows`); `crates/dms-core/tests/lifecycle.rs`; `crates/dms-desktop/src/lib.rs` (candidate notification commands and `DocumentSelection`); `crates/dms-desktop/ui/library.mjs` (`lifecyclePanelMarkup`, `lifecycleLabel`, selection header); `crates/dms-desktop/ui/library.test.mjs`; `docs/product/wireframes/generate.mjs`
**Produces:** An active approval-required candidate visibly reads **Pending approval** in the Library, offers a deliberate **Resend approval request** action, and retains canonical evidence of every resend attempt while preserving the original candidate and review target.
**Status:** in-progress — Phase 2 done; Phase 3 pending.

| Field | Value |
| --- | --- |
| ID | CHG-0036 |
| Status | in-progress |
| External request | Direct operator request: "create a change as recommended. The approval request should also be able to be resend" |
| Affected CAPs | CAP-0002, CAP-0006, CAP-0010, CAP-0011 |
| Decision records | ADR-0004 and ADR-0013 remain applicable; no new ADR is required. This extends the existing operator-maintained review and canonical-event contracts. |

## Current state

- A successfully delivered approval-required candidate moves the document lifecycle and candidate status to `in_review`; the failed-delivery path leaves the candidate retryable in `review_delivery_failed` and the document in `draft`.
- `Workspace::retry_review_notification` still accepts only `review_delivery_failed` and, on successful delivery, establishes `InReview` with a `review_requested` event.
- `Workspace::resend_review_notification` accepts only the active approval-required `InReview` candidate whose source digest still matches. It reuses the snapshotted review ID, digest, approver, and permalink, appends the delivery attempt, and records `review_request_resent` for accepted, confirmed, and failed sends without leaving `in_review`.
- The Library table Lifecycle column and selected-document badge show **Pending approval** for persisted `in_review`. Sort and API values remain `in_review`.
- An active approval-required `in_review` candidate exposes **Resend approval request** beside the decision controls. SMTP sends immediately; `mailto:` keeps the sent-confirmation control. A failed attempt shows the delivery error and remains pending. Delivery-failed, minor, decided, cancelled, and invalidated candidates do not expose the action.
- CAP-0002 currently promises only failed-send redelivery, while CAP-0010 requires workflow history to retain review-request delivery evidence.

## Risk call-out

A resend must not replace the existing candidate, create a second review ID, recalculate the reviewed source digest, rerun content checks, change the snapshotted approver/recipient, or transition the document out of `in_review`. It sends the same canonical review request for the active candidate only.

Delivery is an external side effect. SMTP may fail and `mailto:` requires explicit operator confirmation; either outcome must leave the active review pending and be traceable without claiming that a new approval request or decision exists. The recovery path for an incorrect implementation is the existing workspace backup and the immutable canonical history: do not edit, delete, or rehash prior events.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Preserve review identity while recording resend evidence | done (`cargo test -p dms-core --test lifecycle` — 29 passed) | `cargo test -p dms-core --test lifecycle` exits 0, proving successful and failed resend attempts preserve candidate ID/review ID/digest/approver/lifecycle and append the distinct canonical resend event |
| 2 | Expose Pending approval and deliberate resend in DMS Desktop | done (`cargo test -p dms-desktop --lib` — 89 passed; `node --test crates/dms-desktop/ui/library.test.mjs` — 32 passed) | `cargo test -p dms-desktop --lib` and `node --test crates/dms-desktop/ui/library.test.mjs` exit 0, proving only an active `in_review` approval candidate exposes the resend action and the library never presents a raw `in_review` status to an operator |
| 3 | Publish capability contracts and Library wireframe, then close the change | pending | `node docs/product/wireframes/generate.mjs`, the CAP-0002 PNG render command, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, the repository Markdown-link check, and `git diff --check` all exit 0; CAP/CHG indexes agree |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Preserve review identity while recording resend evidence

**Goal:** The core can resend the review notification for one active `in_review` approval candidate and records each send attempt as immutable evidence without reopening or replacing its review.

Steps:

1. Add an explicit core resend operation separate from failed-delivery retry. It accepts only the active approval-required candidate in `InReview`; it rejects candidates that are draft, delivery-failed, decided, released, invalidated, cancelled, or no longer current.
2. Reuse the original candidate's snapshotted target, requester, approver/recipient, review permalink, source digest, and review ID. Do not allocate a candidate ID or review ID, rerun content conformance, refresh workflow people, or alter lifecycle state.
3. Record every resend attempt in `delivery_attempts`. Add a distinct canonical `review_request_resent` workflow event with the delivery receipt for both accepted/confirmed and failed attempts; extend event rendering and audit export mapping without changing historic event bodies or hashes.
4. Preserve existing failed-delivery retry behavior. A retry that successfully delivers the original request continues to establish `InReview`; the new resend action is only for an already active review.
5. Add focused lifecycle tests for accepted SMTP resend, failed SMTP resend, confirmed `mailto:` resend, and ineligible-status rejection. Assert unchanged candidate/review IDs, source digest, approver snapshot, lifecycle, and that retry and resend events remain distinguishable in workflow history and audit rows.

**Verification gate:** `cargo test -p dms-core --test lifecycle` exits 0 with explicit assertions for retained review identity, retained `in_review` lifecycle, and canonical resend-attempt evidence.

## Phase 2 — Expose Pending approval and deliberate resend in DMS Desktop

**Goal:** Library operators can tell that a document awaits approval and deliberately resend its current review request, while non-active reviews cannot expose the action.

Steps:

1. Map the `in_review` lifecycle to **Pending approval** in the Library table and selected-document badge. Keep `in_review` as the persisted/API enum; do not change lifecycle serialization, sort value, or the underlying transition rules.
2. For an active approval-required candidate in `in_review`, show **Resend approval request** alongside the decision controls, with a short explanation that it repeats the existing request to the snapshotted approver and does not reset approval. Do not show it for minor candidates, delivery-failed retries, decisions, releases, cancelled or invalidated candidates.
3. Add the narrow desktop command and request mapping. SMTP sends immediately; `mailto:` retains an explicit sent-confirmation control before the resend is recorded as confirmed. A failed resend surfaces the delivery error but keeps the document visibly pending approval and allows another resend.
4. Refresh `DocumentSelection` after a resend so the latest delivery attempt and `review_request_resent` entry are shown in **Version history & changes**. Maintain existing review-decision sign-in and authorization requirements.
5. Add adapter and frontend tests for the visible label, command payload, SMTP success/failure, mailto confirmation requirement, action visibility boundaries, and returned selection/history data.

**Verification gate:** `cargo test -p dms-desktop --lib` and `node --test crates/dms-desktop/ui/library.test.mjs` exit 0, proving exact **Pending approval** and **Resend approval request** UI copy plus the active-review-only boundary.

## Phase 3 — Publish capability contracts and Library wireframe, then close the change

**Goal:** Current product records and the lifecycle screen describe pending approval and resending as implemented behavior, with one generated visual reference and a closed CHG receipt.

Steps:

1. Amend CAP-0002 to name **Pending approval** as the operator-facing representation of `in_review`, and to require resend to preserve the active review's candidate, review ID, digest, target, and approver.
2. Amend CAP-0006 to require the Library Lifecycle column and selection badge to use the user-facing pending-approval label for `in_review`.
3. Amend CAP-0010 to define review-request resend delivery behavior for SMTP and `mailto:`, including the retained recipient snapshot and each attempt's delivery evidence. Amend CAP-0011 with `review_request_resent`, its immutable delivery record, and its in-app history presentation.
4. Update the existing CAP-0002 wireframe definition with synthetic pending-approval normal, resend-success, and resend-failure/mailto-confirmation states. Regenerate HTML/index/manifest and render the CAP-0002 PNG. Do not add a proposal-only wireframe to the product inventory.
5. Run all gates. After each passes, record evidence, move this CHG to `docs/changes/archive/`, and update `docs/changes/README.md` from Active to Archive.

**Verification gate:** `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0002-document-lifecycle.png "file://$PWD/html/CAP-0002-document-lifecycle.html" && test -s exports/CAP-0002-document-lifecycle.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .`, and `git diff --check` all exit 0.

## Out of scope

- Changing which target versions require approval, workflow-role selection, approver sign-in, or the decision/release transition rules.
- Sending a new request to a different approver, modifying the existing candidate, or starting a new review from the resend action.
- Automatic resend, scheduled reminders, bulk resend, delivery read receipts, or browser/email approval decisions.
- Rewriting historic `review_requested` events, delivery attempts, or workflow hashes.
