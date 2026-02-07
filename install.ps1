#Requires -Version 5.1
<#
.SYNOPSIS
    ParkHub installer for Windows
.DESCRIPTION
    Downloads and installs the latest ParkHub release for Windows.
    Optionally creates a Windows Service.
.EXAMPLE
    irm https://raw.githubusercontent.com/nash87/parkhub/main/install.ps1 | iex
#>

$ErrorActionPreference = "Stop"
$Repo = "nash87/parkhub"
$ServiceName = "ParkHub"

function Write-Step($msg) { Write-Host "  ℹ  $msg" -ForegroundColor Cyan }
function Write-OK($msg)   { Write-Host "  ✓  $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "  ⚠  $msg" -ForegroundColor Yellow }

Write-Host ""
Write-Host "  🅿️  ParkHub Installer for Windows" -ForegroundColor Green
Write-Host ""

# Get latest version
Write-Step "Fetching latest release..."
$release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$version = $release.tag_name
Write-Step "Latest version: $version"

# Download
$asset = "parkhub-windows-x86_64.zip"
$url = "https://github.com/$Repo/releases/download/$version/$asset"
$tmpDir = Join-Path $env:TEMP "parkhub-install"
$zipPath = Join-Path $tmpDir $asset

New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

Write-Step "Downloading $asset..."
Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

# Extract
$installDir = Join-Path $env:LOCALAPPDATA "ParkHub"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

Write-Step "Extracting to $installDir..."
Expand-Archive -Path $zipPath -DestinationPath $installDir -Force

# Find exe
$exe = Get-ChildItem -Path $installDir -Filter "*.exe" -Recurse | Select-Object -First 1
if (-not $exe) {
    Write-Error "Could not find parkhub-server.exe in archive"
    exit 1
}

# Rename if needed
$target = Join-Path $installDir "parkhub-server.exe"
if ($exe.FullName -ne $target) {
    Move-Item -Path $exe.FullName -Destination $target -Force
}

Write-OK "Installed to $target"

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    $env:Path += ";$installDir"
    Write-OK "Added to PATH"
}

# Optional Windows Service
Write-Host ""
$createService = Read-Host "Create Windows Service? (y/N)"
if ($createService -eq "y" -or $createService -eq "Y") {
    $dataDir = Join-Path $env:ProgramData "ParkHub"
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null

    # Check if NSSM is available, otherwise use sc.exe
    $nssm = Get-Command nssm -ErrorAction SilentlyContinue
    if ($nssm) {
        & nssm install $ServiceName $target
        & nssm set $ServiceName AppDirectory $dataDir
        & nssm set $ServiceName AppEnvironmentExtra "PARKHUB_DATA_DIR=$dataDir" "PARKHUB_PORT=8080"
        & nssm set $ServiceName Description "ParkHub - Parking Management"
        & nssm set $ServiceName Start SERVICE_AUTO_START
        & nssm start $ServiceName
        Write-OK "Service created with NSSM and started"
    } else {
        Write-Warn "NSSM not found. Creating service with sc.exe (limited features)..."
        sc.exe create $ServiceName binPath= "`"$target`"" start= auto
        sc.exe description $ServiceName "ParkHub - Parking Management"
        sc.exe start $ServiceName
        Write-OK "Service created and started"
        Write-Warn "For better service management, install NSSM: choco install nssm"
    }

    Write-Step "Data directory: $dataDir"
    Write-Step "Manage: sc.exe {start|stop|query} $ServiceName"
}

# Cleanup
Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host "    ✓ ParkHub installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "    Start:   parkhub-server.exe" -ForegroundColor Cyan
Write-Host "    Open:    http://localhost:8080" -ForegroundColor Cyan
Write-Host "    Login:   admin / admin" -ForegroundColor Yellow
Write-Host ""
Write-Host "    ⚠  Change your admin password immediately!" -ForegroundColor Red
Write-Host "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host ""
