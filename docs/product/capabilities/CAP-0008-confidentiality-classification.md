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
   has a stable ID and display label; deletion is rejected while a document or
   historical release references the ID.
2. The operator assigns a confidentiality type to the edit root or any folder
   by its edit-root-relative path. A folder configuration applies to that folder
   and all descendant folders unless a nearer folder configuration exists.
3. A document without an explicit type uses the nearest ancestor folder type;
   the root policy is the required fallback. The explorer displays the effective
   type and whether it is inherited or explicitly overridden.
4. An operator can set or clear a document-level override. Clearing it restores
   inherited behaviour without copying a policy into the document record.
5. Changing a folder policy immediately changes the effective type of inheriting
   descendants only. Explicit document overrides and nearer folder policies are
   unchanged.
6. A review request and a released version snapshot the effective type in their
   immutable workflow/release evidence. Later policy changes do not rewrite
   historical records.
7. If a document's effective confidentiality type changes while content review
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