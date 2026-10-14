#requires -Version 5.1
<#
.SYNOPSIS
Native-Windows, read-only-input verification of the already-built DMS converter.
.DESCRIPTION
Requires installed Word, the MSVC-built target\debug\dms.exe and native Windows
uv (https://docs.astral.sh/uv/getting-started/installation/). The pinned Python
verifier dependencies are isolated by uv; no packages are added to the product.
Build first with: cargo build -p dms-cli --locked

OutputRoot must NOT exist (even empty). Its parent must already exist. Also
reserves fresh sibling directories suffixed -repeat and -evidence. Never deletes
or overwrites an existing tree. Runs two full conversions, an overwrite-refusal
probe and a synthetic missing-image failure; never changes real source files.

CLI diagnostics/configuration/source bodies are captured, not echoed. Sanitized
JSON evidence is written under OutputRoot-evidence; rendered PNGs there are as
confidential as the source PDFs. Exit 0 means automated checks passed, NOT visual
G4 sign-off. Inspect the rendered pages (cover/TOC/footer/tables/images), record
layout and heading-numbering findings in CHG-0053 before marking G4 done.
.PARAMETER DmsPath
Explicit already-built executable; defaults to this worktree's debug build.
.PARAMETER PolicyManifest
Operator-reviewed UTF-8 JSON array of exact immediate policy filenames. Must
match CLI discovery and ExpectedCount; extra Markdown files stop before output.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string]$WorkspaceConfig,
    [Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string]$InputRoot,
    [Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string]$OutputRoot,
    [Parameter(Mandatory = $true)][ValidateRange(1, 2147483647)][int]$ExpectedCount,
    [Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string]$PolicyManifest,
    [string]$DmsPath = (Join-Path $PSScriptRoot '..\target\debug\dms.exe')
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
try {
    if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
        throw 'Run from native Windows PowerShell; no Linux/WSL Office fallback exists.'
    }
    if (-not (Test-Path -LiteralPath $DmsPath -PathType Leaf)) {
        throw 'Built CLI missing. In this worktree run native Windows: cargo build -p dms-cli --locked'
    }
    if (-not [type]::GetTypeFromProgID('Word.Application')) {
        throw 'Microsoft Word COM is unavailable. Install/activate desktop Word on this Windows host.'
    }
    $uv = Get-Command uv.exe -CommandType Application -ErrorAction SilentlyContinue
    if (-not $uv) {
        throw 'Install native Windows uv from https://docs.astral.sh/uv/getting-started/installation/ and reopen PowerShell.'
    }
    $arguments = @('run', '--quiet', '--no-project', '--script',
        (Join-Path $PSScriptRoot 'verify_markdown_batch.py'),
        '--workspace-config', $WorkspaceConfig, '--input-root', $InputRoot,
        '--output-root', $OutputRoot, '--expected-count', $ExpectedCount.ToString(),
        '--policy-manifest', $PolicyManifest,
        '--dms', (Resolve-Path -LiteralPath $DmsPath).ProviderPath)
    # Argument array, never Invoke-Expression or a command-string shell.
    & $uv.Source @arguments
    exit $LASTEXITCODE
} catch {
    # Only the static prerequisite messages above are safe to disclose.
    $safe = @('Run from native Windows', 'Built CLI missing.',
        'Microsoft Word COM is unavailable.', 'Install native Windows uv')
    $message = $_.Exception.Message
    if (@($safe | Where-Object { $message.StartsWith($_) }).Count -eq 0) {
        $message = 'Harness startup failed; check executable and verifier access. No configuration contents logged.'
    }
    [Console]::Error.WriteLine($message)
    exit 1
}
