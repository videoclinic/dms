# CAP-0007 — Application-driven source draft → PDF export

| Field | Value |
| --- | --- |
| ID | CAP-0007 |
| Status | not implemented |
| Mechanism | Installed Microsoft Office for Office drafts; local CommonMark HTML print shell + native WebView PDF API for Markdown |
| Tests | Partial phase-5 evidence: [`dms-desktop` adapter tests](../../../crates/dms-desktop/src/export.rs), [Windows/macOS native WebView PDF smoke](https://github.com/videoclinic/dms/actions/runs/31367938246) |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Release invokes an **in-app export** for the library member’s source draft,
   not a manual “export yourself and pick a file” step as the primary path:
   - Office drafts use Microsoft Office desktop apps already installed on the
     machine (Windows and macOS).
   - Markdown (`.md`) drafts render locally as CommonMark HTML inside an app
     print shell and use the native WebView PDF API on the host OS.
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
   interface: Office may use Windows COM or macOS AppleScript/Office automation;
   Markdown uses the print shell plus native WebView PDF APIs. The operator
   outcome is the same on both supported OS.
8. Export first writes to a temporary file in the target publish directory.
   Before commit, the app verifies that the result is non-empty and has a valid
   PDF header, computes its SHA-256 digest, and atomically renames it to the
   final versioned path. Failure removes the temporary file when possible and
   does not commit a version or release record.
9. Both adapters receive one **export chrome** map built only from the release
   context in `.dms` (not from Office document properties or Markdown front
   matter — CAP-0015):
   - candidate version label without the filename `V` prefix (for example `2.0`)
   - effective confidentiality **display label** and stable type **ID**
     (CAP-0008 snapshot at release)
   - optional DMS title and document number when the print shell shows them
10. **Markdown print shell (Option A).** Markdown export does not route through
    Word or a `.docx` template at runtime. The app:
    - strips YAML front matter before CommonMark rendering (front matter is not
      control data and is not PDF body content)
    - wraps the body HTML in a shipped print shell (`shell.html` + `print.css` +
      logo asset) that mirrors the corporate Vorlage chrome: header logo, A4 page
      size and margins, and a footer with page indicator plus the canonical
      captions `Vertraulichkeitsstufe: <display label>` and
      `Version: <major>.<minor>`
    - substitutes footer/header values only from the export chrome map
    - loads the shell from an app-local asset base URL so relative logo/CSS paths
      resolve, then prints via the native WebView PDF API
    CAP-0002 source-draft marker checks remain on the Markdown body; the print
    shell may repeat the same captions on every PDF page and must not be the
    only place those values exist for the review/release gate.
11. **Office placeholder fill.** When an Office draft (or its section
    headers/footers) contains the literal tokens `{CONFIDENTIALITY}` or
    `{VERSION}`, export works on a temporary copy and replaces them with the
    export chrome display label and version label respectively, across body,
    tables, text boxes, and every header/footer part, before invoking Office
    PDF export. Tokens already replaced with concrete values are left unchanged.
    The original draft on disk is never modified by this step.
12. Shipped default print-shell assets are derived from the operator Vorlage
    layout (logo, margins, footer column structure). Pixel-perfect match to
    Word is not required; readable corporate chrome on every PDF page is.
    Workspace override of those assets is out of scope for v1 unless a later
    CAP adds it.

## Non-goals

- Shipping a full Office runtime inside the Tauri binary
- Server-side or cloud conversion services
- Pixel-perfect guarantee beyond what the selected local exporter produces
- Requiring identical Office build numbers across Windows and macOS
- Converting Markdown through Word solely to reuse a `.docx` template
- Treating Office properties or Markdown front matter as authority for chrome
  values

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0007-draft-pdf-export.html`](../wireframes/html/CAP-0007-draft-pdf-export.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0007-draft-pdf-export.png`](../wireframes/exports/CAP-0007-draft-pdf-export.png)

- ADR-0008: [`../../design-decisions.md`](../../design-decisions.md)
- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Integrity: [`CAP-0004-release-integrity.md`](CAP-0004-release-integrity.md)
- Classification: [`CAP-0008-confidentiality-classification.md`](CAP-0008-confidentiality-classification.md)
- Document control data: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
