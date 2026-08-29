# Windows Entra identifier deployment

Deploy the DMS Desktop Microsoft Entra public-client ID and tenant ID to managed Windows devices. Policy writes only those two non-secret identifiers. It does not create an Entra app registration, store tokens, bind a library group, or start OAuth device authorization.

Download `dms-desktop-admx.zip` from the [GitHub Releases](https://github.com/videoclinic/dms/releases) page for the installed version. The zip contains `DMSDesktop.admx` and `en-US/DMSDesktop.adml`.

## Entra control-plane prerequisites

Create the public client, enable device flow, and grant only the existing delegated Graph permissions with admin consent before any device policy. Follow [Microsoft Entra application setup](entra-client-setup.md). Group Policy and Intune do not register that application.

## Manual local setup

Use this path for a single workstation or when no computer policy is assigned.

1. Complete the Configuration flow in [Configure DMS](entra-client-setup.md#configure-dms).
2. Configure and preview the library-specific Entra group and root roles on that same surface.
3. Treat `DMS_ENTRA_CLIENT_ID` and `DMS_ENTRA_TENANT_ID` as a deliberate process-launch override, not as persistent enterprise deployment. When both values are the effective source, DMS validates the cached delegated session or starts one non-blocking device-authorization status at launch. That is OAuth device authorization, not an Entra device object, and it does not open a browser unless the operator chooses **Open sign-in page**.

Windows computer policy, when present, supersedes this path. See [ADR-0028](design-decisions.md) and [ADR-0029](design-decisions.md).

## Elevated local policy test fixture

`deployment/windows/DMSDesktop-test-policy.reg` writes the supplied public
client and tenant UUIDs to the exact machine-policy values (`EntraClientId` and
`EntraTenantId`). Import it only from an elevated Windows session to exercise
the installed app's policy precedence and Configuration state before a managed
rollout:

```text
reg.exe import DMSDesktop-test-policy.reg
reg.exe query HKLM\SOFTWARE\Policies\Videoclinic\DMS
```

Restart DMS Desktop and confirm both identifiers are **Managed by Windows
policy**. This fixture does not establish GPO or Intune delivery evidence.
Before the process-environment scenario or after the check, import
`DMSDesktop-clear-test-policy.reg` from an elevated Windows session, then
restart DMS Desktop. It removes only the two DMS policy values and preserves
the containing registry key:

```text
reg.exe import DMSDesktop-clear-test-policy.reg
```

## Domain Group Policy

Standard Group Policy uses the language-neutral ADMX plus the `en-US` ADML from a Central Store.[1]

1. Copy `DMSDesktop.admx` to `\\<domain>\SYSVOL\<domain>\Policies\PolicyDefinitions\`.
2. Copy `en-US\DMSDesktop.adml` to `\\<domain>\SYSVOL\<domain>\Policies\PolicyDefinitions\en-US\`.
3. In Group Policy Management, edit a Computer Configuration GPO and open **Administrative Templates → Videoclinic → DMS Desktop**.
4. Enable **Microsoft Entra public-client identifiers** and enter both UUID values (Application (client) ID and Directory (tenant) ID).
5. Link the GPO to the intended OU and apply security or WMI filtering.
6. On a target device run `gpupdate /force`, then:

   ```text
   reg.exe query HKLM\SOFTWARE\Policies\Videoclinic\DMS
   ```

   Confirm `EntraClientId` and `EntraTenantId` are the two expected UUIDs.
7. Restart DMS Desktop. Configuration labels both identifiers **Managed by Windows policy** and disables the application form.

## Intune

Intune custom ADMX import is public preview and accepts one `en-US` ADML per template.[2]

1. In the Intune admin center open **Devices → Configuration → Import ADMX** and import `DMSDesktop.admx` with `en-US/DMSDesktop.adml`.
2. Create a **Windows 10 and later** profile whose type is **Imported Administrative templates (Preview)**.
3. Enable the DMS Desktop Entra identifiers policy and enter both UUIDs.
4. Assign the profile to the intended device group and wait until Intune reports the policy as delivered.
5. On a target device run `reg.exe query HKLM\SOFTWARE\Policies\Videoclinic\DMS` and restart DMS Desktop.

Do not import a `de-DE` ADML. Do not add a Windows ADMX dependency; this template is a standalone vendor namespace so Intune import does not require `Microsoft.Policies.Windows`.

## Rollback and troubleshooting

| Symptom | Action |
| --- | --- |
| Need to stop enforcing identifiers | Set the policy to **Not Configured** (or remove both registry values), refresh policy, and restart DMS. Saved OS-user configuration and workspace `.dms` metadata are untouched. |
| Incomplete or malformed policy | DMS blocks Graph on purpose. Fix both UUID values or set **Not Configured**. There is no fallback to environment or saved settings while either policy value exists. |
| Policy present but Configuration still editable | Confirm the query above returns both values on `HKLM`, then restart DMS. User-scoped `HKCU` policy is not used. |
| Assigned to a user group | Reassign to a device group. This template is Computer Configuration writing `HKLM`; user assignment is not the supported path and does not replace device targeting. |
| Intune shows the template but HKLM is empty | This is Computer Configuration. On a workplace-joined (user MDM) device, device-group assignment may ingest PolicyManager without writing `HKLM`. Confirm Azure AD join or that `reg.exe query HKLM\SOFTWARE\Policies\Videoclinic\DMS` returns both values after an Intune sync with no local fixture. |
| Users still must sign in | Expected. Policy does not replace delegated device authorization or library identity-source application. |
| Wrong tenant on every Windows user | Clear the policy as above. A computer policy is shared by every user of the device. |

## Related DMS records

- [Microsoft Entra application setup](entra-client-setup.md)
- [Microsoft Entra workflow identity contract](product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md)
- [Architecture](architecture.md)
- [ADR-0028 and ADR-0029](design-decisions.md)
- [Privacy](privacy.md)
- [ADMX template](deployment/windows/admx/DMSDesktop.admx)
- [ADML strings](deployment/windows/admx/en-US/DMSDesktop.adml)

## Sources

[1]: https://learn.microsoft.com/en-us/troubleshoot/windows-client/group-policy/create-and-manage-central-store
[2]: https://learn.microsoft.com/en-us/intune/intune-service/configuration/administrative-templates-import-custom
