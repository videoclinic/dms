# CHG-0033 — Bounded Office source-history import

**Plan ID:** CHG-0033-bounded-office-source-history-import
**Execution slot:** P0300
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** Phase 1 commit `1590e05`.
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/architecture.md` (Dual-root path model, Document data domains); `docs/privacy.md` (Data classes, Processing principles); `docs/design-decisions.md` (ADR-0003, ADR-0004, ADR-0013, ADR-0017); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcomes 3, 6); `docs/product/capabilities/CAP-0011-approval-evidence.md` (Outcomes 1, 4, 8); `docs/product/capabilities/CAP-0012-audit-export.md` (Outcomes 2, 3); `docs/product/capabilities/CAP-0015-document-control-data.md` (Outcomes 1, 13; Non-goals); `crates/AGENTS.md`; `crates/dms-core/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/lib.rs` (`Workspace::add_document_inner`, schema migration); `crates/dms-core/src/library.rs` (`Workspace::add_documents`); `crates/dms-core/src/audit.rs`; `crates/dms-desktop/src/lib.rs` (`add_library_documents`); `crates/dms-desktop/ui/app.mjs` (library add action); `crates/dms-desktop/ui/library.mjs` (selection detail); `docs/product/wireframes/AGENTS.md`; `docs/product/wireframes/generate.mjs`
**Produces:** The first registration of an Office draft retains at most three newest source-derived, person-and-date observations tied to that imported draft's SHA-256. They are visibly and exportably labelled unverified, never become DMS workflow/release history, and are never re-scanned or overwritten on re-registration.
**Status:** done — Phase 1 committed (`1590e05`); Phase 2 committed (`4a580f0`); Phase 3 gates passed.

First-time Library import of a `.docx`, `.xlsx`, or `.pptx` scans the local OOXML package and stores no more than the three newest attributable source-change observations; the document remains a new DMS `draft` and source-derived data stays separate from DMS workflow and release evidence.

| Field | Value |
| --- | --- |
| ID | CHG-0033 |
| Status | done |
| External request | Direct operator request: "Recover only the last 3 changes from a file that is imported first time to the Library, not the whole history." |
| Affected CAPs | CAP-0006, CAP-0011, CAP-0012, CAP-0015 |
| Decision records | ADR-0030. ADR-0003, ADR-0004, ADR-0013, and ADR-0017 remain applicable. |

## Current state

- Library add calls `Workspace::add_documents`; only a newly created Office document record captures a bounded source-history record before the desktop adapter persists it.
- A newly added document remains a `draft` with empty releases and canonical `workflow_events`; source history is a separate unverified record with no workflow-event, candidate, or release fields.
- The core store is schema v17. Its v16 migration writes `source_history: null` for existing documents and retains `v16.json.bak`.
- CAP-0015 keeps DMS ownership of document control data while allowing bounded, first-import-only Office source provenance that never supplies control fields or source-version history.
- Canonical workflow events remain Entra/local-user evidence with predecessor hashes. Source-derived author strings have no event IDs/hashes or identity authority.
- Word and Excel retain only attributable date-bearing revision groups; PowerPoint retains a sanitized unattributed outcome without client IDs. Microsoft 365 version history remains a OneDrive/SharePoint service, not a sequence of local-file snapshots.
- The Library selection pane and audit reports present at most three `source-derived/unverified` observations without source content, raw metadata, client IDs, or workflow hashes.

## Risk call-out

This is a schema migration and a privacy expansion: embedded revision authors and dates are personal data, and source packages can be malformed or deliberately edited. Retain only the capped, normalized observations plus the imported source SHA-256 and scan outcome; do not retain changed text, comment text, raw XML, Office core properties, client IDs, or an inferred Entra identity. The existing v16 metadata backup and fail-closed migration path are the recovery mechanism.

The three recovered observations are source claims, not DMS actions. They must never receive workflow event IDs/hashes, release versions, candidate versions, approval state, or identity authority. A rescan after source bytes change would rewrite provenance, so it is forbidden: scan only when creating a new document record; re-registering an unregistered record preserves its original source-history record unchanged.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Add bounded, format-aware source-history recovery to `dms-core` | done (`2026-08-28`: `source_history_import` and `workspace` passed) | `cargo test -p dms-core --test source_history_import` and `cargo test -p dms-core --test workspace` exit 0, including v16 migration, cap, attribution, and re-registration cases |
| 2 | Apply recovery only on first Library add and render/export it distinctly | done (`2026-08-28`: `cargo test -p dms-desktop`, `node --test crates/dms-desktop/ui/library.test.mjs`, and `cargo test -p dms-core --test audit` passed) | `cargo test -p dms-desktop`, `node --test crates/dms-desktop/ui/library.test.mjs`, and `cargo test -p dms-core --test audit` exit 0, including the three-row imported-source section and no workflow-chain mutation |
| 3 | Publish contracts, wireframe, and completed records | done (`2026-08-28`: generated and rendered CAP-0006/CAP-0015 wireframes; `cargo fmt`, Clippy, workspace tests, and 112 frontend tests passed) | `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0006-library-explorer.png "file://$PWD/html/CAP-0006-library-explorer.html" && test -s exports/CAP-0006-library-explorer.png && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0015-document-control-data.png "file://$PWD/html/CAP-0015-document-control-data.html" && test -s exports/CAP-0015-document-control-data.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP/ADR/CHG indexes agree |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Add bounded, format-aware source-history recovery to `dms-core`

**Goal:** A new Office document record can retain at most three newest attributable, source-derived observations from the exact bytes first registered, while existing workspaces migrate without changing documents that were already registered.

**Atomicity rationale:** The parser, persistence boundary, migration, batch preflight, and focused fixtures form one core-only schema slice; splitting them would leave an unsafe partially persisted source-history shape.

Steps:

1. Add a Tauri-independent `source_history` module in `crates/dms-core/src/`. Define a serde-defaulted optional document field for one first-import recovery record: imported-source SHA-256, source format, normalized scan outcome, and at most three observations. Each observation contains only an untrusted display author, a validated UTC source timestamp, a format-specific change category, and a compact count of coalesced low-level records. Do not add changed text, comments, core properties, raw XML, a revision ID, a source path, an Entra object ID, or a workflow-event field.
2. Use the existing `zip`, `quick-xml`, `chrono`, and `sha2` dependencies to scan a regular OOXML package without extracting files to disk. Reject traversal/duplicate/oversized package parts safely. A malformed or unsupported revision part produces a sanitized non-fatal scan outcome, not an import failure and not a guessed observation.
3. Implement only attributable, date-bearing source revisions:
   - DOCX: read WordprocessingML revision elements (`w:ins`, `w:del`, move, row, paragraph, and run-property changes). Group elements with the same author and timestamp into one source change so run-level markup cannot consume the three-row budget; preserve only category/count summaries.
   - XLSX: read legacy SpreadsheetML revision parts and their user-name mapping. Emit an observation only when a revision resolves both a non-empty user name and a valid timestamp; group same-person/same-time revision records.
   - PPTX: inspect the Revision Information part through its package relationship. Its client revision records provide an application-client ID and date but no reliable person-name mapping, so record a non-fatal `unattributed_revision_data` outcome and emit no person-and-date observation. Do not display or persist the client ID.
   Sort attributable groups by descending timestamp, retain the newest three, and mark equal-timestamp groups as coalesced rather than inventing an ordering. Do not turn `creator`, `lastModifiedBy`, filesystem times, or comment authors into source changes.
4. Bump `SCHEMA_VERSION` from v16 to v17. Migrate every existing document to `source_history: null`, retain `v16.json.bak`, and preserve the existing unknown-newer-schema read-only behavior. Add migration fixtures and verification before altering `workspace.json`.
5. Extend only the new-record branch of `Workspace::add_document_inner` to calculate the source SHA-256 and attach the recovery result before insertion. Preserve the existing batch-add atomicity: validate all paths and calculate every recovery result before mutating the in-memory document map. The existing-record re-registration branch must retain the old recovery unchanged and must not read the current source again.
6. Add `crates/dms-core/tests/source_history_import.rs` using synthetic OOXML ZIP fixtures. Prove DOCX grouping/cap/order, XLSX user/time resolution, PPTX unattributed handling, malformed packages, missing/no-data packages, v16 migration with backup, first-add persistence, re-registration immutability after the file changes, and no workflow-event/release/candidate creation.

Verification gate: `cargo test -p dms-core --test source_history_import` and `cargo test -p dms-core --test workspace` exit 0.

## Phase 2 — Apply recovery only on first Library add and render/export it distinctly

**Goal:** The normal Add to library action records source-derived observations exactly once, and operators can read/export them without confusing them with DMS workflow history.

Steps:

1. Keep `dms-desktop` and `dms-cli` on the existing `dms-core` add API so both invoke the one-time recovery path. Do not add a desktop-only metadata scanner, a separate import command, or a later rescan action. Return the recovery state only as normal selected-document detail data; errors remain sanitized and do not expose XML or source content.
2. In `crates/dms-desktop/ui/library.mjs`, add a read-only **Imported source changes (unverified)** subsection inside the existing **Version history & changes** topic. It shows at most three newest rows with source person, date/time, and a concise kind/count summary, plus the captured-source-digest relationship in operator wording. It visually separates these rows from the hash-chained DMS workflow blocks and states that DMS did not verify the actor or reconstruct earlier versions. Empty, malformed, no-data, and PowerPoint-unattributed outcomes show a bounded explanatory state rather than raw parser diagnostics. Do not add edit, delete, approve, release, or version controls.
3. Update the Library detail model and tests so a successful one-file add refreshes to the persisted recovery record; batch add handles each new Office source independently; an add failure stays atomic; and selecting a re-registered document renders its original recovery even after external file changes.
4. Extend `crates/dms-core/src/audit.rs` and its report model so a document's source-history section exports the import source SHA-256, scan outcome, and at most three normalized observations. Label every row `source-derived/unverified`; omit content, raw metadata, client IDs, and workflow hashes. Keep report generation itself as the existing `report_generated` event only; never add derived observations to the canonical workflow chain.
5. Add focused core, desktop-adapter, frontend, and audit tests proving three-row rendering/export, DMS workflow history remains unchanged, no Entra matching occurs, content never appears in output, and source-history data stays stable after later source edits.

Verification gate: `cargo test -p dms-desktop`, `node --test crates/dms-desktop/ui/library.test.mjs`, and `cargo test -p dms-core --test audit` exit 0.

## Phase 3 — Publish contracts, wireframe, and completed records

**Goal:** Product records describe bounded, unverified source-history recovery as delivered behavior, including its privacy boundary and first-import-only rule.

Steps:

1. Add ADR-0030 to `docs/design-decisions.md`: DMS preserves at most three attributable Office source-change observations from first import, tied to the original bytes, separate from canonical workflow/release evidence. Record the format boundaries, the no-rescan rule, and the explicit rejection of Entra identity inference and version/release reconstruction.
2. Amend CAP-0015 to replace its source-metadata import non-goal with this bounded provenance outcome while preserving DMS ownership of document control data. Amend CAP-0011 to state that source-derived observations are outside the canonical hash chain. Amend CAP-0012 to specify the bounded, no-content audit representation. Amend CAP-0006 with the selection-pane location and non-interactive states.
3. Amend `docs/architecture.md` and `docs/privacy.md` to classify source-history observations and their SHA-256 binding as local personal-data metadata. State the strict content/raw-XML/client-ID exclusions and the fact that PowerPoint's client IDs are neither retained nor displayed.
4. Update the existing CAP-0006 screen definition in `docs/product/wireframes/generate.mjs` with a synthetic three-row Imported source changes (unverified) section plus an empty/unattributed state. Regenerate its HTML, index, and manifest with `node docs/product/wireframes/generate.mjs`; render the CAP-0006 PNG and the CAP-0015 PNG that shares its generated selection-pane definition into `docs/product/wireframes/exports/`; retain the CAPs' existing HTML/PNG links. Do not create a pending-only wireframe artifact.
5. Factor the existing source-history package-preflight result into a named
   core type so the full workspace Clippy gate accepts the Phase 1 parser
   without suppressing `type_complexity`.
6. Run the complete workspace gates. Once every gate passes, mark phases with concrete evidence, move this CHG to `docs/changes/archive/`, and update `docs/changes/README.md` from Active to Archive. Report the pre-existing CHG-0024/0025/0032 work unchanged.

Verification gate: `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0006-library-explorer.png "file://$PWD/html/CAP-0006-library-explorer.html" && test -s exports/CAP-0006-library-explorer.png && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0015-document-control-data.png "file://$PWD/html/CAP-0015-document-control-data.html" && test -s exports/CAP-0015-document-control-data.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP/ADR/CHG indexes agree.

## Out of scope

- Reconstructing or creating DMS releases, candidate versions, approvals, changelogs, or canonical workflow events from Office data.
- Importing more than three attributable observations, offering a source-history rescan, or rewriting the original capture after re-registration.
- Persisting Word changed text, comments, Office core/custom properties, raw OOXML, Excel cell values, or PowerPoint client IDs.
- Matching Office author strings or any Office identifier to a Microsoft Entra user, e-mail address, owner, editor, or approver.
- Retrieving OneDrive/SharePoint Microsoft 365 version history, snapshots, or document content through Microsoft Graph.
