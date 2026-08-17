# CHG-0022 — Workflow metadata and resizable columns in the Library table

**Plan ID:** CHG-0022-library-table-workflow-columns
**Created:** 2026-08-17
**Depends on:** none
**Context sources:** `docs/product/capabilities/CAP-0006-library-explorer.md` (current-folder contents), `crates/dms-core/src/library.rs` (`LibraryDocumentSummary`, `Workspace::document_summary`), `crates/dms-desktop/ui/library.mjs` (`LIBRARY_COLUMNS`, `libraryTableHeaders`, `setColumnWidth`), `crates/dms-desktop/ui/app.mjs` (column-resize pointer handlers), `crates/dms-desktop/ui/styles.css` (`.table-scroll`, `.col-resize-grip`)
**Produces:** The Library folder-contents table shows each registered document's effective **editor**, **approver**, **confidentiality**, and **next review due** date as their own columns; every table column is pointer-resizable for the active session.
**Status:** done — the contents table carries five additional workflow-metadata columns and all eight columns resize by dragging their header grips; widths are session-only; archived after workspace gate

| Field | Value |
| --- | --- |
| ID | CHG-0022 |
| Status | done |
| External request | Inferred from in-flight work found in the working tree with no CHG record; inferred driver named in the commit proposal and confirmed by the operator before the batch fired |
| Affected CAPs | CAP-0006 |
| Decision records | none |

## Current state

- The folder-contents table rendered five fixed columns (Name, Title, Library state, Lifecycle, Relative path) from hardcoded `<th>` elements; document workflow data (editor, approver, confidentiality, next review due) was only visible in the selection pane after selecting a document.
- `LibraryDocumentSummary` carried only the document ID, lifecycle, and control data, so the UI could not render workflow fields without per-document lookups.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Core metadata fields + table columns | done (`cargo test -p dms-core` 24 passed; frontend `node --test` 98 passed, including the column-assertion tests) | `cargo test --workspace` and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0 |
| 2 | Resizable column widths | done (session-only `column_widths` state, pointer-capture drag on header grips, minimum widths) | same gate |
| 3 | CAP amendment, record closeout, archive | done (CAP-0006 outcome 2 documents the new columns and session-only resize; CHG archived as done) | same gate; CHG archived as done |

## Out of scope

- Persisting column widths across sessions or saved views (session-only, like the splitter widths).
- Column reordering, hiding, or sorting.
- Showing the metadata columns for folder rows (files only; folders render `—`/empty).
