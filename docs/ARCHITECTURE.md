# Architecture Overview

## 1. System Summary

The **Discord Quest Completer** is a desktop application built using **Tauri v2** (Rust backend + Web frontend) and **Vue.js 3** (TypeScript, Vite, Tailwind CSS v4). Its primary purpose is to simulate the presence of Discord-verified games on the host system without requiring full installation of the game assets. Discord's desktop client detects games through process list inspection and directory structure heuristics; this application replicates those signals using a minimal dummy executable runner and an optional direct Discord RPC gateway integration.

```
+-------------------------------------------------------------+
|                     Vue 3 Frontend                          |
|  - UI Views (HomeView, Playground, MainLayout)              |
|  - Composables (useFetchGameList, useGlobalState)           |
|  - Fuzzy Search (Fuse.js) & State Management                |
+------------------------------+------------------------------+
                               | Tauri IPC (invoke / emit)
+------------------------------v------------------------------+
|                   Tauri v2 Core (Rust)                      |
|  - lib.rs: Command Routing & Process Orchestration          |
|  - runner.rs: Activity Builder & Payload Translation        |
|  - rpc.rs: Discord SDK IPC Gateway Client                   |
+------------------------------+------------------------------+
                               | OS Command Execution
        +----------------------+----------------------+
        |                                             |
+-------v-------------------------+         +---------v---------------+
| Windows Native Runner (src-win) |         | Discord Desktop Client  |
| - C++ / Win32 Lightweight Window|         | - Process Detection     |
| - Replicates Game Exe & Directory|        | - Quest Progress Sync   |
+---------------------------------+         +-------------------------+
```

---

## 2. Frontend Architecture (Vue 3 + TypeScript)

- **Entry Point**: `src/main.ts` mounts `App.vue` with global styles configured via Tailwind CSS v4 (`src/theme/style.css`).
- **Core State Management**:
  - `src/composables/app-state.ts`: Defines global application state using VueUse's `createGlobalState`. Manages current page view (`home` vs `playground`), application logs (`logs`), and counter demo state.
  - `src/composables/fetch-gamelist.ts`: Handles fetching detectable game definitions. Uses a three-tier fallback mechanism:
    1. GitHub Pages Mirror: `https://markterence.github.io/discord-quest-completer/detectable.json`
    2. Discord API Endpoint: `https://discord.com/api/applications/detectable`
    3. Bundled Fallback: `src/assets/gamelist.json`
- **User Interface Components**:
  - `src/components/MainLayout.vue`: Application shell providing header, navigation tabs, and footer.
  - `src/pages/HomeView.vue`: Main workspace containing the game search bar (with Fuse.js debounced searching), selected games panel, executable launcher selection, status monitor, and RPC test trigger.
  - `src/components/GameExecutables.vue`: Renders platform-filtered executable paths (e.g. `Binaries/Win64/Game.exe`), handles breadcrumb styling, and emits launch/stop events.
  - `src/pages/Playground.vue`: Developer diagnostic panel allowing manual RPC tests and real-time structured log viewing.

---

## 3. Backend Architecture (Rust / Tauri v2)

- **Application Initialization**: `src-tauri/src/main.rs` invokes `tauri_app_lib::run()` in `src-tauri/src/lib.rs`.
- **Plugin Registry**:
  - `tauri_plugin_dialog`: Native message dialogs for error and warning alerts.
  - `tauri_plugin_http`: High-performance HTTP client for fetching game registries without browser CORS restrictions.
  - `tauri_plugin_opener`: Utility for opening external links.
- **IPC Command Handlers**:
  - `create_fake_game`: Computes the dummy game folder hierarchy `<exe_dir>/games/<app_id>/<path>`, creates directories via `std::fs::create_dir_all`, and copies the bundled runner template (`resources/src-win.exe` or `data/src-win.exe`) to the target executable path.
  - `run_background_process`: Spawns the dummy game binary as a detached child process with the `--title "<game_name>"` flag.
  - `stop_process`: Executes `taskkill /F /IM <exec_name>` on Windows (or `pkill` on Unix platforms) to terminate the running dummy process.
  - `connect_to_discord_rpc_3`: Connects to Discord's local IPC named pipe via `discord-sdk`, updates Rich Presence activity with game metadata, and registers listeners for disconnection.
  - `fetch_gamelist_gh_mirror` / `fetch_gamelist_from_discord`: Proxies HTTP requests using `reqwest` / `tauri_plugin_http`.

---

## 4. Runner Architecture (`src-win`)

- **Design Philosophy**: Minimizes memory footprint and executable size (compiled binary is under 150 KB without requiring external .NET runtimes).
- **Implementation**: Written in C++ (`src-win/src/main.cpp`) and linked via Rust (`src-win/src/main.rs` with `#![no_main] #![no_std]`).
- **Linking Flags**: Uses MSVC flags `/GS-`, `/O1`, `/NODEFAULTLIB`, `/ENTRY:mainEntryPoint`, `/SUBSYSTEM:WINDOWS`, and `/ALIGN:16`.
- **Win32 Runtime Behavior**:
  - Resolves required Win32 APIs dynamically via `LoadLibraryW` and `GetProcAddress` (`shell32.dll`, `user32.dll`, `gdi32.dll`).
  - Parses `--title` command-line argument to customize the displayed window and tray tooltip.
  - Registers the `DQCTray` window class and displays a compact dialog window (415x240 px) with a system tray notification icon (`Shell_NotifyIconW`).

---

## 5. Discord Detection Heuristics

Discord's desktop client identifies running games by scanning active Windows processes and matching their executable file names and parent folder depths against Discord's detectable application list.
When this application creates:
```
<AppDirectory>/games/<Application_ID>/<Relative_Path>/<Game_Executable.exe>
```
and spawns the binary, Discord detects the process as the legitimate verified game. This satisfies the quest criteria (e.g. playing for 15 minutes) without requiring the actual game data files.
