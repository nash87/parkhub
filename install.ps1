#Requires -Version 5.1
<#
.SYNOPSIS
    ParkHub interactive installer for Windows
.DESCRIPTION
    Downloads and installs the latest ParkHub release for Windows.
    Supports Quick Start and Custom Installation modes.
.EXAMPLE
    irm https://raw.githubusercontent.com/nash87/parkhub/main/install.ps1 | iex
#>

$ErrorActionPreference = "Stop"
$Repo = "nash87/parkhub"
$ServiceName = "ParkHub"
$Version = "1.0.0"
$DefaultPort = 7878
$DefaultInstallDir = Join-Path $env:ProgramFiles "ParkHub"
$DefaultDataDir = Join-Path $env:LOCALAPPDATA "ParkHub\data"

function Write-Info($msg)  { Write-Host "  i  $msg" -ForegroundColor Cyan }
function Write-OK($msg)    { Write-Host "  ✓  $msg" -ForegroundColor Green }
function Write-Warn($msg)  { Write-Host "  ⚠  $msg" -ForegroundColor Yellow }
function Write-Err($msg)   { Write-Host "  ✗  $msg" -ForegroundColor Red }
function Write-Step($msg)  { Write-Host "  →  $msg" -ForegroundColor Cyan }

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
    $asset = "parkhub-windows-amd64.exe"
    $url = "https://github.com/$Repo/releases/download/$ver/$asset"

    New-Item -ItemType Directory -Force -Path $installDir | Out-Null

    $target = Join-Path $installDir "parkhub-server.exe"

    Write-Step "Downloading $asset..."
    try {
        Invoke-WebRequest -Uri $url -OutFile $target -UseBasicParsing
    } catch {
        Write-Warn "Download failed (release may not exist yet)"
        return $false
    }

    Write-OK "Installed to $target"
    return $true
}

function Add-ToPath($installDir) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
        $env:Path += ";$installDir"
        Write-OK "Added to PATH"
    } else {
        Write-Info "Already in PATH"
    }
}

function Install-AsService($installDir) {
    $exe = Join-Path $installDir "parkhub-server.exe"
    Write-Step "Installing Windows service..."
    & $exe install
    if ($LASTEXITCODE -eq 0) {
        Write-OK "Service 'ParkHub' installed"
        Write-Step "Starting service..."
        Start-Service -Name $ServiceName -ErrorAction SilentlyContinue
        Write-OK "Service started"
    } else {
        Write-Warn "Service installation failed"
    }
}

function Add-FirewallRule($port) {
    Write-Step "Adding firewall rule for port $port..."
    try {
        New-NetFirewallRule -DisplayName "ParkHub Server" -Direction Inbound -Action Allow -Protocol TCP -LocalPort $port -ErrorAction Stop | Out-Null
        Write-OK "Firewall rule added"
    } catch {
        Write-Warn "Failed to add firewall rule: $_"
    }
}

function New-StartMenuShortcut($installDir) {
    $startMenu = Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\ParkHub"
    New-Item -ItemType Directory -Force -Path $startMenu | Out-Null
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut("$startMenu\ParkHub Server.lnk")
    $shortcut.TargetPath = Join-Path $installDir "parkhub-server.exe"
    $shortcut.WorkingDirectory = $installDir
    $shortcut.Save()
    Write-OK "Start Menu shortcut created"
}

function New-Config($port, $dataDir) {
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
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
    } else {
        Write-Info "Config already exists at $configFile"
    }
}

function Show-Completion($port) {
    $ip = Get-HostIP
    $url = "http://${ip}:${port}"

    Write-Host ""
    Write-Host "  ═══════════════════════════════════════════" -ForegroundColor Green
    Write-Host "    ✓ ParkHub is ready!" -ForegroundColor Green
    Write-Host ""
    Write-Host "    🚗  $url" -ForegroundColor White
    Write-Host ""
    Write-Host "    Open this URL in your browser to start" -ForegroundColor Cyan
    Write-Host "    the onboarding wizard." -ForegroundColor Cyan
    Write-Host "  ═══════════════════════════════════════════" -ForegroundColor Green
    Write-Host ""
}

function Invoke-QuickStart {
    Write-Host ""
    Write-Host "  🚀 Quick Start" -ForegroundColor White
    Write-Host ""

    Write-Info "Platform: Windows/x64"

    $ver = Get-LatestVersion
    Install-Binary $ver $DefaultInstallDir | Out-Null
    New-Config $DefaultPort $DefaultDataDir
    Add-ToPath $DefaultInstallDir
    Install-AsService $DefaultInstallDir
    Add-FirewallRule $DefaultPort
    New-StartMenuShortcut $DefaultInstallDir
    Show-Completion $DefaultPort
}

function Invoke-CustomInstall {
    Write-Host ""
    Write-Host "  ⚙️  Custom Installation" -ForegroundColor White
    Write-Host ""

    # Install directory
    $installDir = Read-Host "  Install directory [$DefaultInstallDir]"
    if ([string]::IsNullOrEmpty($installDir)) { $installDir = $DefaultInstallDir }

    # Port
    $port = Read-Host "  Port [$DefaultPort]"
    if ([string]::IsNullOrEmpty($port)) { $port = $DefaultPort }

    # Data directory
    $dataDir = Read-Host "  Data directory [$DefaultDataDir]"
    if ([string]::IsNullOrEmpty($dataDir)) { $dataDir = $DefaultDataDir }

    # Options
    $doService = Read-Host "  Install as Windows Service? [Y/n]"
    if ([string]::IsNullOrEmpty($doService)) { $doService = "y" }
    $doPath = Read-Host "  Add to PATH? [Y/n]"
    if ([string]::IsNullOrEmpty($doPath)) { $doPath = "y" }
    $doFirewall = Read-Host "  Add firewall rule? [Y/n]"
    if ([string]::IsNullOrEmpty($doFirewall)) { $doFirewall = "y" }
    $doShortcut = Read-Host "  Create Start Menu shortcut? [Y/n]"
    if ([string]::IsNullOrEmpty($doShortcut)) { $doShortcut = "y" }

    Write-Host ""
    $confirm = Read-Host "  Proceed? [Y/n]"
    if ($confirm -match "^[Nn]") { Write-Warn "Aborted."; return }

    $ver = Get-LatestVersion
    Install-Binary $ver $installDir | Out-Null
    New-Config $port $dataDir

    if ($doPath -match "^[Yy]") { Add-ToPath $installDir }
    if ($doService -match "^[Yy]") { Install-AsService $installDir }
    if ($doFirewall -match "^[Yy]") { Add-FirewallRule $port }
    if ($doShortcut -match "^[Yy]") { New-StartMenuShortcut $installDir }

    Show-Completion $port
}

# Main
Clear-Host
Write-Host ""
Write-Host "  ═══════════════════════════════════════════" -ForegroundColor Green
Write-Host "       🚗  ParkHub Installer v$Version  🚗" -ForegroundColor Green
Write-Host "  ═══════════════════════════════════════════" -ForegroundColor Green
Write-Host ""
Write-Host "  Choose installation mode:"
Write-Host ""
Write-Host "    [1] Quick Start (recommended)" -ForegroundColor White
Write-Host "        Download, install as service, ready to go" -ForegroundColor DarkGray
Write-Host ""
Write-Host "    [2] Custom Installation" -ForegroundColor White
Write-Host "        Choose directory, port, service options" -ForegroundColor DarkGray
Write-Host ""
Write-Host "    [q] Quit" -ForegroundColor White
Write-Host ""

$choice = Read-Host "  Your choice [1]"
if ([string]::IsNullOrEmpty($choice)) { $choice = "1" }

switch ($choice) {
    "1" { Invoke-QuickStart }
    "2" { Invoke-CustomInstall }
    "q" { Write-Info "Bye! 👋"; exit 0 }
    default { Write-Err "Invalid choice"; exit 1 }
}
