# CHG-0053 — Markdown batch conversion with retained DOCX

**Plan ID:** CHG-0053-markdown-batch-conversion-retained-docx
**Created:** 2026-10-13
**Depends on:** none; reuse the archived CHG-0004 template pipeline without reopening its release scope.
**Entry checkpoint:** Windows checkout `main` at `9b71542`; Phase 1 changes remain uncommitted by operator instruction.
**Context sources:** Root/documentation/change/product DOX; CAP-0007; CAP-0022; architecture and privacy contracts; ADR-0008 and ADR-0023; CHG-0004 and CHG-0041; all three active CHGs; `crates/dms-core/src/template.rs`; `crates/dms-core/src/frontmatter.rs`; `crates/dms-desktop/src/export.rs`; `crates/dms-cli/src/main.rs`; the operator's workspace configuration and full-corpus assembly/diagnostic Word probes summarized below.
**Produces:** An explicit headless batch command that converts external Markdown through the configured Word template into paired PDF/DOCX artifacts, preserves tables and local images, retains the exact final Word input, and never registers, approves, releases, or mutates the source workspace.
**Status:** planned

| Field | Value |
| --- | --- |
| ID | CHG-0053 |
| Status | planned |
| External request | Direct operator request: Create a CHG in /mnt/c/Users/RaphaelBossek/src/dms/ to accomplish this task; there is already a CHG that should accomplish mass export -- I'm not sure. check this first. Incorporate your findings in the CHG |
| Affected CAPs | CAP-0007, CAP-0022 |
| Decision records | Implementation must amend ADR-0008 and the ADR-0023 boundary in `docs/architecture.md` for shared local Office automation and standalone non-release conversion. No architecture claim changes during planning. |

## Source request

Direct operator request: with ../dms we have a tool to convert/export markdown files to pdf. can you verify how the exported markdown files in docs/reworked/iso-27001/ could be mass-converted to PDF e.g. by calling CLI and using the details from /mnt/c/Users/RaphaelBossek/dms-demo2/edit/.dms/workspace.json The goal is to use the template and keep the docx file what was used to create the PDF

## Existing change ownership

The planning inspection found three active records and 49 archived records. None owns standalone Markdown batch conversion with retained DOCX.

- [CHG-0041 — Library batch execution activity](../archive/CHG-0041-library-batch-execution-activity.md) is the likely remembered batch change. It is done and covers only homogeneous batch add/unregister and a session-only result activity. Its out-of-scope section explicitly excludes bulk release and other lifecycle operations. Do not reopen it or extend its UI for this request.
- [CHG-0004 — Markdown Word-template release](../archive/CHG-0004-markdown-word-template-release.md) is done. Its selected decisions deliberately make DOCX temporary and place only approved versioned PDFs in the publish tree. Reuse its renderer and adapter, not its approval/release transaction for external conversion.
- [CHG-0042 — Publish-root released-document index](CHG-0042-publish-root-released-document-index.md) lists already released PDFs; it does not generate them. Converted artifacts must not enter that index.
- [CHG-0045 — Operator-defined library name](CHG-0045-operator-library-name.md) and [CHG-0050 — UUID MSRV ceiling](CHG-0050-uuid-msrv-ceiling.md) are independent. Preserve their work; do not duplicate their phases here.

CHG-0053 is the sole progress authority for this request. The findings below are baseline evidence, not completed implementation phases.

## Verified findings

### Applicability and real inputs

The initial probes ran against `/home/raphael/src/vc/dms`. During planning, the following files were byte-compared with this Windows checkout and were identical; this grounds the findings without claiming the original build ran in the Windows checkout:

| File | SHA-256 in both checkouts |
| --- | --- |
| `crates/dms-core/src/template.rs` | `89fa5cdbd722937257557ca79bbe24ec276bde5813c5aaee4062581626cc90ed` |
| `crates/dms-desktop/src/export.rs` | `9ee7e9f793110eeb2ad22c075a57954d0e5339d7e31381e9fde71f9b95ead33d` |
| `crates/dms-cli/src/main.rs` | `489c32fa26d05fb5adce23ad39fa9d6287f0ba132fac383cebd35725eeea5a92` |

- Input directory: `/home/raphael/src/vc/iso27001-documentation-rework/docs/reworked/iso-27001/`. It contains 49 root-level policy Markdown files; the 50th recursive Markdown match is `assets/README.md`, not a policy. Use exact policy selection, not an unrestricted recursive glob.
- The supplied configuration is `C:\Users\RaphaelBossek\dms-demo2\edit\.dms\workspace.json` (schema 19 at inspection). Its recorded roots use Windows verbatim paths. The template locator is `Vorlage.docx`, relative to the edit root; contract version is 1.
- Template SHA-256 is `a01d08d58a85e0a1f3ab29d225d5c8b710e7e94a506cbd72f4576ed033622fde`. The file matched the stored digest and passed `validate_markdown_template`.
- The `internal` confidentiality ID maps to `Intern + Unternehmensverbund`. Source frontmatter supplies title/version/classification; the corpus has no invented document numbers. Workspace identities, role routing, and approvals are not conversion inputs.

### Actual failures and supported path

1. Running the existing public assembler and controlled-field filler over all 49 policy files returned **0 assembled, 49 failed**. First failures were: 45 five-column tables, two three-column tables, and two four-column tables. The error is `Markdown tables must contain exactly two columns; found N`. `render_table` in `crates/dms-core/src/template.rs:973` imposes this restriction. Every policy also contains a mandatory five-column `Versionsverlauf`; removing history or flattening it to get an export is not acceptable.
2. There are **25 image references across 10 policies**. The CommonMark handler in `template.rs:680` does not render image events into drawings/media. A diagnostic excerpt preserved the image's alt text but embedded no source screenshot; the only media asset remained the template logo. Successful DOCX assembly alone is not fidelity evidence.
3. CLI `--help` and `document --help` expose no document conversion command. `report generate --format pdf` is an audit-report operation, not a document converter. CAP-0022 currently excludes PDF export and Office integration.
4. `LocalPdfExporter::export` in `crates/dms-desktop/src/export.rs:47` builds temporary `assembled.docx`, fills reserved chrome into temporary `release.docx`, and gives the latter to Word. The temporary directory then deletes both. Retaining `assembled.docx` would retain the wrong intermediate.
5. Windows Word COM is available. A clearly titled **DIAGNOSTIC ONLY** Uhrensynchronisation excerpt, ending before its unsupported version-history table, assembled and exported through Word 16.0 to a real four-page PDF while retaining DOCX. TOC/fields were updated, the DOCX saved, and that same open document exported. PDF parsing and first/last-page inspection confirmed logo, title, TOC, version `3.0`, classification and correct page count; the source image was absent. This was not a complete policy conversion and does not satisfy acceptance.
6. Native Linux export returns `installed Office export is supported only on Windows and macOS`. A WSL caller successfully invoked Windows PowerShell/Word using `wslpath -w` UNC paths. That bridge is diagnostic evidence, not existing Linux DMS support.

The probes did not modify source Markdown, the imported template, demo workspace configuration, or publish root. Temporary diagnostic binaries/documents are not plan dependencies; this record retains the relevant evidence and requires fresh reproducible tests below.

## Planned behavior and safety boundary

These are implementation targets, not claims about current runtime.

- Add `dms convert markdown --workspace-config <workspace.json> --input-root <directory> --output-root <new-directory> [--recursive] [--dry-run]`, retaining global `--json`. This command does **conversion**, never **Release**. Default selection is immediate `.md` children; recursion is explicit. Exclude `.dms` and asset subtrees from document discovery, but allow referenced images under the explicit input root. Reject empty input and report the exact selected set before conversion.
- Read only the export-configuration projection needed for roots, template, contract/digest, and confidentiality catalogue. Support the supplied schema-19 configuration and current schema explicitly; reject unsupported schemas. Do not call an open/migration/save path that rewrites `.dms`, creates backups/instructions, acquires locks, or activates an Entra session. Validate the configured edit-root locator against the actual config location on the native host; do not silently reinterpret Windows paths on Linux.
- This command accepts external sources outside the selected workspace edit/publish roots. Read title, numeric `MAJOR.MINOR` version and confidentiality ID from their flat frontmatter; resolve the ID to an enabled configured label. Missing/invalid values fail for that file. Leave absent document numbers empty. Use the existing optional-variable and reserved-token rules. Do not import owners, approvers, candidates or release history, infer approval from version history, or use external frontmatter to change registered-document authority.
- Require a new output root disjoint from the input tree, workspace edit tree and publish tree after canonical path/symlink checks. No overwrite or resume mode in this change. Mirror selected relative parents and write `<stem>.docx` and `<stem>.pdf`; preflight Windows case-insensitive/path-name collisions. Mark the manifest and command output explicitly as `conversion_only`, not released. A version/classification in document chrome is not evidence of approval.
- Preflight configuration, template SHA/contract, destinations and supported host before invoking Word. Keep deterministic sequential per-file execution; continue independent files after document-specific failure and return nonzero if any fail. Configuration/output failures stop before document work. No silent skipped files, worker queue, retries or parallel Word processes.
- Extend the existing OOXML renderer, preserving the template contract, styles, page setup, logo, headers/footers, numbering and TOC. Support tables with at least the observed three/four/five columns without losing text, column association or inline formatting. Preserve working two-column output; derive additional cell/grid widths from usable template width rather than adding new mandatory template markers.
- Resolve local image references relative to each source, constrain their canonical target to the explicit input root, embed bytes with collision-free package relationships and correct content types, preserve aspect ratio and constrain drawings to printable width. Shared assets across policies must work. Missing, corrupt, unsupported, remote or escaping assets fail clearly; never silently emit only alt text. PNG/JPEG are the required initial formats; other constructs must be explicitly supported or rejected rather than silently losing content. Inventory links, lists and inline HTML in the real corpus as part of fidelity testing; document the output link policy without claiming Markdown-relative links become working PDF links automatically.
- Keep Office automation outside `dms-core` and independent of Tauri. Extract the existing adapter into a shared, narrowly scoped export crate/module consumed by CLI and desktop; do not duplicate COM/AppleScript logic. Desktop release continues to use its approved snapshot and atomic PDF lifecycle contract.
- Retain the final, controlled-field-filled DOCX after Word updates fields, pagination and TOC and saves it. Export that same document to PDF without a later DOCX edit. Validate/read back both final files before recording success. Word failure, save failure, invalid PDF or incomplete output pair must never produce a success entry; clean owned incomplete files or explicitly record any retained diagnostic path. Do not close a user's unrelated Word documents or disable host security globally.
- Write a local manifest containing input relative paths and SHA-256, template identity/digest, referenced-asset digests, effective conversion metadata, final DOCX/PDF paths and SHA-256, renderer/Word version and per-file result/error. Hash the saved final DOCX, not the pre-Word intermediate. Do not copy the workspace's identity directory, source bodies or credentials into logs. Retained DOCX contains the same confidential content as PDF and needs equivalent access controls.
- Provide Windows and macOS adapters with explicit unsupported-host/missing-Word errors. Real Windows full-corpus evidence is required here; macOS installed-Word support must be honestly bounded unless separately exercised. No new Linux renderer or automatic WSL bridge is required.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Preserve multi-column policy tables through template assembly | done (G1: 11 tests passed on Rust 1.88.0; evidence below) | G1: `cargo test -p dms-core --test markdown_template` exits 0 with two/three/four/five-column fidelity coverage. |
| 2 | Embed local source images without silent content loss | pending | G2: `cargo test -p dms-core --test markdown_template --test lifecycle` exits 0 with image, containment, rejection and template-preservation coverage. |
| 3 | Deliver explicit non-release batch CLI and retained Word inputs | pending | G3: `cargo test --workspace` and `cargo run -p dms-cli -- convert markdown --help` exit 0 with batch, read-only configuration, retained-DOCX and unchanged-release coverage. |
| 4 | Prove full-corpus Windows Word conversion and repeatability | pending | G4: The native Windows smoke command specified below exits 0 with 49 verified PDF/DOCX pairs, all 25 image occurrences accounted for, retained source/template/config digests, and inspected rendered evidence. |
| 5 | Integrate contracts, records and project gates | pending | G5: Full integration commands and record checks below pass; CAP/DOX/visual classifications are current; only then archive CHG-0053. |

Keep at most one phase `in-progress`. Every done phase records its actual gate evidence; a fake Office exporter is not a real PDF smoke.

## Phase 1 — Preserve multi-column policy tables through template assembly

**Goal:** The existing public assembler accepts policy tables without dropping cells or changing the operator template.

1. Recheck the source checkpoint and read the applicable core DOX. Add synthetic, non-confidential fixtures to the existing `crates/dms-core/tests/markdown_template.rs` for three/four/five-column tables, including a five-column German version history, long text, empty cells, escaped punctuation, Unicode and inline emphasis.
2. Extend `render_table` and table layout generation in `crates/dms-core/src/template.rs`. Preserve the two-cell template's contract and existing two-column behavior; create valid cell/grid properties for additional columns and fit the printable width. Do not change or preprocess the real corpus to hide unsupported structures.
3. Assert OOXML cell counts, row order, actual cell text/style, valid package XML and byte-identical input/template. Add coverage against text loss across multiple tables, not only parser success.
4. Update CAP-0007's bounded implemented evidence and core DOX in this slice; do not promote the whole CAP while unrelated platform gaps remain.

**Verification gate:** G1: `cargo test -p dms-core --test markdown_template` exits 0 with two/three/four/five-column fidelity coverage.

**Execution evidence:** Phase 1 only, from `9b71542` on Rust 1.88.0 under WSL. The existing untracked CHG and its index entry are preserved; no staging, commit or push.

- G1: `cargo test -p dms-core --test markdown_template` exited 0: 11 passed, 0 failed. The new wider-table tests first failed against the two-column restriction, then passed with the implementation.
- `cargo test -p dms-core --locked`, `cargo clippy -p dms-core --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check` exited 0.
- `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .` exited 0 with no link issues.
- Synthetic OOXML assertions cover separate two/three/four/five-column tables, German version-history headings, exact row/cell text, empty cells, long Unicode text, escaped pipes/asterisks, mixed emphasis/code, inherited run styles, surrounding cell content, and unusable-width rejection before output creation. Wider grids/cell widths sum to the body section's page width minus margins/gutter, independently of a preceding cover section. Two-column widths remain unchanged.
- Tests preserve the cover/TOC, section properties, row/cell/paragraph styling, all non-body fixture package parts (including logo, headers/footers and styles), original Markdown/template bytes, and deterministic output; every output XML/relationship part is parsed.
- Local diff review completed. The independent reviewer returned no review because its provider hit HTTP 429; no independent-review pass is claimed.
- DOX: core ownership guidance and CAP-0007's bounded evidence updated. Parent DOX/indexes, CAP-0022, architecture/privacy and canonical wireframes intentionally unchanged: Phase 1 changes no package ownership, CLI/Office boundary, persistence or desktop interaction. CAP-0007 remains `not implemented` overall.
- No real corpus, workspace configuration or installed Word was accessed. Images, batch conversion, retained final DOCX, real PDF fidelity and workspace-wide integration gates remain in Phases 2–5.
- Record/index checks passed: one active CHG-0053 index entry, both affected CAPs present/indexed, Phase 1 done and Phases 2–5 pending, no active CHG marked done, empty Git index and unchanged `9b71542` HEAD.

## Phase 2 — Embed local source images without silent content loss

**Goal:** Template DOCX contains the source images as real drawings with correct relationships and layout.

1. Add an explicit source/asset context to assembly and update all callers. Keep path validation in core and Office outside it. Reject asset-bearing input when no resolution context is provided instead of ignoring it.
2. Add PNG/JPEG insertion, content types, noncolliding media/relationship IDs, dimensions, aspect-preserving sizing and safe source-relative lookup. Support references to shared assets elsewhere within the input root.
3. Test repeated images, template media/relationship collisions, image-only paragraphs, image text alternatives, images in supported containers, Unicode/space-containing paths, corrupted files, missing files, traversal, absolute/remote references and symlink escapes. Unsupported placement/format fails explicitly rather than claiming success.
4. Verify image bytes and drawing references in DOCX; preserve the template logo and all source/template bytes. Inventory other corpus Markdown constructs and add focused no-content-loss coverage where needed. Synchronize CAP-0007 evidence and relevant core/desktop DOX without adding new UI.

**Verification gate:** G2: `cargo test -p dms-core --test markdown_template --test lifecycle` exits 0 with image, containment, rejection and template-preservation coverage.

## Phase 3 — Deliver explicit non-release batch CLI and retained Word inputs

**Goal:** One explicit CLI invocation produces independently verified output pairs without touching DMS workflow state.

1. Read CLI/desktop DOX, then implement the command contract above in `crates/dms-cli/src/main.rs` with integration tests in `crates/dms-cli/tests/cli.rs`. Add read-only export-configuration validation in core with schema-19/current fixtures; prove no migration, metadata/frontmatter write, lock, Entra or notification call occurs.
2. Extract/reuse placeholder fill and installed-Word automation outside core and Tauri. Add the new ownership boundary and Cargo membership only as needed; update parent/child DOX immediately. Preserve existing desktop release tests and Windows/macOS behavior.
3. Implement final DOCX save before PDF export, paired-artifact validation and sequential batch results. Cover metadata mapping, reserved tokens, absent number, config/template mismatch, no Word, host limitations, collisions, output containment, no overwrite, dry-run, all-success, mixed failures and empty input. Test doubles must inspect the exact DOCX supplied to Word and prove it is the retained artifact; real pagination is phase 4 evidence.
4. Add manifest/hash verification and failure-safe ownership of incomplete files. Never claim a successful pair from a PDF signature alone; verify both artifacts and their recorded digests. Cover sources/assets changing during conversion by using the bytes actually consumed or refusing inconsistent inputs.
5. Update CAP-0022's PDF-export non-goal narrowly for standalone conversion, not approval/release. Add `**Primary surface:** none` to CAP-0022. CAP-0007 remains `**Primary surface:** human` for its existing desktop release surface; document shared rendering improvements separately from external conversion authority. Amend ADR-0008, ADR-0023/architecture and privacy for retained content and the new CLI/adapter boundary in this implementation slice.

**Verification gate:** G3: `cargo test --workspace` and `cargo run -p dms-cli -- convert markdown --help` exit 0 with batch, read-only configuration, retained-DOCX and unchanged-release coverage.

## Phase 4 — Prove full-corpus Windows Word conversion and repeatability

**Goal:** The unmodified real policy corpus converts completely with its template, images, version histories and final Word sources intact.

1. Add a reproducible `scripts/Test-MarkdownBatchExport.ps1` smoke harness with explicit `-WorkspaceConfig`, `-InputRoot`, `-OutputRoot` and `-ExpectedCount` parameters. This script is a planned deliverable, not an existing command. It invokes the built CLI and independently validates the manifest, output pairs, OOXML table/image content, PDF page/text evidence, unchanged inputs/configuration/template/publish tree, and no new release evidence. Use synthetic fixtures in automated tests; do not commit internal policy documents or the workspace identity cache.
2. From native Windows PowerShell in this repository, build `cargo build -p dms-cli --locked`, then run:

   ```powershell
   .\scripts\Test-MarkdownBatchExport.ps1 -WorkspaceConfig 'C:\Users\RaphaelBossek\dms-demo2\edit\.dms\workspace.json' -InputRoot 'L:\src\vc\iso27001-documentation-rework\docs\reworked\iso-27001' -OutputRoot 'C:\Users\RaphaelBossek\dms-conversion-verification\run-1' -ExpectedCount 49
   ```

   Preflight the mapped `L:` source location; use its actual accessible Windows/WSL UNC equivalent explicitly if the mapping is absent. Require a new empty output location and never redirect to the demo publish root. If the source corpus changes, reconcile the explicit file manifest and expected count before execution, rather than silently changing the acceptance denominator.
3. Verify 49 policy pairs, all five-column histories and their text, and all 25 source image occurrences across 10 policies. Count source drawings separately from the repeated template logo. Inspect PDF pages containing tables/images as well as cover, TOC and footer; confirm classification, source version, no unresolved placeholders, no missing text and no clipped columns/images. Persist per-file hashes/counts and non-secret inspection findings in this CHG; do not depend on chat or scratch files for completion evidence.
4. Repeat into a second fresh output root and verify the same complete source/asset coverage and unchanged originals; do not demand byte-identical PDFs because Word metadata can vary. Verify rerunning against an existing output root refuses overwrites. Exercise one controlled failing fixture to prove nonzero exit and an honest failure entry without false success.
5. Keep full output artifacts in the operator-selected local directory with equivalent confidentiality protection. If a review package is needed, create `docs/changes/reviews/CHG-0053/README.md` as `review-only`, link it here, add that ownership to change DOX/index, and keep sanitized evidence there at a stable path. Never place internal corpus copies or live workspace metadata in product wireframes.

**Verification gate:** G4: The native Windows smoke command specified above exits 0 with 49 verified PDF/DOCX pairs, all 25 image occurrences accounted for, retained source/template/config digests, and inspected rendered evidence.

## Phase 5 — Integrate contracts, records and project gates

**Goal:** Current CAPs and runtime contracts describe only proven behavior, with one completed change receipt.

1. Confirm CAP-0007/CAP-0022, architecture, ADR and privacy changes match implementation and test links. Retain honest macOS/Linux limits. Add concise operator invocation/output/failure documentation under the owning documentation contract; no separate progress plan.
2. Complete the DOX pass for changed adapter ownership, CLI input/output boundaries and retained-document disclosure. Update parent indexes where ownership changes. Existing desktop interactions are unchanged, so do not invent a batch UI or replace their canonical visuals with PDF diagnostics. Verify CAP-0007's existing human-surface HTML/PNG links remain valid and CAP-0022 declares `none`. If implementation actually changes a desktop interaction, load the capability-wireframes workflow and regenerate/render/inspect that CAP's canonical HTML/PNG in the same slice; retain any review package at its stable review-only path.
3. Run from the repository root:

   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --locked
   node --test crates/dms-desktop/ui/*.test.mjs
   python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .
   git diff --check
   ```

   Native Windows Word smoke G4 is additionally mandatory; Linux/fake-backed tests do not replace it. If dependency resolution is blocked, coordinate with CHG-0050 rather than silently raising the MSRV or expanding this change.
4. Perform record checks: all affected CAP IDs exist and appear in the product index; this CHG has one matching active/archive index entry; phase rows and numbered sections match; every completed phase cites its passing gate; CAP primary-surface declarations are explicit and truthful; all local links resolve. At planning inspection this repository has no dedicated record validator (`docs/AGENTS.md` explicitly says so). Use its existing record/index checks and strict link/diff gates; run a repository validator if one has been introduced by execution time, and do not invent a passed command.
5. Only after G1–G4 and the full integration gate pass, set CHG-0053 to done, move it to `docs/changes/archive/`, repair active-peer links, and update `docs/changes/README.md`. No other phase may archive this record. Do not commit or push without operator instruction.

**Verification gate:** G5: Full integration commands and record checks above pass; CAP/DOX/visual classifications are current; only then archive CHG-0053.

## Out of scope

- Batch registration, approval, release, notification, integrity-maintenance UI, or extending CHG-0041's activity pane.
- Changing existing released PDFs, release filenames/checksums/history, demo workspace schema or configured template bytes.
- Removing version history, omitting images, flattening tables into prose, or otherwise altering policy sources to make conversion succeed.
- New template authoring, multiple-template routing, server/cloud conversion, Pandoc, LibreOffice, WebView printing or a bundled Office runtime.
- Automatic recursion without explicit scope, resume/overwrite modes, parallel Word automation, background queues or a generalized job system.
- Claiming a complete corpus export from the successful diagnostic excerpt.

## Handoff

Phase 1 is verified and execution is stopped at the requested boundary. Phases 2–5 remain pending, with no phase in progress; the record remains active and `planned` for the next phase. Resume Phase 2 only on operator instruction. Do not commit or push.

## Related contracts

- [CAP-0007](../../product/capabilities/CAP-0007-draft-pdf-export.md)
- [CAP-0022](../../product/capabilities/CAP-0022-headless-core-cli.md)
- [Architecture](../../architecture.md), [decisions](../../design-decisions.md), [privacy](../../privacy.md)
- [Markdown metadata and template variables](../../markdown-frontmatter-and-template-variables.md)
