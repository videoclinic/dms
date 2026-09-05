# CHG-0047 — Candidate approver route and validation feedback

For an approval-required release candidate, the Revision cycle form makes the resolved workflow approver visible and explains where that routing can be changed. Candidate creation does not silently fail when required form values are absent: it returns the existing explicit validation error into the selection pane.

**Plan ID:** CHG-0047-candidate-approver-route-and-validation-feedback
**Created:** 2026-08-31
**Depends on:** CHG-0038 candidate requester and target-outcome semantics
**Entry checkpoint:** CAP-0002/CAP-0015 distinguish the candidate requester from the workflow-selected approver.
**Context sources:** `AGENTS.md`; `docs/AGENTS.md`; `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; CAP-0002 Outcomes 4–5; CAP-0006 Outcome 6; CAP-0015 Outcomes 8, 11, 14; CAP-0019 Outcomes 4–7; `crates/dms-desktop/ui/library.mjs`; `crates/dms-desktop/ui/app.mjs`; `crates/dms-desktop/ui/library.test.mjs`; `docs/product/wireframes/generate.mjs`.
**Produces:** A read-only effective-approver route in the candidate form, an explicit Configuration → Workflow change boundary, and selection-pane feedback for candidate validation errors.
**Status:** done — candidate routing/validation, CAP-linked wireframes, and workspace gate passed

| Field | Value |
| --- | --- |
| ID | CHG-0047 |
| Status | done — candidate routing/validation, CAP-linked wireframes, and workspace gate passed |
| External request | Direct operator request: While clicking on "Create release candidate" in "Create release candidate" (for V1.0) does nothing. Should the UI not offer to change the Approver (by showing the default approver preselected) ? |
| Affected CAPs | CAP-0002, CAP-0006, CAP-0015 |
| Decision records | none — CAP-0019 already assigns approver changes to folder policy or document override. |

## Current state

- The candidate form's required HTML controls can prevent its `submit` event, leaving the existing `lifecycleActionRequest` validation messages unreachable from the selection pane.
- The form identifies its editable person as the requester and says routing selects the approver, but does not show the resolved workflow approver beside this action.
- CAP-0015 permits approver changes only through workflow folder policy or document override. A candidate-time selector would create an unsafe alternate routing path and would permit a misleading mid-flight approver swap.

## UX decision

Do not add an editable approver picker to **Create release candidate**. The candidate form shows the resolved effective approver as a read-only approval route and directs changes to **Configuration → Workflow**. The existing requester picker remains the only editable identity in this form.

The form opts out of browser-native blocking so its established request validation returns a concrete error in the selected document pane. It retains the visible required-field affordances and sends no IPC command when values are invalid.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Make candidate routing and validation feedback explicit | done (`node --test crates/dms-desktop/ui/library.test.mjs` — 33 passed) | `node --test crates/dms-desktop/ui/library.test.mjs` exits 0 with read-only approver-route, no picker, and missing-field validation assertions. |
| 2 | Publish CAP and wireframe evidence | done (`node docs/product/wireframes/generate.mjs`; 1600×1600 Chrome PNGs for CAP-0002, CAP-0006, and CAP-0015) | `node docs/product/wireframes/generate.mjs` plus 1600×1600 headless-Chrome renders for CAP-0002, CAP-0006, and CAP-0015 exit 0. |
| 3 | Workspace gate and close | done (`cargo fmt`, workspace clippy/test, 129 desktop UI tests, and `git diff --check`) | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0. |

## Phase 1 — Make candidate routing and validation feedback explicit

**Goal:** An approval-required candidate visibly names its workflow-selected approver without permitting a candidate-time change; an incomplete submission reports its specific issue in the selection pane.

1. Render a read-only **Approval route** with the resolved effective approver, or explicit unresolved state, in the candidate form. State that changes belong in **Configuration → Workflow**.
2. Do not add an approver field to the candidate command payload or alter core lifecycle policy, role persistence, notification recipients, or approval evidence.
3. Allow candidate form submission to reach `lifecycleActionRequest` even when HTML-required fields are absent. Preserve its existing validation and ensure no IPC request is built for invalid data.
4. Add focused frontend assertions for the route/change boundary and all required candidate values.

Verification gate: `node --test crates/dms-desktop/ui/library.test.mjs`.

## Phase 2 — Publish CAP and wireframe evidence

**Goal:** Product contracts and review screens make the same requester/approver boundary and invalid-submission feedback visible.

1. Amend CAP-0002, CAP-0006, and CAP-0015 without changing approval routing ownership.
2. Update the CAP-0002, CAP-0006, and CAP-0015 generator definitions; regenerate HTML, manifest, index, and PNGs. Do not hand-edit generated outputs.
3. Refresh `crates/dms-desktop/AGENTS.md` if its candidate-form contract changes.

Verification gate: `node docs/product/wireframes/generate.mjs` and matching Chrome renders.

## Phase 3 — Workspace gate and close

**Goal:** Verify the vertical slice, archive this receipt, and refresh the change index.

1. Run the workspace gate and `git diff --check`.
2. Mark completed phases with evidence, set this CHG done, move it to `archive/`, and update `docs/changes/README.md`.

## Out of scope

- Candidate-time workflow-role assignment, multiple approvers, or a mid-review approver swap.
- Changing the requester defaulting rule, target-version policy, notification delivery, or approval decision flow.
