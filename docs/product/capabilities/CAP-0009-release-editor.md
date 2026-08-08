# CAP-0009 — Release editor (host source editor)

| Field | Value |
| --- | --- |
| ID | CAP-0009 |
| Status | not implemented |
| Primary platform | Windows and macOS (Tauri) |
| Editor | Host OS-registered Office application or default text editor |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Selecting **Open draft** for a library document launches the host-registered
   editor for its source format on Windows and macOS: `.docx` → Word, `.xlsx` →
   Excel, `.pptx` → PowerPoint, and `.md` → the host default text editor. The
   app does not silently substitute a different Office handler for Office drafts.
2. The application does not embed an editor, render a custom preview, or
   auto-save the draft. It releases the file handle as soon as the editor is open
   so the host editor can write back at its own cadence.
3. While the draft is open in the host editor, the application remains responsive and
   can show lifecycle, approval, and confidentiality status read-only.
4. Closing the host editor does not change the document's lifecycle state. The next
   lifecycle transition re-reads and re-hashes the draft bytes (CAP-0002
   outcome 6 and ADR-0004).
5. If the host has no registered editor for the draft format, the **Open draft**
   action surfaces a clear message naming the missing handler and points the
   operator at the install location. The action does not fall back silently.
6. The application never blocks on the editor process; the operator can close
   the desktop app while an editor is still running and the next open of
   the workspace re-loads `.dms` cleanly.

## Non-goals

- Replacing the host source editor with an in-app editor
- Auto-saving the draft or auto-committing edits
- Locking the draft against other editor instances
- Supporting source formats other than Markdown and declared Office formats in v1

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0009-release-editor.html`](../wireframes/html/CAP-0009-release-editor.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0009-release-editor.png`](../wireframes/exports/CAP-0009-release-editor.png)

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Architecture: [`../../architecture.md`](../../architecture.md)
- ADR-0011: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
