# Application Features & Customizations

## 1. Baseline Features (Original Implementation)

1. **Detectable Game Catalog Integration**:
   - Queries verified detectable games from GitHub mirror with fallback to official Discord API endpoint and bundled static JSON snapshot (`gamelist.json`).
   - Over 1,000+ verified Discord applications supported.

2. **Fuzzy Search Engine**:
   - Powered by Fuse.js with weighted indexing across game titles, known aliases, and registered executable file paths.
   - Cleans special trademark and copyright symbols (`©`, `™`, `®`) for seamless search ergonomics.

3. **Multi-Executable Path Resolving**:
   - Parses complex executable path hierarchies (e.g. nested subfolders like `Binaries/Win64/Game.exe`).
   - Displays platform badge and folder breadcrumbs.

4. **Zero-Installation Dummy Game Runner**:
   - Automatically provisions folder structures matching Discord's detection criteria.
   - Generates minimal footprint Win32 executable stubs (< 10 KB) without needing .NET or full game downloads.
   - Spawns isolated game window with custom title and process metadata.

5. **Direct Discord RPC (Experimental)**:
   - Direct connection to Discord's local IPC Gateway using `discord-sdk`.
   - Rich presence customization: activity state, details, activity type, and start timestamp.

---

## 2. Independent Custom Features (Implemented & Verified)

### Feature 1: Native Windows System Tray Integration (Stage 5)
- **Status**: **Complete & Verified**
- **Description**: Enables running the application discreetly in the Windows system tray without taskbar clutter.
- **Capabilities**:
  - Native system tray icon using Tauri v2 tray icon subsystem.
  - Persistent user preference: `Minimize to Tray on Close` stored in `settings.json`.
  - When enabled, window close request (`CloseRequested`) is intercepted with `api.prevent_close()` and `window.hide()`.
  - Full Tray Context Menu:
    - *Open Application* (restores and brings window to foreground)
    - *Hide Application* (minimizes window to tray)
    - *Exit Application* (gracefully cleans up resources and terminates via `app.exit(0)`)
  - Left double-click on tray icon restores and focuses window instantly.
  - Background processes, active session tickers, and Discord Rich Presence maintain operation continuously while hidden.

### Feature 2: Persistent Game Session Timer Across Hibernation (Stage 6)
- **Status**: **Complete & Verified**
- **Description**: Robust absolute UNIX timestamp-based session tracking that accurately accounts for sleep, hibernation, and application restarts.
- **Problem Solved**: Standard CPU interval counters and `Instant` monotonically increase only while CPU cycles execute, freezing or resetting when Windows hibernates or restarts.
- **Capabilities**:
  - Source of truth: `elapsed_time = current_unix_timestamp - session_start_timestamp`.
  - Hibernate time counts towards elapsed gameplay time (e.g., 2 hours before hibernation + 30 minutes hibernated = 2 hours 30 minutes after resume).
  - Active session state persisted to disk in `active_session.json` (`session_id`, `app_id`, `game_name`, `executable_name`, `start_timestamp`, `is_active`).
  - Accurate elapsed time synchronization between Vue frontend and Rust backend via real-time reactive ticker.
  - Discord Rich Presence timestamp synchronization: preserves original `start_timestamp` so Discord elapsed timer matches application real-world elapsed time upon resume.
  - Automatic session restoration on application launch if an active session was previously saved.

### Feature 3: Windows Power Event Handling (Stage 7)
- **Status**: **Complete & Verified**
- **Description**: Background monitor detecting system power transitions and wall-clock discontinuities (> 3s gap).
- **Capabilities**:
  - Detects computer sleep and hibernation transitions automatically.
  - Re-evaluates elapsed time immediately on wake.
  - Emits `system_resumed` event to Vue frontend with `gap_seconds` and updated elapsed time.
  - Automatically re-establishes broken Discord IPC connections without losing session start history (`reconnect_discord_rpc_preserve_timestamp`).
  - Resilient error handling preventing crashes if Discord is closed during resume.

### Feature 4: Custom Start Time for Discord Rich Presence
- **Status**: **Complete & Verified**
- **Description**: Dedicated UI controls allowing users to set an arbitrary elapsed duration (e.g. `03:00:00`) for Discord Rich Presence without modifying the actual session start time.
- **Capabilities**:
  - Dedicated UI section in `HomeView.vue` with heading `Custom Start Time`, single input field with placeholder `HH:MM:SS`, `Apply` button, and `Reset` button.
  - Computes custom RPC start timestamp: $\text{custom\_start\_timestamp} = \text{current\_unix\_timestamp}() - \text{entered\_duration\_seconds}$.
  - Discord Rich Presence displays the custom elapsed duration (e.g. approx 3 hours) and continues counting up normally.
  - Complete separation from the persisted session: the actual session start timestamp in `active_session.json` is preserved untouched.
  - Clicking `Reset` immediately restores the actual session start timestamp on Discord RPC and resets the input field.
  - Works seamlessly with power event handling: power resume recalculates RPC presence preserving custom time offset if active.
