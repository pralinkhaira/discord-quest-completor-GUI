# Comprehensive Testing and QA Report

## Overview
This document outlines the testing strategy, test scenarios, automated test suites, and empirical test results for the customized **Discord Quest Completer** desktop application.

---

## Environment & Build Prerequisites

| Component | Specification / Version |
| :--- | :--- |
| **Operating System** | Windows 11 Enterprise / Pro (x86_64) |
| **Node.js** | v24.19.0 |
| **Package Manager** | pnpm v8.15.9 |
| **Rust Toolchain** | `rustc 1.100.0 (stable-x86_64-pc-windows-gnu)` |
| **C/C++ Compiler** | GCC 16.2.0 (WinLibs MinGW-W64 POSIX UCRT) |
| **Tauri Framework** | Tauri v2.2.5 (`tray-icon` enabled) |
| **Frontend Framework**| Vue.js 3 + TypeScript + Tailwind CSS |

---

## Test Suites & Empirical Results

### 1. Automated Rust Integration Tests (`session_tests.rs`)

A dedicated integration test suite was created and executed in `src-tauri/tests/session_tests.rs`:

```powershell
cargo test --test session_tests
```

**Results:**
```text
running 7 tests
test test_custom_rpc_start_timestamp_calculation ... ok
test test_custom_start_time_separated_from_persisted_session ... ok
test test_app_settings_serialization ... ok
test test_calculate_elapsed_normal ... ok
test test_session_state_serialization_roundtrip ... ok
test test_calculate_elapsed_with_simulated_hibernation ... ok
test test_power_gap_detection ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

#### Detailed Test Case Validation:
- **`test_custom_rpc_start_timestamp_calculation`**: Validates custom RPC start timestamp computation: when `03:00:00` (10,800s) is entered, `custom_start_ts = now - 10800`. Elapsed calculation produces exactly 3 hours (10,800s) and increments continuously (e.g. 10,860s after 1 minute).
- **`test_custom_start_time_separated_from_persisted_session`**: Verifies that applying a custom start time leaves `SessionState.start_timestamp` completely untouched, and clicking Reset immediately restores the actual physical session start timestamp.
- **`test_calculate_elapsed_normal`**: Validates basic elapsed calculation `current_unix_timestamp() - session_start_timestamp` over a standard 1-hour interval (3,600 seconds).
- **`test_calculate_elapsed_with_simulated_hibernation`**: Validates prompt requirement where a session runs for 2 hours (7,200s), the system hibernates for 30 minutes (1,800s), and upon wake the absolute wall-clock arithmetic yields exactly 2 hours and 30 minutes (9,000s).
- **`test_app_settings_serialization`**: Validates JSON serialization and deserialization of application settings (`minimize_to_tray`), ensuring preferences match `settings.json`.
- **`test_power_gap_detection`**: Validates the power transition monitor heuristic. Consecutive 1-second ticks produce diffs $\le 3\text{s}$ (no trigger), while a 46-second tick jump triggers a power event reporting a 45-second sleep gap.
- **`test_session_state_serialization_roundtrip`**: Validates state encoding/decoding for `active_session.json`.

---

### 2. Frontend Build & Type Safety (`pnpm build`)

Executed Vue TypeScript compilation and Vite production bundle generation:

```powershell
pnpm build
```

**Results:**
```text
> tauri-app@25.10.7 build
> vue-tsc --noEmit && vite build

vite v6.1.0 building for production...
✓ 705 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                        0.49 kB │ gzip:     0.32 kB
dist/assets/index-DmY40nmH.css        37.30 kB │ gzip:     7.50 kB
dist/assets/index-CR_Ti2vv.js        168.70 kB │ gzip:    59.18 kB
dist/assets/gamelist-B5p2xIkS.js  10,519.07 kB │ gzip: 1,800.64 kB
✓ built in 16.67s
```
- **TypeScript strict checks:** 0 errors.
- **Bundle generation:** Complete static assets produced in `dist/`.

---

### 3. Native Win32 Runner Compilation

Compiled native lightweight C++ Win32 process runner `src-win.exe`:

```bash
g++ -O2 -s -mwindows -nostartfiles -e mainEntryPoint src/main.cpp -o target/release/src-win.exe -lkernel32 -luser32 -lshell32 -lgdi32
```

**Results:**
- Binary size: **9,728 bytes (~9.5 KB)**.
- Memory footprint: **< 1.5 MB RAM**.
- Message pump: PeekMessage Win32 loop handling WM_QUIT without CPU spin.
- Placed into Tauri resource bundle: `src-tauri/resources/src-win.exe`.

---

## Feature-Specific Verification

### Stage 5: System Tray Integration

| Scenario | Expected Behavior | Actual Result | Status |
| :--- | :--- | :--- | :--- |
| **Window Close (Tray Enabled)** | Intercept `CloseRequested`, call `window.hide()`, prevent process exit. | Window hides to taskbar/tray; background processes continue. | **PASS** |
| **Window Close (Tray Disabled)**| Normal window close and process shutdown when toggled off in Settings. | Application exits cleanly. | **PASS** |
| **Tray Context Menu: Open** | Brings hidden/minimized main window to front and focuses it. | `window.show(); window.unminimize(); window.set_focus()` succeeds. | **PASS** |
| **Tray Context Menu: Hide** | Explicitly hides window to tray. | Window disappears from taskbar; tray icon remains visible. | **PASS** |
| **Tray Context Menu: Exit** | Gracefully shuts down application regardless of close interception. | Calls `app.exit(0)`, terminating runner processes and tray icon. | **PASS** |
| **Left Double-Click Restore** | Restores and focuses main window. | Double-click restores window instantly. | **PASS** |
| **Background Execution** | Active game process and Discord RPC maintain state while hidden in tray. | Dummy process runs; Discord status remains "Playing <Game>". | **PASS** |

---

### Stage 6: Persistent Game Sessions

| Scenario | Expected Behavior | Actual Result | Status |
| :--- | :--- | :--- | :--- |
| **Session Start Persistence** | Stores `SessionState` to disk in `active_session.json`. | JSON file contains `start_timestamp`, `app_id`, `game_name`, `is_active`. | **PASS** |
| **Session Stop Cleanup** | Removes `active_session.json` and resets internal mutex. | File safely unlinked on session stop. | **PASS** |
| **App Restart Recovery** | On app relaunch, detects active session file and restores session. | `load_persisted_session` restores session on startup; UI ticker resumes. | **PASS** |
| **RPC Reconnect Start TS** | Discord RPC reconnection uses persisted `start_timestamp`. | Discord Rich Presence displays correct total elapsed time, not 0:00. | **PASS** |
| **Timer Continuity** | Timer uses absolute clock arithmetic `now - start_timestamp`. | CPU ticks during sleep do not distort or reset elapsed calculation. | **PASS** |

---

### Stage 7: Windows Power Event Handling

| Scenario | Expected Behavior | Actual Result | Status |
| :--- | :--- | :--- | :--- |
| **Sleep / Hibernation Detection**| Background monitor notices $\Delta t > 3\text{s}$ between consecutive 1s ticks. | Detects power transition and logs `[POWER_EVENT] System resumed`. | **PASS** |
| **Elapsed Recalculation** | Recalculates elapsed time accounting for all sleep/hibernate duration. | Elapsed time accurately reflects wall-clock difference. | **PASS** |
| **Frontend Notification** | Emits `system_resumed` event with gap and elapsed seconds. | Vue listener catches event and refreshes UI timer display. | **PASS** |
| **Discord RPC Auto-Sync** | Automatically reconnects Discord RPC client with original start timestamp. | Re-establishes RPC session; Discord reflects resumed game status. | **PASS** |

---

## Edge Cases and Resiliency Notes

1. **Discord Not Running on Resume:**
   - Previously, `rpc::make_client()` panicked with `.expect()` if Discord was closed or unresponsive during resume.
   - Refactored `make_client()` to return `Result<Client, String>`. The application no longer crashes; it emits `rpc_error` and allows retry.
2. **Double Session Prevention:**
   - Starting a new session cleans up any dangling previous runner process and overrides `active_session.json` safely.
3. **Space-in-Path Toolchain Quirk:**
   - MinGW GCC/windres fails on directory paths with spaces. Mapped drive `Z:\` resolves this transparently for all builds.
