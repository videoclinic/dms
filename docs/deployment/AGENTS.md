# Deployment

## Purpose

Own Windows deployment runbooks and administrative-template assets for DMS Desktop.

## Ownership

| Path | Owns |
| --- | --- |
| `windows/admx/DMSDesktop.admx` | Language-neutral Computer Configuration policy |
| `windows/admx/en-US/DMSDesktop.adml` | English strings and presentation for that policy |
| `../windows-entra-deployment.md` | Operator guide for manual, GPO, and Intune Entra identifier deployment |

## Local Contracts

- The ADMX target namespace is `Videoclinic.Policies.DMSDesktop` with no `<using>` dependency on Windows or other templates.
- One Computer (`class="Machine"`) policy writes only `EntraClientId` and `EntraTenantId` under `SOFTWARE\Policies\Videoclinic\DMS`.
- Ship `en-US` only. Intune custom-template import accepts one language file per ADMX.
- Do not put the ADMX tree in the NSIS installer. Releases attach `dms-desktop-admx.zip` for Central Store and Intune administrators.

## Work Guidance

- Change registry names or the policy class only together with ADR-0028, CAP-0021, and the validator expected constants.
- Keep operator Entra app-registration steps in `../entra-client-setup.md`; this tree deploys identifiers, it does not create the registration.

## Verification

- `python3 scripts/validate_admx.py docs/deployment/windows/admx/DMSDesktop.admx docs/deployment/windows/admx/en-US/DMSDesktop.adml`
- Relative links in `../windows-entra-deployment.md` resolve.

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.
