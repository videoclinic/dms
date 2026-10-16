# CHG-0054 — Language-selected reusable DE/EN Word conversion templates

The sibling proposal repository `2026-HL` establishes reusable, language-specific
German and English Word templates for Markdown → Word/PDF conversion: an explicit
`Absatz` body/table paragraph style instead of the implicit default, native Word
headings and lists, a preserved dynamic table of contents updated by Word,
printable-width tables, embedded images, and a paired retained DOCX/PDF output.
DMS today supports exactly one workspace template with no language selection.
This change adopts the language-selected reusable-template behavior inside DMS.

**Plan ID:** CHG-0054-language-selected-de-en-word-templates
**Created:** 2026-10-14
**Depends on:** none for this record. The standalone conversion command and the shared `dms-export` Word adapter land under CHG-0053 (in progress) reusing the archived CHG-0004 renderer; this change reuses that boundary and does not reopen their scope.
**Entry checkpoint:** DMS `main` at `050315c84ae3d4506c62f5c0fdc8ef859179976c` (actual `git rev-parse HEAD`). The working tree is intentionally dirty with uncommitted CHG-0053 phase work — root and `crates/dms-core/AGENTS.md`, `docs/changes/README.md`, `docs/changes/active/CHG-0053-markdown-batch-conversion-retained-docx.md`, `docs/product/capabilities/CAP-0022-headless-core-cli.md`, `docs/markdown-frontmatter-and-template-variables.md`, `crates/dms-core/src/{conversion.rs,template.rs,frontmatter.rs,lib.rs}`, `crates/dms-core/tests/conversion.rs`, `crates/dms-cli/tests/cli.rs`, and new `scripts/{Test-MarkdownBatchExport.ps1,verify_markdown_batch.py,test_markdown_batch.py}` plus `crates/dms-core/tests/smoke_contract.rs`. Preserve every existing change; allocate CHG-0054 without touching them.
**Context sources:** Root `AGENTS.md` (Architectural decisions); `docs/AGENTS.md`; `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/capabilities/CAP-0007-draft-pdf-export.md`; `docs/product/capabilities/CAP-0022-headless-core-cli.md`; `docs/markdown-frontmatter-and-template-variables.md`; `docs/architecture.md` (runtime table, standalone-conversion boundary, ADR-0023); `docs/design-decisions.md` (ADR-0008); active CHG-0053 and active CHG-0042/CHG-0045/CHG-0050; `crates/dms-core/src/lib.rs` (`markdown_template: Option<MarkdownTemplateAsset>`); `crates/dms-core/src/template.rs`; sibling `2026-HL` `docs/rendering.md`, `docs/changes/active/CHG-001-english-evidence-package.md`, and the canonical templates `assets/templates/Vorlage.de.docx` / `Vorlage.en.docx` with `assets/templates/AGENTS.md`.
**Produces:** A `.dms` template catalogue that names every reusable Word template with an explicit language, resolves the requested language at assembly, and fails closed on template hash, style or placeholder mismatch — with `Absatz` body/table styling, native headings/lists, preserved dynamic TOC, printable-width tables, embedded images and a DOCX/PDF pair, without rewriting static template language text or inventing dates.
**Status:** planned

| Field | Value |
| --- | --- |
| ID | CHG-0054 |
| Status | planned |
| External request | Direct operator request: I would like to apply these template to all conversion within this project; Adapt this also to the ../dms project as new application-record; Save the teampltes als in /mnt/c/Users/RaphaelBossek/dms-demo2/edit/ environment |
| Affected CAPs | CAP-0007, CAP-0022 |
| Decision records | ADR-0008 (format-specific local PDF export) and ADR-0023 (shared Rust core with a first-class headless CLI) describe the current single-template export/conversion boundary. A language-selected template catalogue is a cross-cutting runtime choice; implementation amends the affected decision surface in the same slice. No new ADR is allocated at planning time. |

## Source request

Direct operator request: I would like to apply these template to all conversion
within this project; Adapt this also to the ../dms project as new application-record;
Save the teampltes als in /mnt/c/Users/RaphaelBossek/dms-demo2/edit/ environment

The operator-focused adoption intent, as established with the sibling `2026-HL`
conversion work, is:

- explicit language-selected reusable German and English Word templates instead
  of one implicit workspace template;
- an explicit `Absatz` body/table paragraph style instead of the implicit default;
- native Word headings and lists;
- a preserved dynamic table of contents that Word updates, with the update
  verified against actual bookmarks and PDF page numbers;
- fixed printable-width tables and preserved images;
- a DOCX/PDF output pair;
- static template language text honored as-is (no hidden English-label rewrite);
- fail-closed on template hash/style/placeholder mismatch, with external
  templates and read-only configuration unchanged by conversions.

## Existing change ownership

The other active records do not own language-selected reusable Word templates.

- [CHG-0053 — Markdown batch conversion with retained DOCX](CHG-0053-markdown-batch-conversion-retained-docx.md) is the standing standalone-conversion progress authority. It owns the `dms convert markdown` command, table/images assembly fidelity, retained final DOCX and the shared `dms-export` adapter, all against the single configured workspace template. This change consumes that boundary and the shared adapter; it does not duplicate the command, the CLI flags, the manifest or the retained-DOCX lifecycle, and does not claim any CHG-0053 phase done.
- [CHG-0042 — Publish-root released-document index](CHG-0042-publish-root-released-document-index.md) lists already released PDFs and is unaffected.
- [CHG-0045 — Operator-defined library name](CHG-0045-operator-library-name.md) and [CHG-0050 — UUID MSRV ceiling](CHG-0050-uuid-msrv-ceiling.md) are independent and must not be duplicated.
- [CHG-0004 — Markdown Word-template release](../../changes/archive/CHG-0004-markdown-word-template-release.md) is the archived single-template release receipt. Reuse its renderer via CHG-0053; do not reopen its release transaction.

CHG-0054 is the sole progress authority for language-selected reusable DE/EN
templates. It stays `planned` with every phase `pending`; nothing here is claimed
implemented.

## Source-backed current state

Baseline at `050315c84ae3d4506c62f5c0fdc8ef859179976c`.

- **One workspace template.** `.dms` stores a single optional template,
  `markdown_template: Option<MarkdownTemplateAsset>` (`crates/dms-core/src/lib.rs`,
  the field used by the conversion tests as `{id, relative_path, sha256,
  contract_version}`). CAP-0007 outcome 10 states "One optional `.docx` below the
  edit root is registered as the active workspace Markdown export-template asset,"
  and the architecture runtime table describes "One reusable `.docx` asset under
  the edit root" for every Markdown release. There is no language field and no
  DE/EN pair.
- **Template drives styles, not an explicit body style contract.** Body assembly
  replaces CommonMark content using "heading/paragraph/list/table prototypes in
  the template" ([`markdown-frontmatter-and-template-variables.md`](../../markdown-frontmatter-and-template-variables.md)).
  No CAP or operator reference names an `Absatz` body/table paragraph style; the
  used style depends on whatever prototype the single template carries.
- **Two prototype cells.** CAP-0007's bounded implemented behavior requires
  exactly two prototype table cells; columns after the first clone the second
  cell's styling. Wider counts are already supported, but the printable-width and
  style contract is tied to this single template.
- **Standalone conversion exists but is single-template.** CAP-0022 outcome 8
  defines `dms convert markdown --workspace-config ... --input-root ... --output-root ...`
  reading schema-18/19/current configuration; it validates the configured
  template contract/digest but has no template-selection flag.
- **No dedicated record validator.** `docs/AGENTS.md` states no `records:check`
  script exists yet; `scripts/` holds only ADMX/winget and the Markdown-batch
  helpers plus `Get-OneDriveM365Group.ps1`. This CHG relies on the existing
  index/link checks, not a validator.

## Sibling template source (read-only reference)

The adoption target is the behavior of the sibling `2026-HL` canonical templates,
inspected read-only at planning time (absolute paths, sibling repo outside DMS):

| Asset | Absolute path | SHA-256 |
| --- | --- | --- |
| German template | `/home/raphael/src/vc/2026-HL/assets/templates/Vorlage.de.docx` | `ce5be5aa46956b4996ecbb5b98e3b7d3f94feaf598efc722fad730b9c137df11` |
| English template | `/home/raphael/src/vc/2026-HL/assets/templates/Vorlage.en.docx` | `edf166374eaf3576c342fd20d6f397b679bdda883c8bd2e283bd0d9ed6d411fc` |

Both packages carry an explicit `Absatz` style used for body paragraphs, native
Überschrift/heading styles, a `Listenabsatz` list style, an
`Inhaltsverzeichnisberschrift` TOC heading with a dynamic TOC, and their own
language-specific static labels. The sibling `assets/templates/AGENTS.md`
contract states the same rules this change adopts: require explicit `de`/`en`
selection, never infer language from prose or silently translate static template
labels, use `Absatz` for body/table paragraphs, preserve heading/list styles and
the dynamic TOC, keep document metadata as controlled placeholders, and do not
embed a document-specific date in a shared template.

TOC styles use hanging indents after their number-tab stops so wrapped titles
retain right-aligned page-number leaders. Byte-identical copies are installed
as `Vorlage.de.docx` and `Vorlage.en.docx` under the demo2 edit root; this does not
register either file or change the current DMS workspace configuration.

This record references that behavior only. DMS does not copy the sibling repo or
its tree; the two `.docx` files and the sibling `AGENTS.md` remain read-only
inputs.

## Target contract (planned behavior — not yet true in runtime)

These are implementation targets for CHG-0054, not current-runtime claims.

- `.dms` stores a reusable **template catalogue**: each entry carries a stable
  template ID, edit-root-relative locator, validation digest, contract version
  and an explicit language tag (`de`, `en`). Do not infer a language or silently
  fall back to the current single `markdown_template`; any required catalogue
  migration must be explicit and validated. Selection is deterministic; an unknown, missing or ambiguous
  language fails closed before any assembly.
- Language selection never rewrites static template text. The template's own
  static language labels (headers, footers, boilerplate) are preserved verbatim;
  there is **no hidden English-label rewrite** and no automatic localisation of
  German footer/field labels. Only the configured controlled fields/tokens are
  filled.
- Generated body and table paragraphs use the template's explicit `Absatz`
  body/table paragraph style rather than an implicit default, and Markdown
  headings/lists map to native Word heading/numbering styles so the dynamic TOC
  can enumerate them.
- The template's dynamic table of contents is retained and updated by Word; the
  saved DOCX and exported PDF must have TOC entries whose page numbers match the
  actual rendered pages and whose bookmarks resolve. Generated tables use fixed
  layout spanning the printable body width; local images are embedded as real
  drawings with preserved aspect ratio.
- Standalone conversion (`dms convert markdown`, from CHG-0053) and the desktop
  release path (CAP-0007) share one template-resolution rule. Conversion writes a
  DOCX/PDF pair and never mutates external templates or the read-only
  configuration.
- Fail-closed on mismatch: a template whose SHA-256, contract version, required
  style, or required placeholder set does not match its catalogue entry stops the
  affected conversion with a clear error and no successful output entry.
- **Generic templates carry no document-specific date.** A reusable DE/EN
  template must not freeze a per-document date such as a DOC-009-style line.
  The operator's decision removes that optional document-specific date line; the
  converter must not invent a release/approval date when the template does not
  supply an explicit source date.

## Constraints and boundaries

- DMS remains the authority for document-control data; Markdown frontmatter and
  template static text are never document-control authority (CAP-0015, CAP-0007).
- The `2026-HL` proposal evidence is confidential. This record references the
  sibling template behavior only; it does not copy proposal documents, images or
  evidence into DMS.
- The sibling template installation filenames (`Vorlage.de.docx`,
  `Vorlage.en.docx` under `/mnt/c/Users/RaphaelBossek/dms-demo2/edit/`) are the
  operator's sibling-side concern. Existing `Vorlage.docx` and
  `.dms/workspace.json` are read-only inputs here and are not modified by this
  change unless separately authorized.
- No CAP status is promoted at planning time. CAP-0007 and CAP-0022 keep their
  current records until implementation and linked tests exist.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Specify the language-selected template catalogue and selection contract | pending | `cargo test -p dms-core --test conversion` exits 0 with explicit-language selection and unknown/ambiguous-language rejection; CAP-0007/CAP-0022 target sections reviewed. |
| 2 | Resolve the selected reusable template for assembly, fail closed on mismatch | pending | `cargo test -p dms-core --locked` exits 0 with DE and EN template assembly, mismatch rejection and byte-identical source/template/configuration. |
| 3 | Structural style fidelity across both templates | pending | `cargo test -p dms-core --test markdown_template` exits 0 with `Absatz` body/table style, native heading/list, printable-width-table, image and retained-dynamic-TOC coverage. |
| 4 | Prove the DOCX/PDF pair and rendered TOC/bookmark page verification | pending | `cargo test --workspace --locked` and `cargo run -p dms-cli -- convert markdown --help` exit 0; a native Windows Word smoke yields a pair whose updated TOC page numbers match the extracted PDF pages. |
| 5 | Integrate contracts, records and project gates | pending | Full integration commands and record checks below pass; CAP/DOX classifications are current; only then archive CHG-0054. |

Keep at most one phase `in-progress`. Phase context work is authorized only when
CHG-0053's single-template command and shared adapter exist to extend.

## Phase 1 — Specify the language-selected template catalogue and selection contract

**Goal:** A source-backed contract for a reusable DE/EN template catalogue and
explicit language selection that matches how CAP-0007 and CAP-0022 describe the
template boundary.

Steps:

1. Re-read the applicable DOX chain, CAP-0007, CAP-0022 and
   `docs/markdown-frontmatter-and-template-variables.md`.
2. Define the catalogue shape (ID, relative locator, digest, contract version,
   language tag), any explicit migration of the current single
   `markdown_template`, and the missing/ambiguous-language failure rules without
   implicit fallback.
3. State the mismatch contract (hash, contract version, required style, required
   placeholder set) and the static-text-preservation rule that forbids hidden
   language rewrites.

**Verification gate:** `cargo test -p dms-core --test conversion` exits 0 with
explicit-language selection and unknown/ambiguous-language rejection coverage;
the CAP-0007/CAP-0022 target sections describe the same contract.

## Phase 2 — Resolve the selected reusable template for assembly, fail closed on mismatch

**Goal:** Assembly resolves the catalogue entry for the requested language and
validates it before any output, without mutating external templates or the
read-only configuration.

Steps:

1. Add catalogue resolution to the existing read-only configuration projection;
   keep selection deterministic and reject unsupported/ambiguous selections.
2. Validate digest, contract version and required style/placeholder presence; on
   mismatch stop the affected conversion with no successful output entry.
3. Prove the source Markdown, both templates and the configuration file stay
   byte-identical across a conversion.

**Verification gate:** `cargo test -p dms-core --locked` exits 0 with DE and EN
template assembly, mismatch rejection and byte-identical
source/template/configuration coverage.

## Phase 3 — Structural style fidelity across both templates

**Goal:** Both templates render body and table paragraphs with the explicit
`Absatz` style, native headings/lists and printable-width tables, and keep the
template's dynamic TOC and images.

Steps:

1. Map generated body/table paragraphs to the configured `Absatz` style and
   Markdown headings/lists to the template's native heading/numbering styles.
2. Keep the dynamic TOC and image embedding from CHG-0053; assert the TOC field
   and heading styles survive assembly.
3. Assert printable-width fixed-layout tables, preserved aspect-ratio images and
   byte-identical source/template bytes for both the DE and EN templates.

**Verification gate:** `cargo test -p dms-core --test markdown_template` exits 0
with `Absatz` body/table style, native heading/list, printable-width-table, image
and retained-dynamic-TOC coverage.

## Phase 4 — Prove the DOCX/PDF pair and rendered TOC/bookmark page verification

**Goal:** One selected-language conversion produces a retained DOCX and an
exported PDF whose updated TOC matches the rendered pages.

Steps:

1. Extend the conversion/release invocation to carry the language selection while
   keeping schema-18/19/current read-only configuration and the existing
   DOCX/PDF pair contract.
2. From native Windows Word, update fields/TOC, save the DOCX, export the PDF,
   then verify the TOC entry page numbers against the extracted PDF pages and
   that TOC bookmarks resolve — not merely that a TOC field exists.
3. Exercise one hash/style/placeholder mismatch fixture and one language with no
   catalogue entry to prove nonzero exit and honest failure entries with no
   false success, and confirm no external template or read-only configuration
   mutation.

**Verification gate:** `cargo test --workspace --locked` and
`cargo run -p dms-cli -- convert markdown --help` exit 0; a native Windows Word
smoke yields a DOCX/PDF pair whose updated TOC page numbers match the extracted
PDF pages.

## Phase 5 — Integrate contracts, records and project gates

**Goal:** CAPs and runtime contracts describe only proven behavior and the change
is a single completed receipt.

Steps:

1. Amend CAP-0007's template outcomes for a reusable catalogue and language
   selection; keep CAP-0022's standalone-conversion contract aligned. Record the
   cross-cutting template-catalogue decision in `docs/design-decisions.md` and
   adjust `docs/architecture.md` and privacy if those surfaces change.
2. Complete the DOX pass for any changed ownership/contract; update the changes
   index. No visual desktop surface is added, so do not invent a wireframe; verify
   CAP-0007's existing human HTML/PNG links remain valid and CAP-0022 stays
   `Primary surface: none`.
3. Run from the repository root:

   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --locked
   node --test crates/dms-desktop/ui/*.test.mjs
   python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary .
   git diff --check
   ```

4. Perform record checks: affected CAP IDs exist and appear in the product index;
   this CHG has one matching active index entry; phase rows and numbered sections
   match; every completed phase cites its passing gate; all local links resolve.
   There is no dedicated record validator (`docs/AGENTS.md`); use the existing
   index/link/diff gates and run a validator if one is introduced by execution
   time.
5. Only after Phases 1–4 and the full integration gate pass, set CHG-0054 to
   done, move it to `docs/changes/archive/`, and refresh `docs/changes/README.md`.
   No other phase may archive this record. Do not commit or push without operator
   instruction.

**Verification gate:** Full integration commands and record checks above pass;
CAP/DOX classifications are current; only then archive CHG-0054.

## Out of scope

- Re-implementing the standalone conversion command, CLI flags, manifest or
  retained-DOCX lifecycle owned by CHG-0053.
- Reopening CHG-0004's approval/release transaction or CHG-0042's released-PDF index.
- Authoring the sibling `2026-HL` templates, installing
  `Vorlage.de.docx`/`Vorlage.en.docx`, or modifying `Vorlage.docx` and
  `.dms/workspace.json`.
- Automatic template translation, hidden English-label rewriting, or treating
  template static text as document-control authority.
- Inventing release/approval dates or document-specific date lines in generic
  templates.
- Server/cloud conversion, Pandoc, LibreOffice or a bundled Office runtime.

## Handoff

CHG-0054 is `planned` with Phases 1–5 `pending`. Execute it only on top of
CHG-0053's delivered single-template command and shared `dms-export` adapter, via
`phased-plan-execution`. Do not create a parallel plan, and do not promote
CAP-0007/CAP-0022 or move this record to `archive/` outside Phase 5.

## Related contracts

- [CAP-0007 — Application-driven source draft → PDF export](../../product/capabilities/CAP-0007-draft-pdf-export.md)
- [CAP-0022 — Headless DMS core CLI](../../product/capabilities/CAP-0022-headless-core-cli.md)
- [Markdown frontmatter and Word template variables](../../markdown-frontmatter-and-template-variables.md)
- [Architecture](../../architecture.md) · [decisions](../../design-decisions.md) · [privacy](../../privacy.md)
- [CHG-0053 — Markdown batch conversion with retained DOCX](CHG-0053-markdown-batch-conversion-retained-docx.md)
