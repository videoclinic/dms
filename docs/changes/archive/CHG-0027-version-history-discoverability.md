# CHG-0027 — Discoverable version history and person-consolidated changes

Make the already-implemented per-document history easy to find: a selected
Library document will expose a visible **View version history & changes** entry
that opens a dedicated **Version history & changes** pane topic, showing the
existing release intervals and per-person consolidated event blocks.

**Plan ID:** CHG-0027-version-history-discoverability
**Execution slot:** P0300
**Created:** 2026-08-21
**Depends on:** none
**Entry checkpoint:** Operator approved the review wireframe and plan in chat on 2026-08-21.
**Context sources:** `docs/changes/AGENTS.md` (Local Contracts); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcomes 2, 6, 8); `docs/product/capabilities/CAP-0011-approval-evidence.md` (Outcome 4); `docs/product/capabilities/CAP-0015-document-control-data.md` (Outcome 11); `crates/dms-desktop/ui/library.mjs` (`DEFAULT_SELECTION_OPEN`, `workflowEvidenceMarkup`, `lifecyclePanelMarkup`, `selectionMarkup`); `crates/dms-desktop/ui/app.mjs` (`handleLibraryClick`, `handleLibraryToggle`); `crates/dms-desktop/ui/library.test.mjs` (selection-pane and fold-state coverage); `crates/dms-desktop/ui/app.test.mjs` (Library click orchestration); `docs/product/wireframes/generate.mjs` (CAP-0006 and CAP-0011 screens).
**Produces:** A discoverable, direct entry to the existing per-document version history, updated CAP contracts and production wireframes, frontend regression coverage, and a closed implementation receipt.
**Status:** done — all verification gates passed.
**Filename convention:** The repository's active-record contract requires `CHG-*.md`; `P0300` is this CHG's execution-order authority and does not change that filename convention.

| Field | Value |
| --- | --- |
| ID | CHG-0027 |
| Status | done |
| External request | Direct operator request: "I've no idea how to open the version history with consolidated changes by user." Follow-up: "with this insights; make a proposal using the wireframes; create the plan do not start implementing before operator approval" |
| Affected CAPs | CAP-0006, CAP-0011, CAP-0015 |
| Decision records | None — this is a capability-local navigation and presentation correction; canonical evidence, identity grouping, and storage remain unchanged |
| Review wireframe | [`CHG-0027-version-history-discoverability-review.html`](CHG-0027-version-history-discoverability-review.html) · [`PNG export`](CHG-0027-version-history-discoverability-review.png) — review-only; not listed in the product wireframe manifest |

## Current state

- The requested information already exists in the selected document's Library
  pane. `workflowEvidenceMarkup` renders newest-first release intervals and one
  **Changes by <person>** block per actor in each interval:
  `crates/dms-desktop/ui/library.mjs:913-991`.
- It is currently nested as **Canonical workflow evidence** at the bottom of
  the **Revision cycle** section (`library.mjs:994-1012`). The section needs two
  disclosures and scrolling to reach it, so its capability is effectively
  undiscoverable.
- The Library selection pane currently documents only **Document control data**,
  **Document review schedule**, **Revision cycle**, and **Releases** as its
  main foldable topics (`CAP-0006:123-151`, `CAP-0015:78-111`). Its Releases
  topic shows the current release profile, not the workflow history
  (`library.mjs:1155-1168`).
- CHG-0026 implemented the requested per-person × release-interval grouping and
  intentionally left the data path unchanged. The UI already receives
  `workflow_events` and the verification verdict in the existing
  `DocumentSelection` payload (`crates/dms-desktop/src/lib.rs:2651-2662`).
- **Implementation finding:** CHG-0026 deliberately namespaces Entra and local
  OS actor keys (`entra:<object-id>` / `local:<os-user>`) to prevent collisions
  (`library.mjs:913-939`). One individual can therefore appear under two
  rendered actor identities. The direct-entry summary must state only the
  current release, never a potentially misleading “N people” count.
- **Documentation finding:** CAP-0011 still says `Status: not implemented`,
  although the current Library code renders its contract and the focused tests
  pass. Phase 2 must correct that stale status while updating its in-app
  location; leaving it false would contradict the shipped vertical slice.
- **Wireframe finding:** CAP-0006 currently renders the Lost source selection
  example (`generate.mjs:455-457`), while the normal selected-document pane
  with this interaction is defined separately. Phase 2 must use the normal
  selection pane for CAP-0006 so the production wireframe actually shows the
  direct history entry; Lost source remains represented by the italic table row.
- No new core API, IPC command, `.dms` schema field, audit export, activity tab,
  or person grouping logic is required. The defect is navigation wording and
  placement only.

## Review proposal

The review wireframe proposes a small, explicit correction in the existing
selection pane:

1. Below the selected document's Source file identity, add a visible secondary
   button: **View version history & changes**. Its summary states the current
   version (or **No released version yet**), but does not count people.
2. Selecting it unfolds and focuses a new session-only **Version history &
   changes** topic in the same selection pane. It does not open a new activity,
   change the selected document, or make a backend call.
3. The topic contains the existing verification verdict and person-consolidated
   blocks, named in operator language. Its content starts with **Current draft
   work**, then prior released version intervals newest first. Fine-grained
   events remain nested under each person.
4. Move the existing evidence disclosure out of **Revision cycle**. That section
   stays focused on candidate/review/release actions; **Releases** keeps current
   PDF and immutable release-profile information.
5. The topic has ordinary selection-section fold behaviour. It is open by
   default for a newly opened Library activity, remains session-only, and keeps
   its fold state across document switches. The direct entry always expands and
   focuses it.

The proposal deliberately replaces the technical operator-facing label
**Canonical workflow evidence** with **Version history & changes**. The
verification verdict still conveys workflow integrity; CAP-0012 remains the
full-fidelity audit-export path.

## Risk call-out

This must remain a frontend-only presentation change. Moving the rendered
history must not alter its newest-first order, release boundaries, actor-key
rules, verification verdict, event retention, audit export fields, or `.dms`
bytes. The safe recovery path is `git restore` of the affected desktop UI,
tests, CAPs, and generated wireframes; no workspace data migration is involved.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 0 | Operator review and approval | done (operator approval in chat, 2026-08-21) | Operator explicitly approves the linked review wireframe and this CHG in chat; then update this row and Status before any implementation edit |
| 1 | Make the existing history directly discoverable in the Library pane | done (`node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs`, 63 passed) | `node --test crates/dms-desktop/ui/library.test.mjs` and `node --test crates/dms-desktop/ui/app.test.mjs` exit 0 with direct-entry, focus/open, placement, and session-fold-state assertions |
| 2 | Update capability contracts and CAP-linked wireframes | done (`node docs/product/wireframes/generate.mjs`; Chrome exported CAP-0006/CAP-0011 PNGs; 21 HTML + 21 PNG pairs) | CAP-0006, CAP-0011, and CAP-0015 state the new topic and direct entry; `node docs/product/wireframes/generate.mjs` exits 0; CAP-0006/CAP-0011 HTML+PNG exports and manifest remain synchronized |
| 3 | Run workspace gates and close the record | done (`cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`, 107 passed) | `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` all exit 0; CHG is archived and indexes are accurate |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate
passes, and `pending` otherwise.

## Phase 0 — Operator review and approval

**Goal:** Confirm the interaction and scope before changing runtime code or
current-state capability records.

Steps:

1. Review the CHG-local HTML and PNG artifacts linked above. They are proposal
   material only and deliberately remain outside `docs/product/wireframes/`.
2. Confirm that **Version history & changes** is the right operator-facing name,
   that the direct entry belongs under Source file identity, and that the topic
   opens in the same selection pane rather than a new activity.
3. On explicit approval, mark Phase 0 `done (operator approval)` and set Phase
   1 to `in-progress`. Do not begin Phase 1 before this checkpoint.

Verification gate: An explicit operator approval is recorded in this CHG and
in the conversation before any runtime, CAP, or production-wireframe edit.

## Phase 1 — Make the existing history directly discoverable in the Library pane

**Goal:** A selected Library document exposes one clear entry to its existing
version history and consolidated per-person changes.

Steps:

1. In `crates/dms-desktop/ui/library.mjs`, define a `history` selection-section
   key in `DEFAULT_SELECTION_OPEN` and keep it session-only with the existing
   `selection_open` state. Default it open for a new Library activity.
2. Extract `workflowEvidenceMarkup` from `lifecyclePanelMarkup` without changing
   event ordering, interval boundaries, actor resolution, grouping, or
   verification formatting. Render it as the body of the new top-level
   **Version history & changes** selection section.
3. Keep the **Revision cycle** section limited to release-candidate, review,
   release, cancel, and obsolete workflows. Keep **Releases** limited to the
   current released PDF/profile. Do not duplicate the history body in either
   section.
4. Add the visible **View version history & changes** control under Source file
   identity. Its compact summary derives only from the already-loaded detail:
   the current release label (or no release). Do not add a people or change
   count because the safe actor namespaces can represent one individual under
   multiple identities.
5. Extend `handleLibraryClick` in `crates/dms-desktop/ui/app.mjs` so the control
   marks the history section open, renders, then focuses its summary and scrolls
   it into the selection pane's view. It must not invoke Tauri, reload the
   document, or create a new activity.
6. Add frontend coverage for: visible direct entry; accessible label; history
   section located separately from Revision cycle; existing **Changes by**
   blocks and verification verdict rendered only in history; default and
   cross-document session fold state; direct entry reopens a folded section.
   Add an app-level click test for state update and focus behaviour if existing
   app test helpers can observe it.

Verification gate: `node --test crates/dms-desktop/ui/library.test.mjs` and
`node --test crates/dms-desktop/ui/app.test.mjs` exit 0, with assertions for the
specified direct-entry, placement, opening, and session-state behaviour.

## Phase 2 — Update capability contracts and CAP-linked wireframes

**Goal:** The current behaviour contract and product visuals match the approved
interaction.

Steps:

1. Amend CAP-0006 Outcome 6 and Outcome 8 to list **Version history & changes**
   as a main selection-pane topic and its visible direct entry; preserve the
   existing pane-scroll and session-only state contracts.
2. Amend CAP-0015 Outcome 11 to include the new history topic in the named
   independently foldable detail sections, while keeping document control,
   schedule, revision cycle, releases, and Actions responsibilities distinct.
3. Set CAP-0011 to `Status: implemented` and amend Outcome 4 so its in-app
   location and plain-language heading are explicit; keep its per-person
   interval grouping, no-digest presentation, and CAP-0012 full-fidelity
   boundary unchanged.
4. Update CAP-0006 and CAP-0011 in `docs/product/wireframes/generate.mjs` to
   show the approved direct entry and history topic. Regenerate the repository
   wireframes and their PNG exports; add no CHG proposal artifact to the product
   manifest.
5. Check all affected CAP links and product-wireframe inventory consistency.

Verification gate: The three CAPs accurately state the approved interaction;
`node docs/product/wireframes/generate.mjs` exits 0; generated CAP-0006 and
CAP-0011 HTML/PNG pairs exist; and `manifest.json` lists each current pair.

## Phase 3 — Run workspace gates and close the record

**Goal:** Ship the single UI vertical slice with executable evidence and a
complete repository record.

Steps:

1. Run `cargo fmt --check`.
2. Run `cargo clippy --workspace --all-targets -- -D warnings`.
3. Run `cargo test --workspace`.
4. Run `node --test crates/dms-desktop/ui/*.test.mjs`.
5. Complete the DOX pass: update `crates/dms-desktop/AGENTS.md` only if the
   named selection-pane topic contract changes; otherwise record it unchanged.
   Confirm `docs/AGENTS.md`, `docs/product/AGENTS.md`, and
   `docs/changes/AGENTS.md` remain accurate.
6. Record passing evidence, move this CHG to `docs/changes/archive/`, and update
   `docs/changes/README.md` in the same change. Commit the frontend, tests,
   CAPs, generated wireframes, and record as one user-visible vertical slice.

Verification gate: `cargo fmt --check`, `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo test --workspace`, and `node --test
crates/dms-desktop/ui/*.test.mjs` all exit 0; the CHG is archived and the change
index points at its archive path.

## Out of scope

- A workspace-wide history page, cross-document timeline, global person filter,
  or new history activity tab.
- New Tauri commands, `dms-core` methods, IPC payload fields, migrations, or
  `.dms` persistence changes.
- Altering the existing actor/grouping rules, release-interval boundaries,
  workflow verification, digest retention, or CAP-0012 report exports.
- Changing the meaning or placement of the current released PDF/profile in the
  existing **Releases** selection topic.
