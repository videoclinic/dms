# CAP-0007 — Application-driven source draft → PDF export

| Field | Value |
| --- | --- |
| ID | CAP-0007 |
| Status | not implemented |
| Mechanism | Installed Microsoft Office for Office drafts; local CommonMark rendering + native WebView PDF API for Markdown |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Release invokes an **in-app export** for the library member’s source draft,
   not a manual “export yourself and pick a file” step as the primary path:
   - Office drafts use Microsoft Office desktop apps already installed on the
     machine (Windows and macOS).
   - Markdown (`.md`) drafts render locally as CommonMark HTML and use the
     native WebView PDF API on the host OS.
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
   Markdown uses native WebView PDF APIs. The operator outcome is the same on
   both supported OS.
8. Export first writes to a temporary file in the target publish directory.
   Before commit, the app verifies that the result is non-empty and has a valid
   PDF header, computes its SHA-256 digest, and atomically renames it to the
   final versioned path. Failure removes the temporary file when possible and
   does not commit a version or release record.

## Non-goals

- Shipping a full Office runtime inside the Tauri binary
- Server-side or cloud conversion services
- Pixel-perfect guarantee beyond what the selected local exporter produces
- Requiring identical Office build numbers across Windows and macOS

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0007-draft-pdf-export.html`](../wireframes/html/CAP-0007-draft-pdf-export.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0007-draft-pdf-export.png`](../wireframes/exports/CAP-0007-draft-pdf-export.png)

- ADR-0008: [`../../design-decisions.md`](../../design-decisions.md)
- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Integrity: [`CAP-0004-release-integrity.md`](CAP-0004-release-integrity.md)
- Classification: [`CAP-0008-confidentiality-classification.md`](CAP-0008-confidentiality-classification.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
