# ==============================================================================
# Discord Quest Completer - One-Click Production Build Script
# ==============================================================================

$ErrorActionPreference = "Stop"

$projectRoot = $PSScriptRoot
$srcTauri = Join-Path $projectRoot "src-tauri"
$finalBuildDir = Join-Path $projectRoot "Final-Build"

# 1. Setup PATH with MinGW and Cargo
$userProfile = [System.Environment]::GetEnvironmentVariable("USERPROFILE")
$mingwBin = "C:\Users\Khaira\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin"
$cargoBin = "$userProfile\.cargo\bin"

[System.Environment]::SetEnvironmentVariable("PATH", "$cargoBin;$mingwBin;" + [System.Environment]::GetEnvironmentVariable("PATH", "Process"), "Process")
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-gnu"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "1. Building Frontend (Vue + Vite)..." -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

Set-Location $projectRoot
& pnpm.cmd run build

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "2. Building Rust Release Binary..." -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Create junction if needed for tauri-build gnu compatibility
$targetReleaseBuild = Join-Path $srcTauri "target\release\build"
$junctionTarget = Join-Path $targetReleaseBuild "build"

if (-not (Test-Path $targetReleaseBuild)) {
    New-Item -ItemType Directory -Path $targetReleaseBuild -Force | Out-Null
}
if (-not (Test-Path $junctionTarget)) {
    cmd /c mklink /J "$junctionTarget" "$targetReleaseBuild" | Out-Null
}

Set-Location $srcTauri
& cargo.exe build --release

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "3. Assembling Output into Final-Build..." -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Ensure Final-Build and subdirectories exist
New-Item -ItemType Directory -Path (Join-Path $finalBuildDir "data") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $finalBuildDir "resources") -Force | Out-Null

# Copy main executable
Copy-Item (Join-Path $srcTauri "target\release\discord-quest-completer.exe") -Destination (Join-Path $finalBuildDir "discord-quest-completer.exe") -Force

# Copy WebView2Loader.dll
$wv2 = Get-ChildItem (Join-Path $srcTauri "target\release\build\webview2-com-sys") -Filter "WebView2Loader.dll" -Recurse | Where-Object { $_.FullName -match "x64" } | Select-Object -First 1
if ($wv2) {
    Copy-Item $wv2.FullName -Destination (Join-Path $finalBuildDir "WebView2Loader.dll") -Force
}

# Copy runner binaries
$runnerSource = Join-Path $srcTauri "resources\src-win.exe"
Copy-Item $runnerSource -Destination (Join-Path $finalBuildDir "data\src-win.exe") -Force
Copy-Item $runnerSource -Destination (Join-Path $finalBuildDir "resources\src-win.exe") -Force

Set-Location $projectRoot

Write-Host ""
Write-Host "BUILD SUCCESSFUL!" -ForegroundColor Green
Write-Host "Output files are located at: $finalBuildDir" -ForegroundColor Green
Get-ChildItem -Path $finalBuildDir -Recurse | Select-Object FullName, Length | Format-Table -AutoSize
