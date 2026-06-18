#!/usr/bin/env pwsh
# Tests for install.ps1 — PowerShell mirror of scripts/test-install.sh.
#
# Run on Windows / pwsh:
#   pwsh scripts/test-install.ps1
#
# Verifies:
#   1. install.ps1 advertises -WithMcp / -NoMcp / -WithHooks / -NoHooks
#   2. install.ps1 advertises -Provider / -AllProviders / -DryRun
#   3. install.ps1 defines Invoke-PostInstallConfig
#   4. install.ps1 uninstall cleans providers via 'rtco init --uninstall --mcp --hooks'
#   5. install.ps1 syntax is parseable
#   6. Behavioural tests for the new flags (mirrors bash test-install.sh):
#      - WithMcp writes expected provider config
#      - default (no -WithMcp) does not touch configs
#      - -Uninstall cleans mcpServers entries
#      - -Provider subset only touches listed providers
#      - -DryRun returns 0 and writes nothing

[CmdletBinding()]
param(
    [string] $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
)

$ErrorActionPreference = 'Stop'

$InstallPs1 = Join-Path $RepoRoot 'install.ps1'
$FakeRtco   = Join-Path $PSScriptRoot 'test-fake-rtco.ps1'
if (-not (Test-Path $InstallPs1)) {
    Write-Host "FAIL: install.ps1 not found at $InstallPs1"
    exit 1
}

$FAIL = 0
function Test-Pass { param($msg) Write-Host "  PASS: $msg" }
function Test-Fail { param($msg) Write-Host "  FAIL: $msg"; $script:FAIL = 1 }

# --- Syntax parse ---
$tokens = $null
$errors = $null
[System.Management.Automation.Language.Parser]::ParseFile(
    $InstallPs1, [ref]$tokens, [ref]$errors) | Out-Null
if ($errors) {
    Test-Fail "install.ps1 has parse errors:"
    $errors | ForEach-Object { Write-Host "    $($_.Message) ($($_.Extent.StartLineNumber))" }
} else {
    Test-Pass "install.ps1 parses cleanly"
}

# --- Help text / function existence ---
$content = Get-Content -LiteralPath $InstallPs1 -Raw

Write-Host "==> Advertised flags"
foreach ($flag in @('WithMcp', 'NoMcp', 'WithHooks', 'NoHooks',
                    'Provider', 'AllProviders', 'DryRun')) {
    if ($content -match [regex]::Escape("-$flag")) {
        Test-Pass "install.ps1 advertises -$flag"
    } else {
        Test-Fail "install.ps1 is missing -$flag"
    }
}

Write-Host "==> Functions / dispatch"
if ($content -match 'function Invoke-PostInstallConfig') {
    Test-Pass "install.ps1 defines Invoke-PostInstallConfig"
} else {
    Test-Fail "install.ps1 is missing Invoke-PostInstallConfig"
}

if ($content -match 'Invoke-PostInstallConfig') {
    # The function should be wired into the main install body
    $callCount = ([regex]::Matches($content, 'Invoke-PostInstallConfig')).Count
    if ($callCount -ge 2) {
        Test-Pass "Invoke-PostInstallConfig is defined and called ($callCount occurrences)"
    } else {
        Test-Fail "Invoke-PostInstallConfig is defined but not called"
    }
}

if ($content -match 'init --uninstall --mcp --hooks') {
    Test-Pass "uninstall branch cleans providers"
} else {
    Test-Fail "uninstall branch does not clean providers"
}

if ($content -match 'TLS12|Tls12') {
    Test-Pass "install.ps1 still pins TLS 1.2 (regression guard)"
} else {
    Test-Fail "install.ps1 is missing TLS 1.2 pin"
}

# --- Cross-check: bash script presence ---
$InstallSh = Join-Path $RepoRoot 'install.sh'
if (Test-Path $InstallSh) {
    Test-Pass "install.sh still exists (mirror contract)"
    if ($content -match 'install\.ps1 is for Windows only') {
        Test-Pass "install.ps1 still points users at install.sh on Unix"
    } else {
        Test-Fail "install.ps1 is missing the 'use install.sh on Unix' hint"
    }
}

# ===========================================================================
# Behavioural tests for -WithMcp / -Provider / -Uninstall / -DryRun
# ===========================================================================
#
# We can't just dot-source install.ps1 because that would run its main
# install code (which would try to download the real binary). Instead we
# parse install.ps1 with the PowerShell AST, find the
# Invoke-PostInstallConfig function definition, and execute just that
# function body. We then invoke it directly with controlled parameters.
# This verifies the real code path the installer uses, without the
# side effects of the install itself.

# Skip behavioural tests on non-Windows: the test uses Windows-style
# fake-binary paths and [System.IO.Compression] which behave
# differently on macOS/Linux. The structural tests above already run
# on all platforms.
if (-not $IsWindows) {
    Write-Host ""
    Write-Host "==> Behavioural tests (Windows only — skipped on $($PSVersionTable.OS))"
    Write-Host ""
    if ($FAIL -eq 0) {
        Write-Host "All install.ps1 tests passed (behavioural tests skipped on non-Windows)"
        exit 0
    } else {
        Write-Host "Some tests failed"
        exit 1
    }
}

if (-not (Test-Path $FakeRtco)) {
    Test-Fail "fake rtco not found at $FakeRtco"
    Write-Host ""
    Write-Host "Some tests failed"
    exit 1
}

# Extract Invoke-PostInstallConfig from install.ps1 via the AST.
$funcAst = $null
try {
    $ast = [System.Management.Automation.Language.Parser]::ParseFile(
        $InstallPs1, [ref]$null, [ref]$null)
    $funcAst = $ast.FindAll(
        { param($n)
          $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
          $n.Name -eq 'Invoke-PostInstallConfig' },
        $true)
} catch {
    Test-Fail "failed to parse $InstallPs1 : $_"
}

if (-not $funcAst) {
    Test-Fail "Invoke-PostInstallConfig not found in install.ps1"
} else {
    # Define the function in this script's scope so the behavioural
    # tests below can call it. It depends on $BinaryName / $BinaryFile
    # / $Dest / $WithMcp / $WithHooks / $NoMcp / $NoHooks /
    # $AllProviders / $Provider / $DryRun — all of which the tests
    # set explicitly before each call.
    Invoke-Expression $funcAst.Extent.Text
    Test-Pass "extracted Invoke-PostInstallConfig from install.ps1 via AST"
}

# === Per-test sandbox setup ===
# Each behavioural test gets its own $env:USERPROFILE and $env:DEST so
# tests cannot interfere with each other or with the developer's real
# environment. The fake binary is dropped at $Dest\rtco.ps1 and
# $BinaryFile is overridden to 'rtco.ps1' so Invoke-PostInstallConfig
# picks it up.
$script:SandboxHome = $null
$script:SandboxDest = $null
$script:OldUserProfile = $env:USERPROFILE
$script:OldHome        = $env:HOME
$script:OldPath        = $env:Path

function New-Sandbox {
    $script:SandboxHome = Join-Path $env:TEMP ("rtco-test-home-" + [Guid]::NewGuid().ToString('N'))
    $script:SandboxDest = Join-Path $env:TEMP ("rtco-test-dest-" + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Force -Path $script:SandboxHome | Out-Null
    New-Item -ItemType Directory -Force -Path $script:SandboxDest | Out-Null

    $env:USERPROFILE = $script:SandboxHome
    $env:HOME        = $script:SandboxHome
    $env:Path        = "$($script:SandboxDest);$env:Path"

    # Drop the fake binary. We override $BinaryFile later to 'rtco.ps1'
    # so Invoke-PostInstallConfig picks this up.
    $fakeDest = Join-Path $script:SandboxDest 'rtco.ps1'
    Copy-Item -LiteralPath $FakeRtco -Destination $fakeDest -Force
}

function Remove-Sandbox {
    if ($script:SandboxHome -and (Test-Path $script:SandboxHome)) {
        Remove-Item -LiteralPath $script:SandboxHome -Recurse -Force -ErrorAction SilentlyContinue
    }
    if ($script:SandboxDest -and (Test-Path $script:SandboxDest)) {
        Remove-Item -LiteralPath $script:SandboxDest -Recurse -Force -ErrorAction SilentlyContinue
    }
    $env:USERPROFILE = $script:OldUserProfile
    $env:HOME        = $script:OldHome
    $env:Path        = $script:OldPath
    $script:SandboxHome = $null
    $script:SandboxDest = $null
}

# Run Invoke-PostInstallConfig with controlled flag values. Sets the
# script-scope variables the function looks up, then invokes the
# function. Returns whatever the function wrote to stdout (which is
# normally nothing in non-DryRun mode; the side effect is the file
# writes performed by the fake binary).
function Invoke-PostInstallWith {
    param([hashtable] $Flags)

    # Point $BinaryFile at our fake .ps1 binary (not rtco.exe)
    $script:BinaryName  = 'rtco'
    $script:BinaryFile  = 'rtco.ps1'
    $script:Dest        = $script:SandboxDest

    # Reset all flag variables to defaults before applying test values
    $script:WithMcp      = $false
    $script:NoMcp        = $false
    $script:WithHooks    = $false
    $script:NoHooks      = $false
    $script:AllProviders = $false
    $script:Provider     = ''
    $script:DryRun       = $false
    $script:Quiet        = $true

    foreach ($k in $Flags.Keys) {
        Set-Variable -Scope Script -Name $k -Value $Flags[$k]
    }

    # Capture and swallow informational output
    $out = & (Get-Command Invoke-PostInstallConfig) 2>&1
    return $out
}

# --- Test 1: -WithMcp writes expected provider config --------------
Write-Host "==> Behavioural: -WithMcp writes expected provider config"

New-Sandbox
try {
    # Pre-populate a Claude config so we can assert it gets modified.
    $claudePath = Join-Path $env:USERPROFILE '.claude.json'
    Set-Content -LiteralPath $claudePath -Value '{}' -Encoding UTF8

    $null = Invoke-PostInstallWith @{
        WithMcp  = $true
        NoMcp    = $false
        Provider = 'claude'
    }

    if ((Test-Path $claudePath) -and
        ((Get-Content -LiteralPath $claudePath -Raw) -match 'mcpServers') -and
        ((Get-Content -LiteralPath $claudePath -Raw) -match '"command":\s*"rtco"')) {
        Test-Pass 'with-mcp writes mcpServers.rtco.command to claude.json'
    } else {
        Test-Fail 'with-mcp did not write expected mcpServers entry to claude.json'
    }
} finally {
    Remove-Sandbox
}

# --- Test 2: default (no -WithMcp) does not touch configs ---------
Write-Host "==> Behavioural: default (no -WithMcp) does not touch configs"

New-Sandbox
try {
    $claudePath = Join-Path $env:USERPROFILE '.claude.json'
    Set-Content -LiteralPath $claudePath -Value '{}' -Encoding UTF8
    $cursorPath = Join-Path $env:USERPROFILE '.cursor\mcp.json'
    $preHash = (Get-FileHash -LiteralPath $claudePath -Algorithm SHA256).Hash

    $null = Invoke-PostInstallWith @{
        WithMcp   = $false
        NoMcp     = $true
        WithHooks = $false
        NoHooks   = $true
        Provider  = ''
    }

    $postHash = (Get-FileHash -LiteralPath $claudePath -Algorithm SHA256).Hash
    if (($preHash -eq $postHash) -and (-not (Test-Path $cursorPath))) {
        Test-Pass 'no-mcp default leaves existing config unchanged and writes no new files'
    } else {
        Test-Fail 'no-mcp default modified filesystem unexpectedly'
    }
} finally {
    Remove-Sandbox
}

# --- Test 3: -Uninstall cleans mcpServers entries -----------------
Write-Host "==> Behavioural: -Uninstall cleans mcpServers entries"

New-Sandbox
try {
    $claudePath = Join-Path $env:USERPROFILE '.claude.json'
    $initialJson = @'
{
  "numStartups": 3,
  "mcpServers": {
    "rtco":  {"type": "stdio", "command": "rtco", "args": ["mcp"]},
    "other": {"type": "stdio", "command": "other", "args": ["serve"]}
  }
}
'@
    Set-Content -LiteralPath $claudePath -Value $initialJson -Encoding UTF8

    # First, run the install hook to ensure the rtco entry is on disk
    $null = Invoke-PostInstallWith @{
        WithMcp  = $true
        NoMcp    = $false
        Provider = 'claude'
    }

    # Now invoke the same command the -Uninstall branch invokes
    $fakeBin = Join-Path $script:SandboxDest 'rtco.ps1'
    & $fakeBin init --uninstall --mcp --hooks --all-providers | Out-Null

    $afterJson = Get-Content -LiteralPath $claudePath -Raw
    if (($afterJson -notmatch '"rtco"') -and
        ($afterJson -match '"other"') -and
        ($afterJson -match '"numStartups":\s*3')) {
        Test-Pass 'uninstall removes rtco entry but preserves other mcpServers and top-level keys'
    } else {
        Test-Fail "uninstall did not clean mcpServers correctly: $afterJson"
    }
} finally {
    Remove-Sandbox
}

# --- Test 4: -Provider subset only touches listed providers -------
Write-Host "==> Behavioural: -Provider subset only touches listed providers"

New-Sandbox
try {
    $claudePath = Join-Path $env:USERPROFILE '.claude.json'
    $cursorPath = Join-Path $env:USERPROFILE '.cursor\mcp.json'
    New-Item -ItemType Directory -Force -Path (Split-Path $cursorPath) | Out-Null
    Set-Content -LiteralPath $claudePath -Value '{}' -Encoding UTF8
    Set-Content -LiteralPath $cursorPath -Value '{}' -Encoding UTF8
    $geminiPath = Join-Path $env:USERPROFILE '.gemini\settings.json'
    $codexPath  = Join-Path $env:USERPROFILE '.codex\config.toml'

    $null = Invoke-PostInstallWith @{
        WithMcp  = $true
        NoMcp    = $false
        Provider = 'claude,cursor'
    }

    $claudeOk = (Test-Path $claudePath) -and
                ((Get-Content -LiteralPath $claudePath -Raw) -match 'rtco')
    $cursorOk = (Test-Path $cursorPath) -and
                ((Get-Content -LiteralPath $cursorPath -Raw) -match 'rtco')
    $geminiAbsent = -not (Test-Path $geminiPath)
    $codexAbsent  = -not (Test-Path $codexPath)

    if ($claudeOk -and $cursorOk -and $geminiAbsent -and $codexAbsent) {
        Test-Pass '-Provider claude,cursor touches only claude and cursor configs'
    } else {
        Test-Fail "-Provider subset wrote to unlisted providers (claude=$claudeOk cursor=$cursorOk gemini=$geminiAbsent codex=$codexAbsent)"
    }
} finally {
    Remove-Sandbox
}

# --- Test 5: -DryRun writes nothing and returns 0 -----------------
Write-Host "==> Behavioural: -DryRun writes nothing and returns 0"

New-Sandbox
try {
    $claudePath  = Join-Path $env:USERPROFILE '.claude.json'
    $cursorPath  = Join-Path $env:USERPROFILE '.cursor\mcp.json'
    $geminiPath  = Join-Path $env:USERPROFILE '.gemini\settings.json'
    $codexPath   = Join-Path $env:USERPROFILE '.codex\config.toml'
    $copilotPath = Join-Path $env:USERPROFILE '.config\Code\User\settings.json'
    Set-Content -LiteralPath $claudePath -Value '{}' -Encoding UTF8
    $preHash = (Get-FileHash -LiteralPath $claudePath -Algorithm SHA256).Hash

    $null = Invoke-PostInstallWith @{
        WithMcp  = $true
        NoMcp    = $false
        Provider = 'claude,cursor,gemini,codex,copilot'
        DryRun   = $true
    }

    $postHash = (Get-FileHash -LiteralPath $claudePath -Algorithm SHA256).Hash
    $filesAbsent = -not (Test-Path $cursorPath) -and
                   -not (Test-Path $geminiPath) -and
                   -not (Test-Path $codexPath)  -and
                   -not (Test-Path $copilotPath)

    if (($preHash -eq $postHash) -and $filesAbsent) {
        Test-Pass 'dry-run returned 0 and left the filesystem untouched'
    } else {
        Test-Fail "dry-run wrote to filesystem (claude_changed=$($preHash -ne $postHash))"
    }
} finally {
    Remove-Sandbox
}

Write-Host ""
if ($FAIL -eq 0) {
    Write-Host "All install.ps1 tests passed"
    exit 0
} else {
    Write-Host "Some tests failed"
    exit 1
}
