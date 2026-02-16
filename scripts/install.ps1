$ErrorActionPreference = "Stop"

function Show-Logo {
    Clear-Host
    Write-Host "
 __      __  _____   __      __  ______   __   __   ______   __   __   ______   __  __   ______    
/\ \  __/\ \/\  __-./\ \  __/\ \/\  ___\ /\ '-.\ \ /\  ___\ /\ '-.\ \ /\  __ \ /\ \/ /  /\  ___\   
\ \ \/\ \ \ \ \ \/\ \ \ \/\ \ \ \ \  __\ \ \ \-.  \ \ \___  \ \ \-.  \ \ \  __ \ \ \  _'-\ \  __\   
 \ \_\ \_\ \_\ \____-\ \_\ \_\ \_\ \_____\\ \_\\'\_\ \/\_____\\ \_\\'\_\ \ \_\ \_\ \ \_\ \_\ \_____\ 
  \/_/\/_/\/_/\/____/ \/_/\/_/\/_/\/_____/ \/_/ \/_/  \/_____/ \/_/ \/_/  \/_/\/_/  \/_/\/_/\/_____/ 
                                                                                                     
    " -ForegroundColor Green
    Write-Host "                🐍 WovenSnake Installer" -ForegroundColor Cyan
    Write-Host "                Dependencies, neatly woven." -ForegroundColor Gray
    Write-Host ""
}

function Get-UserConfirmation {
    param([string]$Prompt, [bool]$Default=$true)
    $DefaultStr = if ($Default) { "[Y/n]" } else { "[y/N]" }
    $Response = Read-Host -Prompt "$Prompt $DefaultStr"
    if ([string]::IsNullOrWhiteSpace($Response)) { return $Default }
    return $Response -match "^[yY]"
}

function Get-InstallDir {
    $Default = "$env:USERPROFILE\.wovensnake\bin"
    $Input = Read-Host -Prompt "Install Location [$Default]"
    if ([string]::IsNullOrWhiteSpace($Input)) { return $Default }
    return $Input
}

Show-Logo

# --- Configuration Phase ---
Write-Host ":: Configuration" -ForegroundColor Yellow

# 1. Install Location
$InstallDir = Get-InstallDir

# 2. Add to PATH
$AddToPath = Get-UserConfirmation "Add to user PATH?" $true

# 3. Global Python Management (Future wiring)
$ManagePython = Get-UserConfirmation "Enable global Python management?" $false

Write-Host "`n:: Installation Plan" -ForegroundColor Yellow
Write-Host "   Location: $InstallDir"
Write-Host "   Add Path: $AddToPath"
Write-Host "   Py Mgmt : $ManagePython"
Write-Host ""

if (-not (Get-UserConfirmation "Proceed with installation?")) {
    Write-Host "Installation cancelled." -ForegroundColor Red
    exit 0
}

# --- Execution Phase ---
Write-Host "`n:: Installing..." -ForegroundColor Cyan

# Prepare Directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Download Binary
$Repo = "jackby03/wovensnake"
$ExeName = "woven.exe"
$AssetUrl = "https://github.com/$Repo/releases/latest/download/woven-windows-amd64.exe"
$DestPath = Join-Path $InstallDir $ExeName

try {
    Write-Host "   Downloading $ExeName..." -NoNewline
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $AssetUrl -OutFile $DestPath
    Write-Host " Done!" -ForegroundColor Green
} catch {
    Write-Host " Failed!" -ForegroundColor Red
    Write-Error "Failed to download WovenSnake. Please check your internet connection."
    exit 1
}

# Update PATH
if ($AddToPath) {
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($CurrentPath -notlike "*$InstallDir*") {
        Write-Host "   Adding to PATH..." -NoNewline
        $NewPath = "$CurrentPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        $env:Path += ";$InstallDir"
        Write-Host " Done!" -ForegroundColor Green
    } else {
        Write-Host "   Already in PATH." -ForegroundColor DarkGray
    }
}

# Python Mgmt Setup (Placeholder)
if ($ManagePython) {
    $PyDir = "$env:USERPROFILE\.wovensnake\python"
    if (-not (Test-Path $PyDir)) {
        New-Item -ItemType Directory -Path $PyDir -Force | Out-Null
        Write-Host "   Created Python management directory: $PyDir"
    }
}

# --- Validation Phase ---
Write-Host "`n:: Verification" -ForegroundColor Yellow
if (Test-Path $DestPath) {
    try {
        $Version = & $DestPath --version
        Write-Host "   Verifying binary... $Version" -ForegroundColor Green
        
        Write-Host "`n✨ Installation Successful!" -ForegroundColor Green
        Write-Host "You may need to restart your terminal for PATH changes to take effect." -ForegroundColor Gray
        Write-Host "Try: woven init" -ForegroundColor Cyan
    } catch {
         Write-Warning "Binary installed but failed to execute. You may need to install VC++ Redistributables."
    }
} else {
    Write-Error "Installation failed: Binary not found."
    exit 1
}
