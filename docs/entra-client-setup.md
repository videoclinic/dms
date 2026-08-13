# Microsoft Entra application setup for DMS

This guide is for the Microsoft 365 / Entra administrator who creates the
**Application (client) ID** and grants the Microsoft Graph access used by the
DMS desktop application.

DMS is a local desktop **public client**. It signs a person in interactively
with the OAuth device authorization flow, reads one configured group's direct
user members for workflow routing, and resolves the signed-in approver. It does
not use a client secret, certificate, application permissions, background
service identity, SharePoint, OneDrive, or document-content access.

## What to prepare

- A Microsoft Entra tenant used by the DMS operator and its approvers.
- One dedicated Entra security group per DMS library where practical. An
  existing Microsoft 365 group is acceptable only when its membership exactly
  matches the workflow population.
- A person who can grant tenant-wide admin consent for Microsoft Graph
  `User.Read.All` and `GroupMember.Read.All`.
- The group's **Object ID**. This is the value entered later as the library
  group ID, not the group display name.

DMS accepts direct, enabled **user** members only. Nested groups and other
Entra directory object types do not become eligible DMS workflow people.

## Create the app registration

1. Sign in to the [Microsoft Entra admin center](https://entra.microsoft.com/)
   in the intended tenant. Select **Entra ID → App registrations → New
   registration**.[1]
2. Name the application, for example `DMS desktop workflow`.
3. Select **Accounts in this organizational directory only** (single tenant).
   DMS is configured with one specific tenant ID and does not support a shared
   multi-tenant registration.[1]
4. Select **Register**.
5. On **Overview**, copy both values:
   - **Application (client) ID** — this is DMS's *Public client ID*.
   - **Directory (tenant) ID** — this is DMS's *Tenant ID*.[1]

Do **not** create a client secret or upload a certificate. A desktop public
client cannot keep either credential secret, and DMS never uses a
client-credentials / app-only flow.

## Enable public-client device sign-in

1. In the app registration, open **Authentication**.
2. Under **Advanced settings**, set **Allow public client flows** to **Yes** and
   save.[2]
3. Do not add a redirect URI for DMS. DMS uses the device authorization flow,
   which starts at the tenant's `/devicecode` endpoint and completes through the
   browser sign-in prompt; it does not use a local callback URI.[5]

## Configure Microsoft Graph permissions

Open **API permissions**. Keep only the following **Delegated permissions**
under **Microsoft Graph**:

| Permission | Why DMS needs it | Consent |
| --- | --- | --- |
| `User.Read.All` | Read each direct user member's display name, email address, and `accountEnabled` state, and resolve the signed-in approver through Microsoft Graph `/me`. `User.ReadBasic.All` is insufficient because DMS must exclude disabled accounts. | **Tenant admin consent required.** [4] |
| `GroupMember.Read.All` | Read the configured group's basic properties and direct membership to preview/refresh eligible people. | **Tenant admin consent required.** [4] |

Then:

1. Select **Add a permission → Microsoft Graph → Delegated permissions**.
2. Add `User.Read.All` and `GroupMember.Read.All` if they are not already listed.
3. Confirm that both are **Delegated**, not **Application**.
4. Select **Grant admin consent for _<tenant>_** and confirm.
5. Refresh the page and verify that both permissions show **Granted for
   _<tenant>_**.[1][4]

DMS requests `openid`, `profile`, and `offline_access` during sign-in in
addition to the Graph permissions. Those are OpenID Connect / token scopes, not
extra Microsoft Graph data permissions. `offline_access` permits Entra to issue
a refresh token; DMS stores its delegated token cache in the operating system's
credential store.[5]

Do **not** add `Directory.Read.All`, `Group.Read.All`, `User.ReadWrite.All`,
`Sites.*`, `Files.*`, or any write permission for DMS. They are unnecessary for
the implemented DMS Graph calls. In particular, `Group.Read.All` also permits
access to group content such as conversations and files, while
`GroupMember.Read.All` is limited to basic group properties and memberships.[4]

## Configure DMS

1. Start DMS without `DMS_ENTRA_CLIENT_ID` or `DMS_ENTRA_TENANT_ID` set, unless
   your organization intentionally manages these values through the process
   environment.
2. Open a DMS library, then go to **Configuration → Workflow → Manage identity
   source**.
3. In **Application Entra configuration**, enter the copied **Application
   (client) ID** and **Directory (tenant) ID**, then save.
4. In **Library Entra group**, enter the group's **Object ID**.
5. Select **Sign in and preview group**. Complete the displayed device sign-in
   with an account that can read the selected group's membership.
6. Check the returned tenant, group, and eligible-person list, then explicitly
   apply the source binding.

The client and tenant IDs are non-secret, OS-user-wide DMS settings shared by
local libraries. The group object ID is library-specific DMS metadata. DMS
stores delegated access/refresh tokens in the OS credential store, not in the
library `.dms` folder.[5]

### Environment-managed configuration

Set both variables before launching DMS when a deployment mechanism owns the
registration settings:

```text
DMS_ENTRA_CLIENT_ID=<Application (client) ID>
DMS_ENTRA_TENANT_ID=<Directory (tenant) ID>
```

A non-empty variable overrides the corresponding saved DMS value for that
process and makes the field read-only in Configuration. Both values must be
valid UUIDs; an invalid or incomplete configuration blocks Graph use rather
than silently falling back to saved settings.

## Verification checklist

- **API permissions** lists only delegated `User.Read.All` and
  `GroupMember.Read.All` for DMS's Microsoft Graph use.
- Both Graph permissions display **Granted for _<tenant>_**.
- **Authentication** has **Allow public client flows** set to **Yes**.
- DMS can complete device sign-in and preview the intended tenant and group.
- The preview lists the expected direct, enabled users and excludes nested-group
  members.
- A test workflow-role selection and approver sign-in succeed without granting
  filesystem, SharePoint, or OneDrive access through DMS.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Consent fails, DMS receives HTTP 403, or Graph returns a user ID without profile/account-status fields | Confirm `User.Read.All` and `GroupMember.Read.All` are **delegated** permissions and both show tenant-wide admin consent. Sign in again after consent is granted so the token contains the new scope. The signed-in person must also be authorized to read the group membership; delegated access is constrained by both the app permission and the user's own access. [3][4] |
| Group preview is empty or misses people | Confirm the group contains direct, enabled user members. DMS intentionally ignores nested groups and non-user directory objects. |
| DMS says the client or tenant ID is missing or invalid | Recopy the IDs from the app registration **Overview**; use the Application (client) ID and Directory (tenant) ID, not object IDs. Check environment overrides first. |
| DMS says a token cache is invalid or sign-in is required | Restart the device sign-in flow. DMS does not put tokens in `.dms`; it uses the OS credential store. |
| An approver can sign in but cannot approve | Confirm they are still a direct enabled member of the configured group and are the review's snapshotted effective approver. |

## Security boundary

Treat the Entra group as the DMS workflow roster, not as filesystem or
SharePoint authorization. DMS does not create users or groups, change
membership, grant file access, or upload document data to Microsoft Graph.
Filesystem and SharePoint/OneDrive permissions remain separately administered.

## Related DMS records

- [Microsoft Entra workflow identity contract](product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md)
- [Architecture](architecture.md)
- [ADR-0021 and ADR-0024](design-decisions.md)
- [Privacy](privacy.md)

## Sources

[1] https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-register-app
[2] https://learn.microsoft.com/en-us/entra/identity-platform/scenario-desktop-app-configuration
[3] https://learn.microsoft.com/en-us/graph/api/group-list-members?view=graph-rest-1.0
[4] https://learn.microsoft.com/en-us/graph/permissions-reference
[5] https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code
