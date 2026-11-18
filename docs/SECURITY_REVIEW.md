# Security and Safety Review

## 1. Executive Summary

This document reviews the security posture, external risks, IPC boundaries, and process safety of the Discord Quest Completer codebase.

---

## 2. Process Management and Safety

### 2.1 Indiscriminate Process Termination (`taskkill`)
- **Location**: `src-tauri/src/lib.rs` line 283 (`stop_process`)
- **Current Mechanism**:
  ```rust
  let output = std::process::Command::new("taskkill")
      .arg("/F")
      .arg("/IM")
      .arg(&process_name)
      .output()
  ```
- **Risk Assessment**: `taskkill /F /IM <name>` terminates **all** running processes with the specified executable image name across the current user session without verifying whether the PID was spawned by this application.
- **Impact**: If a user runs the dummy launcher for a game that they also legitimately have open, the real game process will be forcibly terminated, causing potential loss of unsaved game data.
- **Recommended Remediation**: Store the spawned child process PID when calling `Command::spawn()` and terminate strictly by PID (`taskkill /F /PID <pid>` or Win32 `TerminateProcess`), falling back to image name only when PID tracking is unavailable.

### 2.2 Orphan Process Retention
- **Risk Assessment**: `run_background_process` spawns child processes but does not maintain process ownership in application state. If the user exits the main Tauri application while a dummy game is active, the child process continues running indefinitely.
- **Recommended Remediation**: Implement an application exit hook in Tauri to safely enumerate and terminate any active dummy game runner processes spawned during the session, or attach them to a Windows Job Object configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.

---

## 3. Filesystem and Path Safety

### 3.1 Path Resolution and Directory Traversal
- **Location**: `src-tauri/src/lib.rs` (`game_folder_path`)
- **Current Mechanism**:
  ```rust
  fn game_folder_path(exe_dir: &Path, path: &str, app_id: i64) -> PathBuf {
      let normalized_path = Path::new(path).to_string_lossy().to_string();
      exe_dir.join("games").join(app_id.to_string()).join(normalized_path)
  }
  ```
- **Risk Assessment**: The `path` parameter originates from the detectable game database or frontend payload. If `path` contains directory traversal sequences (`..`), files could theoretically be created outside the `<exe_dir>/games` boundary.
- **Mitigation Status**: `GameExecutables.vue` applies client-side path validation (`isValidPath` filtering illegal characters `['>', '<', ':', '"', '|', '?', '*']`).
- **Recommended Remediation**: Add strict backend path canonicalization to ensure all generated paths remain strictly within `<exe_dir>/games/`.

### 3.2 Permission Model in System Directories
- **Risk Assessment**: If the executable is placed in a protected system path (such as `C:\Program Files\` or `C:\Windows\`), attempting to create directories and copy binaries into `<exe_dir>/games/` fails with an OS access denied error (`ERROR_ACCESS_DENIED`) unless running with administrator elevation.
- **Recommendation**: Ensure the application recommends portable user folders (e.g., user profile or dedicated user directories) or uses `%LOCALAPPDATA%` for generated game stubs.

---

## 4. Discord Platform and Terms of Service (ToS) Review

### 4.1 Process Simulation (Quest Completion)
- The application mimics game detection by creating a process with the expected image name. It does **not** inject code, modify Discord memory, hook system calls, bypass anti-cheat systems, or spoof authentication tokens.
- Discord detects the process through legitimate Win32 toolhelp process enumeration APIs (`CreateToolhelp32Snapshot`).

### 4.2 Direct RPC Gateway Integration
- The experimental RPC feature connects directly to Discord's local IPC named pipe using Discord's official SDK protocol.
- Sending rich presence updates using arbitrary third-party Application IDs may trigger Discord's heuristic detection for self-botting or automated status spoofing. The application properly gates this behind a clear user disclaimer modal.

---

## 5. WebView2 and IPC Security Configuration

- **Content Security Policy (CSP)**:
  - Configured in `src-tauri/tauri.conf.json`:
    ```json
    "csp": {
      "default-src": "'self' customprotocol: asset:",
      "connect-src": "ipc: http://ipc.localhost",
      "img-src": "'self' asset: http://asset.localhost blob: data:",
      "style-src": "'unsafe-inline' 'self'"
    }
    ```
- **Capability Scopes**:
  - `src-tauri/capabilities/default.json` explicitly restricts HTTP origins:
    - `https://markterence.github.io`
    - `https://discord.com`
- **Safety**: Remote scripts are prevented from execution within the WebView2 environment.

---

## 6. Rust Robustness and Panic Audit

- In `src-tauri/src/rpc.rs`:
  ```rust
  let user = match &*user.0.borrow() {
      ds::wheel::UserState::Connected(user) => user.clone(),
      ds::wheel::UserState::Disconnected(err) => panic!("failed to connect to Discord: {}", err),
  };
  ```
- **Risk Assessment**: If Discord is not running or the user disconnects Discord while attempting RPC connection, `panic!` crashes the entire Tauri application thread.
- **Recommended Remediation**: Refactor `rpc.rs` and `runner.rs` to return `Result<Client, String>` cleanly without panicking.
