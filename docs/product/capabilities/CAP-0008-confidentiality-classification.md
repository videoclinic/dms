# CAP-0008 — Inherited document confidentiality classification

| Field | Value |
| --- | --- |
| ID | CAP-0008 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/` |
| Tests | Partial phases 2, 9f.1, and 9f.3 evidence: [core policy tests](../../../crates/dms-core/tests/policies.rs), [desktop adapter commands](../../../crates/dms-desktop/src/lib.rs), [Library override-form tests](../../../crates/dms-desktop/ui/library.test.mjs), [Document-defaults policy tests](../../../crates/dms-desktop/ui/configuration.test.mjs) |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The operator configures a workspace list of confidentiality types. Each type
   has a stable, portable filename-safe ID (lowercase letters, digits, and
   hyphens; for example `restricted`) and a display label. Renaming a label
   never changes its ID. Deletion is rejected while a folder policy, document,
   or historical release references the ID.
2. Under **Configuration → Document defaults**, confidentiality uses the same
   defaults-first policy layout as CAP-0019: a compact workspace-default summary
   identifies the edit root's type and enabled-type count, while direct folder
   policies are the primary workspace. The persistent Configuration navigation
   also exposes Workspace, Workflow, and Notifications so this policy editor is
   not a dead-end page. **Manage confidentiality types** opens catalogue
   administration as a secondary surface and returns to Document defaults when
   dismissed; the full type list and its controls do not occupy a permanent
   Configuration column.
3. The folder-policy editor shows an
   edit-root-relative folder tree containing the edit root and every accessible
   descendant folder, including empty folders and folders without library
   documents; `<edit-root>/.dms` is excluded. Selecting a tree node is the only
   way to choose the editor target. The editor shows that selected relative path;
   it does not accept an arbitrary typed path and does not reuse Library
   navigation selection.
4. The editor targets only the selected edit root or existing folder under it.
   The operator chooses one enabled confidentiality type from the workspace
   catalogue. **Save folder policy** creates a direct policy for the selected
   folder or replaces that folder's existing direct policy; it does not create
   policies for ancestors or descendants.
5. The edit root always has a direct policy and is the required fallback. Its
   type can be replaced but its policy cannot be removed. **Remove folder
   policy** is available only for a non-root folder and removes that folder's
   direct policy record. It neither deletes or moves the folder nor changes a
   child folder policy or a document-level override.
6. A folder policy applies to that folder and all descendant folders unless a
   nearer direct folder policy exists. Removing a non-root policy immediately
   makes the selected folder and every descendant without a nearer policy
   inherit the nearest remaining ancestor policy; it does not copy that type
   into any folder or document record.
7. For each **library document**, the app resolves the effective type from the
   document's current edit-root-relative source path and the nearest ancestor
   folder policy; the root policy is the required fallback. A file that is not
   in the library has no DMS confidentiality classification. The explorer's
   library row and single-document selection pane display the effective type,
   its source folder, and whether it is inherited or explicitly overridden.
8. For exactly one selected library document, the selection pane offers an
   **Override confidentiality** action. The operator can choose one enabled
   type as a document-level override or clear the existing override. Clearing it
   restores inherited behaviour without copying a policy into the document
   record.
9. Changing, adding, or removing a folder policy immediately changes the
   effective type of inheriting descendants only. Explicit document overrides
   and nearer folder policies are unchanged.
10. A review request and a released version snapshot the effective type ID and
   display label in their immutable workflow/release evidence. Later policy
   changes or label renames do not rewrite historical records or PDF filenames.
11. If a document's effective confidentiality type changes while an
   approval-required content review is open or after approval but before release,
   the request/approval is invalidated and a new review is required. Historical
   snapshots remain unchanged.
12. The folder-exceptions table follows CAP-0005's growing-table interaction;
    its text filter case-insensitively matches the edit-root-relative path and
    confidentiality type before pagination.

## Non-goals

- Encrypting document files or `.dms`
- Enforcing access rights from a confidentiality label
- Applying a policy to arbitrary paths outside the edit root

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0008-confidentiality-classification.html`](../wireframes/html/CAP-0008-confidentiality-classification.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0008-confidentiality-classification.png`](../wireframes/exports/CAP-0008-confidentiality-classification.png)

- Storage: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0010: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)