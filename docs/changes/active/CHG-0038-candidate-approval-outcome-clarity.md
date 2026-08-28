# CHG-0038 — Candidate, release, and Entra identity clarity

Make the Library candidate form state, for the currently selected target, whether **Create release candidate** starts approval or leaves the document ready for direct release; its requester picker uses the accurate **Choose requesting editor** placeholder and preselects the verified Entra session actor by exact object ID when one is available; make the Releases pane distinguish immutable owner, requester, editor, and approval evidence at release time.

**Plan ID:** CHG-0038-candidate-approval-outcome-clarity
**Execution slot:** P0700
**Created:** 2026-08-27
**Depends on:** CHG-0035-entra-session-required-for-group-bound-library#phase-2
**Entry checkpoint:** CHG-0035 Phase 2 evidence proves the active bound-library session carries a verified Entra actor through the desktop adapter without exposing credentials.
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `docs/product/capabilities/CAP-0002-document-lifecycle.md` (Outcome 4); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcome 6); `docs/product/capabilities/CAP-0011-approval-evidence.md` (Outcomes 1, 6); `docs/product/capabilities/CAP-0015-document-control-data.md` (Outcome 11, 14); `docs/product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md` (Operational details 1, 10); `docs/changes/active/CHG-0035-entra-session-required-for-group-bound-library.md` (Phase 2); `docs/changes/archive/CHG-0017-candidate-approval-copy.md`; `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md` (Local Contracts); `crates/dms-core/src/lifecycle.rs` (`ReleaseRecord`); `crates/dms-desktop/src/lib.rs` (`CurrentReleaseSelection`, `ReleaseProfileSelection`, `document_selection`); `crates/dms-desktop/ui/library.mjs` (`candidateTargetHelpText`, `syncCandidateTargetForm`, `externalLifecycleMarkup`, release snapshot markup); `crates/dms-desktop/ui/library.test.mjs`; `docs/product/wireframes/generate.mjs` (CAP-0002, CAP-0006, and CAP-0015 selection/candidate forms)
**Produces:** A target-sensitive candidate form whose outcome text distinguishes direct-release candidates from approval-request candidates, whose requester picker cannot be mistaken for the approval target, whose verified Entra actor is preselected by immutable object ID without fuzzy identity matching, and whose Releases pane names immutable release-time workflow identities without substituting current roles.
**Status:** pending — queued after P0600 and blocked until CHG-0035 Phase 2 evidence exists.

| Field | Value |
| --- | --- |
| ID | CHG-0038 |
| Status | pending |
| External request | Direct operator request: "Creating a next minor version, where an approval would be optional, is not clear from UI perspective, if a approval process will be started by pressing \"Create release condidate\". The first \"Requesting editor\" drop-down element is called \"Choose person\" and could be e.g. renamed to \"Choose person for review (approval)\" or something similar. If you have a better sugesstion make the proposal." |
| Affected CAPs | CAP-0002, CAP-0006, CAP-0011, CAP-0015, CAP-0021 |
| Decision records | none — approval-required target resolution, requester/approver identities, group-bound session identity, and delivery semantics already have durable contracts. |

## Current state

- The existing idle-draft form uses one **Create release candidate** submit label for every target mode, as intentionally established by archived CHG-0017 (`docs/changes/archive/CHG-0017-candidate-approval-copy.md:21-25`).
- The runtime target helper already resolves first release, Next minor, Next major, and manual targets, but phrases the consequence only as `stays in draft for direct PDF export` or `opens approver review after notification` (`crates/dms-desktop/ui/library.mjs:517-555`).
- The form places a generic paragraph above the target picker and labels the requester select's empty option **Choose person** (`crates/dms-desktop/ui/library.mjs:1025-1027`, `crates/dms-desktop/ui/library.mjs:1054-1056`).
- The selected person is the candidate's requester/editor snapshot, not the approver/reviewer; the effective approver derives from workflow routing and is separately snapshotted for approval-required candidates (`docs/product/capabilities/CAP-0002-document-lifecycle.md:47-48`, `docs/product/capabilities/CAP-0002-document-lifecycle.md:63-74`).
- The responsible Editor is selected separately in **Configuration → Workflow** at the edit root or a folder and inherited by documents; the candidate form currently lets any eligible person become its requester (`docs/product/capabilities/CAP-0019-inherited-workflow-role-routing.md:29-45`; `crates/dms-desktop/ui/library.mjs:1021-1029`). A conditional staged Editor picker is yet another operation: it updates future document control only after a successful release.
- CHG-0035 is pending. Its Phase 2 will provide the active group-bound session's verified Entra tenant/object ID to desktop mutations; before that phase lands, the candidate form has no session principal it can safely preselect (`docs/changes/active/CHG-0035-entra-session-required-for-group-bound-library.md:61-74`).
- Every `ReleaseRecord` already retains immutable `owner`, `editor`, `approver`, and `requester` snapshots, but the desktop `CurrentReleaseSelection` projects only the release-time control profile and Owner snapshot (`crates/dms-core/src/lifecycle.rs:277-302`; `crates/dms-desktop/src/lib.rs:285-301`; `crates/dms-desktop/src/lib.rs:2663-2681`).
- The Library Releases section consequently displays only **Owner** beneath **Immutable current release profile**, while the current folder/table and Document control data distinguish the live effective **Editor** and **Approver** roles (`crates/dms-desktop/ui/library.mjs:1115-1118`, `crates/dms-desktop/ui/library.mjs:1148`).
- The generated CAP-0002 and CAP-0015 screens reproduce the same generic candidate-form explanation and requester terminology (`docs/product/wireframes/generate.mjs:108-125`, `docs/product/wireframes/generate.mjs:1334-1341`).

## UX proposal

Keep the operation name **Create release candidate**. It accurately covers both paths and CHG-0017 deliberately made it approval-neutral. Do not rename the requester control to **Choose person for review (approval)**: that would falsely suggest the selected person is the approver and would be wrong for approval-optional minor candidates.

Use **Choose requesting editor** as the empty option beneath the existing **Requesting editor** label, followed by concise explanatory copy: **This person is recorded as the requester; the assigned approver is selected by the workflow when approval is required.**

For a group-bound library after CHG-0035 Phase 2, preselect the current verified Entra actor only when its immutable object ID exactly equals one of the candidate form's `eligible_people` object IDs. Keep the picker editable so an operator can explicitly create a candidate on behalf of another eligible requester; the new actor evidence from CHG-0035 and the chosen requester snapshot must remain distinguishable. Do not fuzzy-match display names, email addresses, local OS usernames, or a persisted local-to-Entra mapping: those are mutable, ambiguous, and weaker than the object ID already available from the session.

In **Releases**, keep Owner but make every identity release-time explicit. Rename the subheading to **Immutable release snapshot** and group its fields as:

- **Owner at release** — document-control owner snapshot.
- **Requested by** — candidate requester snapshot.
- **Responsible editor at release** — workflow editor snapshot.
- **Approval** — **Not required** for a minor/direct release, or **Approved by <name>** for an approval-required release.

Do not display current effective Editor/Approver values in this snapshot and do not replace missing historic fields from current routing. A legacy missing value reads **Unrecorded**. This preserves the important distinction visible in the screenshot: a current effective Editor/Approver can differ from the person or role recorded when the PDF was released.

Make the existing target-sensitive helper the primary **What happens next** message directly below Target version:

- For an approval-optional target: **No approval request will be sent. Creating this candidate keeps the document in Draft; you can export and release Vx.y directly.**
- For an approval-required target: **Approval is required. Creating this candidate starts approval and sends the request to the assigned approver. The document becomes Pending approval after delivery succeeds.**
- For an incomplete Manual target: keep version-entry guidance and state that approval outcome appears after a valid target is entered.

The message must update when target mode or Manual major/minor inputs change. It describes the existing lifecycle only: it neither changes target resolution nor sends a notification before submission. For `mailto:`, delivery remains unconfirmed until the existing explicit confirmation flow completes.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Make candidate outcome, release identities, and Entra default explicit in the Library | pending | `node --test crates/dms-desktop/ui/library.test.mjs` exits 0, proving optional, required, first-release, and Manual outcome copy; **Choose requesting editor**; exact actor-ID preselection; editable explicit override; no fuzzy fallback; and distinct immutable release identities |
| 2 | Publish the CAP contracts and matching candidate/release wireframes | pending | `node docs/product/wireframes/generate.mjs` and 1600×1600 headless-Chrome renders of CAP-0002, CAP-0006, and CAP-0015 exit 0; generated screens contain the approved copy |
| 3 | Run the workspace gate and close the change record | pending | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, Markdown-link check, and `git diff --check` all exit 0; CAP/CHG indexes agree and CHG-0038 is archived |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Make candidate outcome, release identities, and Entra default explicit in the Library

**Goal:** Selecting a candidate target tells an operator whether submission will initiate approval, the requester picker defaults only to the verified Entra actor, and the current release snapshot names every distinct historical identity.

Steps:

1. Keep **Create release candidate** as the form title and submit label. Do not add a second submit action or a separate preview/review-request action.
2. Update `candidateTargetHelpText` (or its directly owned presentation helper) with the proposed outcome copy. Reuse `syncCandidateTargetForm` so the message changes for Next minor, Next major, first-release, and complete Manual inputs; incomplete Manual inputs must not claim an approval outcome.
3. Preserve `effectiveCandidateTarget`, target-mode values, candidate request payload, approval-required calculation, lifecycle transitions, notification dispatch, and `mailto:` confirmation semantics.
4. Replace the requester select's empty option with **Choose requesting editor**. Keep **Requesting editor** as the field label and add the proposed requester-versus-approver explanation adjacent to that field; do not expose an approver picker in this form.
5. Consume only CHG-0035's non-secret active-session actor projection. If the library is group-bound and that actor's object ID is present in `eligible_people`, render that existing option selected and state that it is the signed-in actor's default. Keep the select enabled so an explicit different eligible requester may be selected. If the session actor is missing or not an exact eligible-person match, show the normal blank **Choose requesting editor** state and do not synthesize a match.
6. Extend only the desktop selection projection from the existing `ReleaseRecord` snapshots. Add requester, editor, approver, and approval-required/approval-evidence data to the current-release selection; do not change `dms-core` release persistence, a `.dms` schema, or historic release records.
7. Render **Immutable release snapshot** with **Owner at release**, **Requested by**, **Responsible editor at release**, and the conditional **Approval** result. For an approval-required record, use its immutable approver snapshot only after the existing approval-chain evidence is present; for an approval-optional record render **Not required**. Never source these values from current `effective_workflow_roles`.
8. Render legacy/missing release snapshots as **Unrecorded** rather than borrowing current document-control or routing values.
9. Add focused frontend and desktop-adapter assertions for exact text and updates across target modes, including a Manual target on either side of the approval boundary. Assert exact-ID preselection, explicit override, missing/no-match fallback, and no display-name/email/local-username fuzzy matching. Assert both approval-required and direct-release identity summaries and that a later role change cannot rewrite a current release's displayed snapshots. Assert that generic **Choose person** is absent from the candidate form while unrelated Configuration pickers remain unchanged.

**Verification gate:** `cargo test -p dms-desktop --lib` and `node --test crates/dms-desktop/ui/library.test.mjs` exit 0 with exact assertions for the candidate's optional/required/Manual outcome copy, requester terminology, exact actor-ID default, editable override, no-match fallback, and immutable release identity summary.

## Phase 2 — Publish the CAP contracts and matching candidate/release wireframes

**Goal:** The current product contracts and candidate/release visual references state the same target-sensitive outcome and release-time identity semantics as the runtime UI.

Steps:

1. Amend CAP-0002 outcome 4 and CAP-0015 outcome 14 to require a target-sensitive message that distinguishes no approval request/direct release from approval-required/Pending approval, while retaining one **Create release candidate** action and an editable exact-ID session-actor requester default.
2. Amend CAP-0006 outcome 6 and CAP-0015 outcome 11 to distinguish current effective Editor/Approver from immutable release-time Owner, requester, editor, and approval result. Amend CAP-0011 to state the release summary is derived from immutable evidence and never reconstructed from current routing.
3. Amend CAP-0021 to state that a verified group-bound session actor may be preselected only by immutable object ID in an eligible-person picker; it never authorizes a fuzzy or local-OS identity match.
4. Amend `crates/dms-desktop/AGENTS.md`'s Revision cycle and Releases contracts with the required outcome message, **Choose requesting editor** terminology, exact-ID session-actor default, and immutable release snapshot terminology because it owns the durable Library UI contract.
5. Update the CAP-0002 and CAP-0015 candidate forms plus the CAP-0006 Releases example in `docs/product/wireframes/generate.mjs` with synthetic approval-optional and approval-required examples, a signed-in requester default, the requester explanation, and the exact release snapshot labels. Regenerate HTML, `index.html`, and `manifest.json`; do not hand-edit generated outputs.
6. Render the regenerated CAP-0002, CAP-0006, and CAP-0015 HTML pages to their existing PNG paths and visually inspect that the changed helper text and release identity groups wrap without colliding with controls.

**Verification gate:** `(cd docs/product/wireframes && node generate.mjs && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0002-document-lifecycle.png "file://$PWD/html/CAP-0002-document-lifecycle.html" && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0006-library-explorer.png "file://$PWD/html/CAP-0006-library-explorer.html" && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0015-document-control-data.png "file://$PWD/html/CAP-0015-document-control-data.html" && test -s exports/CAP-0002-document-lifecycle.png && test -s exports/CAP-0006-library-explorer.png && test -s exports/CAP-0015-document-control-data.png)` exits 0, and the rendered PNGs visibly show the approved wording without overlap or clipping.

## Phase 3 — Run the workspace gate and close the change record

**Goal:** The UI copy, contracts, generated artifacts, and change-progress receipt remain consistent and verified.

Steps:

1. Run the focused and workspace gates. Fix only regressions caused by this change; retain unrelated working-tree changes.
2. Run `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .` and verify CAP/CHG links and indexes, including the new/archived CHG path.
3. After every gate passes, record phase evidence, set this CHG to done, move it to `docs/changes/archive/`, and move its README entry from Active to Archive.

**Verification gate:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .`, and `git diff --check` all exit 0; `docs/changes/README.md` lists CHG-0038 only in Archive and `docs/changes/active/CHG-0038-candidate-approval-outcome-clarity.md` no longer exists.

## Out of scope

- Changing which versions require approval, the effective target-version algorithm, or first-release rules.
- Changing notification delivery, `mailto:` confirmation, review identity, approver authorization, or Pending approval semantics.
- Adding a requester/approver picker to the candidate form, automatic approver reassignment, or a second candidate-submit action.
- Fuzzy display-name/email/local-OS-user matching, a persisted local-to-Entra mapping, or any automatic selection when the verified actor lacks an exact eligible-person object-ID match.
- Rewriting release records, backfilling legacy snapshots, modifying workflow evidence, or substituting current Owner/Editor/Approver values into historic release summaries.
- Renaming generic **Choose person** placeholders outside the candidate form.
