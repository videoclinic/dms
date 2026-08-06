# CAP-0008 — Inherited document confidentiality classification

| Field | Value |
| --- | --- |
| ID | CAP-0008 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/` |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The operator configures a workspace list of confidentiality types. Each type
   has a stable, portable filename-safe ID (lowercase letters, digits, and
   hyphens; for example `restricted`) and a display label. Renaming a label
   never changes its ID. Deletion is rejected while a folder policy, document,
   or historical release references the ID.
2. The folder-policy editor targets only the edit root or an existing folder
   under it, identified by its edit-root-relative path. **Save folder policy**
   creates a direct policy for the selected folder or replaces that folder's
   existing direct policy; it does not create policies for ancestors or
   descendants.
3. The edit root always has a direct policy and is the required fallback. Its
   type can be replaced but its policy cannot be removed. **Remove folder
   policy** is available only for a non-root folder and removes that folder's
   direct policy record. It neither deletes or moves the folder nor changes a
   child folder policy or a document-level override.
4. A folder policy applies to that folder and all descendant folders unless a
   nearer direct folder policy exists. Removing a non-root policy immediately
   makes the selected folder and every descendant without a nearer policy
   inherit the nearest remaining ancestor policy; it does not copy that type
   into any folder or document record.
5. A document without an explicit type uses the nearest ancestor folder type;
   the root policy is the required fallback. The explorer displays the effective
   type, its source folder, and whether it is inherited or explicitly overridden.
6. An operator can set or clear a document-level override. Clearing it restores
   inherited behaviour without copying a policy into the document record.
7. Changing, adding, or removing a folder policy immediately changes the
   effective type of inheriting descendants only. Explicit document overrides
   and nearer folder policies are unchanged.
8. A review request and a released version snapshot the effective type ID and
   display label in their immutable workflow/release evidence. Later policy
   changes or label renames do not rewrite historical records or PDF filenames.
9. If a document's effective confidentiality type changes while content review
   is open or after approval but before release, the request/approval is
   invalidated and a new review is required. Historical snapshots remain
   unchanged.

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