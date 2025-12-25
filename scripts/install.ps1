$ErrorActionPreference = "Stop"

$Repo = "jackby03/wovensnake"
$InstallDir = "$env:USERPROFILE\.wovensnake\bin"
$ExeName = "woven.exe"
$AssetUrl = "https://github.com/$Repo/releases/latest/download/wovensnake-windows-amd64.exe"

Write-Host "🧶 WovenSnake Installer" -ForegroundColor Green
Write-Host "-----------------------" -ForegroundColor DarkGray

# 1. Prepare Directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Write-Host "Created install directory: $InstallDir" -ForegroundColor Gray
}

# 2. Download Binary
$DestPath = Join-Path $InstallDir $ExeName
Write-Host "Downloading latest version..." -ForegroundColor Yellow
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $AssetUrl -OutFile $DestPath
    Write-Host "Download complete!" -ForegroundColor Green
} catch {
    Write-Error "Failed to download WovenSnake. Please check your internet connection or the repository URL."
    exit 1
}

# 3. Update PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Host "Adding to PATH..." -ForegroundColor Yellow
    $NewPath = "$CurrentPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path += ";$InstallDir" # Update current session too
    Write-Host "Path updated! (You might need to restart your terminal)" -ForegroundColor Green
} else {
    Write-Host "Already in PATH." -ForegroundColor Gray
}

Write-Host "`n✨ WovenSnake successfully installed!" -ForegroundColor Cyan
Write-Host "Try it now: woven --help" -ForegroundColor Gray
