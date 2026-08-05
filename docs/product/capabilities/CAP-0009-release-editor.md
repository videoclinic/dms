# CAP-0009 — Release editor (host Office)

| Field | Value |
| --- | --- |
| ID | CAP-0009 |
| Status | not implemented |
| Primary platform | Windows and macOS (Tauri) |
| Editor | Host OS-registered Microsoft Office application |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Selecting **Open draft** for a library document launches the host-registered
   Microsoft Office application for the draft format (`.docx` → Word,
   `.xlsx` → Excel, `.pptx` → PowerPoint) on Windows and macOS. The app does
   not silently substitute a non-Office handler.
2. The application does not embed an editor, render a custom preview, or
   auto-save the draft. It releases the file handle as soon as Office is open
   so Office can write back at its own cadence.
3. While the draft is open in Office, the application remains responsive and
   can show lifecycle, approval, and confidentiality status read-only.
4. Closing Office does not change the document's lifecycle state. The next
   lifecycle transition re-reads and re-hashes the draft bytes (CAP-0002
   outcome 6 and ADR-0004).
5. If the host has no Office handler registered for the draft format, the
   **Open draft** action surfaces a clear message naming the missing handler
   and points the operator at the install location. The action does not fall
   back to a different binary silently.
6. The application never blocks on the Office process; the operator can
   close the desktop app while Office is still running and the next open of
   the workspace re-loads `.dms` cleanly.

## Non-goals

- Replacing Office with an in-app editor
- Auto-saving the draft or auto-committing edits
- Locking the draft against other Office instances
- Supporting non-Office draft formats (`.odt`, `.ods`, etc.) in v1

## Links

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Architecture: [`../../architecture.md`](../../architecture.md)
- ADR-0011: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
