# Build-WingetKit.ps1
#
# Renders the three winget-pkgs multi-file manifests (version, installer,
# defaultLocale) for a Videoclinic.DMS release into an output directory laid
# out exactly the way the winget-pkgs repo expects, plus a one-page
# SUBMIT.md that walks the operator through `wingetcreate submit`.
#
# Why a script and not a workflow PR? The microsoft/winget-pkgs community
# repo's PR bot rejects first-submissions from unattended workflows (the
# publisher ID has to be vouched-for by a human-trusted opener) and
# auto-closes duplicate-version PRs. We do not try to script around that;
# the operator runs `wingetcreate submit` themselves. This script just makes
# the three files pain-free to generate for every release.
#
# Usage (PowerShell 5.1 or pwsh 7+):
#   pwsh scripts/winget/Build-WingetKit.ps1 `
#     -Version 0.1.0 `
#     -InstallerUrl "https://github.com/videoclinic/dms/releases/download/v0.1.0/DMS_Desktop_0.1.0_x64-setup.exe" `
#     -InstallerSha256 ed2ab70ce12ca43e6063ba450cf9241e334f20aea327c8a44bc0ed557bc91b59 `
#     -OutputDir /tmp/winget-bundle
#
# Used by .github/workflows/release-windows.yml after the GitHub Release
# publish step to produce the same bundle, plus a `winget-bundle.zip` job
# artifact the operator downloads.

[CmdletBinding()]
param(
  [Parameter(Mandatory)] [string] $Version,
  [Parameter(Mandatory)] [string] $InstallerUrl,
  [Parameter(Mandatory)] [string] $InstallerSha256,
  [Parameter(Mandatory)] [string] $OutputDir,
  [string] $Publisher = 'Videoclinic',
  [string] $PackageName = 'DMS Desktop',
  [string] $Moniker = 'dms-desktop',
  [string] $ShortDescription = 'Operator-controlled document control for ISO 27001-style workflows.',
  [string] $ReleaseNotesUrl = 'https://github.com/videoclinic/dms/releases/tag/v0.1.0',
  [string] $PublisherUrl = 'https://videoclinic.de/',
  [string] $PrivacyUrl = 'https://github.com/videoclinic/dms/blob/main/docs/privacy.md',
  [string] $License = 'MIT',
  [string] $LicenseUrl = 'https://github.com/videoclinic/dms/blob/main/LICENSE'
)

$ErrorActionPreference = 'Stop'

# Sanity checks: schema requires lowercase PackageIdentifier, a 4-32 char
# segment on each side of the dot, no spaces, no slashes, no colons.
if ($Publisher -notmatch '^[A-Za-z][A-Za-z0-9.-]{0,31}$') {
  throw "Publisher '$Publisher' is not a valid winget publisher id (^[A-Za-z][A-Za-z0-9.-]{0,31}$)."
}
if ($InstallerSha256 -notmatch '^[A-Fa-f0-9]{64}$') {
  throw "InstallerSha256 must be 64 hex chars; got '$InstallerSha256'."
}

# Manifests path under winget-pkgs: manifests/<lowercase first letter>/<Publisher>/<PackageName>/<Version>/.
# PackageName on disk drops spaces (the path component must match the
# PackageIdentifier with dots; "DMS Desktop" becomes "DMSDesktop"). The
# user-facing PackageName inside the manifest keeps the space.
$packageNameForPath = $PackageName -replace '\s+', ''
$packageId = "$Publisher.$packageNameForPath" -replace '[^A-Za-z0-9.]', ''
$firstLetter = $Publisher.ToLowerInvariant().Substring(0,1)
$manifestDir = Join-Path $OutputDir ('manifests/{0}/{1}/{2}/{3}' -f $firstLetter, $Publisher, $packageNameForPath, $Version)
$null = New-Item -ItemType Directory -Path $manifestDir -Force

# --- version file -----------------------------------------------------------
$versionFile = Join-Path $manifestDir ("$packageId.yaml")
@"
PackageIdentifier: $packageId
PackageVersion: $Version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
"@ | Set-Content -Path $versionFile -NoNewline -Encoding utf8

# --- defaultLocale file -----------------------------------------------------
$localeFile = Join-Path $manifestDir ("$packageId.locale.en-US.yaml")
@"
PackageIdentifier: $packageId
PackageVersion: $Version
PackageLocale: en-US
Publisher: $Publisher
PublisherUrl: $PublisherUrl
PrivacyUrl: $PrivacyUrl
PackageName: $PackageName
License: $License
LicenseUrl: $LicenseUrl
ShortDescription: $ShortDescription
Moniker: $Moniker
Tags:
- document-management
- iso-27001
- compliance
- tauri
ReleaseNotesUrl: $ReleaseNotesUrl
ManifestType: defaultLocale
ManifestVersion: 1.6.0
"@ | Set-Content -Path $localeFile -NoNewline -Encoding utf8

# --- installer file ---------------------------------------------------------
$installerFile = Join-Path $manifestDir ("$packageId.installer.yaml")
@"
PackageIdentifier: $packageId
PackageVersion: $Version
Platform:
- Windows.Desktop
MinimumOSVersion: 10.0.17763.0
InstallerType: nullsoft
InstallModes:
- silent
- silentWithProgress
Installers:
- Architecture: x64
  InstallerUrl: $InstallerUrl
  InstallerSha256: $InstallerSha256
ManifestType: installer
ManifestVersion: 1.6.0
"@ | Set-Content -Path $installerFile -NoNewline -Encoding utf8

# --- SUBMIT.md --------------------------------------------------------------
$submitFile = Join-Path $OutputDir 'SUBMIT.md'
@"
# Submitting $packageId v$Version to winget-pkgs

## Files in this bundle

```
manifests/v/$Publisher/$PackageName/$Version/
  $packageId.yaml                 # version
  $packageId.locale.en-US.yaml    # defaultLocale
  $packageId.installer.yaml       # installer
```

These three files match the [winget-pkgs multi-file layout](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest) for `manifests/<first-letter>/<Publisher>/<PackageName>/<Version>/`. The first letter is `$firstLetter`.

## One-time setup

1. Install wingetcreate on a Windows host:
   ```
   winget install wingetcreate
   ```
2. Fork https://github.com/microsoft/winget-pkgs to your account and clone your fork.

## Submit the PR

From the root of your winget-pkgs fork, with this bundle unpacked at `./winget-bundle/`:

```
wingetcreate submit --manifests ./winget-bundle/manifests
```

`wingetcreate` will open a PR against microsoft/winget-pkgs. The first submission for a brand-new publisher id (`$Publisher`) usually takes 1-3 days of manual review by a winget-pkgs maintainer; subsequent versions are typically merged within hours.

## Verification after merge

```
winget install --id $packageId --version $Version
dms-desktop --version
start dms://about
```

The first command should fetch the signed NSIS installer, the second should print `$Version`, and the third should open the DMS Desktop window via the registered `dms://` protocol handler (ADR-0027).
"@ | Set-Content -Path $submitFile -NoNewline -Encoding utf8

Write-Output "wrote: $versionFile"
Write-Output "wrote: $localeFile"
Write-Output "wrote: $installerFile"
Write-Output "wrote: $submitFile"
