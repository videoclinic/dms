# CAP-0021 — Microsoft Entra workflow identity source

| Field | Value |
| --- | --- |
| ID | CAP-0021 |
| Status | partial — Phase 9j live identity-source setup, refresh, and sign-in |
| Authority | Microsoft Entra ID group |
| Storage | `<edit-root>/.dms/` group binding and display cache; OS-user app-global Entra settings; OS credential store token cache |
| Tests | [Core policy and schema migration tests](../../../crates/dms-core/tests/policies.rs), [desktop Graph and configuration tests](../../../crates/dms-desktop/src/graph.rs), [configuration UI tests](../../../crates/dms-desktop/ui/configuration.test.mjs) |

## Implemented subset

1. **Configuration → Workflow → Manage identity source** uses a distinct
   app-global public-client/tenant configuration card plus a per-library group
   object ID card, starts delegated
   Microsoft Entra device authorization, previews the resolved tenant/group and
   eligible direct enabled user members, and applies the binding only after
   explicit confirmation. The apply action is single-flight per rendered form;
   the preview is consumed only after the workspace binding saves successfully,
   so a failed save remains retryable. First setup requires the operator to
   select the edit-root editor and approver from that preview, then persists the
   binding, people cache, and both root roles atomically. A later replacement
   does not remap existing role references to people in the new group.
- The **Application Entra configuration** card confirms a successful save with
  an in-place status notice and remains on the same Configuration secondary
  surface; saving global Entra configuration never changes the active
  Configuration activity.
- The device-flow sign-in page opens in the host's default browser through a
  Tauri shell opener that accepts only `https:` URLs, plus
  `http://localhost` and `http://127.0.0.1` for local development; the in-app
  WebView never navigates to the verification URI.
- Delegated Microsoft Entra tokens remain exclusively in the OS credential
  store. When their serialized UTF-16 representation exceeds the Windows
  Credential Manager per-password limit, DMS writes UTF-16-safe fragments
  under a versioned manifest and continues to read the previous single-entry
  cache format.
- An expired or failed Microsoft Entra device-flow challenge surfaces an
  explicit **Sign in again** control on the same surface. It reissues a fresh
  challenge with the operator's transient last group ID, and the Graph adapter
  discards expired pending challenges.
- Saving app-global Entra settings clears any rendered challenge or preview
  invalidated when the Graph client adopts that configuration.
2. Replacing a binding retains historical evidence, invalidates stale workflow
   candidates, and leaves existing role references unresolved rather than mapping
   them to the replacement group.
3. The desktop adapter refreshes the direct enabled user members from Microsoft
   Graph before a workflow role assignment; it persists the display cache and
   refresh timestamp only in `<edit-root>/.dms`.
   A successful direct-member response with zero eligible users is represented
   separately from refresh, permission, tenant, inaccessible-group, and
   disabled-only failures.
4. An approver sign-in command uses delegated device authorization, resolves
   `/me` to an immutable tenant/object-ID actor, and leaves the actor available
   to the lifecycle adapter. Approval-decision composition remains phase 9k.
5. The public-client and tenant IDs are app-global OS-user configuration.
   Non-empty `DMS_ENTRA_CLIENT_ID` and `DMS_ENTRA_TENANT_ID` process values
   override their stored counterparts and are read-only in Configuration;
   invalid non-empty values fail closed. The identity-source overview shows
   these effective IDs with the library-bound group label and object ID. Its
   Group ID control opens
   `https://myaccount.microsoft.com/groups/<encoded-group-id>` through the host
   browser rather than navigating the app WebView. Delegated access and refresh
   tokens are stored only in the OS credential store; `.dms` retains neither
   tokens, client/tenant IDs, nor client secrets.

## Full capability contract (remaining outcomes are not all present)

When implemented, the following must hold:

1. Before a workspace can configure workflow roles or submit a review, app-global
   configuration provides a valid Microsoft Entra public-client ID and tenant ID,
   and the workspace has an identity-source binding containing one group object
   ID and a display label. The application does not
   create or modify the Entra group.
2. While a binding exists, **Configuration → Workflow** shows the group label,
   connection state, eligible-person count, and last refresh in a compact
   summary. **Manage identity source** opens a secondary surface whose
   current-source overview shows the effective app-global public-client ID and
   tenant ID separately from the library-bound group label/object ID. Its Group
   ID is a host-browser control for
   `https://myaccount.microsoft.com/groups/<encoded-group-id>`. First setup and
   replacement stay on this secondary surface with a visible return to Workflow;
   the identity source has no independent Configuration-navigation entry. The
   global IDs/group object ID form, preview, and replacement warning do not
   permanently occupy a main Configuration column.
3. First setup and a later binding replacement require an operator to configure
   the public-client/tenant IDs supplied by Microsoft 365 administration and
   then enter the library group object ID, sign in interactively, preview the
   resolved source, select the required edit-root editor and approver during
   first setup, and explicitly apply the binding. First setup persists the
   binding, people cache, and both roles atomically. Replacing a binding marks
   every live workflow-role policy `unresolved`; the app never attempts to map
   roles between groups. Historical evidence remains unchanged.
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
   membership. The delegated app registration has tenant-admin consent for
   `GroupMember.Read.All` and `User.Read.All`; the latter is required to read
   profile fields and `accountEnabled`, so `User.ReadBasic.All` is insufficient.
6. The application refreshes membership before a person is assigned as Owner or to a role,
   before a review request is submitted, and before a review decision is
   recorded. It stores a person’s immutable Entra object ID in a role policy and
   uses cached display name/email only for presentation and notification. A
   cache is never authorization truth. Owner, editor, approver, requester, and
   actor equality uses tenant-scoped object ID, never display name or email.
7. A successful direct-member refresh containing zero eligible users may expose
   the literal presentation placeholders `<owner>` and `<editor>`. They have no
   object ID, are never persisted as an Entra identity, approver, actor, or mail
   recipient, and cannot pass candidate/release validation. A missing binding,
   failed refresh, tenant mismatch, inaccessible group, disabled-only response, or
   missing role identity is explicit. A policy that can no longer resolve to an
   eligible person is `unresolved`; new review submission and approval decisions
   fail closed until an operator reroutes it. Membership changes never silently
   rewrite folder/document policies or historical workflow evidence. A later
   successful non-empty refresh requires explicit real-person selections; staged
   Owner/Editor changes apply only with the next successful release.
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
11. The read-only eligible-people table follows CAP-0005's growing-table
    interaction; its text filter case-insensitively matches person display name,
    email address, object ID, and account state before pagination.

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
- ADR-0021, ADR-0024: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`CHG-0001`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
  owns the broader Tauri bootstrap and remaining phase 9l integration evidence;
  archived [`CHG-0002`](../../changes/archive/CHG-0002-entra-configuration-ux-fixes.md)
  records the completed Entra configuration UX corrections; archived
  [`CHG-0003`](../../changes/archive/CHG-0003-retry-safe-entra-identity-application.md)
  records reliable identity-source application.
