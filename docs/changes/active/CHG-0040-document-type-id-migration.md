# CHG-0040 — Document type-ID migration

Retain a document type’s stable source ID while mapping it to an enabled replacement ID for every future release candidate: existing document control data, active evidence, and released snapshots keep the source ID; candidates created after the mapping snapshot the replacement ID, which becomes the immutable release profile on successful release. Every Document control data view resolves that ID through the current catalogue and shows `Display label (type-id)` to the operator.

**Plan ID:** CHG-0040-document-type-id-migration
**Execution slot:** P0810
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** CHG-0030 is archived with the retained confidentiality type-ID migration pattern; document types require the same candidate/release snapshot boundary without confidentiality policies, PDF filenames, or Markdown frontmatter projection.
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `docs/product/capabilities/CAP-0001-local-folder-dms.md` (Outcome 8); `docs/product/capabilities/CAP-0002-document-lifecycle.md` (Outcomes 7, 11, 19); `docs/product/capabilities/CAP-0013-library-maintenance.md` (Outcome 12); `docs/product/capabilities/CAP-0015-document-control-data.md` (Outcomes 1, 9–10); `docs/changes/archive/CHG-0030-confidentiality-type-id-migration.md`; `crates/AGENTS.md`; `crates/dms-core/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/catalogues.rs` (`DocumentType`, `Workspace::configure_document_type`); `crates/dms-core/src/lib.rs` (`SCHEMA_VERSION`, `Workspace::open`, `Workspace::update_control`); `crates/dms-core/src/lifecycle.rs` (`CandidateMetadataSnapshot`, candidate-current validation and invalidation); `crates/dms-desktop/src/lib.rs` (`configure_document_type`, Tauri command registration); `crates/dms-desktop/ui/configuration.mjs` (`documentTypesMarkup`, `configurationMutationRequest`); `crates/dms-core/tests/policies.rs`; `crates/dms-core/tests/lifecycle.rs`; `crates/dms-desktop/ui/configuration.test.mjs`; `docs/product/wireframes/generate.mjs`.
**Produces:** A persisted, non-cyclic document type replacement mapping; candidates and releases that snapshot the resolved replacement type ID without rewriting live document control data or historical evidence; Document control data views that render the current catalogue label followed by that retained ID as `Display label (type-id)`; an explicit Configuration → Document defaults migration control; focused core/desktop coverage; and current CAP/wireframe evidence.
**Status:** pending — queued after P0800; implementation has not begun.
**Filename convention:** The repository's active-record contract requires `CHG-*.md`; `P0810` is this CHG's execution-order authority and does not change that filename convention.

| Field | Value |
| --- | --- |
| ID | CHG-0040 |
| Status | pending |
| External request | Direct operator request: "What we did with CHG-0030 for confidentiality type id realise accordingly to \"Document types\"" Follow-up: "for document control data the Display label not the type-id" and clarification: show `{Display label} ({type-id})`. |
| Affected CAPs | CAP-0001, CAP-0002, CAP-0013, CAP-0015 |
| Decision records | No new ADR. This applies the established stable-catalogue and candidate/release-snapshot boundary to document types; it does not change the confidentiality-specific ADR-0010 policy model. |

## Current state

- `DocumentType` persists only `id`, `label`, and `enabled`; disabling it is rejected only while a current document control profile refers to that ID (`crates/dms-core/src/catalogues.rs:5-42`).
- Document control keeps its `document_type` as a mutable current-profile value. `Workspace::update_control` validates only enabled configured IDs, records an auditable before/after change, invalidates stale candidates, and re-syncs Markdown's unrelated controlled keys (`crates/dms-core/src/lib.rs:687-724`).
- Candidate and release metadata snapshot a complete `DocumentControl`, so a document type selected at candidate submission becomes immutable release-profile evidence (`crates/dms-core/src/lifecycle.rs:80-87, 1071-1090`).
- `Workspace::open` migrates v1–v15, validates the converted shape, retains a versioned `.dms/workspace.v<old>.json.bak`, then atomically saves (`crates/dms-core/src/lib.rs:35, 436-513`).
- Configuration → Document defaults already renders the document-type catalogue and emits only `configure_document_type`; confidentiality migration is a separate secondary configuration surface because it also owns folder policy (`crates/dms-desktop/ui/configuration.mjs:219-274, 441-452`).
- The Library selection pane currently shows the raw `document_type` ID in its Document control data summary, shows only the label in the edit selector, and interpolates the raw ID again in its current-release profile (`crates/dms-desktop/ui/library.mjs:1117-1148`). Its detail response already carries `document_types`, so this presentation rule needs no second persistence field.
- CHG-0030's migration retains source references, invalidates affected active candidates, and projects a replacement only where confidentiality owns future-release state. Document types deliberately have no folder policy, PDF filename segment, or Markdown frontmatter key (`docs/changes/archive/CHG-0030-confidentiality-type-id-migration.md:26-34, 48-59`).

## Risk call-out

The source document type must not be renamed, deleted, or rewritten across live document control data, open-candidate/review evidence, or release history. Those values are audit evidence or describe the current draft. Only a candidate created after a valid mapping may snapshot the replacement ID. If creating or changing a mapping changes an active candidate's resolved document type, invalidate that candidate instead of releasing a review approved for the source type. A display-label rename changes only operator presentation: it must never replace the retained ID or create a second label snapshot in document-control, candidate, or release records.

The schema migration changes portable `.dms` metadata. Follow the existing open-time migration transaction: deserialize and validate the v16 → v17 shape before replacing `workspace.json`, retain `workspace.v16.json.bak`, and do not retry after a partial failure without restoring the original metadata from that backup.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Implement retained document-type mapping and candidate/release resolution | pending | `cargo test -p dms-core` exits 0 with v16→v17 backup migration, retained-reference, invalid mapping, active-candidate invalidation, candidate snapshot, and release-profile coverage. |
| 2 | Expose the Document defaults migration control and human-readable document-type views | pending | `cargo test -p dms-desktop`, `node --test crates/dms-desktop/ui/configuration.test.mjs`, and `node --test crates/dms-desktop/ui/library.test.mjs` exit 0 with explicit source/replacement command, refreshed snapshot, and `Display label (type-id)` in current and immutable Document control data views. |
| 3 | Publish current contracts, wireframe evidence, and close | pending | `node docs/product/wireframes/generate.mjs`, CAP-0013 and CAP-0015 PNG render commands, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree. |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Retained mapping and future candidate snapshots

**Goal:** A source document type remains a valid current-profile and historical ID, while a non-cyclic enabled replacement is resolved only into future candidate and release-profile snapshots.

Steps:

1. Add an optional `replacement_type_id` to the persisted `DocumentType`, increment the workspace schema to v17, and add the v16 → v17 migration that defaults every catalogue entry to `null`. Preserve the current migration safety sequence and fixture proof that `.dms/workspace.v16.json.bak` is retained.
2. Add a core `Workspace::migrate_document_type(source_type_id, replacement_type_id)` operation. It validates portable IDs, requires both configured IDs, requires an enabled distinct target, rejects cyclic chains, preserves the source type and every existing document-control reference, and persists the source mapping. Updating a document type must preserve its existing mapping; neither source nor target may be disabled while it is the source or target of a retained mapping.
3. Resolve the complete replacement chain only while constructing and validating future candidate metadata. The candidate’s `metadata.control.document_type` must hold the resolved replacement ID; the live `Document.control.document_type` remains the source ID. When a mapping changes, use the existing stale-candidate path so any affected active candidate is invalidated with canonical evidence and returns the document to `draft`.
4. Preserve candidate/release immutability: an existing candidate, its review decision, and every historical release keep their already-snapshotted source/replacement document type. A successful later release stores the candidate's replacement snapshot in its immutable profile.
5. Add focused core tests for mapping persistence and v16 migration backup; missing, disabled, identical, and cyclic targets; disabled source/target rejection; retained live document profile; active-candidate invalidation; future-candidate replacement snapshot; release-profile retention; and no mutation of Markdown frontmatter or release PDF filenames. Keep `DocumentControl.document_type` and every candidate/release snapshot as IDs only; display labels remain a frontend catalogue lookup.

Verification gate: `cargo test -p dms-core` exits 0 with v16→v17 backup migration, retained-reference, invalid mapping, active-candidate invalidation, candidate snapshot, and release-profile coverage.

## Phase 2 — Document defaults migration control and document-type presentation

**Goal:** Configuration → Document defaults lets an operator explicitly select an enabled replacement document type for future candidates without reproducing core validation in the WebView, while Document control data renders every resolved document type as `Display label (type-id)`.

Steps:

1. Add a narrow desktop `migrate_document_type` command that accepts explicit source and replacement IDs, calls the core operation, saves through the existing workspace-configuration mutation path, and returns the refreshed configuration snapshot. Register it with the Tauri invoke handler and prove its persistence/error path in adapter tests.
2. Extend the existing **Document types** catalogue rows with a **Future release type** selector containing only other enabled document types and a **Migrate future candidates** / **Update future-candidate migration** action. Keep document-type management in Document defaults; do not create a confidentiality-style secondary page, because document types have no folder-policy surface.
3. Add a `document-type-migration` form mapper that submits only trimmed explicit source/replacement IDs. The catalogue's migration selector and each Document control data rendering use `Display label (type-id)`; the type ID remains the form value and never becomes a display-label lookup key. Keep disabled state when no eligible target exists, error presentation, and state refresh consistent with the existing catalogue mutations; do not reproduce enabled-target, reference, or cycle validation in JavaScript.
4. In the Library selection pane, resolve `detail.control.document_type` and each candidate/current-release profile type ID against `detail.document_types`, then show `Display label (type-id)` in the current-control summary, edit selector, and immutable profile. A catalogue label rename refreshes those presentation strings; it must not mutate document control, candidate, release, or audit data. Preserve an explicit visible ID-only fallback only for corrupt/legacy detail responses whose catalogue entry is unavailable.
5. Add adapter and frontend tests for command registration/persisted snapshot, exact request payload, selector eligibility, retained source-ID copy, current-control and immutable-profile `Display label (type-id)` rendering, and the distinction that the mapping changes future candidates—not the current document profile, existing evidence, PDF names, or Markdown frontmatter.

Verification gate: `cargo test -p dms-desktop`, `node --test crates/dms-desktop/ui/configuration.test.mjs`, and `node --test crates/dms-desktop/ui/library.test.mjs` exit 0 with explicit source/replacement command, refreshed snapshot, `Display label (type-id)` rendering, and no duplicated frontend validation.

## Phase 3 — Current contracts, wireframes, and close

**Goal:** Current product records distinguish retained document-control IDs from future candidate/release snapshots and specify the `Display label (type-id)` presentation rule accurately.

Steps:

1. Amend CAP-0001 only to identify document-type replacement mappings as workspace metadata when implemented. Amend CAP-0013 to replace its simple add/rename/disable rule with retained-ID migration and reference protection. Amend CAP-0015 to state that document-type migrations preserve current control data and history while candidates prepared after the mapping snapshot the replacement ID; all Document control data views render the current catalogue's `Display label (type-id)`, so a later label rename changes presentation only. Amend CAP-0002 only where its candidate/release-profile snapshot outcome needs the document-type migration consequence. Do not alter CAP-0008 or ADR-0010's confidentiality-specific policy contract.
2. Update the CAP-0013 and CAP-0015 wireframe definitions with a Document defaults **Future release type** selector/action and a current-control plus later candidate/release profile rendered as `Display label (type-id)`. Regenerate generated HTML, index, manifest, and PNG exports; visually inspect both PNGs. Do not hand-edit generated outputs.
3. Update the active CAP and CHG indexes, record passing evidence in this CHG, mark phases done only after their gates pass, then archive the CHG and refresh `docs/changes/README.md`. Update the relevant `AGENTS.md` only if the implementation changes a durable ownership or operating contract.

Verification gate: `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0013-library-maintenance.png "file://$PWD/html/CAP-0013-library-maintenance.html" && test -s exports/CAP-0013-library-maintenance.png)`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0015-document-control-data.png "file://$PWD/html/CAP-0015-document-control-data.html" && test -s exports/CAP-0015-document-control-data.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree.

## Out of scope

- Rewriting existing document-control profiles, open-candidate/review evidence, release profiles, audit events, source files, or historical PDFs.
- Replacing a persisted document type ID with its display label, or snapshotting a mutable display label as candidate/release evidence.
- Adding document type to PDF filenames, Office markers, Markdown controlled frontmatter, confidentiality policies, or export chrome.
- Renaming or deleting a document type ID, bulk-releasing documents, or providing an implicit migration based on matching display labels.
- Adding a `dms` CLI migration command; CHG-0030's corresponding migration is deliberately available through the core and DMS Desktop configuration surface only.
