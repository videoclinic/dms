# CHG-0040 — Document type-ID migration

Retain a document type’s stable source ID while mapping it to an enabled replacement ID for every future release candidate: existing document control data, active evidence, and released snapshots keep the source ID; candidates created after the mapping snapshot the replacement ID, which becomes the immutable release profile on successful release. Every Document control data view resolves that ID through the current catalogue and shows `Display label (type-id)` to the operator.

**Plan ID:** CHG-0040-document-type-id-migration
**Execution slot:** P0810
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** CHG-0030 is archived with the retained confidentiality type-ID migration pattern; document types require the same candidate/release snapshot boundary without confidentiality policies, PDF filenames, or Markdown frontmatter projection.
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `docs/product/capabilities/CAP-0001-local-folder-dms.md` (Outcome 8); `docs/product/capabilities/CAP-0002-document-lifecycle.md` (Outcomes 7, 11, 19); `docs/product/capabilities/CAP-0013-library-maintenance.md` (Outcome 12); `docs/product/capabilities/CAP-0015-document-control-data.md` (Outcomes 1, 9–10); `docs/changes/archive/CHG-0030-confidentiality-type-id-migration.md`; `crates/AGENTS.md`; `crates/dms-core/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/catalogues.rs` (`DocumentType`, `Workspace::configure_document_type`); `crates/dms-core/src/lib.rs` (`SCHEMA_VERSION`, `Workspace::open`, `Workspace::update_control`); `crates/dms-core/src/lifecycle.rs` (`CandidateMetadataSnapshot`, candidate-current validation and invalidation); `crates/dms-desktop/src/lib.rs` (`configure_document_type`, Tauri command registration); `crates/dms-desktop/ui/configuration.mjs` (`documentTypesMarkup`, `configurationMutationRequest`); `crates/dms-core/tests/policies.rs`; `crates/dms-core/tests/lifecycle.rs`; `crates/dms-desktop/ui/configuration.test.mjs`; `docs/product/wireframes/generate.mjs`.
**Produces:** A persisted, non-cyclic document type replacement mapping; candidates and releases that snapshot the resolved replacement type ID without rewriting live document control data or historical evidence; Document control data views that render the current catalogue label followed by that retained ID as `Display label (type-id)`; an explicit Configuration → Document defaults migration control; focused core/desktop coverage; and current CAP/wireframe evidence.
**Status:** done — all three phases passed their gates, the reviewer cleared the final documentation correction, and the reviewed delta is integrated in the primary checkout without a commit.
**Filename convention:** The repository's active-record contract requires `CHG-*.md`; `P0810` is this CHG's execution-order authority and does not change that filename convention.

| Field | Value |
| --- | --- |
| ID | CHG-0040 |
| Status | done |
| External request | Direct operator request: "What we did with CHG-0030 for confidentiality type id realise accordingly to \"Document types\"" Follow-up: "for document control data the Display label not the type-id" and clarification: show `{Display label} ({type-id})`. |
| Affected CAPs | CAP-0001, CAP-0002, CAP-0013, CAP-0015 |
| Decision records | No new ADR. This applies the established stable-catalogue and candidate/release-snapshot boundary to document types; it does not change the confidentiality-specific ADR-0010 policy model. |

## Current state

- `DocumentType` persists `id`, `label`, `enabled`, and an optional replacement ID. A mapping retains source references, requires an enabled non-cyclic target, preserves mappings across source-label updates, and prevents disabling either side (`crates/dms-core/src/catalogues.rs`).
- Document control keeps its `document_type` as a mutable current-profile value. `Workspace::update_control` validates only enabled configured IDs, records an auditable before/after change, invalidates stale candidates, and re-syncs Markdown's unrelated controlled keys (`crates/dms-core/src/lib.rs:687-724`).
- Candidate and release metadata snapshot a complete `DocumentControl`; candidate construction and freshness validation resolve any document-type replacement while live control data remains unchanged (`crates/dms-core/src/lifecycle.rs`).
- `Workspace::open` migrates through v19, validates the converted shape, retains a versioned `.dms/workspace.v<old>.json.bak`, then atomically saves. The v18 → v19 migration writes `replacement_type_id: null` for existing document types (`crates/dms-core/src/lib.rs`).
- Configuration → Document defaults renders document-type catalogue edits and explicit source/replacement migration forms through `migrate_document_type`; the adapter delegates validation to core, saves through the existing configuration mutation path, and returns a refreshed snapshot. Confidentiality migration remains a separate secondary surface (`crates/dms-desktop/ui/configuration.mjs`, `crates/dms-desktop/src/lib.rs`).
- The Library selection pane resolves retained `document_type` IDs through `detail.document_types` as `Display label (type-id)` in its current-control summary, edit selector, and immutable current-release profile. Missing catalogue entries remain visible as IDs, including the selected edit option; presentation never follows replacement mappings or writes labels into metadata (`crates/dms-desktop/ui/library.mjs`).
- CHG-0030's migration retains source references, invalidates affected active candidates, and projects a replacement only where confidentiality owns future-release state. Document types deliberately have no folder policy, PDF filename segment, or Markdown frontmatter key (`docs/changes/archive/CHG-0030-confidentiality-type-id-migration.md:26-34, 48-59`).

## Risk call-out

The source document type must not be renamed, deleted, or rewritten across live document control data, open-candidate/review evidence, or release history. Those values are audit evidence or describe the current draft. Only a candidate created after a valid mapping may snapshot the replacement ID. If creating or changing a mapping changes an active candidate's resolved document type, invalidate that candidate instead of releasing a review approved for the source type. A display-label rename changes only operator presentation: it must never replace the retained ID or create a second label snapshot in document-control, candidate, or release records.

The schema migration changes portable `.dms` metadata. Follow the existing open-time migration transaction: deserialize and validate the v18 → v19 shape before replacing `workspace.json`, retain `workspace.v18.json.bak`, and do not retry after a partial failure without restoring the original metadata from that backup.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Implement retained document-type mapping and candidate/release resolution | done (`cargo fmt --all -- --check` and `cargo test -p dms-core` passed) | `cargo test -p dms-core` exits 0 with v18→v19 backup migration, retained-reference, invalid mapping, active-candidate invalidation, candidate snapshot, and release-profile coverage. |
| 2 | Expose the Document defaults migration control and human-readable document-type views | done (all three gate commands exit 0; 92 desktop, 27 Configuration, and 40 Library tests pass) | `cargo test -p dms-desktop`, `node --test crates/dms-desktop/ui/configuration.test.mjs`, and `node --test crates/dms-desktop/ui/library.test.mjs` exit 0 with explicit source/replacement command, refreshed snapshot, and `Display label (type-id)` in current and immutable Document control data views. |
| 3 | Publish current contracts, wireframe evidence, and close | done (all named commands exit 0; 188 Rust and 145 frontend tests pass; both 1600×1600 PNGs visually inspected) | `node docs/product/wireframes/generate.mjs`, CAP-0013 and CAP-0015 PNG render commands, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree. |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Retained mapping and future candidate snapshots

**Goal:** A source document type remains a valid current-profile and historical ID, while a non-cyclic enabled replacement is resolved only into future candidate and release-profile snapshots.

Steps:

1. Add an optional `replacement_type_id` to the persisted `DocumentType`, increment the workspace schema to v19, and add the v18 → v19 migration that defaults every catalogue entry to `null`. Preserve the current migration safety sequence and fixture proof that `.dms/workspace.v18.json.bak` is retained.
2. Add a core `Workspace::migrate_document_type(source_type_id, replacement_type_id)` operation. It validates portable IDs, requires both configured IDs, requires an enabled distinct target, rejects cyclic chains, preserves the source type and every existing document-control reference, and persists the source mapping. Updating a document type must preserve its existing mapping; neither source nor target may be disabled while it is the source or target of a retained mapping.
3. Resolve the complete replacement chain only while constructing and validating future candidate metadata. The candidate’s `metadata.control.document_type` must hold the resolved replacement ID; the live `Document.control.document_type` remains the source ID. When a mapping changes, use the existing stale-candidate path so any affected active candidate is invalidated with canonical evidence and returns the document to `draft`.
4. Preserve candidate/release immutability: an existing candidate, its review decision, and every historical release keep their already-snapshotted source/replacement document type. A successful later release stores the candidate's replacement snapshot in its immutable profile.
5. Add focused core tests for mapping persistence and v18 migration backup; missing, disabled, identical, and cyclic targets; disabled source/target rejection; retained live document profile; active-candidate invalidation; future-candidate replacement snapshot; release-profile retention; and no mutation of Markdown frontmatter or release PDF filenames. Keep `DocumentControl.document_type` and every candidate/release snapshot as IDs only; display labels remain a frontend catalogue lookup.

Verification gate: `cargo test -p dms-core` exits 0 with v18→v19 backup migration, retained-reference, invalid mapping, active-candidate invalidation, candidate snapshot, and release-profile coverage.

## Phase 2 — Document defaults migration control and document-type presentation

**Goal:** Configuration → Document defaults lets an operator explicitly select an enabled replacement document type for future candidates without reproducing core validation in the WebView, while Document control data renders every resolved document type as `Display label (type-id)`.

Steps:

1. Add a narrow desktop `migrate_document_type` command that accepts explicit source and replacement IDs, calls the core operation, saves through the existing workspace-configuration mutation path, and returns the refreshed configuration snapshot. Register it with the Tauri invoke handler and prove its persistence/error path in adapter tests.
2. Extend the existing **Document types** catalogue rows with a **Future release type** selector containing only other enabled document types and a **Migrate future candidates** / **Update future-candidate migration** action. Keep document-type management in Document defaults; do not create a confidentiality-style secondary page, because document types have no folder-policy surface.
3. Add a `document-type-migration` form mapper that submits only trimmed explicit source/replacement IDs. The catalogue's migration selector and each Document control data rendering use `Display label (type-id)`; the type ID remains the form value and never becomes a display-label lookup key. Keep disabled state when no eligible target exists, error presentation, and state refresh consistent with the existing catalogue mutations; do not reproduce enabled-target, reference, or cycle validation in JavaScript.
4. In the Library selection pane, resolve `detail.control.document_type` and each candidate/current-release profile type ID against `detail.document_types`, then show `Display label (type-id)` in the current-control summary, edit selector, and immutable profile. A catalogue label rename refreshes those presentation strings; it must not mutate document control, candidate, release, or audit data. Preserve an explicit visible ID-only fallback only for corrupt/legacy detail responses whose catalogue entry is unavailable.
5. Add adapter and frontend tests for command registration/persisted snapshot, exact request payload, selector eligibility, retained source-ID copy, current-control and immutable-profile `Display label (type-id)` rendering, and the distinction that the mapping changes future candidates—not the current document profile, existing evidence, PDF names, or Markdown frontmatter.

Verification gate: `cargo test -p dms-desktop`, `node --test crates/dms-desktop/ui/configuration.test.mjs`, and `node --test crates/dms-desktop/ui/library.test.mjs` exit 0 with explicit source/replacement command, refreshed snapshot, `Display label (type-id)` rendering, and no duplicated frontend validation.

### Phase 2 evidence

- `cargo test -p dms-desktop` — exit 0; 92 tests pass. The adapter tests check invoke-handler registration, the returned mapping against a reopened workspace, and unchanged saved metadata after missing, disabled, identical, and cyclic mapping errors.
- `node --test crates/dms-desktop/ui/configuration.test.mjs` — exit 0; 27 tests pass. Coverage includes retained source IDs, enabled distinct target options, disabled empty-target controls, mapping preselection/action text, exact trimmed-ID payloads, refreshed state/error presentation, and future-candidate-only copy.
- `node --test crates/dms-desktop/ui/library.test.mjs` — exit 0; 40 tests pass. Coverage includes current/immutable catalogue labels, label renames without detail mutation, ID-only submission, distinct current/release IDs, disabled retained entries, and absent-catalogue fallback.
- `cargo fmt --all -- --check` and `git diff --check` — exit 0. The initial Library run exited 1 on its old label-only expectation; the initial format check exited 1 on new assertion wrapping. Both were corrected before the passing gates above.
- The existing configuration submit/refresh/error path is reused. `ui/styles.css` shares the existing responsive migration-row rule between both catalogues; no new secondary page or frontend domain validation was added.
- Verification is Linux/WSL adapter and framework-free Node coverage, not live WebView IPC, native Windows/macOS execution, or an independent review. The primary-checkout overlay still matches the task packet hash. CAPs, wireframes, indexes, Phase 3, and all AGENTS.md files remain unchanged: this slice retains existing ownership and core-validation contracts; product-contract publication remains Phase 3 work.

## Phase 3 — Current contracts, wireframes, and close

**Goal:** Current product records distinguish retained document-control IDs from future candidate/release snapshots and specify the `Display label (type-id)` presentation rule accurately.

Steps:

1. Amend CAP-0001 only to identify document-type replacement mappings as workspace metadata when implemented. Amend CAP-0013 to replace its simple add/rename/disable rule with retained-ID migration and reference protection. Amend CAP-0015 to state that document-type migrations preserve current control data and history while candidates prepared after the mapping snapshot the replacement ID; all Document control data views render the current catalogue's `Display label (type-id)`, so a later label rename changes presentation only. Amend CAP-0002 only where its candidate/release-profile snapshot outcome needs the document-type migration consequence. Do not alter CAP-0008 or ADR-0010's confidentiality-specific policy contract.
2. Update the CAP-0013 and CAP-0015 wireframe definitions with a Document defaults **Future release type** selector/action and a current-control plus later candidate/release profile rendered as `Display label (type-id)`. Regenerate generated HTML, index, manifest, and PNG exports; visually inspect both PNGs. Do not hand-edit generated outputs.
3. Update the active CAP and CHG indexes, record passing evidence in this CHG, mark phases done only after their gates pass, then archive the CHG and refresh `docs/changes/README.md`. Update the relevant `AGENTS.md` only if the implementation changes a durable ownership or operating contract.

Verification gate: `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0013-library-maintenance.png "file://$PWD/html/CAP-0013-library-maintenance.html" && test -s exports/CAP-0013-library-maintenance.png)`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0015-document-control-data.png "file://$PWD/html/CAP-0015-document-control-data.html" && test -s exports/CAP-0015-document-control-data.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree.

### Phase 3 evidence

- Scope: CAP-0001 outcome 8; CAP-0002 candidate snapshot boundary; CAP-0013
  document-type catalogue slice; CAP-0015 retained IDs and current-catalogue
  presentation. CAP-0013 retains its overall `not implemented` status and links
  the implemented slice's tests. No runtime or confidentiality-contract changes.
- `node docs/product/wireframes/generate.mjs` — exit 0. It regenerated all 21
  HTML screens, index, and manifest; only CAP-0013/CAP-0015 HTML and CAP-0013
  manifest navigation changed. The index and other screens remain byte-identical.
- Both exact Chrome screenshot commands in the Phase 3 gate above — exit 0,
  including `test -s`. Final exports are 1600×1600: CAP-0013 is 176903 bytes;
  CAP-0015 is 255941 bytes after the scoped corrections below. Chromium emitted
  non-fatal GPU/portal warnings.
- CAP-0013 visual inspection passed: it highlights Configuration
  and Document defaults, shows the two Future release type selectors and both
  migration actions, and explains retained IDs/reference protection. Its broader
  Maintenance reference is folded and explicitly separate. CAP-0015 shows
  Procedure (procedure) in current control and the edit selector, Policy (policy)
  in the immutable V1.3 release, and Procedure (procedure) for earlier V1.2.
  Its wireframe status agrees with its implemented CAP; the PDF path wraps.
- CAP-0015 correction: the prior inspection incorrectly reported the full
  explanation visible; the 1600×1600 PNG clipped it after “Every label comes
  from the”. Only CAP-0015's scroll-area stack gaps and hint margins were
  tightened in `generate.mjs`; all explanation text and the docked Actions
  footer are unchanged. Generator and the exact CAP-0015 Chrome gate command
  (including `test -s`) exited 0. Full-image and full-resolution crop inspection
  confirm both candidate-boundary and presentation-only paragraphs are visible
  through “not stored IDs.”, with the Open latest released PDF button above
  Actions. That explanation-only inspection did not resolve the footer's
  separate fixed-height Evaluate changes with Claude label clipping.
  A repeat generator run exited 0 and left artifacts byte-identical. Only the
  generator, CAP-0015 HTML/PNG, and this evidence changed from the correction
  baseline; other screens, index, manifest, CAPs, and runtime files are unchanged.
- CAP-0015 Actions correction: the screen-scoped footer button rule now uses
  automatic height with a 2rem minimum, 0.125rem/0.375rem padding, 0.75rem text,
  and 1.1 line-height. All existing labels and the docked, foldable Actions
  structure are preserved. Generator and the exact CAP-0015 Chrome gate command
  above (including `test -s`) exited 0; the export is 1600×1600, 255941 bytes.
  Full-image and full-resolution crop inspection confirm all six Actions labels
  fit without clipping or overlap, including the complete “Evaluate changes
  with Claude” label. Both release explanation paragraphs remain visible through
  “not stored IDs.”, and Open latest released PDF stays above the docked footer.
  This follow-up edits only the CAP-0015 generator CSS, regenerated CAP-0015
  HTML/PNG, and this receipt. The Rust/frontend results below are retained Phase 3
  evidence, not reruns for these CSS-only corrections.
- `cargo fmt --all -- --check` — exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo test --workspace` — exit 0; 188 passed, 0 failed, 0 ignored.
- `node --test crates/dms-desktop/ui/*.test.mjs` — exit 0; 145 passed, 0 failed.
- `git diff --check` — exit 0 before and after archival and after the scoped
  CAP-0015 explanation and Actions corrections. Generator repeatability and
  manifest/HTML/PNG inventory checks passed for all 21 screens; every screen
  appears in the HTML index and its CAP links both artifacts.
- Final review found one CAP-0002 outcome-4 antecedent ambiguity: `It may also
  stage` could appear to assign Owner/Editor staging to a document-type
  migration. It was corrected to `The candidate may also stage`, then a fresh
  configured reviewer-model session returned `No findings`. The primary checkout
  reran every Phase 3 gate with the reviewer-cleared diff.
- DOX pass: root, documentation, product, changes, and wireframe AGENTS.md
  contracts remain unchanged because ownership, operating rules, and child
  boundaries did not change. All four affected CAP statuses agree with the
  product index; the change index moves CHG-0040 to the archive. Sidecar CHG
  links follow it. A stdlib audit resolved all 384 product/change Markdown links
  and matched both CHG index sections to disk (4 active, 45 archived).
- Supplemental whole-product status auditing found one unrelated baseline
  mismatch: CAP-0011 says `implemented`, while the product index says
  `not implemented`. Both files are byte-identical to HEAD and remain untouched.
  An additional untracked-file whitespace check flags the copied sidecars'
  intentional Markdown hard-break spaces; these are preserved, not runtime or
  task-packet gate failures. No repository `records:check` command exists.
- This receipt describes the isolated worktree only. Phase 1/2 runtime overlay
  is preserved. No commit, push, deployment, secret access, primary-checkout
  integration, or new CHG was performed. Linux/WSL tests and static PNG inspection
  do not establish live WebView, installed-Office, Windows, or macOS behavior.
  The orchestrator's fresh configured reviewer-model review remains pending;
  this implementer does not claim an independent `No findings` verdict.

## Out of scope

- Rewriting existing document-control profiles, open-candidate/review evidence, release profiles, audit events, source files, or historical PDFs.
- Replacing a persisted document type ID with its display label, or snapshotting a mutable display label as candidate/release evidence.
- Adding document type to PDF filenames, Office markers, Markdown controlled frontmatter, confidentiality policies, or export chrome.
- Renaming or deleting a document type ID, bulk-releasing documents, or providing an implicit migration based on matching display labels.
- Adding a `dms` CLI migration command; CHG-0030's corresponding migration is deliberately available through the core and DMS Desktop configuration surface only.
