# CAP-0007 — Application-driven source draft → PDF export

**Primary surface:** human

| Field | Value |
| --- | --- |
| ID | CAP-0007 |
| Status | not implemented |
| Mechanism | Installed Microsoft Word for Office drafts and Markdown converted into a temporary DOCX from the workspace Word-template asset |
| Tests | Partial phase-5 and CHG-0004 evidence: [`dms-desktop` adapter/unit tests](../../../crates/dms-desktop/src/export.rs) cover template-backed temporary DOCX assembly, controlled-field fill, installed-Office dispatch through a test double, valid-PDF enforcement, Win32-safe Word COM paths, and unchanged source/template bytes; [core lifecycle tests](../../../crates/dms-core/tests/lifecycle.rs) cover fail-closed missing, changed, invalid, and unconfigured template states without export or release evidence, plus the asset context a Markdown release passes to the assembler; [desktop Configuration tests](../../../crates/dms-desktop/src/lib.rs) and [frontend tests](../../../crates/dms-desktop/ui/configuration.test.mjs) cover template selection, validation display, removal confirmation, and Library exclusion; [core template tests](../../../crates/dms-core/tests/markdown_template.rs) cover strict custom properties, temporary DOCX assembly, template-package preservation, embedded source images, and explicit image rejection. A retained Windows installed-Word smoke released the controlled A.8.29 Markdown as a four-page A4 PDF with refreshed title, table of contents, version, and confidentiality, no unresolved template markers, matching SHA-256 release evidence, and unchanged source/template bytes. Windows/macOS CI runs the fake-backed pipeline; installed-Word evidence on macOS remains pending, so this CAP is not promoted. |

## Bounded implemented behavior

The shared Markdown assembler supports two-, three-, four- and five-column
policy tables, including German version histories, long text, empty cells,
escaped punctuation and Unicode. It retains cell order, paragraph/cell styling
and template run properties while rendering inline emphasis and code. Multiple
tables remain separate and preserve their cell text. Ordered Markdown lists use
native decimal Word numbering, keep source start values and restart independently;
unordered lists retain the template's bullet prototype.

The template still requires exactly two prototype cells and no new markers.
All column counts clone the
second-cell styling for columns after the first and receive equal fixed widths
from the body section's page width minus left/right margins and gutter. Missing
or invalid dimensions fail rather than guessing a page size. Assembly preserves
source/template bytes and section properties. Non-body package parts are
preserved except for optional-variable substitution and image relationship /
content-type additions when images are embedded.

Markdown image references become real OOXML drawings over embedded package
media. Assembly takes an explicit source/asset context (input root plus source
path); without it an asset-bearing document is rejected instead of emitting
alternative text alone. Parent-relative references such as `../assets/shared.png`
are allowed when their canonical target stays inside the input root. Remote,
absolute, fragment-bearing and escaping references, missing, structurally
corrupt or unsupported assets, symlink escapes and raw HTML `<img>` markup fail
explicitly. PNG and JPEG bytes are embedded verbatim with non-colliding media
part and relationship IDs shared by repeated references; each drawing occurrence
has its own ID above the maximum `wp:docPr` / `pic:cNvPr` ID across all template
XML parts, including preserved headers, footers, footnotes and comments.
Drawings preserve aspect ratio within the body printable width and carry the
Markdown alternative text. Alt-text line breaks become spaces; inline labels,
code, math and literal HTML stay in the description rather than leaking body
runs or changing surrounding formatting. An image-only paragraph contains only
a drawing run. Content-type defaults are added once
per case-insensitive extension; conflicting template declarations fail before
output creation. The template logo and unrelated package parts remain unchanged,
as do the source, template and asset files.

PNG validation checks chunk lengths, CRCs, header encoding fields, nonempty
image data and termination. JPEG validation checks segment/frame/scan lengths,
nonempty scan data and termination. These are structural checks, not pixel
decoding; malformed compressed payloads may still require a decoder to detect.
Successful tests use independently decoded synthetic images, while header-only
stubs and selected structural corruptions are explicit rejection cases.

Evidence: `assembler_preserves_two_three_four_and_five_column_table_fidelity`,
`assembler_sizes_wide_tables_without_grid_or_cell_widths`,
`assembler_embeds_local_source_images_as_drawings_and_package_media`,
`assembler_rejects_unusable_or_uncontained_source_images`,
`assembler_resolves_parent_relative_shared_images_inside_input_root`,
`assembler_rejects_header_only_and_structurally_corrupt_images`,
`assembler_deduplicates_jpeg_content_types_and_rejects_template_conflicts`,
`assembler_avoids_template_media_relationship_and_drawing_id_collisions`,
`assembler_avoids_drawing_ids_in_preserved_package_parts`,
`assembler_keeps_image_alt_inline_events_out_of_body_runs`,
`assembler_renders_supported_markdown_and_preserves_non_body_package_parts`,
`assembler_renders_authentication_delivery_steps_as_one_native_decimal_list` and
`assembler_preserves_other_markdown_constructs_without_losing_content` in the
[core template tests](../../../crates/dms-core/tests/markdown_template.rs).
This is OOXML assembly evidence, not full-corpus Word/PDF layout verification.
Standalone batch conversion and retained final DOCX are a separate headless
authority under [CAP-0022](CAP-0022-headless-core-cli.md), not release evidence.
Shared full-width layout and optional template-TOC removal are covered by the
core template tests; desktop release retains the default TOC. Full-corpus Word
rendering remains pending under
[CHG-0053](../../changes/active/CHG-0053-markdown-batch-conversion-retained-docx.md).
The overall CAP remains `not implemented` while the platform gaps above remain.

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Release invokes an **in-app export** for the library member’s source draft,
   not a manual “export yourself and pick a file” step as the primary path:
   - Office drafts use Microsoft Office desktop apps already installed on the
     machine (Windows and macOS).
   - Markdown (`.md`) drafts are assembled into a temporary DOCX from the active
     workspace Word-template asset and exported by installed Microsoft Word.
2. Export writes only to the computed publish path for the new version
   (`<publish-root>/<relative-parent>/<stem>_VMAJOR.MINOR_<confidentiality-type-id>.pdf`).
   The confidentiality type ID is the effective type at release, snapshotted in
   the release record; later policy or display-label changes never rename the
   PDF. Example: `Policy_V1.0_restricted.pdf`.
3. If the selected exporter is unavailable (including missing or unlicensed
   Office for an Office draft), cannot render the draft, or otherwise fails,
   release aborts with a clear error; no partial version record is committed as
   successful. The release transaction is atomic: a successful record only
   exists when the export produced a valid, non-empty PDF, its SHA-256 was
   computed, and the atomic rename to the versioned path succeeded.
4. Supported draft types for v1 are `.md` and at least `.docx`; `.xlsx` /
   `.pptx` are supported as implemented. Unsupported types fail closed with a
   clear message.
5. The source draft file is not deleted or replaced by the export.
6. After a successful export, CAP-0004 checksum runs on the produced PDF bytes.
7. Format-specific adapters are an implementation detail behind one export
   interface. Office drafts use Windows COM or macOS AppleScript/Office
   automation directly. Markdown first uses the existing Rust CommonMark and
   OOXML stack to create a temporary DOCX from the active workspace template,
   then uses the same installed-Word PDF adapter. The operator outcome is the
   same on both supported OS.
8. Export first writes to a temporary file in the target publish directory.
   Before commit, the app verifies that the result is non-empty and has a valid
   PDF header, computes its SHA-256 digest, and atomically renames it to the
   final versioned path. Failure removes the temporary file when possible and
   does not commit a version or release record.
9. Both adapters receive one **export chrome** map built only from the release
   context in `.dms` (not from source Office properties or Markdown frontmatter
   — CAP-0015):
   - candidate version label without the filename `V` prefix (for example `2.0`)
   - effective confidentiality **display label** and stable type **ID**
     (CAP-0008 snapshot at release)
   - DMS title and optional document number when the template exposes them
10. **Workspace Markdown template.** One optional `.docx` below the edit root is
    registered as the active workspace Markdown export-template asset. `.dms`
    stores its stable ID, relative path, validation digest, and contract version.
    The asset is reusable across Markdown documents and excluded from controlled
    document lifecycle, notes, release history, and permalinks. Markdown review
    and release fail closed when no valid template is configured.
11. **Template and field fill.** The template preserves its styles, page setup,
    headers, footers, relationships, and media while providing unambiguous body
    insertion/style prototypes for headings, paragraphs, lists, and tables.
    Generated temporary DOCX custom properties `DMS_TITLE`,
    `DMS_DOCUMENT_NUMBER`, `DMS_VERSION`, and `DMS_CONFIDENTIALITY` are filled
    from export chrome. Word `DOCPROPERTY` fields expose those values in the
    template and are refreshed before PDF export together with any template table
    of contents. The literal tokens `{TITLE}`,
    `{DOCUMENT_NUMBER}`, `{VERSION}`, and `{CONFIDENTIALITY}` remain supported in
    temporary Office copies and custom-property values. The source Office draft,
    Markdown source, and imported template are never modified during release.
12. Markdown frontmatter is managed for registered library members before body
    conversion. DMS prefills and overwrites controlled keys `title`,
    `document_number` (when set), `version`, and `confidentiality` from
    document control, effective library confidentiality **type ID**, and the
    candidate target version. Non-controlled keys remain operator template
    variables. Frontmatter never overwrites `.dms` control data. The generated
    DOCX and PDF always receive controlled chrome values from the release
    snapshot for `TITLE`, `DOCUMENT_NUMBER`, `VERSION`, and `CONFIDENTIALITY`
    (chrome confidentiality uses the display label). Additional flat
    ASCII-identifier frontmatter keys act as optional Word-template **variable
    definitions**: each key `name` fills matching `{NAME}` placeholders
    (uppercased) in temporary package XML during Markdown→DOCX assembly. Those
    variables never become document-control data and never override the four
    reserved controlled tokens. Operator reference:
    [`../../markdown-frontmatter-and-template-variables.md`](../../markdown-frontmatter-and-template-variables.md).

## Non-goals

- Shipping a full Office runtime inside the Tauri binary
- Server-side or cloud conversion services
- Pixel-perfect guarantee beyond what the selected local exporter produces
- Requiring identical Office build numbers across Windows and macOS
- A second WebView, HTML, altChunk, Pandoc, LibreOffice, or cloud fallback for
  Markdown conversion
- Treating source Office properties or Markdown frontmatter as authority for
  controlled output field values

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0007-draft-pdf-export.html`](../wireframes/html/CAP-0007-draft-pdf-export.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0007-draft-pdf-export.png`](../wireframes/exports/CAP-0007-draft-pdf-export.png)

- ADR-0008: [`../../design-decisions.md`](../../design-decisions.md)
- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Integrity: [`CAP-0004-release-integrity.md`](CAP-0004-release-integrity.md)
- Classification: [`CAP-0008-confidentiality-classification.md`](CAP-0008-confidentiality-classification.md)
- Document control data: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- Markdown Word-template implementation receipt: [`../../changes/archive/CHG-0004-markdown-word-template-release.md`](../../changes/archive/CHG-0004-markdown-word-template-release.md)
- Implementation receipt: [`../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md)
