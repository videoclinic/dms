# CAP-0007 — Application-driven Office → PDF export

| Field | Value |
| --- | --- |
| ID | CAP-0007 |
| Status | not implemented |
| Mechanism | Preinstalled Microsoft Office on the host (Windows and macOS desktop apps) |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Release invokes an **in-app export** that converts the library member’s
   Office draft to PDF using Microsoft Office desktop apps already installed on
   the machine (Windows and macOS), not a manual “export yourself and pick a
   file” step as the primary path.
2. Export writes only to the computed publish path for the new version
   (`<publish-root>/<relative-parent>/<stem>_VMAJOR.MINOR.pdf`).
3. If required Office application is missing, not licensed, or export fails,
   release aborts with a clear error; no partial version record is committed as
   successful. The release transaction is atomic: a successful record only
   exists when the export produced a valid, non-empty PDF, its SHA-256 was
   computed, and the atomic rename to the versioned path succeeded.
4. Supported draft types for v1 are declared (at least `.docx`; `.xlsx` /
   `.pptx` as implemented). Unsupported types fail closed with a clear message.
5. The Office draft file is not deleted or replaced by the export.
6. After a successful export, CAP-0004 checksum runs on the produced PDF bytes.
7. Platform-specific automation (e.g. Windows COM vs macOS AppleScript/Office
   automation) is an implementation detail behind one export interface; operator
   outcome is the same on both supported OS.
8. Export first writes to a temporary file in the target publish directory.
   Before commit, the app verifies that the result is non-empty and has a valid
   PDF header, computes its SHA-256 digest, and atomically renames it to the
   final versioned path. Failure removes the temporary file when possible and
   does not commit a version or release record.

## Non-goals

- Shipping a full Office runtime inside the Tauri binary
- Server-side or cloud conversion services
- Pixel-perfect guarantee beyond what the installed Office export produces
- Requiring identical Office build numbers across Windows and macOS

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0007-office-pdf-export.html`](../wireframes/html/CAP-0007-office-pdf-export.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0007-office-pdf-export.png`](../wireframes/exports/CAP-0007-office-pdf-export.png)

- ADR-0008: [`../../design-decisions.md`](../../design-decisions.md)
- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Integrity: [`CAP-0004-release-integrity.md`](CAP-0004-release-integrity.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
