# CAP-0019 — Inherited editor and approver routing

| Field | Value |
| --- | --- |
| ID | CAP-0019 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/` |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The workspace maintains a workflow-person roster. Each person has a stable
   ID, display label, and email address. Deletion is rejected while a folder
   policy, document override, or historical workflow event references the ID;
   the person can instead be disabled for future assignment.
2. The operator assigns one responsible editor and one approver to the edit root
   or any folder by its edit-root-relative path. The root assignments are the
   required defaults. A folder may change either role without changing the
   other.
3. Each role derives independently from the nearest configured ancestor folder
   assignment. A document may explicitly override either role; clearing that
   override restores inheritance for that role without copying a policy into the
   document record.
4. The CAP-0006 library navigator (row metadata and selection pane) shows
   the effective editor and approver and
   whether each is inherited or explicitly overridden.
5. A review request uses the document's effective approver. The approval email
   and its local-app deep link address that person and the specific review
   request (CAP-0010). The responsible editor is workflow-routing and audit
   metadata; it is not an alternate approver.
6. Changing a policy changes only inheriting descendants. A change to the
   effective approver while a review is open invalidates that review and requires
   a new request; explicit overrides and nearer folder assignments remain
   unchanged.
7. Review requests and released versions snapshot the effective editor and
   approver IDs. Later policy or roster changes do not rewrite historical
   evidence.
8. Role routing does not grant or revoke filesystem access and does not prevent
   Office editing. The application records the local OS user for workflow events;
   filesystem ACLs remain the access-control boundary.

## Non-goals

- Directory-backed authentication or identity proof
- Application-enforced editing permissions
- Groups, multiple approvers, or RACI matrices in v1
- Assigning roles to paths outside the edit root

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0019-inherited-workflow-role-routing.html`](../wireframes/html/CAP-0019-inherited-workflow-role-routing.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0019-inherited-workflow-role-routing.png`](../wireframes/exports/CAP-0019-inherited-workflow-role-routing.png)

- Local store: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Notification: [`CAP-0010-notification-transport.md`](CAP-0010-notification-transport.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0019: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
