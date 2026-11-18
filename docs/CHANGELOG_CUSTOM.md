# Custom Build Changelog

All notable changes, architectural adaptations, and new features introduced in this customized independent build are documented in this file.

---

## [25.10.8-custom] - 2026-11-18

### Stage 3: Baseline Build & Environment Stabilization
- Verified toolchain requirements on Windows (Node.js v24.19.0, pnpm v8.15.9, Rust 1.100.0 GNU target, WinLibs MinGW GCC 16.2.0).
- Resolved toolchain space-in-path limitation with `windres.exe` via virtual drive mapping `subst Z: "D:\Github\Discord Quest Completer"`.
- Resolved PE export table ordinal overflow (`export ordinal too large: 199524`) by tuning crate-types in `src-tauri/Cargo.toml` to `["staticlib", "rlib"]`.
- Compiled native 9.5 KB Win32 runner binary `src-win.exe` with GCC and embedded it in `src-tauri/resources/src-win.exe`.

### Stage 4: Independent Build Preparation
- Preserved original MIT License and author attribution in all modified files.
- Maintained application naming and branding on dedicated isolated branch `my-custom-build`.

### Stage 5: System Tray Integration
- Implemented native Windows system tray using Tauri v2 `TrayIconBuilder` in `src-tauri/src/tray.rs`.
- Added customizable Context Menu with three actions:
  - **Open Application**: Brings the hidden/minimized main window to front and focuses it.
  - **Hide Application**: Hides the main window to the system tray.
  - **Exit Application**: Cleanly terminates the application via `app.exit(0)`.
- Implemented left double-click on tray icon to quickly restore and focus window.
- Added persistent user preference `minimize_to_tray` saved in `settings.json`.
- Intercepted `tauri::WindowEvent::CloseRequested` to hide the window instead of exiting when `minimize_to_tray` is enabled.
- Ensured background processes, timers, and Discord RPC continue uninterrupted while hidden.
- Created `SettingsView.vue` with toggle switch and test buttons for tray interactions.

### Stage 6: Persistent Game Sessions & Sleep/Hibernation Resilience
- Implemented absolute UNIX timestamp-based session tracking (`elapsed = current_unix_timestamp() - session_start_timestamp`) in `src-tauri/src/session.rs`.
- Replaced volatile tick counters with persistent `active_session.json` storage, surviving app crashes, restarts, sleep, and hibernation.
- Added session recovery on application startup in both Rust backend and Vue frontend (`tryOnMounted`).
- Designed continuous elapsed quest timer banner in `src/pages/HomeView.vue` displaying live formatted `HH:MM:SS` time.
- Integrated Discord Rich Presence timestamp preservation: reconnected RPC sessions reuse original `start_timestamp`, preventing timer resets.
- Implemented clean manual "Stop Quest" button terminating processes, RPC, and persistent session state.

### Stage 7: Windows Power Event Handling & Auto-Resync
- Implemented background power transition monitor in `src-tauri/src/session.rs` detecting wall-clock discontinuities (> 3s gap).
- Emitted `system_resumed` event with `gap_seconds` and recalculated elapsed time.
- Implemented automatic Discord RPC reconnection after system resume (`reconnect_discord_rpc_preserve_timestamp`).
- Refactored `rpc::make_client()` in `src-tauri/src/rpc.rs` from panic/expect to `Result<Client, String>`, eliminating crashes when Discord is closed or restarting during wake.

### Stage 8: Testing & Verification
- Created comprehensive integration test suite `src-tauri/tests/session_tests.rs`:
  - `test_calculate_elapsed_normal`: PASS
  - `test_calculate_elapsed_with_simulated_hibernation` (2h before + 30m hibernate = 2h 30m): PASS
  - `test_app_settings_serialization`: PASS
  - `test_power_gap_detection`: PASS
  - `test_session_state_serialization_roundtrip`: PASS
  - `test_custom_rpc_start_timestamp_calculation`: PASS
  - `test_custom_start_time_separated_from_persisted_session`: PASS
- Verified frontend TypeScript compilation and Vite production build (`pnpm build`): 0 errors.
- Verified debug and release Tauri builds (`cargo build` and `cargo build --release`): production binary `discord-quest-completer.exe` (33.5 MB) generated successfully.
- Authored detailed testing report in `docs/TESTING.md`.

### Additional Feature: Custom Start Time for Discord Rich Presence
- Implemented dedicated **Custom Start Time** section in `src/pages/HomeView.vue`:
  - Input field with placeholder `HH:MM:SS`
  - `Apply` and `Reset` action buttons
  - Live feedback banner displaying active custom RPC elapsed time
- Implemented `set_custom_start_time`, `reset_custom_start_time`, and `get_custom_start_time` Tauri commands in `src-tauri/src/lib.rs`.
- Separated custom RPC start timestamp from the underlying persisted `SessionState.start_timestamp` (`active_session.json` remains untouched).
- Dynamic Rich Presence synchronization: when applied, Discord RPC immediately counts up from the entered duration; when reset, Discord RPC restores the actual physical session start timestamp.

---

## [Baseline] - Original Repository State (v25.10.7)

### Features
- Detectable verified games browser with fallback to GitHub mirror and Discord API.
- Fuse.js powered fuzzy search.
- Win32 lightweight dummy executable runner (`src-win.exe`).
- Multi-executable launch and stop support.
- Experimental Discord RPC test gateway.
