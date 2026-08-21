# CHG-0029 — Newest version-history entry expanded by default

Render only the newest person-consolidated entry in **Version history & changes**
expanded by default; keep all older actor entries folded while retaining their
nested events unchanged.

**Plan ID:** CHG-0029-newest-history-entry-default
**Execution slot:** P0500
**Created:** 2026-08-21
**Depends on:** none
**Entry checkpoint:** Direct operator request specifies the required initial state.
**Context sources:** `docs/changes/AGENTS.md` (Local Contracts); `docs/product/capabilities/CAP-0011-approval-evidence.md` (Outcome 4); `crates/dms-desktop/ui/library.mjs` (`workflowEvidenceMarkup`, `workflowEventActor`); `crates/dms-desktop/ui/library.test.mjs` (person-group fixture and markup assertions); `docs/product/wireframes/generate.mjs` (CAP-0011 screen and shared selection pane); `crates/dms-desktop/AGENTS.md` (selection-pane contract).
**Produces:** An initial expansion rule that exposes only the most recent actor block, with current CAP-0011 and static wireframes documenting it.
**Status:** done — all verification gates passed; this record is ready for archive.
**Filename convention:** The repository's active-record contract requires `CHG-*.md`; `P0500` is this CHG's execution-order authority and does not change that filename convention.

| Field | Value |
| --- | --- |
| ID | CHG-0029 |
| Status | done |
| External request | Direct operator request: "unfold only the newest entry in \"Version history & changes · valid\"; fold all older entries by default" |
| Affected CAPs | CAP-0011 |
| Decision records | None — initial disclosure state is capability-local presentation |

## Current state

- `workflowEvidenceMarkup` receives history newest-first, renders intervals
  newest-first, and renders actor groups in first-seen order within each
  interval: `crates/dms-desktop/ui/library.mjs:954-989`.
- Every `workflow-actor-block` currently receives the `open` attribute,
  expanding the entire history whenever its parent topic is opened.
- The current UI fixture renders five actor blocks:
  `crates/dms-desktop/ui/library.test.mjs:598-606`.
- The Version history topic itself stays folded in a fresh Library activity under
  CHG-0028. This change governs its nested actor blocks only after an operator
  opens that topic.

## Risk call-out

The newest entry is defined by the existing rendered order: the first actor
block in the first (newest) interval. Do not re-sort timestamps, merge actor
identities, or change release interval boundaries while changing the `open`
attribute. The fallback for malformed timestamps remains the existing render
order. Recovery is a frontend-only `git restore`; `.dms` event data is never
modified.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Render only the newest actor block open | done (`node --test crates/dms-desktop/ui/library.test.mjs`, 31 passed) | `node --test crates/dms-desktop/ui/library.test.mjs` exits 0 with one open actor-block assertion against the multi-actor fixture |
| 2 | Synchronize CAP-0011 and wireframes | done (`node docs/product/wireframes/generate.mjs`; Chrome exported CAP-0011 PNG; 21 HTML + 21 PNG pairs) | CAP-0011 states the nested disclosure default; `node docs/product/wireframes/generate.mjs` exits 0; CAP-0011 HTML/PNG are current |
| 3 | Run workspace gates and close the record | done (`cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`, 106 passed) | `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` all exit 0; CHG is archived and indexes are accurate |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate
passes, and `pending` otherwise.

## Phase 1 — Render only the newest actor block open

**Goal:** Expanding Version history & changes reveals the latest actor entry
without expanding older entries.

Steps:

1. In `workflowEvidenceMarkup`, carry a single expansion flag across all
   intervals and actor groups. Apply `open` only to the first rendered actor
   block; omit it for every later block.
2. Extend the existing multi-actor frontend fixture to assert exactly one
   `workflow-actor-block` has `open`, that it is the first block under Current
   draft work, and that the remaining blocks have no `open` attribute.
3. Preserve no-digest presentation, actor labels, event counts, ordering, and
   empty-state rendering.

Verification gate: `node --test crates/dms-desktop/ui/library.test.mjs` exits 0
with the expansion rule asserted against the existing multi-actor fixture.

## Phase 2 — Synchronize CAP-0011 and wireframes

**Goal:** The evidence contract and static visual reference state the new nested
expansion default.

Steps:

1. Amend CAP-0011 Outcome 4: only the newest actor block is initially expanded;
   older actor blocks start folded when Version history & changes is opened.
2. Amend `crates/dms-desktop/AGENTS.md` only if its selection-pane contract needs
   this nested disclosure rule.
3. Update the CAP-0011 screen in `docs/product/wireframes/generate.mjs` so its
   newest actor block is open and the two older actor blocks are folded.
4. Regenerate the product wireframes and export CAP-0011 PNG. Do not add a CHG
   review artifact to the product manifest.

Verification gate: CAP-0011 and desktop DOX match the rule; generator exits 0;
CAP-0011 HTML/PNG show exactly one open actor block.

## Phase 3 — Run workspace gates and close the record

**Goal:** Deliver the small presentation change with proof and a complete record.

Steps:

1. Run `cargo fmt --check`.
2. Run `cargo clippy --workspace --all-targets -- -D warnings`.
3. Run `cargo test --workspace`.
4. Run `node --test crates/dms-desktop/ui/*.test.mjs`.
5. Complete the DOX pass; record evidence, archive this CHG, update
   `docs/changes/README.md`, and commit the vertical slice.

Verification gate: All listed commands exit 0; the CHG is archived and indexed.

## Out of scope

- Changing the folded state of the parent Version history & changes topic.
- Persisting nested actor-block disclosure choices.
- Changing event ordering, release intervals, actor resolution, event content,
  digest visibility, or audit exports.
