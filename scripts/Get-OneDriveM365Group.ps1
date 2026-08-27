# Get-OneDriveM365Group.ps1
#
# Resolves registered OneDrive for Business sync roots, including the OneDrive
# client's cached SharePoint mount points where available, to their backing
# Microsoft 365 Group. For "Add shortcut to OneDrive", it resolves the local
# relative path through Microsoft Graph and reads the public remoteItem facet.
# Shell metadata is only a fallback.
#
# Requires Windows PowerShell 5.1 or PowerShell 7+, the Microsoft.Graph
# Authentication module, and delegated Files.Read + Group.Read.All +
# Sites.Read.All. The script opens an interactive Microsoft Graph sign-in for
# every directory under a registered OneDrive for Business root.
#
# Examples:
#   .\Get-OneDriveM365Group.ps1
#   .\Get-OneDriveM365Group.ps1 -Path 'C:\Users\Raphael\OneDrive - Contoso\Project'
#   .\Get-OneDriveM365Group.ps1 -Path 'C:\Users\Raphael\OneDrive - Contoso\Project' | Format-Table
#
# The Registry identifies local OneDrive account roots. Microsoft Graph's
# remoteItem facet is the public identity mapping for a folder shared or added
# to the user's OneDrive. The Graph lookup is the authority for whether the
# backing site has a Microsoft 365 Group. SharePoint communication sites and
# user-shared OneDrive folders have no Microsoft 365 Group and are reported as
# such. Private OneDrive client state is not read.

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [ValidateNotNullOrEmpty()]
    [string[]] $Path
)

$ErrorActionPreference = 'Stop'

function Get-NormalizedPath {
    param([Parameter(Mandatory)] [string] $Value)

    $fullPath = [System.IO.Path]::GetFullPath($Value)
    return $fullPath.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
}

function Test-PathWithinRoot {
    param(
        [Parameter(Mandatory)] [string] $Candidate,
        [Parameter(Mandatory)] [string] $Root
    )

    $normalizedCandidate = Get-NormalizedPath $Candidate
    $normalizedRoot = Get-NormalizedPath $Root
    $comparison = [System.StringComparison]::OrdinalIgnoreCase

    return $normalizedCandidate.Equals($normalizedRoot, $comparison) -or
        $normalizedCandidate.StartsWith($normalizedRoot + [System.IO.Path]::DirectorySeparatorChar, $comparison)
}

function Get-OneDriveBusinessAccounts {
    $accountsPath = 'HKCU:\Software\Microsoft\OneDrive\Accounts'
    if (-not (Test-Path -LiteralPath $accountsPath)) {
        return @()
    }

    $accounts = @()
    foreach ($accountKey in Get-ChildItem -LiteralPath $accountsPath) {
        if ($accountKey.PSChildName -notmatch '^Business\d+$') {
            continue
        }

        $properties = Get-ItemProperty -LiteralPath $accountKey.PSPath
        if ([string]::IsNullOrWhiteSpace($properties.UserFolder)) {
            continue
        }

        $accounts += [pscustomobject]@{
            Name = $accountKey.PSChildName
            UserFolder = Get-NormalizedPath $properties.UserFolder
            RegistryPath = $accountKey.PSPath
        }
    }

    return $accounts
}

function Get-CachedMountPoints {
    param([Parameter(Mandatory)] $Account)

    # ScopeIdToMountPointPathCache is OneDrive-client implementation state, not
    # a Windows API. Treat it only as an extra local discovery signal.
    $cachePath = Join-Path $Account.RegistryPath 'ScopeIdToMountPointPathCache'
    if (-not (Test-Path -LiteralPath $cachePath)) {
        return @()
    }

    $cacheKey = Get-Item -LiteralPath $cachePath
    $mountPoints = @()
    foreach ($valueName in $cacheKey.GetValueNames()) {
        $value = $cacheKey.GetValue($valueName)
        if ($value -is [string] -and (Test-Path -LiteralPath $value -PathType Container)) {
            $mountPoints += Get-NormalizedPath $value
        }
    }

    return $mountPoints
}

function Get-ShellRemoteUri {
    param([Parameter(Mandatory)] [string] $LocalPath)

    # Split-Path does not allow -LiteralPath with -Parent or -Leaf. Use the
    # filesystem object so paths with wildcard characters remain literal.
    $directory = Get-Item -LiteralPath $LocalPath -Force
    if (-not $directory.PSIsContainer -or $null -eq $directory.Parent) {
        return $null
    }

    $shell = New-Object -ComObject Shell.Application
    try {
        $folder = $shell.Namespace($directory.Parent.FullName)
        if ($null -eq $folder) {
            return $null
        }

        $item = $folder.ParseName($directory.Name)
        if ($null -eq $item) {
            return $null
        }

        $remoteUri = $item.ExtendedProperty('System.StorageProviderFileRemoteUri')
        if ($remoteUri -is [string] -and $remoteUri -match '^https://') {
            return $remoteUri
        }

        return $null
    }
    finally {
        if ($null -ne $shell) {
            [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($shell)
        }
    }
}

function Test-UriWithinSite {
    param(
        [Parameter(Mandatory)] [string] $RemoteUri,
        [Parameter(Mandatory)] [string] $SiteUri
    )

    try {
        $remote = [Uri] $RemoteUri
        $site = [Uri] $SiteUri
    }
    catch {
        return $false
    }

    if (-not $remote.Host.Equals($site.Host, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $false
    }

    $sitePath = $site.AbsolutePath.TrimEnd('/')
    $remotePath = $remote.AbsolutePath.TrimEnd('/')
    return $remotePath.Equals($sitePath, [System.StringComparison]::OrdinalIgnoreCase) -or
        $remotePath.StartsWith($sitePath + '/', [System.StringComparison]::OrdinalIgnoreCase)
}

function Assert-GraphAuthentication {
    $requiredCommands = @('Connect-MgGraph', 'Disconnect-MgGraph', 'Invoke-MgGraphRequest')
    $missingCommands = @($requiredCommands | Where-Object {
        $null -eq (Get-Command -Name $_ -ErrorAction SilentlyContinue)
    })
    if ($missingCommands.Count -eq 0) {
        return
    }

    if (-not (Get-Module -ListAvailable -Name Microsoft.Graph.Authentication)) {
        throw 'Missing Microsoft.Graph.Authentication. Install it for the current user with: Install-Module Microsoft.Graph.Authentication -Scope CurrentUser'
    }

    $loadedAuthenticationAssembly = [AppDomain]::CurrentDomain.GetAssemblies() | Where-Object {
        $_.GetName().Name -eq 'Microsoft.Graph.Authentication'
    } | Select-Object -First 1
    if ($null -ne $loadedAuthenticationAssembly) {
        throw 'The current PowerShell host has a Graph Authentication assembly loaded without the required commands. Close this host and run the script from a clean `pwsh -NoProfile` session.'
    }

    Import-Module Microsoft.Graph.Authentication -ErrorAction Stop
    $stillMissing = @($requiredCommands | Where-Object {
        $null -eq (Get-Command -Name $_ -ErrorAction SilentlyContinue)
    })
    if ($stillMissing.Count -ne 0) {
        throw "Microsoft.Graph.Authentication did not provide: $($stillMissing -join ', ')."
    }
}

function Get-UnifiedGroups {
    $groups = @()
    $uri = "https://graph.microsoft.com/v1.0/groups?`$select=id,displayName&`$filter=groupTypes/any(c:c eq 'Unified')"
    do {
        $response = Invoke-MgGraphRequest -Method GET -Uri $uri
        $groups += @($response.value)
        $uri = $response.'@odata.nextLink'
    } while (-not [string]::IsNullOrWhiteSpace($uri))

    return $groups
}

function Get-GraphShortcutUri {
    param(
        [Parameter(Mandatory)] [string] $LocalPath,
        [Parameter(Mandatory)] [string] $OneDriveRoot
    )

    $relativePath = $LocalPath.Substring($OneDriveRoot.Length).TrimStart([char[]]@('\', '/'))
    if ([string]::IsNullOrWhiteSpace($relativePath)) {
        return $null
    }

    # A child of an added shortcut is not necessarily a remoteItem itself.
    # Query it and then each ancestor up to the OneDrive root until the shortcut
    # entry exposes the backing SharePoint item.
    $segments = @($relativePath -split '[\\/]')
    for ($segmentCount = $segments.Count; $segmentCount -ge 1; $segmentCount--) {
        $encodedPath = @($segments[0..($segmentCount - 1)] | ForEach-Object {
            [Uri]::EscapeDataString($_)
        }) -join '/'
        $uri = "https://graph.microsoft.com/v1.0/me/drive/root:/$encodedPath?`$select=remoteItem"

        try {
            $item = Invoke-MgGraphRequest -Method GET -Uri $uri
        }
        catch {
            continue
        }

        if ($null -eq $item.remoteItem) {
            continue
        }

        $remoteItem = $item.remoteItem
        if ($remoteItem.sharepointIds.siteUrl) {
            return $remoteItem.sharepointIds.siteUrl
        }
        if ($remoteItem.webUrl) {
            return $remoteItem.webUrl
        }
    }

    return $null
}

$accounts = Get-OneDriveBusinessAccounts
if ($accounts.Count -eq 0) {
    throw 'No OneDrive for Business account roots were found under HKCU:\Software\Microsoft\OneDrive\Accounts.'
}

$candidates = @()
if ($Path) {
    foreach ($inputPath in $Path) {
        if (-not (Test-Path -LiteralPath $inputPath -PathType Container)) {
            Write-Warning "Skipping missing directory: $inputPath"
            continue
        }

        $candidates += [pscustomobject]@{
            LocalPath = Get-NormalizedPath $inputPath
            DiscoverySource = 'explicit-path'
        }
    }
}
else {
    foreach ($account in $accounts) {
        $candidates += [pscustomobject]@{
            LocalPath = $account.UserFolder
            DiscoverySource = 'registry-account-root'
        }

        foreach ($mountPoint in Get-CachedMountPoints $account) {
            $candidates += [pscustomobject]@{
                LocalPath = $mountPoint
                DiscoverySource = 'registry-cached-mount-point'
            }
        }
    }
}

$candidates = @($candidates | Sort-Object LocalPath -Unique)
$localResults = @()
foreach ($candidate in $candidates) {
    $account = $accounts | Where-Object { Test-PathWithinRoot -Candidate $candidate.LocalPath -Root $_.UserFolder } | Select-Object -First 1
    if ($null -eq $account) {
        $localResults += [pscustomobject]@{
            LocalPath = $candidate.LocalPath
            DiscoverySource = $candidate.DiscoverySource
            OneDriveAccount = $null
            OneDriveRoot = $null
            RemoteUri = $null
            Resolution = 'not-under-a-registered-onedrive-business-root'
            GroupId = $null
            GroupUrl = $null
            GroupDisplayName = $null
        }
        continue
    }

    $localResults += [pscustomobject]@{
        LocalPath = $candidate.LocalPath
        DiscoverySource = $candidate.DiscoverySource
        OneDriveAccount = $account.Name
        OneDriveRoot = $account.UserFolder
        RemoteUri = Get-ShellRemoteUri $candidate.LocalPath
        Resolution = 'pending-graph-path-lookup'
        GroupId = $null
        GroupUrl = $null
        GroupDisplayName = $null
    }
}

$graphCandidates = @($localResults | Where-Object { $_.Resolution -eq 'pending-graph-path-lookup' })
if ($graphCandidates.Count -eq 0) {
    $localResults
    return
}

Assert-GraphAuthentication
Connect-MgGraph -Scopes @('Files.Read', 'Group.Read.All', 'Sites.Read.All') -NoWelcome | Out-Null
try {
    $groups = Get-UnifiedGroups
    foreach ($result in $graphCandidates) {
        $graphRemoteUri = Get-GraphShortcutUri -LocalPath $result.LocalPath -OneDriveRoot $result.OneDriveRoot
        if ($graphRemoteUri) {
            $result.RemoteUri = $graphRemoteUri
        }
        if (-not $result.RemoteUri) {
            $result.Resolution = 'no-public-remote-item-mapping'
            continue
        }

        $matchedGroup = $null
        foreach ($group in $groups) {
            try {
                $site = Invoke-MgGraphRequest -Method GET -Uri ("https://graph.microsoft.com/v1.0/groups/{0}/sites/root?`$select=id,webUrl" -f $group.Id)
            }
            catch {
                continue
            }

            if (Test-UriWithinSite -RemoteUri $result.RemoteUri -SiteUri $site.webUrl) {
                $matchedGroup = $group
                break
            }
        }

        if ($null -eq $matchedGroup) {
            $result.Resolution = 'no-microsoft-365-group-for-backing-site'
            continue
        }

        $result.Resolution = 'resolved'
        $result.GroupId = $matchedGroup.Id
        $result.GroupUrl = "https://myaccount.microsoft.com/groups/$($matchedGroup.Id)"
        $result.GroupDisplayName = $matchedGroup.DisplayName
    }
}
finally {
    Disconnect-MgGraph | Out-Null
}

$localResults
