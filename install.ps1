\xEF\xBB\xBF#Requires -Version 5.1
<#
.SYNOPSIS
    ParkHub interactive installer for Windows
.DESCRIPTION
    Downloads and installs ParkHub with User or System install modes.
    User install requires no admin rights. System install adds a Windows Service.
.EXAMPLE
    irm https://raw.githubusercontent.com/nash87/parkhub/main/install.ps1 | iex
#>

$ErrorActionPreference = "Stop"
$Repo = "nash87/parkhub"
$ServiceName = "ParkHub"
$Version = "2026.2.9"
$DefaultPort = 7878

function Write-Info($msg)  { Write-Host "  i  $msg" -ForegroundColor Cyan }
function Write-OK($msg)    { Write-Host "  OK  $msg" -ForegroundColor Green }
function Write-Warn($msg)  { Write-Host "  ⚠  $msg" -ForegroundColor Yellow }
function Write-Err($msg)   { Write-Host "  ✗  $msg" -ForegroundColor Red }
function Write-Step($msg)  { Write-Host "  →  $msg" -ForegroundColor Cyan }

function Test-IsAdmin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-Arch {
    if ([Environment]::Is64BitOperatingSystem) {
        if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { return "arm64" }
        return "amd64"
    }
    return "386"
}

function Get-HostIP {
    try {
        $ip = (Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.PrefixOrigin -eq "Dhcp" -or $_.PrefixOrigin -eq "Manual" } | Select-Object -First 1).IPAddress
        if ($ip) { return $ip }
    } catch {}
    return "localhost"
}

function Get-LatestVersion {
    Write-Step "Fetching latest release..."
    try {
        $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
        $ver = $release.tag_name
        Write-OK "Latest version: $ver"
        return $ver
    } catch {
        Write-Warn "Could not fetch latest version, using v$Version"
        return "v$Version"
    }
}

function Install-Binary($ver, $installDir) {
    $arch = Get-Arch
    $asset = "parkhub-windows-$arch.exe"
    $url = "https://github.com/$Repo/releases/download/$ver/$asset"

    try { New-Item -ItemType Directory -Force -Path $installDir | Out-Null } catch {
        Write-Err "Cannot create directory: $installDir"
        return $false
    }

    $target = Join-Path $installDir "parkhub-server.exe"

    Write-Step "Downloading $asset ($arch)..."
    try {
        Invoke-WebRequest -Uri $url -OutFile $target -UseBasicParsing
    } catch {
        Write-Warn "Download failed (release may not exist yet for $arch)"
        return $false
    }

    # Unblock downloaded file (Windows SmartScreen / antivirus workaround)
    try { Unblock-File -Path $target -ErrorAction SilentlyContinue } catch {}
    Write-Info "File unblocked (SmartScreen/antivirus workaround)"
    Write-OK "Installed to $target"
    return $true
}

function Add-ToUserPath($dir) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$dir*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$dir", "User")
        $env:Path += ";$dir"
        Write-OK "Added to user PATH"
    } else { Write-Info "Already in user PATH" }
}

function Add-ToSystemPath($dir) {
    $sysPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    if ($sysPath -notlike "*$dir*") {
        [Environment]::SetEnvironmentVariable("Path", "$sysPath;$dir", "Machine")
        $env:Path += ";$dir"
        Write-OK "Added to system PATH"
    } else { Write-Info "Already in system PATH" }
}

function New-UserShortcut($installDir) {
    $startMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
    try {
        $shell = New-Object -ComObject WScript.Shell
        $shortcut = $shell.CreateShortcut("$startMenu\ParkHub Server.lnk")
        $shortcut.TargetPath = Join-Path $installDir "parkhub-server.exe"
        $shortcut.WorkingDirectory = $installDir
        $shortcut.Save()
        Write-OK "User Start Menu shortcut created"
    } catch { Write-Warn "Failed to create shortcut: $_" }
}

function New-SystemShortcut($installDir) {
    $startMenu = Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\ParkHub"
    try {
        New-Item -ItemType Directory -Force -Path $startMenu | Out-Null
        $shell = New-Object -ComObject WScript.Shell
        $shortcut = $shell.CreateShortcut("$startMenu\ParkHub Server.lnk")
        $shortcut.TargetPath = Join-Path $installDir "parkhub-server.exe"
        $shortcut.WorkingDirectory = $installDir
        $shortcut.Save()
        Write-OK "System Start Menu shortcut created"
    } catch { Write-Warn "Failed to create shortcut: $_" }
}

function Install-AsService($installDir, $dataDir) {
    $exe = Join-Path $installDir "parkhub-server.exe"
    Write-Step "Installing Windows service..."
    try {
        & $exe install
        if ($LASTEXITCODE -eq 0) {
            # Set PARKHUB_DATA_DIR for the service
            $regPath = "HKLM:\SYSTEM\CurrentControlSet\Services\$ServiceName"
            if (Test-Path $regPath) {
                $envBlock = [Environment]::GetEnvironmentVariable("PARKHUB_DATA_DIR", "Machine")
                if (-not $envBlock) {
                    [Environment]::SetEnvironmentVariable("PARKHUB_DATA_DIR", $dataDir, "Machine")
                }
            }
            Write-OK "Service '$ServiceName' installed"
            Write-Step "Starting service..."
            Start-Service -Name $ServiceName -ErrorAction SilentlyContinue
            Write-OK "Service started"
        } else { Write-Warn "Service installation returned non-zero exit code" }
    } catch { Write-Warn "Service installation failed: $_" }
}

function Add-FirewallRule($port) {
    Write-Step "Adding firewall rule for port $port..."
    try {
        # Remove existing rule if any
        Remove-NetFirewallRule -DisplayName "ParkHub Server" -ErrorAction SilentlyContinue
        New-NetFirewallRule -DisplayName "ParkHub Server" -Direction Inbound -Action Allow -Protocol TCP -LocalPort $port -ErrorAction Stop | Out-Null
        Write-OK "Firewall rule added"
    } catch { Write-Warn "Failed to add firewall rule: $_" }
}

function New-Config($port, $dataDir) {
    try { New-Item -ItemType Directory -Force -Path $dataDir | Out-Null } catch {
        Write-Err "Cannot create data directory: $dataDir"
        return
    }
    $configFile = Join-Path $dataDir "config.toml"
    if (-not (Test-Path $configFile)) {
        @"
# ParkHub Configuration
[server]
port = $port
host = "0.0.0.0"

[storage]
data_dir = "$($dataDir -replace '\\', '\\\\')"

[auth]
session_timeout = 86400
"@ | Set-Content -Path $configFile -Encoding UTF8
        Write-OK "Created config at $configFile"
    } else { Write-Info "Config already exists at $configFile" }
}

function Show-Completion($port, $mode) {
    $ip = Get-HostIP
    $url = "http://${ip}:${port}"
    Write-Host ""
    Write-Host "  ═══════════════════════════════════════════" -ForegroundColor Green
    Write-Host "    OK ParkHub is ready! ($mode install)" -ForegroundColor Green
    Write-Host ""
    Write-Host "      $url" -ForegroundColor White
    Write-Host ""
    Write-Host "    Open this URL in your browser to start" -ForegroundColor Cyan
    Write-Host "    the onboarding wizard." -ForegroundColor Cyan
    Write-Host "  ═══════════════════════════════════════════" -ForegroundColor Green
    Write-Host ""
}

# ─── Mode A: User Install ───
function Invoke-UserInstall {
    Write-Host ""
    Write-Host "  > User Install (no admin required)" -ForegroundColor White
    Write-Host ""

    $arch = Get-Arch
    Write-Info "Platform: Windows/$arch"

    $installDir = Join-Path $env:LOCALAPPDATA "ParkHub"
    $dataDir    = Join-Path $env:LOCALAPPDATA "ParkHub\data"

    $ver = Get-LatestVersion
    if (-not (Install-Binary $ver $installDir)) {
        Write-Err "Installation failed."
        return
    }

    New-Config $DefaultPort $dataDir

    # Set PARKHUB_DATA_DIR for the user
    [Environment]::SetEnvironmentVariable("PARKHUB_DATA_DIR", $dataDir, "User")
    Write-OK "Set PARKHUB_DATA_DIR=$dataDir"

    Add-ToUserPath $installDir
    New-UserShortcut $installDir
    Show-Completion $DefaultPort "User"

    Write-Info "Start ParkHub manually or from the Start Menu."
    Write-Info "To start now: parkhub-server.exe --headless (from $installDir)"
}

# ─── Mode B: System Install ───
function Invoke-SystemInstall {
    Write-Host ""
    Write-Host "  >  System Install (Windows Service)" -ForegroundColor White
    Write-Host ""

    $arch = Get-Arch
    Write-Info "Platform: Windows/$arch"

    $installDir = Join-Path $env:ProgramFiles "ParkHub"
    $dataDir    = Join-Path $env:ProgramData "ParkHub"

    $ver = Get-LatestVersion
    if (-not (Install-Binary $ver $installDir)) {
        Write-Err "Installation failed."
        return
    }

    New-Config $DefaultPort $dataDir

    # Set PARKHUB_DATA_DIR system-wide
    [Environment]::SetEnvironmentVariable("PARKHUB_DATA_DIR", $dataDir, "Machine")
    Write-OK "Set PARKHUB_DATA_DIR=$dataDir (system)"

    Add-ToSystemPath $installDir
    Install-AsService $installDir $dataDir
    Add-FirewallRule $DefaultPort
    New-SystemShortcut $installDir
    Show-Completion $DefaultPort "System"
}

# ─── Main ───
Clear-Host
Write-Host ""
Write-Host "  ═══════════════════════════════════════════" -ForegroundColor Green
Write-Host "         ParkHub Installer v$Version  " -ForegroundColor Green
Write-Host "  ═══════════════════════════════════════════" -ForegroundColor Green
Write-Host ""

$isAdmin = Test-IsAdmin

if ($isAdmin) {
    Write-Host "  Running as Administrator OK" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Choose installation mode:"
    Write-Host ""
    Write-Host "    [1] User Install" -ForegroundColor White
    Write-Host "        Install to %LOCALAPPDATA%\ParkHub (current user only)" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "    [2] System Install (recommended)" -ForegroundColor White
    Write-Host "        Install to Program Files, run as Windows Service" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "    [q] Quit" -ForegroundColor White
    Write-Host ""

    $choice = Read-Host "  Your choice [2]"
    if ([string]::IsNullOrEmpty($choice)) { $choice = "2" }

    switch ($choice) {
        "1" { Invoke-UserInstall }
        "2" { Invoke-SystemInstall }
        "q" { Write-Info "Bye! "; exit 0 }
        default { Write-Err "Invalid choice"; exit 1 }
    }
} else {
    Write-Host "  Running as standard user" -ForegroundColor Yellow
    Write-Host "  TIP: Run as Administrator for system-wide install with Windows Service" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Choose installation mode:"
    Write-Host ""
    Write-Host "    [1] User Install" -ForegroundColor White
    Write-Host "        Install to %LOCALAPPDATA%\ParkHub (no admin needed)" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "    [q] Quit" -ForegroundColor White
    Write-Host ""

    $choice = Read-Host "  Your choice [1]"
    if ([string]::IsNullOrEmpty($choice)) { $choice = "1" }

    switch ($choice) {
        "1" { Invoke-UserInstall }
        "q" { Write-Info "Bye! "; exit 0 }
        default { Write-Err "Invalid choice"; exit 1 }
    }
}
