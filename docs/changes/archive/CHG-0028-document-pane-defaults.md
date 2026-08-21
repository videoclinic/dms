# CHG-0028 — Document-pane initial disclosure state

Make a selected Library document open with **Releases** expanded, while
**Version history & changes**, **Revision cycle**, **Document control data**, and
**Document review schedule** start folded; keep **Actions** unchanged and remove
the Version history anchor button.

**Plan ID:** CHG-0028-document-pane-defaults
**Execution slot:** P0400
**Created:** 2026-08-21
**Depends on:** none
**Entry checkpoint:** Operator did not choose a clarification option and directed the agent to use its best judgement; the recommended approval path was selected on 2026-08-21.
**Context sources:** `docs/changes/AGENTS.md` (Local Contracts); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcomes 2, 6, 8); `docs/product/capabilities/CAP-0011-approval-evidence.md` (Outcome 4); `docs/product/capabilities/CAP-0015-document-control-data.md` (Outcome 11 and Outcome 14); `crates/dms-desktop/AGENTS.md` (Library selection-pane contract); `crates/dms-desktop/ui/library.mjs` (`DEFAULT_SELECTION_OPEN`, `openVersionHistory`, `selectionMarkup`); `crates/dms-desktop/ui/app.mjs` (`focusVersionHistorySummary`, `handleLibraryClick`); `crates/dms-desktop/ui/library.test.mjs`; `crates/dms-desktop/ui/app.test.mjs`; `docs/product/wireframes/generate.mjs` (CAP-0006/CAP-0011/CAP-0015 screens).
**Produces:** An approved, tested initial disclosure order for the single-document Library pane, matching CAP contracts and CAP-linked wireframes without a runtime link to repository-only CAP files.
**Status:** done — all verification gates passed.
**Filename convention:** The repository's active-record contract requires `CHG-*.md`; `P0400` is this CHG's execution-order authority and does not change that filename convention.

| Field | Value |
| --- | --- |
| ID | CHG-0028 |
| Status | done |
| External request | Direct operator request: "Redesign the document control pane as follows: (1) Releases unfolded (2) Version history & changes folded (3) Revision cycle folded (4) Document control data folded (5) Document review schedule folded (*) (a) Actions as is (b) Remove View version history & changes because it's only a anchro to the folded element; I would expect a link to the CAP-0011-approval-evidence page" |
| Affected CAPs | CAP-0006, CAP-0011, CAP-0015 |
| Decision records | None — disclosure defaults and in-pane presentation are capability-local |
| Review wireframe | [`CHG-0028-document-pane-defaults-review.html`](CHG-0028-document-pane-defaults-review.html) · [`PNG export`](CHG-0028-document-pane-defaults-review.png) — review-only; not listed in the product wireframe manifest |

## Current state

- The recently completed CHG-0027 sets all five main detail topics open by
default through `DEFAULT_SELECTION_OPEN`:
  `crates/dms-desktop/ui/library.mjs:46-53`.
- CHG-0027 added **View version history & changes** under Source file identity.
  It calls `openVersionHistory`, renders, then focuses the history summary:
  `library.mjs:72-74`; `crates/dms-desktop/ui/app.mjs`.
- `Actions` is already an independently foldable, bottom-docked disclosure and
  defaults open. It must not change.
- The primary selection pane already has all required information for a Releases
  default; no core query, IPC command, persistence, or lifecycle rule changes
  are required.
- CAP files are repository product contracts. The packaged DMS Desktop app loads
  app-local frontend assets and has no configured, installed, or stable URL for
  `CAP-0011-approval-evidence.md`; making a runtime CAP link would be broken for
  operators outside a source checkout.

## Review proposal

- Default the main document-detail scroller to **Releases: open** and the other
  four topics closed: **Version history & changes**, **Revision cycle**,
  **Document control data**, and **Document review schedule**.
- Preserve the user’s session-only fold choices across document switches exactly
  as today. These are only fresh-activity defaults; no preference or `.dms`
  value is added.
- Remove the **View version history & changes** control and its focus helper.
  The history topic remains available as its own named disclosure.
- Do **not** replace the removed control with a CAP-0011 hyperlink in the runtime
  app. That would expose a build-repository path as an operator feature and is
  not a deployable documentation surface. If a documentation website is later
  introduced with a stable operator URL, it needs its own capability and plan.

## Risk call-out

The main risk is regressively resetting an operator’s session choice when a
selected document changes. The implementation must alter only
`DEFAULT_SELECTION_OPEN`; `setSelectionSectionOpen` and selection changes must
continue preserving the existing `selection_open` object. Recovery is a
frontend-only `git restore` of the UI, tests, CAPs, and regenerated wireframes;
no `.dms` data changes.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 0 | Operator review and approval | done (operator-directed best-judgement approval, 2026-08-21) | Operator explicitly approves this CHG and its review wireframe in chat before runtime, CAP, or production-wireframe edits |
| 1 | Apply disclosure defaults and remove the redundant anchor | done (`node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs`, 62 passed) | `node --test crates/dms-desktop/ui/library.test.mjs` and `node --test crates/dms-desktop/ui/app.test.mjs` exit 0 with default-order, session-persistence, and no-anchor assertions |
| 2 | Synchronize CAP contracts and product wireframes | done (`node docs/product/wireframes/generate.mjs`; Chrome exported CAP-0006/CAP-0015 PNGs; 21 HTML + 21 PNG pairs) | CAP-0006, CAP-0011, and CAP-0015 describe the approved default state; `node docs/product/wireframes/generate.mjs` exits 0; CAP-0006/CAP-0011/CAP-0015 HTML+PNG outputs are current |
| 3 | Run workspace gates and close the record | done (`cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`, 106 passed) | `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` all exit 0; CHG is archived and indexes are accurate |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate
passes, and `pending` otherwise.

## Phase 0 — Operator review and approval

**Goal:** Confirm the proposed information hierarchy and the deliberate absence
of a runtime CAP link before changing any current behaviour.

Steps:

1. Review the linked CHG-local HTML and PNG proposal.
2. Confirm that fresh document selections should start with Releases expanded
   and all other main topics folded, while Actions stays as it is.
3. Confirm the runtime boundary: remove the anchor without substituting a
   repository CAP link. If an operator documentation URL is required instead,
   stop and create a separate capability proposal.
4. On explicit approval, mark this phase `done (operator approval)` and Phase 1
   `in-progress` before implementation.

Verification gate: Explicit operator approval is recorded in this CHG and the
conversation before implementation starts.

## Phase 1 — Apply disclosure defaults and remove the redundant anchor

**Goal:** New Library activities present Releases first without losing
session-only disclosure choices later in the activity.

Steps:

1. In `ui/library.mjs`, change `DEFAULT_SELECTION_OPEN` to `releases: true` and
   `history`, `revision`, `control`, and `schedule`: `false`; retain `actions:
   true`.
2. Remove `openVersionHistory`, the anchor markup below Source file identity,
   and its `aria-controls` target. Keep the named history disclosure intact.
3. Remove `focusVersionHistorySummary` and the corresponding click path from
   `ui/app.mjs`; no backend call or new interaction replaces it.
4. Update Library and app tests to prove the exact fresh defaults, session-state
   preservation across document switches, absence of the anchor/focus helper,
   present history disclosure, and unchanged Actions default.

Verification gate: `node --test crates/dms-desktop/ui/library.test.mjs
crates/dms-desktop/ui/app.test.mjs` exits 0 with the stated assertions.

## Phase 2 — Synchronize CAP contracts and product wireframes

**Goal:** Current documentation and static product visuals describe the approved
initial state rather than the superseded history anchor.

Steps:

1. Amend CAP-0006 Outcome 6 and Outcome 8: remove the direct history action and
   state the fresh disclosure defaults plus session-only preservation.
2. Amend CAP-0015 Outcome 11 and Outcome 14: distinguish Releases as the open
   fresh default, the other four main topics as initially folded, and Actions as
   unchanged.
3. Amend CAP-0011 Outcome 4: history remains a folded, named in-pane topic;
   remove the direct-anchor claim and state no repository-CAP runtime link.
4. Update `crates/dms-desktop/AGENTS.md` to match the settled selection-pane
   contract.
5. Update CAP-0006, CAP-0011, and CAP-0015 in
   `docs/product/wireframes/generate.mjs`; regenerate their HTML, manifest,
   index, and matching PNG exports. Do not add the CHG review artifact to the
   product manifest.

Verification gate: Affected CAPs and desktop DOX match the approved interaction;
`node docs/product/wireframes/generate.mjs` exits 0; 21 HTML/PNG pairs remain in
the manifest; CAP-0006/CAP-0011/CAP-0015 outputs show the correct defaults.

## Phase 3 — Run workspace gates and close the record

**Goal:** Deliver the narrow disclosure-default vertical slice with executable
evidence and complete repository records.

Steps:

1. Run `cargo fmt --check`.
2. Run `cargo clippy --workspace --all-targets -- -D warnings`.
3. Run `cargo test --workspace`.
4. Run `node --test crates/dms-desktop/ui/*.test.mjs`.
5. Complete the DOX pass against `docs/AGENTS.md`, `docs/product/AGENTS.md`,
   `docs/changes/AGENTS.md`, and `crates/dms-desktop/AGENTS.md`.
6. Record the passed gates, archive this CHG and its review artifacts, and update
   `docs/changes/README.md` in the same change. Commit the UI, tests, CAPs,
   generated wireframes, and CHG receipt as one vertical slice.

Verification gate: Every listed command exits 0; the CHG is archived and the
change index points to it.

## Out of scope

- A runtime browser link to repository-only CAP Markdown or to an unspecified
  documentation site.
- A workspace-wide history screen, new activity tab, external documentation
  subsystem, IPC command, `.dms` schema change, or saved preference.
- Any change to workflow event grouping, release records, verification verdicts,
  audit-export content, or Actions behaviour.
