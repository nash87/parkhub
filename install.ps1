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

function Install-Binary($ver) {
    $asset = "parkhub-windows-x86_64.zip"
    $url = "https://github.com/$Repo/releases/download/$ver/$asset"
    $tmpDir = Join-Path $env:TEMP "parkhub-install"
    $zipPath = Join-Path $tmpDir $asset

    New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

    Write-Step "Downloading $asset..."
    try {
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
    } catch {
        Write-Warn "Download failed (release may not exist yet)"
        return $false
    }

    $installDir = Join-Path $env:LOCALAPPDATA "ParkHub"
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null

    Write-Step "Extracting to $installDir..."
    Expand-Archive -Path $zipPath -DestinationPath $installDir -Force

    $exe = Get-ChildItem -Path $installDir -Filter "*.exe" -Recurse | Select-Object -First 1
    if (-not $exe) {
        Write-Err "Could not find parkhub-server.exe in archive"
        return $false
    }

    $target = Join-Path $installDir "parkhub-server.exe"
    if ($exe.FullName -ne $target) {
        Move-Item -Path $exe.FullName -Destination $target -Force
    }

    # Add to PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
        $env:Path += ";$installDir"
        Write-OK "Added to PATH"
    }

    Write-OK "Installed to $target"
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    return $true
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

function Start-ParkHub($port, $dataDir) {
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

    Write-Info "Platform: Windows/$([System.Environment]::Is64BitOperatingSystem ? 'x64' : 'x86')"

    $ver = Get-LatestVersion
    Install-Binary $ver | Out-Null
    New-Config $DefaultPort $DefaultDataDir
    Start-ParkHub $DefaultPort $DefaultDataDir
}

function Invoke-CustomInstall {
    Write-Host ""
    Write-Host "  ⚙️  Custom Installation" -ForegroundColor White
    Write-Host ""

    Write-Info "Platform: Windows"

    # Port
    $port = Read-Host "  Port [$DefaultPort]"
    if ([string]::IsNullOrEmpty($port)) { $port = $DefaultPort }

    # Data directory
    $dataDir = Read-Host "  Data directory [$DefaultDataDir]"
    if ([string]::IsNullOrEmpty($dataDir)) { $dataDir = $DefaultDataDir }

    # TLS
    $tls = Read-Host "  Enable TLS? [y/N]"
    if ([string]::IsNullOrEmpty($tls)) { $tls = "n" }

    # Admin
    $adminUser = Read-Host "  Admin username [admin]"
    if ([string]::IsNullOrEmpty($adminUser)) { $adminUser = "admin" }

    $adminPass = Read-Host "  Admin password [auto-generate]" -AsSecureString
    $plainPass = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($adminPass))
    if ([string]::IsNullOrEmpty($plainPass)) {
        $plainPass = -join ((65..90) + (97..122) + (48..57) | Get-Random -Count 16 | ForEach-Object { [char]$_ })
        Write-Info "Generated password: $plainPass"
    }

    # Use case
    Write-Host ""
    Write-Host "  Use-case type:" -ForegroundColor White
    Write-Host "    [1] Corporate"
    Write-Host "    [2] Residential"
    Write-Host "    [3] Family"
    Write-Host "    [4] Rental"
    Write-Host "    [5] Public"
    $useCaseNum = Read-Host "  Your choice [1]"
    if ([string]::IsNullOrEmpty($useCaseNum)) { $useCaseNum = "1" }
    $useCaseMap = @{ "1" = "corporate"; "2" = "residential"; "3" = "family"; "4" = "rental"; "5" = "public" }
    $useCase = $useCaseMap[$useCaseNum]
    if (-not $useCase) { $useCase = "corporate" }

    # Organization
    $orgName = Read-Host "  Organization name [My Parking]"
    if ([string]::IsNullOrEmpty($orgName)) { $orgName = "My Parking" }

    # Self-registration
    $selfReg = Read-Host "  Enable self-registration? [Y/n]"
    if ([string]::IsNullOrEmpty($selfReg)) { $selfReg = "y" }
    $selfRegBool = $selfReg -match "^[Yy]"

    # Demo data
    $demoData = Read-Host "  Load demo data? [y/N]"
    if ([string]::IsNullOrEmpty($demoData)) { $demoData = "n" }
    $demoBool = $demoData -match "^[Yy]"

    Write-Host ""
    Write-Host "  📋 Summary" -ForegroundColor White
    Write-Info "Port:              $port"
    Write-Info "Data directory:    $dataDir"
    Write-Info "TLS:               $tls"
    Write-Info "Admin:             $adminUser"
    Write-Info "Use-case:          $useCase"
    Write-Info "Organization:      $orgName"
    Write-Info "Self-registration: $selfRegBool"
    Write-Info "Demo data:         $demoBool"
    Write-Host ""

    $confirm = Read-Host "  Proceed? [Y/n]"
    if ($confirm -match "^[Nn]") {
        Write-Warn "Aborted."
        return
    }

    $ver = Get-LatestVersion
    Install-Binary $ver | Out-Null
    New-Config $port $dataDir
    Start-ParkHub $port $dataDir

    Write-Host "  Admin credentials:" -ForegroundColor White
    Write-Host "    Username: $adminUser" -ForegroundColor Cyan
    Write-Host "    Password: $plainPass" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  ⚠  Save these credentials! Change password after first login." -ForegroundColor Red
    Write-Host ""
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
Write-Host "        Default settings, ready in 2 minutes" -ForegroundColor DarkGray
Write-Host ""
Write-Host "    [2] Custom Installation" -ForegroundColor White
Write-Host "        Configure settings before first start" -ForegroundColor DarkGray
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
