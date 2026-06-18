<#
test-fake-rtco.ps1 -- fake rtco binary for test-install.ps1.

Mirrors scripts/test-fake-rtco.sh. Simulates `rtco init --mcp
--hooks [--uninstall] --provider <list> [--all-providers] [--dry-run]`
by writing provider config files to $env:USERPROFILE in the expected
shapes. Invoked by the install.ps1 post-install hook to verify the
-WithMcp / -Provider / -Uninstall / -DryRun plumbing without requiring
a real install (network, cargo, etc.).

This is NOT a real rtco binary.
#>

[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]] $Args
)

$ErrorActionPreference = 'Stop'

# Parse args. We accept: init, --mcp, --hooks, --uninstall, --dry-run,
# --all-providers, --provider <list>, --provider=<list>.
$Providers = ''
$DoMcp = $false
$DoHooks = $false
$DoUninstall = $false
$DryRun = $false
$AllProviders = $false

for ($i = 0; $i -lt $Args.Count; $i++) {
    $a = $Args[$i]
    switch -Wildcard ($a) {
        'init' { }
        '--mcp' { $DoMcp = $true }
        '--hooks' { $DoHooks = $true }
        '--uninstall' { $DoUninstall = $true }
        '--dry-run' { $DryRun = $true }
        '--all-providers' { $AllProviders = $true }
        '--provider' {
            $i++
            if ($i -lt $Args.Count) { $Providers = $Args[$i] }
        }
        '--provider=*' {
            $Providers = $a.Substring('--provider='.Length)
        }
        default { }
    }
}

# Resolve provider list
if ([string]::IsNullOrEmpty($Providers) -or $AllProviders) {
    $Providers = 'claude cursor cline windsurf copilot opencode codex gemini amazonq warp'
}

function Write-JsonMcp {
    param(
        [string] $FilePath,
        [string] $Key,
        [hashtable] $Extra
    )
    if ($DryRun) { return }
    $dir = Split-Path -Parent $FilePath
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
    if (-not (Test-Path $FilePath)) { Set-Content -LiteralPath $FilePath -Value '{}' -Encoding UTF8 }

    try {
        $obj = Get-Content -LiteralPath $FilePath -Raw | ConvertFrom-Json
    } catch {
        $obj = $null
    }
    if ($null -eq $obj) { $obj = [pscustomobject]@{} }
    # Convert PSCustomObject -> hashtable for safe mutation
    $ht = @{}
    if ($obj.PSObject.Properties) {
        foreach ($p in $obj.PSObject.Properties) { $ht[$p.Name] = $p.Value }
    }

    if ($DoUninstall) {
        if ($ht.ContainsKey($Key) -and $ht[$Key].PSObject.Properties['rtco']) {
            $sub = @{}
            foreach ($p in $ht[$Key].PSObject.Properties) { $sub[$p.Name] = $p.Value }
            $sub.Remove('rtco')
            $ht[$Key] = [pscustomobject]$sub
        }
    } else {
        $entry = @{
            type    = 'stdio'
            command = 'rtco'
            args    = @('mcp')
        }
        if ($Extra) {
            foreach ($k in $Extra.Keys) { $entry[$k] = $Extra[$k] }
        }
        if ($ht.ContainsKey($Key) -and $ht[$Key]) {
            $sub = @{}
            foreach ($p in $ht[$Key].PSObject.Properties) { $sub[$p.Name] = $p.Value }
            $sub['rtco'] = $entry
            $ht[$Key] = [pscustomobject]$sub
        } else {
            $ht[$Key] = [pscustomobject]@{ rtco = $entry }
        }
    }

    ($ht | ConvertTo-Json -Depth 10) | Set-Content -LiteralPath $FilePath -Encoding UTF8
}

function Write-Provider {
    param([string] $Name)
    switch ($Name) {
        'claude'   { Write-JsonMcp -FilePath (Join-Path $env:USERPROFILE '.claude.json') -Key 'mcpServers' }
        'cursor'   { Write-JsonMcp -FilePath (Join-Path $env:USERPROFILE '.cursor\mcp.json') -Key 'mcpServers' }
        'gemini'   { Write-JsonMcp -FilePath (Join-Path $env:USERPROFILE '.gemini\settings.json') -Key 'mcpServers' -Extra @{ trust = $true } }
        'copilot'  { Write-JsonMcp -FilePath (Join-Path $env:USERPROFILE '.config\Code\User\settings.json') -Key 'mcpServers' }
        'cline'    { Write-JsonMcp -FilePath (Join-Path $env:USERPROFILE '.config\Code\User\globalStorage\saoudrizwan.claude-dev\settings\cline_mcp_settings.json') -Key 'mcpServers' }
        'windsurf' { Write-JsonMcp -FilePath (Join-Path $env:USERPROFILE '.codeium\windsurf\mcp_config.json') -Key 'mcpServers' }
        'opencode' { Write-JsonMcp -FilePath (Join-Path $env:USERPROFILE '.config\opencode\config.json') -Key 'mcp' }
        'amazonq'  { Write-JsonMcp -FilePath (Join-Path $env:USERPROFILE '.aws\amazonq\mcp.json') -Key 'mcpServers' }
        'warp'     { Write-JsonMcp -FilePath (Join-Path $env:USERPROFILE '.warp\mcp_config.json') -Key 'mcpServers' }
        'codex' {
            if ($DryRun) { return }
            $file = Join-Path $env:USERPROFILE '.codex\config.toml'
            $dir = Split-Path -Parent $file
            if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
            if ($DoUninstall) { return }
            @"

[mcp_servers.rtco]
type = "stdio"
command = "rtco"
args = ["mcp"]
"@ | Add-Content -LiteralPath $file
        }
    }
}

# Split providers on comma or whitespace, then write each
$plist = @()
foreach ($p in ($Providers -split '[,\s]+')) {
    $t = $p.Trim()
    if ($t) { $plist += $t }
}

foreach ($p in $plist) {
    Write-Provider -Name $p
}

exit 0
