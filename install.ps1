<#
install.ps1 -- one-shot installer for rtco on Windows.

Usage:
  irm "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.ps1" | iex

To pin a version or pass flags through `irm | iex`:
  & ([scriptblock]::Create((irm 'https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.ps1'))) -Version v0.1.0 -EasyMode

Or download once and run directly:
  irm "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.ps1" -OutFile install.ps1
  .\install.ps1 -Version v0.1.0 -EasyMode -Verify

Flags:
  -Dest <path>       Install location. Default: $env:USERPROFILE\.local\bin
  -System            Shortcut for -Dest "$env:ProgramFiles\rtco" (admin)
  -Version vX.Y.Z    Pin a specific release. Default: latest
  -EasyMode          Append the install dir to the *user* PATH if missing
  -Verify            Run `rtco --version` after install
  -Quiet             Suppress info logs
  -Uninstall         Remove the binary and any easy-mode PATH entry
                     (also calls `rtco init --uninstall --mcp --hooks` if the binary is on PATH)
  -WithMcp           After install, register the rtco MCP server in every detected provider
                     (claude, cursor, cline, windsurf, copilot, opencode, codex, gemini, amazonq, warp)
  -NoMcp             Skip the MCP auto-config step
  -WithHooks         After install, register rtco hooks in every detected provider
  -NoHooks           Skip the hooks auto-config step
  -Provider <list>   Comma-separated provider list, e.g. claude,cursor
  -AllProviders      Probe every known provider regardless of -Provider
  -DryRun            Print the actions that would be taken; do not modify provider configs
  -Help              Show this help and exit

Example:
  irm https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.ps1 | iex
  # pipe the above into:
  #   -WithMcp -WithHooks -AllProviders
#>

[CmdletBinding()]
param(
    [string] $Dest    = "$env:USERPROFILE\.local\bin",
    [switch] $System,
    [string] $Version = "",
    [switch] $EasyMode,
    [switch] $Verify,
    [switch] $Quiet,
    [switch] $Uninstall,
    [switch] $WithMcp,
    [switch] $NoMcp,
    [switch] $WithHooks,
    [switch] $NoHooks,
    [string] $Provider = "",
    [switch] $AllProviders,
    [switch] $DryRun,
    [switch] $Help
)

$ErrorActionPreference = 'Stop'
# Disable the slow IE-style progress bar in Invoke-WebRequest, which can
# turn a 2-second download into several minutes on Windows PowerShell.
$ProgressPreference    = 'SilentlyContinue'

# Force TLS 1.2 (and 1.3 if available). Windows PowerShell 5.1 still
# defaults to TLS 1.0/1.1 for .NET HTTP clients, which GitHub releases
# / api.github.com now reject -- surfaces as "The request was aborted:
# The connection was closed unexpectedly." The -bor preserves any newer
# protocols the runtime already has enabled.
try {
    [Net.ServicePointManager]::SecurityProtocol =
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
} catch { }

$BinaryName = 'rtco'
$BinaryFile = "$BinaryName.exe"
$Owner      = 'quangdang46'
$Repo       = 'rust_token_cost_optimizer'
if ($System) { $Dest = "$env:ProgramFiles\$BinaryName" }

function Write-Info { param($msg) if (-not $Quiet) { Write-Host "==> [$BinaryName] $msg" -ForegroundColor Cyan } }
function Write-Warn { param($msg) Write-Host "!! [$BinaryName] $msg" -ForegroundColor Yellow }
function Write-Ok   { param($msg) if (-not $Quiet) { Write-Host "[OK] $msg" -ForegroundColor Green } }
function Die        { param($msg) Write-Host "ERROR: $msg" -ForegroundColor Red; exit 1 }

if ($Help) {
    $self = $MyInvocation.MyCommand.Path
    if (-not $self) { $self = $PSCommandPath }
    if ($self -and (Test-Path $self)) {
        $content = Get-Content -Raw $self
        if ($content -match '(?s)<#(.*?)#>') { Write-Host $matches[1].Trim() }
    } else {
        Write-Host "$BinaryName installer for Windows. Re-run with -Help on a downloaded copy for full text."
    }
    exit 0
}

function Get-Platform {
    if ($IsLinux -or $IsMacOS) {
        Die "install.ps1 is for Windows only. On Linux / macOS use install.sh:`n  curl -fsSL https://raw.githubusercontent.com/$Owner/$Repo/master/install.sh | bash"
    }
    $arch = $env:PROCESSOR_ARCHITECTURE
    # WOW64 reports x86 even on a 64-bit OS when the host PowerShell is 32-bit;
    # PROCESSOR_ARCHITEW6432 reflects the real OS bitness in that case.
    if ($env:PROCESSOR_ARCHITEW6432) { $arch = $env:PROCESSOR_ARCHITEW6432 }
    switch -Wildcard ($arch) {
        'AMD64'  { return 'windows-x86_64' }
        'x86_64' { return 'windows-x86_64' }
        'ARM64'  { Die "Windows on ARM64 isn't published yet. Build from source via cargo." }
        default  { Die "unsupported architecture: $arch" }
    }
}

function Invoke-Uninstall {
    $target = Join-Path $Dest $BinaryFile
    # Best-effort: clean MCP entries and hooks before removing the binary.
    # Failures here are warnings, not fatal.
    $bin = Get-Command $BinaryName -ErrorAction SilentlyContinue
    if ($bin) {
        Write-Info "Cleaning MCP/hooks for known providers…"
        try {
            & $bin init --uninstall --mcp --hooks --all-providers | Out-Null
        } catch {
            Write-Warn "Could not clean provider configs: $_"
        }
    } else {
        Write-Warn "$BinaryName not on PATH; skipping provider cleanup"
    }
    if (Test-Path $target) {
        Remove-Item -LiteralPath $target -Force
        Write-Ok "removed $target"
    } else {
        Write-Warn "no binary at $target"
    }
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($userPath -and (($userPath -split ';') -contains $Dest)) {
        $entries = $userPath -split ';' | Where-Object { $_ -and ($_ -ne $Dest) }
        [Environment]::SetEnvironmentVariable('Path', ($entries -join ';'), 'User')
        Write-Ok "removed $Dest from user PATH"
    }
    Write-Ok "uninstalled"
    exit 0
}
if ($Uninstall) { Invoke-Uninstall }

# === Post-install configuration ===
# Build the `rtco init` arg vector from -WithMcp / -WithHooks /
# -Provider / -AllProviders / -DryRun flags, then invoke the
# just-installed binary. Best-effort.
function Invoke-PostInstallConfig {
    # Decide whether to run. Default to opt-out unless explicit.
    $doMcp = [bool] $WithMcp
    $doHooks = [bool] $WithHooks
    if (-not $WithMcp -and -not $NoMcp -and -not $AllProviders -and [string]::IsNullOrEmpty($Provider)) {
        $doMcp = $false
    }
    if (-not $WithHooks -and -not $NoHooks -and -not $AllProviders -and [string]::IsNullOrEmpty($Provider)) {
        $doHooks = $false
    }
    if (-not $doMcp -and -not $doHooks) { return }

    $bin = Join-Path $Dest $BinaryFile
    if (-not (Test-Path $bin)) {
        Write-Warn "$bin not found; skipping post-install configuration"
        return
    }

    $args = @('init')
    if ($doMcp)    { $args += '--mcp' }
    if ($doHooks)  { $args += '--hooks' }
    if (-not [string]::IsNullOrEmpty($Provider)) { $args += @('--provider', $Provider) }
    if ($AllProviders) { $args += '--all-providers' }
    if ($DryRun)        { $args += '--dry-run' }

    Write-Info "Configuring providers: rtco $($args -join ' ')"
    if ($DryRun) { return }
    try {
        & $bin @args | Out-Host
    } catch {
        Write-Warn "Provider configuration returned non-zero: $_"
        Write-Warn "Retry manually with: rtco init --mcp"
    }
}

function Resolve-Version {
    if ($script:Version) {
        if (-not $script:Version.StartsWith('v')) { $script:Version = "v$script:Version" }
        return
    }
    try {
        $resp = Invoke-RestMethod -Uri "https://api.github.com/repos/$Owner/$Repo/releases/latest" `
            -Headers @{ 'Accept' = 'application/vnd.github.v3+json' } -TimeoutSec 30
        if ($resp.tag_name) {
            $script:Version = $resp.tag_name
            Write-Info "latest: $script:Version"
            return
        }
    } catch {
        Write-Warn "GitHub API request failed; falling back to redirect probe ($($_.Exception.Message))"
    }
    try {
        $resp = Invoke-WebRequest -Uri "https://github.com/$Owner/$Repo/releases/latest" `
            -MaximumRedirection 0 -UseBasicParsing -ErrorAction SilentlyContinue
        $loc  = $resp.Headers.Location
        if ($loc -and $loc -match '/tag/(v[0-9][^/?#]*)') {
            $script:Version = $matches[1]
            return
        }
    } catch { }
    Die "could not resolve latest version. Pass -Version vX.Y.Z to pin."
}

function Get-FileWithRetry {
    param(
        [Parameter(Mandatory)] [string] $Url,
        [Parameter(Mandatory)] [string] $OutPath,
        [int] $MaxRetries = 3,
        [int] $TimeoutSec = 120
    )
    for ($attempt = 1; $attempt -le $MaxRetries; $attempt++) {
        try {
            Invoke-WebRequest -Uri $Url -OutFile $OutPath -TimeoutSec $TimeoutSec -UseBasicParsing
            return $true
        } catch {
            if ($attempt -lt $MaxRetries) {
                Write-Warn "download attempt $attempt failed; retrying in 3s..."
                Start-Sleep -Seconds 3
            } else {
                Write-Warn "download failed: $($_.Exception.Message)"
                return $false
            }
        }
    }
    return $false
}

function Update-UserPath {
    $current = $env:Path -split ';'
    if ($current -contains $Dest) { return }
    if ($EasyMode) {
        $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        $entries  = if ($userPath) { $userPath -split ';' } else { @() }
        if ($entries -notcontains $Dest) {
            $newPath = (($entries + $Dest) | Where-Object { $_ }) -join ';'
            [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
            Write-Ok "added $Dest to user PATH"
            Write-Warn "open a new PowerShell window for the change to take effect."
        }
    } else {
        Write-Warn "$Dest is not on your PATH. Either:"
        Write-Warn "  - rerun with -EasyMode to add it permanently, or"
        Write-Warn "  - prepend manually:  `$env:Path = '$Dest;' + `$env:Path"
    }
}

function Install-BinaryAtomic {
    param([string] $SourcePath, [string] $DestPath)
    $tmp = "$DestPath.tmp.$PID"
    Copy-Item -LiteralPath $SourcePath -Destination $tmp -Force
    try {
        Move-Item -LiteralPath $tmp -Destination $DestPath -Force
    } catch {
        Remove-Item -LiteralPath $tmp -ErrorAction SilentlyContinue
        Die "failed to write $DestPath ($($_.Exception.Message))"
    }
}

$tempDir = Join-Path $env:TEMP "$BinaryName-install-$PID"
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null
try {
    if (-not (Test-Path $Dest)) { New-Item -ItemType Directory -Force -Path $Dest | Out-Null }
    $platform = Get-Platform
    Write-Info "platform: $platform | dest: $Dest"
    Resolve-Version

    $archive     = "$BinaryName-$Version-$platform.zip"
    $base        = "https://github.com/$Owner/$Repo/releases/download/$Version"
    $archivePath = Join-Path $tempDir $archive

    Write-Info "downloading $archive"
    if (-not (Get-FileWithRetry -Url "$base/$archive" -OutPath $archivePath)) {
        Die "failed to download $archive -- pin a release that exists or build from source."
    }

    # Tolerant sidecar parser: accepts either a bare hash or
    # `<hash>  <filename>` (GNU sha256sum -c format).
    $sumPath = "$archivePath.sha256"
    if (Get-FileWithRetry -Url "$base/$archive.sha256" -OutPath $sumPath -MaxRetries 1 -TimeoutSec 30) {
        $expected = (Get-Content -LiteralPath $sumPath -Raw).Trim().Split()[0]
        $actual   = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLower()
        if ($expected.ToLower() -ne $actual) {
            Die "checksum mismatch for $archive`n  expected: $expected`n  actual:   $actual"
        }
        Write-Info "checksum verified"
    } else {
        Write-Warn "no checksum sidecar -- skipping verification"
    }

    $extractDir = Join-Path $tempDir 'extract'
    Expand-Archive -LiteralPath $archivePath -DestinationPath $extractDir -Force
    $bin = Get-ChildItem -LiteralPath $extractDir -Recurse -Filter $BinaryFile -File | Select-Object -First 1
    if (-not $bin) { Die "$BinaryFile not found inside $archive" }

    Install-BinaryAtomic -SourcePath $bin.FullName -DestPath (Join-Path $Dest $BinaryFile)
    Update-UserPath

    if ($Verify) { & (Join-Path $Dest $BinaryFile) --version | Out-Host }

    # Best-effort post-install provider configuration. Skipped silently
    # unless the user passed -WithMcp / -WithHooks / -AllProviders.
    Invoke-PostInstallConfig

    Write-Host ""
    Write-Host "[OK] $BinaryName installed -> $(Join-Path $Dest $BinaryFile)" -ForegroundColor Green
    try {
        $v = & (Join-Path $Dest $BinaryFile) --version 2>$null
        if ($v) { Write-Host "   version: $v" }
    } catch { }
    Write-Host "   $BinaryName --help"
} finally {
    if (Test-Path $tempDir) { Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue }
}
