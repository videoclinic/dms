# CAP-0021 — Microsoft Entra workflow identity source

| Field | Value |
| --- | --- |
| ID | CAP-0021 |
| Status | not implemented |
| Authority | Microsoft Entra ID group |
| Storage | `<edit-root>/.dms/` binding and display cache; OS credential store token cache |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Before a workspace can configure workflow roles or submit a review, it has a
   Microsoft Entra identity-source binding. The binding contains the expected
   tenant ID, one group object ID, and a display label. The desktop client ID is
   product configuration, not a workspace secret. The application does not
   create or modify the Entra group.
2. While a binding exists, **Configuration** shows its tenant display, group
   label/object ID, connection state, eligible-person count, and last refresh in
   a compact current-source summary. **Manage identity source** opens first setup
   and replacement as a secondary surface; the Tenant ID/group object ID form,
   preview, and replacement warning do not permanently occupy a main
   Configuration column.
3. First setup and a later binding replacement require an operator to enter the
   tenant ID and group object ID supplied by Microsoft 365 administration, sign
   in interactively, preview the resolved source, and explicitly apply the
   binding. Replacing a binding marks every live workflow-role policy
   `unresolved`; the app never attempts to map roles between groups. Historical
   evidence remains unchanged.
4. A Microsoft 365 administrator supplies a dedicated Entra security group for
   the workspace, or an existing Microsoft 365 group only when its membership
   exactly matches the intended workflow population. The source is explicit;
   the application never infers a group from a mapped SharePoint or OneDrive
   path, site permission, sharing link, or local OS account.
5. The application signs in interactively to the configured tenant and uses
   Microsoft Graph to list the group's direct **user** members. It treats other
   directory object types and nested groups as ineligible. The people picker is
   read-only: it can refresh, search, and select an eligible person for a DMS
   routing policy, but cannot add, edit, disable, or delete a user or group
   membership.
6. The application refreshes membership before a person is assigned to a role,
   before a review request is submitted, and before a review decision is
   recorded. It stores a person’s immutable Entra object ID in a role policy and
   uses cached display name/email only for presentation and notification. A
   cache is never authorization truth.
7. A failed refresh, tenant mismatch, inaccessible group, disabled account, or
   missing role identity is explicit. A policy that can no longer resolve to an
   eligible person is `unresolved`; new review submission and approval decisions
   fail closed until an operator reroutes it. Membership changes never silently
   rewrite folder/document policies or historical workflow evidence.
8. A review request snapshots the effective approver’s Entra tenant/object ID,
   display name, and email. A decision requires interactive sign-in; the app
   accepts it only from the snapshotted tenant/object ID while that user remains
   an eligible member of the bound group. The decision event records that
   authenticated identity as CAP-0011 evidence.
9. The application stores no Entra password, client secret, or delegated token
   in `.dms`. The delegated-token cache belongs in the OS credential store. Graph
   requests carry no draft/released document bytes, root paths, approval comments,
   or document-control data.
10. The identity source routes workflow responsibility and verifies an approval
   actor only. It does not grant or revoke filesystem, SharePoint, or OneDrive
   access. Those access-control systems remain independently administered.

## Non-goals

- Application user, group, or Microsoft 365 tenant administration
- Background directory synchronization or a copied Entra user directory
- Reading SharePoint site permissions or OneDrive sharing as a workflow roster
- SharePoint/OneDrive document-content synchronization
- Role assignment to a group, multiple approvers, or RACI matrices in v1
- Browser-based approval or digital signatures

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0021-microsoft-entra-workflow-identity.html`](../wireframes/html/CAP-0021-microsoft-entra-workflow-identity.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0021-microsoft-entra-workflow-identity.png`](../wireframes/exports/CAP-0021-microsoft-entra-workflow-identity.png)

- Workflow-role routing: [`CAP-0019-inherited-workflow-role-routing.md`](CAP-0019-inherited-workflow-role-routing.md)
- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Local store: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- Architecture: [`../../architecture.md`](../../architecture.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0021: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
