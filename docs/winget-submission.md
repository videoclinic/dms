# winget submission

The Windows distribution is the signed NSIS installer published to GitHub Releases (ADR-0027). On top of that, the project also submits manifests to the [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) community repo so users can install with `winget install VideoClinic.DMSDesktop`.

The release workflow (`/.github/workflows/release-windows.yml`) builds the three winget-pkgs multi-file manifests (`version`, `installer`, `defaultLocale`) for every stable release and uploads them as a `winget-bundle` job artifact. The workflow does **not** open the PR itself — see "Why no auto-PR" below.

## Package identity

| Field | Value |
| --- | --- |
| `PackageIdentifier` | `Videoclinic.DMSDesktop` |
| `Publisher` | `Videoclinic` |
| `PackageName` | `DMS Desktop` (user-facing; the path component drops the space) |
| `Moniker` | `dms-desktop` |
| `License` | MIT |
| `InstallerType` | `nullsoft` (NSIS) |
| `Architecture` | `x64` only (ADR-0002, ARM64 deferred) |
| `MinimumOSVersion` | `10.0.17763.0` (Windows 10 1809) |

## Operator runbook

After the release workflow ships a release, the operator submits the winget manifests in three steps.

1. **Download the winget-bundle artifact** from the release run on GitHub Actions (it is uploaded at the end of every stable release run).
2. **Install wingetcreate** on a Windows host: `winget install wingetcreate`.
3. **Run `wingetcreate submit` from a winget-pkgs fork**:

   ```
   wingetcreate submit --manifests <unpacked-bundle>/manifests
   ```

   `wingetcreate` opens a PR against `microsoft/winget-pkgs`. The first submission for the `Videoclinic` publisher id is reviewed manually by a winget-pkgs maintainer (1-3 days); subsequent versions are typically merged within hours.

## Why no auto-PR

The release workflow deliberately does not push to `microsoft/winget-pkgs` itself:

- The community bot rejects first-submissions for an unseen publisher id from unattended workflows — the publisher has to be vouched for by a human-trusted opener.
- The bot auto-closes duplicate-version PRs, so a partial automation that sometimes fails leaves orphaned PRs the operator has to clean up.
- A winget-pkgs PR is a contractual moment (the project is asking Microsoft to ship this to all Windows users via `winget install`); making the operator the one who hits enter is the right amount of friction for that action.

The bundle is generated deterministically (the same SHA-256, the same URL, the same version) on every release run, so `wingetcreate submit` is a one-liner.

## Files

| Path | Role |
| --- | --- |
| `scripts/winget/Build-WingetKit.ps1` | Renders the three manifests + `SUBMIT.md` from `-Version`, `-InstallerUrl`, `-InstallerSha256`, `-OutputDir` |
| `docs/winget-submission.md` | This file |
| `.github/workflows/release-windows.yml` `Build winget submission kit` step | Calls the script with the values from the just-published release, uploads the bundle as a job artifact |

## Verification after winget-pkgs merge

```
winget install --id VideoClinic.DMSDesktop --version <version>
dms-desktop --version
start dms://about
```

The first command should fetch the signed NSIS installer, the second should print the version, and the third should open the DMS Desktop window via the registered `dms://` protocol handler (ADR-0027).
