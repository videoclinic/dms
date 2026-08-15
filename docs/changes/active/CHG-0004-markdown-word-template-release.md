# CHG-0004 — Markdown Word-template release pipeline

| Field | Value |
| --- | --- |
| ID | CHG-0004 |
| Status | in-progress — Phases 1–4 complete; Phase 5 pending |
| External request | Direct operator request: “In order to export Markdown files to PDF the idea was to use a Word template like .temp/Vorlage.docx This template could/should be imported into the library for reuse. The frontmatter should be then used in the Word document template as metadata for \"auto-fields\" if possible.” |
| Affected CAPs | CAP-0001, CAP-0002, CAP-0005, CAP-0006, CAP-0007, CAP-0015 |
| Decision records | ADR-0008 |

**Plan ID:** `CHG-0004-markdown-word-template-release`
**Execution slot:** P0195
**Created:** 2026-08-15
**Depends on:** `CHG-0001#phase-9k.5` (`202e796`)
**Plan family:** `CHG-0001-tauri-local-dms-bootstrap`
**Entry checkpoint:** Phase 9k.5 is committed and pushed at `202e796`; `main`
is synchronized with `origin/main`.
**Context sources:** ADR-0008; CAP-0002 outcomes 4 and 19; CAP-0005
Configuration contracts; CAP-0006 library membership boundaries; CAP-0007;
CAP-0015 outcomes 1 and 13; `crates/dms-core/src/lib.rs` workspace schema and
migration tests; `crates/dms-core/src/lifecycle.rs` content checks and export
requests; `crates/dms-desktop/src/export.rs`; Configuration and Library adapters,
frontend modules, and focused tests.
**Produces:** A workspace-selected reusable Word template asset and one
Office-backed Markdown release path that generates a temporary DOCX, fills
release-controlled fields, exports it through installed Word, and preserves the
existing atomic PDF commit and evidence boundary.

## Goal

Replace the WebView Markdown PDF path with a canonical Word-template pipeline.
An operator imports one `.docx` under the edit root as the workspace Markdown
export template. The asset has a stable template ID and edit-relative locator,
is reusable by every Markdown library document, and is excluded from controlled
document lifecycle actions.

The Markdown source keeps flat YAML frontmatter as source metadata. DMS release
state remains authoritative: the candidate/release snapshot supplies title,
document number, target version, and effective confidentiality to the generated
Word document. Frontmatter version and confidentiality are required to match the
candidate snapshot; title and document number are checked when present. A
mismatch blocks review/release with expected and detected values. Frontmatter
never overwrites `.dms` control data.

## Selected decisions

- Markdown release requires licensed Microsoft Word on Windows and macOS.
- One workspace has one active Markdown export-template asset imported from a
  `.docx` under the edit root.
- The template asset is workspace configuration, not an ordinary controlled
  document; it has no notes, review, release, or document permalink.
- The generated DOCX is temporary. Only the versioned PDF enters the publish
  tree; the Markdown source and imported template are never rewritten.
- The release snapshot is authoritative for controlled fields. Frontmatter is a
  validation surface, not a synchronization source.
- Word `DOCPROPERTY` fields are the selected visible field mechanism. The
  temporary-copy filler writes `DMS_TITLE`, `DMS_DOCUMENT_NUMBER`, `DMS_VERSION`,
  and `DMS_CONFIDENTIALITY` custom-property values from the release snapshot;
  literal placeholders remain deterministic package values, not a second export
  path.
- Do not add Pandoc, LibreOffice, a cloud converter, or another installed runtime
  prerequisite. Markdown-to-Word assembly uses the existing Rust/CommonMark and
  OOXML stack plus installed Word for final pagination and PDF export.

## Scope

- Workspace schema migration for one optional template asset: stable UUID,
  edit-root-relative `.docx` path, validation digest, and validated template
  contract version.
- Configuration → Document defaults controls to import, replace, inspect, and
  remove the active template. Markdown review/release fails closed when no valid
  template is configured.
- Library classification that distinguishes the template asset from controlled
  documents and from files available to add. It is not counted as a document or
  offered document lifecycle actions.
- A strict reusable-template contract derived from the operator Vorlage:
  preserved package styles, headers, footers, relationships, media, page setup,
  and field/token locations; explicit body insertion/style prototypes for
  headings, paragraphs, lists, and tables; clear validation errors for missing
  or ambiguous required parts.
- Markdown frontmatter parsing and snapshot comparison without copying source
  values into DMS document control data.
- Temporary DOCX generation from the CommonMark event stream, followed by the
  existing installed-Word PDF export adapter, PDF validation, SHA-256, atomic
  rename, and release commit.
- Removal of the WebView Markdown printer, print-shell assets, and native
  WebView PDF smoke once replacement gates prove the Word route.
- CAP, ADR, architecture, DOX, wireframe, packaging, and external-smoke updates.

## Out of scope

- Multiple templates per workspace or per-document template selection
- Treating templates as controlled documents
- Importing control data from Word properties or frontmatter
- Editing the source Markdown or imported template during release
- Pandoc, LibreOffice, server-side conversion, or bundled Office
- Supporting arbitrary YAML objects or executable template expressions
- Claiming macOS installed-Word evidence before an external macOS smoke exists

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Template field mechanism feasibility | complete | Focused temporary-copy tests prove all four controlled values fill XML custom-property values without mutating the source; a Windows Word smoke using the operator template produces a readable multi-page A4 PDF with visible title, number, version, and confidentiality; the CHG selects `DOCPROPERTY` fields and records only non-secret path, page-count, and checksum evidence |
| 2 | Workspace template asset, frontmatter, and OOXML contract | complete | Schema-v14 migration preserves every v13 workspace and adds no template implicitly; core tests prove in-root `.docx` import, stable identity, replace/remove, changed-template revalidation, symlink/outside-root refusal, lifecycle exclusion, required/optional frontmatter comparisons, deterministic template validation, CommonMark block insertion, preservation of headers/footers/styles/media/relationships, and valid DOCX packaging; CLI remains headless and does not invoke Word |
| 3 | Configuration and Library template surfaces | complete | Desktop/frontend tests prove Document defaults import/replace/remove/status controls, clear missing/invalid template errors, and Library exclusion from controlled/available counters and actions; CAP-linked HTML/PNG wireframes render without clipping |
| 4 | Canonical Word-backed Markdown release | complete | Focused exporter/lifecycle tests prove Markdown → temporary template DOCX → installed-Word adapter → validated PDF; version/confidentiality/title/number come from the release snapshot; frontmatter mismatches block with expected/detected details; export failure leaves no release/version; the original Markdown/template remain byte-identical; WebView Markdown runtime code and smoke are removed |
| 5 | Platform, records, and packaging closeout | pending | Rust format/workspace tests/Clippy, frontend tests, release builds, strict record/link/table checks, and `git diff --check` pass; Windows external smoke releases the A.8.29 Markdown through the imported Vorlage and retains non-secret PDF path/checksum evidence; macOS keeps explicit pending installed-Word evidence unless separately proven; CAP/ADR/architecture/DOX describe only evidenced behaviour |

## Phase 1 — Template field mechanism feasibility

1. Add failing temporary-copy tests for the four release-controlled field values:
   title, document number, version, and confidentiality.
2. Extend Office XML fill across custom-property values while preserving source
   bytes and the existing PDF handoff boundary.
3. Add `DOCPROPERTY` custom properties and visible fields to the operator
   `.temp/Vorlage.docx` without making that ignored operator asset a repository
   dependency.
4. Spike field updates through Word on the
   Windows host. Select them only if repeated open/update/export runs produce
   stable visible values; otherwise pin literal temporary-copy replacement.
5. Exercise the real operator template with representative controlled values and
   inspect the generated PDF. Record only non-secret paths, dimensions, page
   count, and checksum evidence.
6. Update this CHG's selected field mechanism before marking the phase done.

### Phase 1 evidence — 2026-08-15

- `dms-desktop` tests
  `docx_export_replaces_body_header_and_footer_tokens_on_a_copy` and
  `office_placeholder_fill_supports_word_document_property_values` prove all
  four controlled values are filled in the temporary package, including
  `docProps/custom.xml`, while the source bytes remain unchanged.
- The ignored operator asset `.temp/Vorlage.docx` now contains visible Word
  `DOCPROPERTY` fields for title, document number, version, and confidentiality;
  SHA-256:
  `30c70677f23c9fa1ea8a6c9b1f4e1f4f5906ebde92632d1eb1c58db7e056bedf`.
- Two Windows Microsoft Word exports from the same filled temporary DOCX produced
  identical extracted text with all four visible values. The repeated output is
  a two-page A4 PDF at
  `C:\Users\Raphael_Bossek\AppData\Local\Temp\dms-template-fields-filled-repeat.pdf`;
  SHA-256:
  `a2c57428ed3b126d6206e411a52f9d0554fd881af5625bdf966f9a50a7c0c1e5`.
- `cargo fmt --check`, `cargo test --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`, repository-wide strict
  Markdown link validation, changed-contract table validation, and
  `git diff --check` pass.

## Phase 2 — Workspace template asset, frontmatter, and OOXML contract

1. Add schema v14 and migration/backup tests for an optional workspace template
   record without assigning an implicit template.
2. Add core operations for import, inspect, replace, revalidate, and remove. The
   source must be an ordinary `.docx` file under the edit root and may not be a
   registered controlled document.
3. Exclude the active template path from addable/unsupported/library document
   counts and lifecycle operations while keeping it visible as workspace
   configuration.
4. Parse only the supported flat frontmatter keys. Require `version` and
   `confidentiality`; compare optional `title` and `document_number` when present.
   Reject duplicates, non-scalar values, malformed delimiters, and mismatches
   with explicit expected/detected details.
5. Keep DMS title, document number, target version, and effective
   confidentiality authoritative in every export request.
6. Add a small tracked synthetic DOCX fixture with the same structural contract
   as the operator Vorlage but no proprietary logo or business content.
7. Add failing tests for template package validation, body/style prototypes,
   preserved headers/footers/media/relationships, Markdown headings,
   paragraphs, emphasis, links, lists, code, and tables.
8. Implement the smallest direct OOXML assembler using existing
   `pulldown-cmark`, `quick-xml`, and `zip` dependencies. Do not implement a
   parallel HTML/altChunk/Pandoc fallback.

### Phase 2 evidence — 2026-08-15

- Schema v14 stores one optional stable template ID, edit-root-relative path,
  SHA-256, and template-contract version. The v13 migration retains
  `workspace.v13.json.bak` and leaves the template unset.
- Core import/revalidation tests prove stable replacement identity,
  changed/missing/invalid state reporting, in-root regular-file enforcement,
  symlink and outside-root refusal, removal, persistence, and exclusion from
  document registration and Library rows/counters.
- Markdown lifecycle tests require scalar `version` and `confidentiality`
  frontmatter and expose expected/detected mismatch evidence for optional
  `title` and `document_number`; source values never mutate `.dms` control data.
- The tracked synthetic `markdown-template.docx` fixture contains no proprietary
  branding. Focused assembler tests cover headings, paragraphs, emphasis, links,
  ordered/unordered lists, code, and two-column tables; output is byte-stable,
  XML-well-formed, and preserves every non-body package part including styles,
  headers, footers, media, and relationships.
- `cargo fmt --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` pass. The headless CLI
  remains free of Word invocation.

## Phase 3 — Configuration and Library template surfaces

1. Add the template asset controls under Configuration → Document defaults.
2. Use the native file picker constrained to `.docx` under the edit root; show
   exact source path, validation state, and replacement/removal consequences.
3. Keep the asset out of controlled-document selection, counts, batch actions,
   notes, lifecycle, and permalinks.
4. Add adapter/frontend tests and update the CAP-0005/CAP-0006/CAP-0007
   wireframes and DOX contracts.

### Phase 3 evidence — 2026-08-15

- Configuration → Document defaults exposes native `.docx` selection, exact
  template ID and relative path, validation state, stable-ID replacement
  consequences, and explicitly confirmed removal consequences. Picker cancel
  leaves the configuration unchanged.
- The desktop adapter delegates import/removal and validation to `dms-core`.
  Focused adapter coverage proves persistence, valid-state reporting, confirmed
  removal, and that the configured asset is absent from Library rows until its
  configuration is removed.
- Frontend tests prove the configured/unconfigured surfaces, escaped exact path,
  validation labels, narrow IPC requests, and replacement/removal explanations.
  The complete frontend suite passes all 84 tests.
- CAP-0005, CAP-0006, and CAP-0007 HTML and PNG wireframes were regenerated at
  1440×1100. Visual inspection found no clipping, overlap, broken glyph, or
  action-hierarchy defect; CAP-0007 contains no stale WebView/print-shell path.
  The 21-entry manifest resolves non-empty HTML/PNG pairs.
- `cargo fmt --all -- --check`, `cargo test --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`, strict repository
  links, Markdown table validation, and `git diff --check` pass.

## Phase 4 — Canonical Word-backed Markdown release

1. Route `.md` export through template resolution, frontmatter validation,
   direct OOXML assembly, controlled-field fill, and the existing Word PDF
   adapter.
2. Update Word fields before export when Phase 1 selected `DOCPROPERTY`; otherwise
   replace the selected literal tokens in the temporary package.
3. Preserve the existing temporary-target, valid-PDF, digest, atomic rename, and
   rollback behaviour.
4. Remove the native WebView Markdown printer, print-shell assets, platform PDF
   API dependencies, and their obsolete smoke path. Do not retain dual runtime
   exporters.
5. Add regression coverage for the A.8.29 failure class: frontmatter is accepted
   as the Markdown source marker surface, body duplication is unnecessary, and
   the PDF receives the release snapshot's controlled values.

### Phase 4 evidence — 2026-08-15

- `dms-core` resolves and revalidates the configured template immediately before
  Markdown export. Missing, changed, or invalid assets fail closed before the
  exporter runs and leave lifecycle state and version allocation unchanged.
- The desktop exporter assembles Markdown into a temporary template-backed DOCX,
  fills title, document number, version, and confidentiality from the `.dms`
  release snapshot, then invokes the existing installed-Office adapter. Focused
  tests prove the source Markdown and configured template remain byte-identical.
- Direct `.docx` export retains its existing temporary-copy Office path. The
  native WebView Markdown printer, print assets, platform PDF dependencies, and
  `DMS_DESKTOP_EXPORT_SMOKE` route are removed; Windows/macOS CI now runs the
  deterministic fake-backed template/Office test.
- The reusable template contract requires `docProps/custom.xml` with exactly one
  placeholder for each `DMS_TITLE`, `DMS_DOCUMENT_NUMBER`, `DMS_VERSION`, and
  `DMS_CONFIDENTIALITY` property.
- `cargo fmt --all -- --check`, `cargo test --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and all 84 frontend
  tests pass. Installed-Word PDF evidence remains exclusively a Phase 5 gate.

## Phase 5 — Platform, records, and packaging closeout

1. Run all local workspace, frontend, records, wireframe, and package gates.
2. Build the native Windows application and release the controlled A.8.29
   Markdown using the imported operator Vorlage and installed Word.
3. Verify PDF page count, visible template chrome, title/number/version/
   confidentiality fields, filename, SHA-256, publish-tree placement, and release
   evidence without retaining document content in the CHG.
4. Update CAP statuses conservatively: record Windows evidence and keep macOS
   installed-Word evidence explicit if unavailable.
5. Mark this CHG done and archive it. CHG-0001 Phase 9l may resume only from that
   pushed checkpoint.

## Risks and stop conditions

- Stop Phase 1 if direct OOXML assembly cannot preserve the operator template or
  representative CommonMark structures without a second converter/runtime.
- Stop rather than silently accepting a template with ambiguous body/style
  prototypes or fields.
- Stop on any mutation of the source Markdown or imported template during
  release.
- Stop if controlled values can be sourced from frontmatter instead of the
  release snapshot.
- Stop on a release record without a valid atomically committed PDF.
- Stop CAP promotion on any unproved host claim.
