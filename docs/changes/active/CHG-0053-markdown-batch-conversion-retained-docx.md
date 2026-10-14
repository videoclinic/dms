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
| External request | Direct operator request: Create a CHG in /mnt/c/Users/RaphaelBossek/src/dms/ to accomplish this task; there is already a CHG that should accomplish mass export -- I'm not sure. check this first. Incorporate your findings in the CHG. Follow-on: Incorporate the findings regarding converting the markdown document in the ../dms project using the application-records skill for future improvements. Using the dms cli tool would be prefered then. |
| Affected CAPs | CAP-0007, CAP-0022 |
| Decision records | ADR-0008 and ADR-0023 in `docs/architecture.md` describe the shared local Word adapter and standalone non-release conversion boundary. |

## Source request

Direct operator request: with ../dms we have a tool to convert/export markdown files to pdf. can you verify how the exported markdown files in docs/reworked/iso-27001/ could be mass-converted to PDF e.g. by calling CLI and using the details from /mnt/c/Users/RaphaelBossek/dms-demo2/edit/.dms/workspace.json The goal is to use the template and keep the docx file what was used to create the PDF

Follow-on direct operator request: Are the DOC-002 translation/render insights relevant for `../dms`; if so, create the corresponding change record.

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

### DOC-001 review-render findings

- The supplied `/mnt/c/Users/RaphaelBossek/dms-demo/edit/.dms/workspace.json` is schema 18. Its configured `Vorlage.docx` locator, contract version 1 and SHA-256 `a01d08d58a85e0a1f3ab29d225d5c8b710e7e94a506cbd72f4576ed033622fde` were verified. A configuration-only converter must support that schema as well as schema 19 and the current schema.
- The current `dms --help` exposes no `convert` command. An external review renderer can call `dms-core` assembly and Word, but that is a stopgap, not the preferred product command.
- The template's static TOC is a `w:sdt` with `w:docPartGallery` value `Table of Contents`. The CLI must retain and update it by default, and remove only that structured-document tag when an explicit `--without-toc` option is requested.
- Current two-column assembly inherits the template's `w:tblW type="auto"` and a 6,041-twip grid, while the template's printable width is 9,072 twips. Every generated Markdown table must instead use fixed layout, `w:tblW` and a grid/cell-width sum equal to the printable width; this applies to two-column tables as well as three-, four- and five-column tables.
- Microsoft Word opened a DOCX from the WSL UNC path but rejected `ExportAsFixedFormat` to that UNC destination. The CLI's native-Windows adapter remains the supported path; if a WSL bridge is intentionally supported later, it must stage the temporary PDF under the Windows user profile, copy the validated result back, and clean it up. This does not establish Linux DMS export support.

### DOC-002 translation-render adaptation findings

The DOC-002 local review pair confirms that these findings belong to CHG-0053;
it already owns standalone Markdown conversion with retained final DOCX, local
image embedding, Word field/TOC updates and native rendered-output inspection.
No second CHG is created because that would duplicate its progress authority.

1. The first external renderer invocation passed no `MarkdownAssetContext` and
   correctly failed on the local process-map image. This validates the Phase-2
   fail-closed contract; it is not a new `dms-core` defect. DMS release and
   standalone conversion already pass their explicit context. Phase 4 must use
   the built CLI rather than a wrapper that calls assembly with `None`.
2. Word produced a valid retained DOCX and PDF from the same post-save document.
   The rendered process map was embedded, readable and within the printable
   width. This is a single-document adaptation observation, not the required
   49-document native-Windows proof. Phase 4 remains responsible for counting
   every source image occurrence separately from template media and for visual
   inspection of image/table pages.
3. A template that auto-numbers heading styles must not also receive source
   headings with manual section-number prefixes. The resulting duplicate numbers
   are a source/template composition issue, not a reason for the CLI to rewrite
   Markdown. Phase 4 must inspect the generated TOC and headings, record the
   operator-selected source/template convention, and fail the smoke harness on
   duplicate heading numbers or unresolved TOC fields.
4. The external review renderer locally translated static German footer labels.
   That localisation is outside the standalone converter's contract: DMS
   preserves static template text and fills only configured controlled fields.
   Word strips trailing ordinary spaces from edited field-label runs, so any
   template-localisation path must preserve a separator (for example a
   non-breaking space) and verify the rendered footer. Do not add a hidden
   English-label rewrite to `dms convert markdown`.
5. The four-page DOC-002 pair left the final three review-criteria bullets on a
   mostly blank page. This is layout evidence that structural DOCX/PDF checks do
   not replace the Phase-4 visual review; it does not authorize automatic source
   rewriting, margin changes or content removal.

## Planned behavior and safety boundary

These are implementation targets, not claims about current runtime.

- Add `dms convert markdown --workspace-config <workspace.json> --input-root <directory> --output-root <new-directory> [--recursive] [--dry-run] [--without-toc]`, retaining global `--json`. This command does **conversion**, never **Release**. Default selection is immediate `.md` children; recursion is explicit. Exclude `.dms` and asset subtrees from document discovery, but allow referenced images under the explicit input root. Reject empty input and report the exact selected set before conversion.
- Read only the export-configuration projection needed for roots, template, contract/digest, and confidentiality catalogue. Support schema 18, schema 19 and the current schema explicitly; reject unsupported schemas. Do not call an open/migration/save path that rewrites `.dms`, creates backups/instructions, acquires locks, or activates an Entra session. Validate the configured edit-root locator against the actual config location on the native host; do not silently reinterpret Windows paths on Linux.
- This command accepts external sources outside the selected workspace edit/publish roots. Read title, numeric `MAJOR.MINOR` version and confidentiality ID from their flat frontmatter; resolve the ID to an enabled configured label. Missing/invalid values fail for that file. Leave absent document numbers empty. Use the existing optional-variable and reserved-token rules. Do not import owners, approvers, candidates or release history, infer approval from version history, or use external frontmatter to change registered-document authority.
- Require a new output root disjoint from the input tree, workspace edit tree and publish tree after canonical path/symlink checks. No overwrite or resume mode in this change. Mirror selected relative parents and write `<stem>.docx` and `<stem>.pdf`; preflight Windows case-insensitive/path-name collisions. Mark the manifest and command output explicitly as `conversion_only`, not released. A version/classification in document chrome is not evidence of approval.
- Preflight configuration, template SHA/contract, destinations and supported host before invoking Word. Keep deterministic sequential per-file execution; continue independent files after document-specific failure and return nonzero if any fail. Configuration/output failures stop before document work. No silent skipped files, worker queue, retries or parallel Word processes.
- Extend the existing OOXML renderer, preserving the template contract, styles, page setup, logo, headers/footers and numbering. Retain/update the template TOC by default; `--without-toc` removes only its TOC structured-document tag. Support two-, three-, four- and five-column tables without losing text, column association or inline formatting. Every generated table uses fixed layout and table/grid/cell widths that sum to the printable body width; do not add mandatory template markers.
- Resolve local image references relative to each source, constrain their canonical target to the explicit input root, embed bytes with collision-free package relationships and correct content types, preserve aspect ratio and constrain drawings to printable width. Shared assets across policies must work. Missing, corrupt, unsupported, remote or escaping assets fail clearly; never silently emit only alt text. PNG/JPEG are the required initial formats; other constructs must be explicitly supported or rejected rather than silently losing content. Inventory links, lists and inline HTML in the real corpus as part of fidelity testing; document the output link policy without claiming Markdown-relative links become working PDF links automatically.
- Keep Office automation outside `dms-core` and independent of Tauri. Extract the existing adapter into a shared, narrowly scoped export crate/module consumed by CLI and desktop; do not duplicate COM/AppleScript logic. Desktop release continues to use its approved snapshot and atomic PDF lifecycle contract.
- Retain the final, controlled-field-filled DOCX after Word updates fields, pagination and TOC and saves it. Export that same document to PDF without a later DOCX edit. Validate/read back both final files before recording success. Word failure, save failure, invalid PDF or incomplete output pair must never produce a success entry; clean owned incomplete files or explicitly record any retained diagnostic path. Do not close a user's unrelated Word documents or disable host security globally.
- Write a local manifest containing input relative paths and SHA-256, template identity/digest, referenced-asset digests, effective conversion metadata, final DOCX/PDF paths and SHA-256, renderer/Word version and per-file result/error. Hash the saved final DOCX, not the pre-Word intermediate. Do not copy the workspace's identity directory, source bodies or credentials into logs. Retained DOCX contains the same confidential content as PDF and needs equivalent access controls.
- Provide Windows and macOS adapters with explicit unsupported-host/missing-Word errors. Real Windows full-corpus evidence is required here; macOS installed-Word support must be honestly bounded unless separately exercised. No new Linux renderer or automatic WSL bridge is required.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Preserve multi-column policy tables through template assembly | done (G1: 11 tests passed on Rust 1.88.0; evidence below) | G1: `cargo test -p dms-core --test markdown_template` exits 0 with two/three/four/five-column fidelity coverage. |
| 2 | Embed local source images without silent content loss | done (third-correction G2: 20 markdown_template + 31 lifecycle tests passed on Rust 1.88.0; evidence below) | G2: `cargo test -p dms-core --test markdown_template --test lifecycle` exits 0 with image, containment, rejection and template-preservation coverage. |
| 3 | Deliver explicit non-release batch CLI and retained Word inputs | done (corrected G3: 250 workspace tests and CLI help passed; review findings corrected) | G3: `cargo test --workspace --locked` and `cargo run -p dms-cli -- convert markdown --help` exit 0 with schema-18/19/current read-only configuration, retained-DOCX, full-width tables, explicit TOC removal and unchanged-release coverage. |
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

Review correction scope: retain canonical in-root parent-relative assets, the escaping fixture, case-insensitive JPEG defaults, structural image validation and package collision coverage. The explicitly authorized third correction addresses only the final review's two remaining defects: scan all package XML parts for drawing IDs and isolate inline image-alt events from body output while preserving separators. Completion requires a fresh corrected G2; Phase 3–5 work remains unauthorized.

1. Add an explicit source/asset context to assembly and update all callers. Keep path validation in core and Office outside it. Reject asset-bearing input when no resolution context is provided instead of ignoring it.
2. Add PNG/JPEG insertion, content types, noncolliding media/relationship IDs, dimensions, aspect-preserving sizing and safe source-relative lookup. Support references to shared assets elsewhere within the input root.
3. Test repeated images, template media/relationship collisions, image-only paragraphs, image text alternatives, images in supported containers, Unicode/space-containing paths, corrupted files, missing files, traversal, absolute/remote references and symlink escapes. Unsupported placement/format fails explicitly rather than claiming success.
4. Verify image bytes and drawing references in DOCX; preserve the template logo and all source/template bytes. Inventory other corpus Markdown constructs and add focused no-content-loss coverage where needed. Synchronize CAP-0007 evidence and relevant core/desktop DOX without adding new UI.

**Verification gate:** G2: `cargo test -p dms-core --test markdown_template --test lifecycle` exits 0 with image, containment, rejection and template-preservation coverage.

**Corrected execution evidence:** Phase 2 review corrections in the operator-selected isolated worktree `/home/raphael/.hermes/profiles/hermes-vc/cache/scratch/dms-chg0053-phase2`, HEAD `27c61f179d7a762510d3208cc28b45ef43a272bf`, Rust 1.88.0 under WSL. Synthetic fixtures only; no external corpus, workspace configuration, credentials or installed Word accessed. No staging, commit, push or primary-checkout edits.

- G2: `cargo test -p dms-core --test markdown_template --test lifecycle` exited 0 after the authorized third correction: 20 passed (markdown_template) and 31 passed (lifecycle), 0 failed. The two focused regressions first failed against the pre-correction source: a preserved header ID `9000` produced body IDs `1` / `2`, and multiline alternative text became `line oneline two`.
- `cargo test -p dms-core --locked` exited 0: 108 tests passed, 0 failed; 0 doc-tests. `cargo fmt --all -- --check`, `cargo clippy -p dms-core --all-targets --locked -- -D warnings` and `git diff --check` exited 0. `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .` exited 0 with no issues. No desktop-suite, workspace-wide or installed-Word rerun is claimed for this third correction.
- API remains `assemble_markdown_docx(template, markdown, destination, assets: Option<&MarkdownAssetContext>)`. The context carries input root and source path; resolution canonicalizes the root and source directory. `ExportRequest.markdown_assets` supplies the edit-root/registered-source context for Markdown release. Missing context rejects asset-bearing input.
- Containment: both `input/policies/a.md` and `b.md` resolve `../assets/shared.png` and its percent-encoded equivalent to one shared in-root asset. The negative fixture now exists at the actual `../outside.png` target; raw/encoded escapes and Unix symlink escapes fail canonical containment. Remote/scheme, absolute, missing, non-file, fragment/query and raw HTML image rejection remains covered without destination creation.
- Content types: two distinct JPEG files produce two media parts and exactly one JPEG default. A mixed-case existing `JpEg` default is reused byte-for-byte. Lower/upper-case wrong MIME declarations and contradictory defaults in either order reject before output creation; source template bytes remain unchanged.
- Collision coverage: an occupied `word/media/dms-image-1.png`, existing `rId900` image relationship and `wp:docPr` ID `7000` survive unchanged. New media uses `dms-image-2.png` / `rId901`; repeated occurrences receive distinct drawing IDs `7001` and `7002` while sharing the media relationship.
- Final-review medium finding — corrected: drawing allocation scans every loaded `.xml` package part with namespace-aware matching for both `wp:docPr` and `pic:cNvPr`. New occurrence IDs exceed the package maximum; allocation rejects exhaustion instead of wrapping. Focused fixtures independently place the maximum in either property in headers, footers, footnotes and comments, including a preserved header ID `1` that formerly collided. Alternate namespace prefixes and single-quoted attributes are covered. Preserved parts, logo, template and asset bytes remain unchanged.
- Final-review low finding — corrected: a dedicated image-alt collector consumes the complete inline subtree without mutating body formatting or emitting runs. Soft breaks and both hard-break spellings retain a space separator; emphasis, code, link labels, literal inline HTML, math, escaped labels and nested-image alt text remain in drawing descriptions. Tests require exactly one drawing run and no text/break run in image-only paragraphs, unchanged surrounding italic text, and continued raw HTML image rejection. Parser options remain unchanged; defined footnote references inside image syntax are not recognized as images by the current parser, so that syntax is not claimed as image-alt coverage.
- Image checks: PNG validates chunk bounds/CRCs, IHDR encoding fields, nonempty IDAT and IEND; JPEG walks segment/frame/scan lengths, nonempty entropy-coded scan data and EOI rather than returning at SOF. Tests reject both former header-only stubs, bad PNG CRC/length, truncated PNG/JPEG, missing JPEG scan and empty JPEG scan. No new Cargo dependencies or manifest/lockfile edits.
- Validation is structural, not compressed-pixel decoding. A malformed payload with plausible structure can still pass; no general corrupt-image or Word-renderability guarantee is claimed. Positive fixtures are real decodable images, not stubs. Native 96-DPI sizing, downscaling to printable width, aspect ratio, alt text, image-only/list/table placement, Unicode/space paths and repeated references remain tested.
- Fixture generation: `ffmpeg -v error -f lavfi -i color=c=<color>:s=<size> -frames:v 1 -threads 1 crates/dms-core/tests/fixtures/<file>` generated `markdown-wide.png` (red, 4000x2000), `markdown-small.png` (blue, 40x20), `markdown-photo.jpeg` (green, 640x360) and `markdown-second.jpeg` (yellow, 32x16). Each passed `ffmpeg -v error -xerror -i <fixture> -frames:v 1 -f framemd5 -` with one decoded frame of the expected dimensions; the template's 1x1 logo also passed via ZIP extraction to stdin. FFmpeg is only a fixture-generation/independent-verification tool, not a build or runtime dependency.
- Preservation: source/template/asset files stay byte-identical and repeated assembly is deterministic. Body XML is replaced; document relationships and content types are extended for media; unrelated package parts and template logo bytes are preserved. Optional frontmatter-variable substitution remains an existing exception to package-part identity.
- Tests: the [core template suite](../../../crates/dms-core/tests/markdown_template.rs) retains parent-relative containment, structural corruption, JPEG defaults/conflicts, media/relationship/body-drawing collision, image-placement and construct coverage, augmented by `assembler_avoids_drawing_ids_in_preserved_package_parts` and `assembler_keeps_image_alt_inline_events_out_of_body_runs`. The lifecycle exporter double still checks the release-side asset context; lifecycle and desktop pass-through code are untouched by this correction.
- Review disposition: both final-review defects are addressed with failing-before/passing-after regressions and local source/test review. No new independent-review verdict is claimed by the correction worker; primary-checkout integration remains outside this authorization.
- Construct inventory remains synthetic parser-event coverage, not a real-corpus inventory. Word/PDF layout fidelity, standalone batch conversion, retained final DOCX and full-corpus verification remain Phases 3–5; their scope is unchanged.
- DOX/CAP: core guidance and CAP-0007 state package-wide drawing-ID allocation and image-alt event isolation alongside the existing containment, preservation and structural-validation limits. CAP-0007 remains `not implemented` overall. Parent/desktop DOX, ownership indexes, CAP-0022, architecture/privacy and canonical wireframes are intentionally unchanged: no package ownership, CLI/Office boundary, persistence or desktop interaction changed. The changes index follows this record's status.

## Phase 3 — Deliver explicit non-release batch CLI and retained Word inputs

**Goal:** One explicit CLI invocation produces independently verified output pairs without touching DMS workflow state.

1. Read CLI/desktop DOX, then implement the command contract above in `crates/dms-cli/src/main.rs` with integration tests in `crates/dms-cli/tests/cli.rs`. Add read-only export-configuration validation in core with schema-18/19/current fixtures; prove no migration, metadata/frontmatter write, lock, Entra or notification call occurs.
2. Extract/reuse placeholder fill and installed-Word automation outside core and Tauri. Add the new ownership boundary and Cargo membership only as needed; update parent/child DOX immediately. Preserve existing desktop release tests and Windows/macOS behavior.
3. Implement final DOCX save before PDF export, paired-artifact validation and sequential batch results. Make every generated table span the printable body width, including two-column tables, and retain template TOC by default while testing explicit `--without-toc` removal. Add structural regressions that require the default DOCX to retain the template TOC field and English rendered heading, and require `--without-toc` to remove only the TOC structured-document tag. Markdown must not be silently rewritten to remove a source-authored static TOC; callers provide the desired Markdown body. Cover metadata mapping, reserved tokens, absent number, config/template mismatch, no Word, host limitations, collisions, output containment, no overwrite, dry-run, all-success, mixed failures and empty input. Test doubles must inspect the exact DOCX supplied to Word and prove it is the retained artifact; real pagination is phase 4 evidence.
4. Add manifest/hash verification and failure-safe ownership of incomplete files. Never claim a successful pair from a PDF signature alone; verify both artifacts and their recorded digests. Cover sources/assets changing during conversion by using the bytes actually consumed or refusing inconsistent inputs.
5. Update CAP-0022's PDF-export non-goal narrowly for standalone conversion, not approval/release. Add `**Primary surface:** none` to CAP-0022. CAP-0007 remains `**Primary surface:** human` for its existing desktop release surface; document shared rendering improvements separately from external conversion authority. Amend ADR-0008, ADR-0023/architecture and privacy for retained content and the new CLI/adapter boundary in this implementation slice.

**Verification gate:** G3: `cargo test --workspace` and `cargo run -p dms-cli -- convert markdown --help` exit 0 with batch, read-only schema-18/19/current configuration, retained-DOCX, full-width-table and explicit-TOC coverage, and unchanged-release coverage.

**Execution evidence:** Phase 3 in the selected isolated worktree
`/home/raphael/.hermes/profiles/hermes-vc/cache/scratch/dms-chg0053-phase3`, branch
`agent/chg-0053-phase3`, Rust 1.88.0 under WSL. No primary-checkout edits,
staging, commit, push, deployment, live credentials, real corpus/configuration
or installed Word access. Implementation workers shared only non-overlapping
paths; the parent integrated their partial results and completed verification.

- Corrected G3: `cargo test --workspace` exited 0: 250 passed, 0 failed, 0 ignored. This
  includes 18 CLI integration, 20 conversion-domain, 23 template, 31 lifecycle,
  91 desktop and 10 shared-export tests, plus the unchanged domain suites.
  `cargo run -p dms-cli -- convert markdown --help` exited 0 and exposes all
  three required paths, `--recursive`, `--dry-run`, `--without-toc` and global
  `--json`.
- `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `git diff --check` exited 0. Focused development runs of
  `cargo test -p dms-core --test conversion`,
  `cargo test -p dms-core --test conversion --test markdown_template`,
  `cargo test -p dms-export`, and `cargo test -p dms-cli` passed. The initial
  integration attempt exposed a missing SHA-256 test import and unfinished
  ownership docs; both were corrected before G3. No dependency versions or
  MSRV changed.
- `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .`
  exited 0 with no issues. Read-only record checks passed: five phase rows,
  Phases 1–3 done and 4–5 pending, no active phase, one matching CHG index entry,
  both affected CAPs indexed with their correct primary surfaces, unchanged
  wireframes/UI paths, selected branch and empty Git index. There is no dedicated
  repository record validator; none is claimed.
- Delayed review disposition: M1's PDF reservation now rejects nonempty or
  symlink targets; private staging replaces the predictable filename. M2 clears
  Word-version receipts on failed pairs. M3 no longer forwards native stderr.
  M4 rejects a supplied config-file symlink before canonicalization. M5 treats
  whitespace-only numbers as absent while preserving nonblank values exactly.
  The two M4/M5 regressions failed before correction and passed afterward with
  `cargo test -p dms-core --test conversion review_`; the full G3 and strict
  Clippy gates were rerun successfully. M6's input/output-root replacement and
  cleanup redirects are covered; residual hostile check/use races remain outside
  the local operator-owned filesystem contract. M7's schema-20 rejection is
  intentional: the supported set remains exactly 18, 19 and current. This is
  parent disposition of the review, not a new independent sign-off.
- Fresh macOS review disposition (reviewer report `20261014_145058_9ee343`,
  final findings; source/tests re-read at unchanged HEAD
  `0c4251dc1b7b0798b03eda311219c5dd49c59fb6`):
  1. **High — corrected.** The script captures `open`'s result, rejects a
     missing result or a full-name alias different from the supplied source,
     and only then assigns the save/export/close target. No `active document`
     lookup remains. The error handler cannot close the unverified result and
     never saves a document. An unresolvable Word handle fails closed instead
     of falling back to a user's active document.
  2. **Medium — corrected.** Running state is captured before Word commands.
     Temporary release (`retain_docx=false`) hides Word before opening the copy;
     success and error paths quit only when Word was initially stopped and no
     documents remain. Existing user sessions are never quit. Retained
     conversion neither sets application visibility nor quits Word. The empty
     document guard also preserves documents opened while the operation runs.
  3. **Medium — not a bug; unchanged.** Preflight is exactly
     `osascript -e 'id of application "Microsoft Word"'`, not a tell/open/run
     command. The [AppleScript application-class reference](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_classes.html)
     identifies `id` as an AppleScript-handled application property;
     [Enhanced Application Object Model](https://macosxautomation.com/applescript/features/appobject.html)
     explicitly says retrieving it does not require the application to run.
     No current test or documentation demonstrates launch on this path.
     Replacing it with a launch-and-cleanup probe would add side effects rather
     than fix a confirmed defect. This is documented-semantics evidence, not a
     native macOS execution claim.
- Both new static regressions first failed with
  `cargo test -p dms-export macos_script_` (exit 101: 0 passed, 2 failed,
  8 filtered out): active-document binding and absent running-state/lifecycle
  handling. After correction, `cargo test -p dms-export` exited 0 (10 passed,
  0 failed, 0 ignored; 0 doc-tests) and
  `cargo test -p dms-desktop export::tests` exited 0 (5 passed, 0 failed,
  86 filtered out). The assertions check validation-before-binding,
  success/error cleanup targets, release-only hiding, guarded quitting on both
  outcomes, retained-only saving, and retained-mode absence of visibility/quit
  commands. Existing update/paginate/save/export ordering and no-resave
  assertions remain intact; the former whole-script visibility/quit bans now
  check retained mode alongside explicit release assertions.
- Review-correction reruns also passed with exit 0:
  `cargo fmt --all -- --check`, `cargo test --workspace`,
  `cargo run -p dms-cli -- convert markdown --help`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `git diff --check`. CLI help retains all three required paths, `--recursive`,
  `--dry-run`, `--without-toc` and global `--json`. Counts above were summed from
  the actual test summaries, not inferred from the added tests.
- `conversion.rs` reads only roots/template/catalogue from schema 18, 19 or
  current JSON. Synthetic configurations deliberately contain unusable identity
  and document fields; existing lock/manual instructions and full workspace
  bytes stay unchanged. There is no Workspace open/save/migration, identity,
  notification or release call in the conversion path. Dry-run assembles
  privately without Word preflight/calls or output creation.
- Configuration/template mismatch, unsupported schema, strict title/version/
  enabled-classification mapping, absent number and reserved-token precedence
  are covered. Root checks, symlink aliases/replacements, Windows device names,
  case-insensitive file/directory collisions, explicit recursion/exclusions,
  empty input and no-overwrite are tested. Independent document failures
  continue in deterministic order; existing files are never cleanup targets.
- `dms-export` is the minimal new shared ownership boundary. CLI and desktop
  reuse one placeholder implementation and one native script per OS, with an
  explicit retained-versus-release mode. Controlled fill uses private staging;
  token replacement is non-recursive and cannot cascade inserted field values.
  Word diagnostics are sanitized. Windows uses Win32-compatible paths. macOS
  validates its opened document before mutation/cleanup and leaves other
  documents alone. Retained conversion leaves application visibility alone;
  temporary release preserves hiding and guarded quit-if-started behavior.
- The shared Word double inspects the controlled-field-filled DOCX at the exact
  retained path, simulates a save, and proves the adapter leaves those saved
  bytes intact. Native-script tests require field/TOC/pagination updates and
  save before PDF export, with no later DOCX save. These are structural/port
  tests, not execution of COM/AppleScript or proof of real pagination.
- Final-pair validation reads ZIP entries/CRCs and XML, required OOXML roots/
  body, split controlled tokens, PDF EOF and parsed nonempty PDF text. Tests
  reject Word failure, incomplete/invalid PDF and invalid DOCX, remove owned
  partial outputs and avoid false success hashes/version receipts. Manifests
  are read back and contain conversion-only metadata and final artifact hashes.
  Captured Markdown/template and actually embedded asset bytes supply digests;
  mutation tests prove later original-file changes do not falsify those hashes.
- Every generated table, including two columns, uses printable-width fixed
  layout. Default TOC tests retain its field and English heading. Explicit TOC
  removal is namespace-aware (including alternate prefixes and foreign-attribute
  rejection), preserves unrelated SDTs/package parts and keeps source-authored
  static contents. Image and desktop release regressions remain green.
- CAP-0022 declares `Primary surface: none` and narrowly adds non-release
  conversion. CAP-0007 retains `Primary surface: human` and its bounded release
  status. ADR-0008, ADR-0023/architecture, privacy and root/crate DOX describe
  the actual boundary and retained confidential DOCX. Wireframes and UI assets
  are untouched. Documentation/product/change parent DOX and product index
  intentionally remain unchanged: no documentation ownership, CAP ID or visual
  surface was added. The Rust child index includes `dms-export`.
- Correction DOX pass: `crates/dms-export/AGENTS.md` records validated macOS
  document ownership and mode-specific lifecycle/preflight rules. Parent DOX,
  child indexes, CAP/ADR/privacy records and desktop source/tests are unchanged
  by these corrections: no new ownership boundary, API, capability, authority,
  data class or UI interaction was introduced. Findings 1–2 have local
  failing-before/passing-after evidence; Finding 3 is rejected on the cited
  semantics. No fresh independent-review sign-off is claimed.
- Remaining gates: native Windows full-corpus evidence and macOS installed-Word
  verification are not claimed. Image checks remain structural, not decoding;
  parsed output is not a visual-fidelity guarantee. Link labels are preserved,
  but Markdown-relative links are not rewritten into PDF navigation. Concurrent
  hostile filesystem replacement is not an OS sandbox; detected root changes
  fail closed and may leave owned artifacts for operator inspection. Phase 4
  and Phase 5 stay pending; this record is not archived.
- macOS-specific residual risk: WSL has no `osascript`; these are static script
  and Rust adapter tests, not AppleScript compilation or installed-Word runs.
  Word's returned-handle/full-name behavior and PDF-save document identity need
  native verification. If binding or cleanup fails, the owned Word document may
  remain open; the script deliberately does not guess another document or force
  quit with documents remaining. Concurrent user/process changes are not made
  atomic by the initial running-state and final document-count checks.

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

Phases 1–3 are done with their bounded evidence above. Phases 4–5 remain pending
and no phase is in progress, so the record returns to `planned`. Native Word/full-
corpus fidelity and final change integration remain unverified. Work remains
uncommitted in the isolated branch without primary-checkout or real-data access.
Start Phase 4 only on operator instruction.

## Related contracts

- [CAP-0007](../../product/capabilities/CAP-0007-draft-pdf-export.md)
- [CAP-0022](../../product/capabilities/CAP-0022-headless-core-cli.md)
- [Architecture](../../architecture.md), [decisions](../../design-decisions.md), [privacy](../../privacy.md)
- [Markdown metadata and template variables](../../markdown-frontmatter-and-template-variables.md)
