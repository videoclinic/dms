# CAP-0019 — Inherited Microsoft Entra editor and approver routing

| Field | Value |
| --- | --- |
| ID | CAP-0019 |
| Status | not implemented |
| Identity source | Microsoft Entra workspace group (CAP-0021) |
| Storage | `<edit-root>/.dms/` routing policies and identity references |
| Tests | Partial phases 2 and 9f.4 evidence: [core policy tests](../../../crates/dms-core/tests/policies.rs), [desktop adapter commands](../../../crates/dms-desktop/src/lib.rs), [workflow route tests](../../../crates/dms-desktop/ui/configuration.test.mjs); live identity refresh remains phase 9i work |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The workspace uses the configured Microsoft Entra group from CAP-0021 as its
   read-only people source. A routing policy references an immutable Entra user
   object ID, not an application profile. The application exposes no user CRUD
   or group-membership management.
2. Under **Configuration → Workflow**, workflow-role routing uses the same
   defaults-first policy layout as CAP-0008: a compact people-source summary
   shows the bound group and its refresh/connection state, while an
   edit-root-relative tree and selected-folder editor define routing defaults
   and exceptions. The persistent Configuration navigation also exposes
   Workspace, Document defaults, and Notifications. **Manage identity source**
   opens CAP-0021 as a secondary surface with a visible return to Workflow;
   Entra setup does not occupy a permanent routing column or a separate
   Configuration-navigation entry.
3. The operator selects the edit root or an existing folder from that tree, then
   assigns one responsible editor and one approver. The root assignments are the
   required defaults after a valid identity-source binding. A folder may change
   either role without changing the other. The picker lists only currently
   eligible direct user members of the bound Entra group; it does not accept an
   arbitrary typed path or reuse Library navigation selection.
4. Each role derives independently from the nearest configured ancestor folder
   assignment. A document may explicitly override either role; clearing that
   override restores inheritance for that role without copying a policy into the
   document record.
5. The CAP-0006 library navigator (row metadata and selection pane) shows the
   effective editor and approver, their Entra display name/email, and whether
   each is inherited or explicitly overridden. It also exposes an unresolved
   identity rather than displaying a stale person as active.
6. An approval-required review request uses the document's effective approver
   only after the app refreshes and resolves that person in the bound Entra
   group. The approval email and its local-app deep link address that person and
   the specific review request (CAP-0010). A direct minor release performs the
   same refresh and resolution before it snapshots the effective approver as the
   publication-notification recipient. The responsible editor is workflow-routing
   and audit metadata; it is not an alternate approver.
7. Changing a policy changes only inheriting descendants. A change to the
   effective approver while a review is open invalidates that review and requires
   a new request; explicit overrides and nearer folder assignments remain
   unchanged. If Entra membership later makes the selected person ineligible,
   the policy becomes unresolved; the app never selects a replacement itself.
8. Review requests and released versions snapshot the effective editor and
   approver Entra tenant/object IDs plus their display name/email at that time.
   Later policy, group, or profile changes do not rewrite historical evidence.
9. Recording a review decision requires interactive Entra sign-in. The signed-in
   tenant/object ID must match the review's snapshotted approver and remain
   eligible in the bound group. The application records that authenticated actor
   and the local OS user in the canonical event chain (CAP-0011).
10. Role routing does not grant or revoke filesystem access and does not prevent
   source-file editing. Filesystem, SharePoint, and OneDrive ACLs remain the
   access-control boundary.
11. The folder-exceptions table follows CAP-0005's growing-table interaction;
    its text filter case-insensitively matches the edit-root-relative path,
    effective editor, effective approver, and routing state before pagination.

## Non-goals

- Application-enforced editing permissions
- Application-managed workflow users or group membership
- Group role assignees, multiple approvers, or RACI matrices in v1
- Assigning roles to paths outside the edit root

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0019-inherited-workflow-role-routing.html`](../wireframes/html/CAP-0019-inherited-workflow-role-routing.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0019-inherited-workflow-role-routing.png`](../wireframes/exports/CAP-0019-inherited-workflow-role-routing.png)

- Local store: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Notification: [`CAP-0010-notification-transport.md`](CAP-0010-notification-transport.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Workflow identity: [`CAP-0021-microsoft-entra-workflow-identity.md`](CAP-0021-microsoft-entra-workflow-identity.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0019, ADR-0021: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
