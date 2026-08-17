# CHG-0017 — Candidate form approval-neutral copy

**Plan ID:** CHG-0017-candidate-approval-copy
**Created:** 2026-08-17
**Depends on:** CHG-0011-revision-cycle-candidate-clarity
**Entry checkpoint:** CHG-0016 is archived as done; `b95f155` is the current `main` tip. Unrelated membership-vs-obsolescence working-tree edits are left untouched.
**Context sources:** `docs/product/capabilities/CAP-0002-document-lifecycle.md#outcomes` (outcome 4), `docs/product/capabilities/CAP-0015-document-control-data.md#outcomes` (outcome 14), `crates/dms-desktop/AGENTS.md#local-contracts`, `crates/dms-desktop/ui/library.mjs` (`externalLifecycleMarkup`, `candidateTargetHelpText`), `docs/product/wireframes/generate.mjs` (CAP-0002 / CAP-0015 candidate forms)
**Produces:** One **Create release candidate** action for approval-optional and approval-required targets, the review content-check override field on that same form, and **approval optional** on later Next minor options, without changing Effective target resolution.
**Status:** done — approval-neutral Create release candidate copy shipped; archived after workspace gate

| Field | Value |
| --- | --- |
| ID | CHG-0017 |
| Status | done |
| External request | Direct operator request: (1) The "Create release condidate" should be adopted if the approval is skipped or not; The wording "Create release candidate" does not say something about the approval (2) "Review content-check overreide reason (only wen needed)" should also be adopted to the approval process, if possible or not (3) Add the "approval optional" to the "Next minor" selection in "Target version"; do not change the "Effective target" logic |
| Affected CAPs | CAP-0002, CAP-0015 |
| Decision records | none |

## Current state

- Runtime titles and submits the idle-draft form as **Create release candidate** for every target mode.
- Runtime keeps **Review content-check override reason (only when needed)** on that form for every target mode.
- Later Next minor options are labeled **approval optional**. First-release Next minor stays `Next minor · V1.0 (first release)` because `V1.0` still requires approval.
- CAP-0002/CAP-0015 wireframes use one Create release candidate action, show the override field, and label later Next minor **approval optional**.
- `candidateTargetHelpText` / Effective target resolution are unchanged.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Contract, runtime copy, and focused tests | done (`node --test crates/dms-desktop/ui/library.test.mjs` 27/27) | `node --test crates/dms-desktop/ui/library.test.mjs` exits 0; later Next minor is labeled approval optional; first-release Next minor is not; Effective target strings stay unchanged |
| 2 | CAP-linked wireframes | done (`node generate.mjs` + 1600×1600 CAP-0002/CAP-0015 Chrome exports) | `node generate.mjs` plus 1600×1600 CAP-0002/CAP-0015 Chrome exports exit 0; both screens use Create release candidate, show the override field, and label later Next minor approval optional |
| 3 | Workspace gate and change-record closeout | done (`cargo test --workspace`; `clippy -D warnings`; frontend 94/94; link check 0) | Workspace Rust/frontend/link/diff checks exit 0; CHG-0017 archived as done |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, `pending` otherwise.

## Phase 1 — Contract, runtime copy, and focused tests

**Goal:** The candidate form uses approval-neutral **Create release candidate** copy, keeps the review override field for every target, and labels later Next minor options **approval optional**. Effective target logic does not change.

Steps:

1. Amend CAP-0002 outcome 4 and CAP-0015 outcome 14 with the copy contract.
2. Label later Next minor options `Next minor · Vn.n (approval optional)`. Keep first-release Next minor as `Next minor · V1.0 (first release)`.
3. Leave `candidateTargetHelpText` and Effective target resolution unchanged.
4. Assert the new Next minor label, first-release exception, and always-visible override field in Library frontend tests. Keep existing Effective target assertions.

**Verification gate:** `node --test crates/dms-desktop/ui/library.test.mjs` exits 0.

## Phase 2 — CAP-linked wireframes

**Goal:** CAP-0002 and CAP-0015 wireframes show one Create release candidate action, the review override field, and approval-optional Next minor.

Steps:

1. In `docs/product/wireframes/generate.mjs`, drop the Preview review-request action, keep one Create release candidate control, add the override field, and label later Next minor **approval optional**.
2. Regenerate HTML, then export CAP-0002 and CAP-0015 PNGs at 1600×1600.

**Verification gate:** `(cd docs/product/wireframes && node generate.mjs && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0002-document-lifecycle.png "file://$PWD/html/CAP-0002-document-lifecycle.html" && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0015-document-control-data.png "file://$PWD/html/CAP-0015-document-control-data.html" && test -s exports/CAP-0002-document-lifecycle.png && test -s exports/CAP-0015-document-control-data.png)` exits 0.

## Phase 3 — Workspace gate and change-record closeout

**Goal:** Repository-wide checks pass and the completed behavior lives only in current CAPs plus an archived CHG receipt.

Steps:

1. Run focused frontend tests, then the Rust/frontend workspace gates.
2. Run Markdown link validation and `git diff --check`.
3. Perform the DOX closeout against every changed path.
4. Archive this CHG and refresh the change index.

**Verification gate:** `cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/*.test.mjs && python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary . && git diff --check` exits 0.

## Out of scope

- Changing whether a target requires approval, Effective target resolution, or first-release `V1.0` approval.
- Changing content-check rules, override persistence, or release-time override copy.
- Changing changelog, effective date, requester, or handover fields.
- Touching the unrelated membership-vs-obsolescence working-tree edits.
