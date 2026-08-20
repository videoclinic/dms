# CHG-0026 — Person-consolidated workflow evidence without digest values

Rebuild the **Canonical workflow evidence** disclosure in the DMS Desktop Library
selection pane so that a document's event history renders as one consolidated
visual block per (person × release interval) — all workflow events made by the
same person between two releases (or from history start) collapse into a single
expandable element with the fine-grained events nested inside — and stop
displaying digest values (event hash, predecessor hash, event ID) anywhere in
that pane, while the persisted chain, the Verify workflow verdict, and CAP-0012
report exports remain untouched.

**Plan ID:** CHG-0026-person-consolidated-workflow-evidence
**Execution slot:** P0200
**Created:** 2026-08-19
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `docs/changes/AGENTS.md` (Local Contracts); `docs/product/capabilities/CAP-0011-approval-evidence.md` (whole file, especially Outcomes 4 and 8); `docs/product/capabilities/CAP-0015-document-control-data.md` (Outcome 11, pane section layout); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcome 8, selection-pane actions); `crates/dms-desktop/ui/library.mjs` (`workflowEventMarkup`, `lifecyclePanelMarkup`, `selectionMarkup`); `crates/dms-desktop/ui/styles.css` (`.workflow-evidence`, `.workflow-event` rules); `crates/dms-desktop/ui/library.test.mjs` (evidence fixture and assertions); `crates/dms-core/src/lifecycle.rs` (`WorkflowEventType`, `WorkflowEventBody`); `crates/dms-desktop/src/lib.rs` (`DocumentSelectionDetail.workflow_events`, `workflow_verification`); `docs/product/wireframes/generate.mjs` (CAP-0011 screen, `event()` helper); `docs/product/wireframes/manifest.json`
**Produces:** A desktop build whose Library selection-pane evidence disclosure renders per-person interval blocks with no digest values, the amended CAP-0011 presentation contract, and a regenerated CAP-0011 wireframe (HTML + PNG).
**Status:** done — all three phases passed their gates; this record is ready for archive.
**Filename convention:** The repository's active-record contract requires `CHG-*.md`; `P0200` is the execution order authority for this CHG and does not conflict with CHG-0025's `P0100`.

| Field | Value |
| --- | --- |
| ID | CHG-0026 |
| Status | done |
| External request | Direct operator request: "The audito/changelog of a file is fine grained today. I would like to drop the digest value presentation in DMS Desktop and consolidate all changes in the UI for all changes done between releases -- as long done by the same person. If the person who makes changes result in a new visual element in the DMS Desktop UI." Follow-up confirmation: one consolidated visual block per (person × release interval); fine-grained events nested/expandable inside; digest values gone from that pane. |
| Affected CAPs | CAP-0011 |
| Decision records | None — capability-local presentation rule recorded in CAP-0011; the ADR-0013 canonical chain is unchanged |

## Current state

- The evidence disclosure renders one `article.workflow-event` per event with
  the full event hash, predecessor hash, and event ID in a `<dl>`:
  `crates/dms-desktop/ui/library.mjs:913-918` (`workflowEventMarkup`) and
  `library.mjs:930-938` (`lifecyclePanelMarkup` disclosure, summary shows only
  the verification verdict).
- Supporting styles live at `crates/dms-desktop/ui/styles.css:407-414`
  (`.workflow-evidence`, `.workflow-event` grid rules).
- The pane payload already carries everything grouping needs:
  `crates/dms-desktop/src/lib.rs:214-223` exposes
  `DocumentSelectionDetail.workflow_events: Vec<WorkflowEvent>` (newest first)
  plus `workflow_verification`; release events are inside that chain, so
  interval boundaries are derivable in the frontend with **no IPC or schema
  change**.
- Event types are `crates/dms-core/src/lifecycle.rs:306-327`; event bodies
  carry `requester`, `editor`, `approver` (`PersonSnapshot`),
  `authenticated_actor`, and `local_os_user`
  (`lifecycle.rs:356-381`).
- Existing UI test: `crates/dms-desktop/ui/library.test.mjs:472-481` (single
  event fixture with `event_hash: "abc123"`, `predecessor_hash: null`) and
  `library.test.mjs:557-568` (disclosure label and event label assertions).
- CAP-0011 Outcome 4 currently requires each history row to retain "its event
  hash, predecessor hash, type, timestamp, changelog, … revision digest"; this
  CHG changes that presentation contract. CAP-0011 is `Status: not
  implemented`, so the outcome is a contract, not a shipped claim.
- The CAP-0011 wireframe renders per-event `hash … ← pred …` hint lines:
  `docs/product/wireframes/generate.mjs:636-657` (screen) and
  `generate.mjs:1178-1184` (`event()` helper, also used by the CAP-0002
  screen's callout area).
- Digest values also appear outside this pane and are deliberately not in
  scope: `ui/maintenance.mjs:61,77,188` (release/approval chain heads, PDF
  digest, backup manifest digest), `ui/reports.mjs:117` (report SHA-256),
  `ui/assistance.mjs:47` (payload digests), and CAP-0012 CSV/PDF export
  columns (`crates/dms-core/src/audit.rs` `content_digest` /
  `evidence_hash` fields).

## Risk call-out

Grouping is presentation-only: it must not mutate, re-order, or reinterpret
the persisted SHA-256 chain. The safe limits are (1) no change to
`WorkflowEvent`, `WorkflowEventBody`, IPC shapes, or `.dms` schema;
(2) the disclosure summary keeps the Verify workflow verdict
(`valid` / `tampered at <event-id>` / `invalid`) so an operator who sees a
consolidated block can still see chain state without digests; and
(3) CAP-0012 reports remain the full-fidelity export where hashes and digests
are the matching mechanism. No data is destroyed; `.dms` bytes are untouched,
so recovery from any failure in this CHG is `git restore` on the frontend
files plus re-running the UI tests.

One grouping hazard: actor identity must key on the stable identifier
(Entra object ID when the event records a `PersonSnapshot`, local OS user
string otherwise), never on the mutable display name — a later Entra rename
or local-user change must not re-key or split existing blocks. The two key
spaces are disjoint and must be namespaced so an OS user named like an object
ID never collides with one.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Rebuild the evidence disclosure as per-person interval blocks without digest values | done (`node --test crates/dms-desktop/ui/library.test.mjs`, 30 passed) | `node --test crates/dms-desktop/ui/library.test.mjs` exits 0, including assertions that a two-person × two-interval fixture renders one consolidated block per (person × interval) and that the pane markup contains no `abc123`, `Hash`, or `Predecessor` presentation |
| 2 | Amend CAP-0011 and regenerate the CAP-0011 wireframe | done (`node docs/product/wireframes/generate.mjs`; headless Chrome CAP-0011 export; manifest 21 screens) | CAP-0011 Outcome 4 states the per-person interval-block presentation and the absence of digest values in the pane; `node docs/product/wireframes/generate.mjs` exits 0; `html/CAP-0011-approval-evidence.html` and `exports/CAP-0011-approval-evidence.png` are regenerated and `manifest.json` still lists 21 screens |
| 3 | Run the workspace gate and close the record | done (`cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`, 98 passed) | `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` all exit 0; CHG-0026 moved to `docs/changes/archive/` and `docs/changes/README.md` refreshed |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate
passes, and `pending` otherwise.

## Phase 1 — Rebuild the evidence disclosure as per-person interval blocks without digest values

**Goal:** The disclosure's body shows consolidated per-person blocks instead of
flat per-event rows, and no digest value is displayed anywhere in the pane.

Steps:

1. In `crates/dms-desktop/ui/library.mjs`, stop rendering the per-event
   `<dl>` with **Event ID**, **Hash**, and **Predecessor** in
   `workflowEventMarkup`. A fine-grained event row keeps only its type label,
   timestamp, and comment (first present of `changelog`, `decision_comment`,
   `operator_comment`), plus the requested target version and target-version
   mode when the body carries them.
2. Add a grouping step ahead of rendering that works on
   `detail.workflow_events` (newest first):
   - Split the chain into intervals at each `release` event: an interval is
     the events strictly after the previous `release` event through the next
     `release` event, which belongs to the interval it concludes; events after
     the last `release` form the open interval. Label a concluded interval by
     its release version (`V<major>.<minor>` from the release event's
     `target_version`) and the open interval **Current draft work**. A
     `release_withdrawn` event is an ordinary member of whichever interval it
     falls into; it never opens a new interval.
   - Resolve each event's acting person: `review_decision_*` and
     `periodic_review_completed` → `body.approver`; `review_requested`,
     `release`, `review_cancelled`, `decision_outcome_notified`, and
     `minor_publication_notified` → `body.requester`; every other event →
     `body.local_os_user`. When a resolved `PersonSnapshot` is absent, fall
     back to `body.local_os_user`.
   - Key the group by the stable identity — Entra: object ID of the resolved
     person; local: OS user string — with disjoint namespaces so the two kinds
     can never collide. Display the `PersonSnapshot` display name (or the OS
     user string); never key by display name.
   - Render each (person × interval) group as one consolidated visual element:
     a `<details class="workflow-actor-block">` labelled **Changes by
     <name>** with the event count and the group's first/last timestamp in the
     summary, fine-grained rows nested inside, open by default. Groups within
     an interval are ordered by their newest event; intervals render newest
     first, matching the existing history rule.
3. Keep the disclosure summary as `Canonical workflow evidence · <verdict>`
   (existing behaviour); the verdict is the pane's only remaining chain
   indicator. Keep the "No canonical workflow evidence has been recorded."
   empty state for a zero-event chain.
4. Add `.workflow-interval` and `.workflow-actor-block` rules to
   `crates/dms-desktop/ui/styles.css` consistent with the existing disclosure
   styling (`.workflow-evidence`, `.workflow-event`). Do not persist the new
   blocks' open/closed state; session-only, like every other pane disclosure.
5. Update `crates/dms-desktop/ui/library.test.mjs`:
   - Extend the fixture to a two-interval chain (events, one `release`
     event, post-release events) with two distinct acting people plus one
     local-OS-user event, and one `release_withdrawn` event.
   - Assert: one `.workflow-actor-block` per (person × interval); the
     interval labels include the release version and **Current draft work**;
     the withdrawn event renders inside its interval, not as a boundary.
   - Assert the pane markup contains no digest presentation:
     `assert.doesNotMatch` the fixture's hash string, `<dt>Hash</dt>`,
     `<dt>Predecessor</dt>`, and `<dt>Event ID</dt>`.
   - Keep the existing `document control data changed` label, tampered-verdict,
     and empty-state assertions working; add a zero-event case if absent.

Recovery: this phase touches only `ui/library.mjs`, `ui/styles.css`, and
`ui/library.test.mjs`; on any inconsistency, `git restore` those three files
and re-run `node --test crates/dms-desktop/ui/library.test.mjs`.

Verification gate: `node --test crates/dms-desktop/ui/library.test.mjs` exits
0, including the new per-person block, interval-boundary, and no-digest
assertions.

## Phase 2 — Amend CAP-0011 and regenerate the CAP-0011 wireframe

**Goal:** The capability contract describes the new presentation, and the
CAP-0011 wireframe illustrates per-person interval blocks without digest
values.

Steps:

1. Rewrite CAP-0011 Outcome 4 (`docs/product/capabilities/CAP-0011-approval-evidence.md`):
   the history lists every event newest first, consolidated into one visual
   block per (acting person × release interval); each block's summary shows
   the person, event count, and time span, with fine-grained events nested
   showing type, timestamp, comment, and requested target version/mode when
   present. No event hash, predecessor hash, event ID, or revision digest is
   displayed in the pane; the disclosure summary carries the Verify workflow
   verdict, and full-fidelity evidence remains available through CAP-0012
   exports. Data retention per event is unchanged — only presentation changes.
   Leave Outcomes 1–3, 5–9 untouched.
2. Update CAP-0011's Tests row so the evidence-test link still resolves to
   `crates/dms-desktop/ui/library.test.mjs` and names the new
   consolidation assertions.
3. In `docs/product/wireframes/generate.mjs`, update the CAP-0011 screen to
   show two interval sections each containing per-person blocks, and remove
   the `hash … ← pred …` hint line from the `event()` helper (keep the helper
   for the CAP-0002 screen, now hash-free there as well). Keep the callout
   "Verify workflow recomputed each event hash from its canonical body" — it
   describes the verdict, not a displayed value.
4. Regenerate: `node docs/product/wireframes/generate.mjs`, then headless
   Chrome screenshots into `exports/` per the wireframe AGENTS.md
   verification (manifest lists every HTML and PNG; screen count stays 21).
5. Check that CAP-0011's Links section still resolves (wireframe HTML/PNG,
   sibling CAPs, design decisions).

Verification gate: CAP-0011 Outcome 4 states the per-person interval-block
presentation and the absence of digest values in the pane;
`node docs/product/wireframes/generate.mjs` exits 0;
`html/CAP-0011-approval-evidence.html` and
`exports/CAP-0011-approval-evidence.png` are regenerated and
`manifest.json` still lists 21 screens.

## Phase 3 — Run the workspace gate and close the record

**Goal:** The full repository gate passes and the record is archived with
evidence.

Steps:

1. Run `cargo fmt --check`,
   `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, and
   `node --test crates/dms-desktop/ui/*.test.mjs`.
2. Confirm `docs/product/README.md` CAP index and the
   `docs/changes/README.md` active index are consistent; set every phase
   `done (<evidence>)` and this CHG's Status to `done`, move the file to
   `docs/changes/archive/`, and refresh `docs/changes/README.md` in the same
   change.
3. Commit the vertical slice (frontend code, styles, tests, CAP-0011,
   wireframe regeneration, and record updates) as one
   `feat(desktop): …` commit.

Verification gate: `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, and
`node --test crates/dms-desktop/ui/*.test.mjs` all exit 0; CHG-0026 is in
`docs/changes/archive/` and `docs/changes/README.md` reflects it.

## Out of scope

- Digest presentation outside the selection-pane evidence disclosure: the
  Audit & reports release-history chain heads, PDF digests, and backup
  manifest digest (`ui/maintenance.mjs`), recent-report SHA-256
  (`ui/reports.mjs`), and the Claude assistance payload digests
  (`ui/assistance.mjs`).
- CAP-0012 audit report CSV/PDF columns — event and content digests stay
  report fields; they are the report's matching mechanism.
- CAP-0011's Verify workflow routine, the ADR-0013 canonical event body, any
  `.dms` schema or migration, and any new IPC command — grouping is pure
  presentation over the existing `workflow_events` payload.
- Workspace-wide or multi-document evidence consolidation; only the
  per-document selection pane changes.
- Persisting open/closed state for the new interval or person blocks.
- Renaming workflow event types or adding new event types.
