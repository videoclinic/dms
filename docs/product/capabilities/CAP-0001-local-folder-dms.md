# CAP-0001 — Local folder DMS metadata store

| Field | Value |
| --- | --- |
| ID | CAP-0001 |
| Status | not implemented |
| Primary platform | Windows and macOS (Tauri) |
| Storage | Hidden `.dms/` under the edit root |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Operator configures a DMS workspace with two rooted paths:
   - **edit root** — tree where Office and Markdown source drafts are edited
   - **publish root** — tree where released versioned PDFs are written
2. Both absolute roots and a stable opaque workspace ID are persisted in `.dms`
   and restored when the workspace is reopened. The ID is assigned when the
   workspace is initialized and remains unchanged for its lifetime.
3. Metadata lives only under `<edit-root>/.dms/` (name change requires ADR/CAP
   update). No database server.
4. Library entries store a **stable document ID** plus the current draft path
   **relative to the edit root** (and matching relative publish path segments).
   Absolute paths are not the durable identity.
5. On release, the app reconstructs the document’s relative directory tree
   under the publish root (creates missing folders) so edit layout and publish
   layout stay aligned.
6. Closing and reopening the workspace restores roots, library membership,
   document control data, and process state from `.dms`.
7. If `.dms` is missing under a chosen edit root, the app initializes it only
   after an explicit confirm, including choosing or confirming the publish root.
8. `.dms` stores the workspace confidentiality catalogue, document-type
   catalogue, Microsoft Entra tenant/group binding, read-only identity display
   cache, relative folder policies that reference Entra user object IDs, and
   document-control-data fields required by CAP-0008, CAP-0015, CAP-0019, and
   CAP-0021. It stores no SMTP password, OAuth token, or other credential.

## Non-goals

- Multi-master sync across machines
- Transparent encryption of `.dms` (filesystem ACLs remain the operator’s control)
- Treating edit root and publish root as the same folder by default (they may
  coincide only if the operator deliberately sets them equal)

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0001-local-folder-dms.html`](../wireframes/html/CAP-0001-local-folder-dms.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0001-local-folder-dms.png`](../wireframes/exports/CAP-0001-local-folder-dms.png)

- Architecture: [`../../architecture.md`](../../architecture.md)
- Workflow identity: [`CAP-0021-microsoft-entra-workflow-identity.md`](CAP-0021-microsoft-entra-workflow-identity.md)
- ADR-0001, ADR-0006, ADR-0009, ADR-0010, ADR-0015, ADR-0021: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
