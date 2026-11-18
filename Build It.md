# Building Discord Quest Completer

This document describes how to compile the application and where to find the generated output files.

---

## Method 1: One-Click Build Script *(Recommended)*

A build script `build.ps1` is located in the repository root. Open PowerShell and run:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force
.\build.ps1
```

This automates all required steps:
1. Sets up the MinGW and Cargo toolchain in `PATH`.
2. Compiles the Vue + TypeScript frontend into `dist/`.
3. Compiles the Rust backend with embedded production assets into `discord-quest-completer.exe`.
4. Packages and copies all runtime dependencies into `Final-Build/`.

---

## Method 2: Manual Step-by-Step Compilation

### Prerequisites
* **Node.js** & **pnpm**
* **Rust** (`stable-x86_64-pc-windows-gnu` toolchain)
* **MinGW-w64** (containing `gcc` and `windres.exe`)

---

### Step 1: Set Up PowerShell Environment

Open PowerShell and add MinGW and Cargo to your session's `PATH`:

```powershell
$mingwBin = "C:\Users\Khaira\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin"
$cargoBin = "$env:USERPROFILE\.cargo\bin"
[System.Environment]::SetEnvironmentVariable("PATH", "$cargoBin;$mingwBin;" + [System.Environment]::GetEnvironmentVariable("PATH", "Process"), "Process")
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-gnu"
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force
```

---

### Step 2: Build the Frontend (Vue + Vite)

Compile the web application assets into `dist/`:

```powershell
cd "D:\Github\extra\discord-quest-completor-GUI"
pnpm run build
```

---

### Step 3: Build the Rust Desktop Executable

Compile the Rust application in release mode:

```powershell
# Create junction if needed for tauri-build gnu compatibility
if (-not (Test-Path "src-tauri\target\release\build\build")) {
    cmd /c mklink /J "src-tauri\target\release\build\build" "src-tauri\target\release\build"
}

cd "src-tauri"
cargo build --release
```

---

## Output Locations

### 1. Compiler Output (Raw)
* **Main Executable:**  
  `src-tauri\target\release\discord-quest-completer.exe`
* **WebView2 Runtime Loader:**  
  `src-tauri\target\release\build\webview2-com-sys\...\out\x64\WebView2Loader.dll`

---

### 2. Standalone Package (`Final-Build/`)

The standalone, portable folder ready to run or distribute is located at:

```
D:\Github\extra\discord-quest-completor-GUI\Final-Build\
```

Contents:
| File / Folder | Description |
|---|---|
| `discord-quest-completer.exe` | Complete standalone GUI application with embedded UI assets and simplified Discord presence |
| `WebView2Loader.dll` | 64-bit WebView2 runtime loader required on Windows |
| `data\src-win.exe` | Runner binary used by the app to simulate game processes |
| `resources\src-win.exe` | Runner backup resource location |
