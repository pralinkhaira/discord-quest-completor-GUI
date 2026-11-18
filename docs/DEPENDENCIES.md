# Dependencies and Prerequisites

## 1. System Requirements & Prerequisites

| Requirement | Minimum Version | Purpose |
|---|---|---|
| **Operating System** | Windows 10 / 11 (64-bit) | Host platform; required for Win32 runner & process detection |
| **Node.js** | 20.0.0+ (LTS recommended) | JavaScript runtime for Vite frontend bundling |
| **pnpm** | 8.0.0+ | Package manager used for managing frontend modules |
| **Rust & Cargo** | Stable (1.78+) | Rust toolchain required for building Tauri core and runner |
| **Visual Studio C++ Build Tools** | 2019 / 2022 | MSVC `cl.exe` and `link.exe` for compiling `src-win` native C++ code |
| **Microsoft Edge WebView2** | Evergreen Runtime | Embedded browser engine powering the Tauri UI (pre-installed on Windows 11) |

---

## 2. Frontend Dependencies (`package.json`)

### Runtime Dependencies
- `vue` (`^3.5.13`): Progressive reactive UI framework.
- `@tauri-apps/api` (`^2.8.0`): Client-side bridge for Tauri commands, events, and core IPC.
- `@tauri-apps/plugin-dialog` (`~2`): Native OS dialog alerts and confirmations.
- `@tauri-apps/plugin-http` (`~2`): Network client bypassing browser CORS restrictions.
- `@tauri-apps/plugin-opener` (`^2`): Inter-process utility for opening external URLs in default browser.
- `@vueuse/core` (`^13.1.0`): Collection of Vue composition utilities (`createGlobalState`, `onClickOutside`, `refDebounced`).
- `@vueuse/integrations` (`^13.9.0`): VueUse integration wrapper for third-party libraries (Fuse.js).
- `fuse.js` (`^7.1.0`): Client-side lightweight fuzzy-search engine for matching games and executable names.
- `tailwindcss` (`^4.1.3`): Utility-first styling engine.
- `@tailwindcss/vite` (`^4.1.3`): Vite plugin integration for Tailwind v4.
- `lodash-es` (`^4.17.21`): Modular JavaScript utility functions (`isEmpty`).
- `nanoid` (`^5.1.5`): Unique ID generator for selected game instances.

### Build & Development Dependencies
- `@tauri-apps/cli` (`^2.8.4`): Tauri command-line tooling for building and running desktop applications.
- `vite` (`^6.0.3`): Next-generation frontend build tool and dev server.
- `@vitejs/plugin-vue` (`^5.2.1`): Official Vue Single-File Component plugin for Vite.
- `typescript` (`~5.6.2`): Type checking and language services.
- `vue-tsc` (`^2.1.10`): Type-checking compiler for Vue components.
- `npm-run-all` (`^4.1.5`): CLI tool to run multiple npm-scripts in parallel or sequentially.

---

## 3. Rust Backend Dependencies (`src-tauri/Cargo.toml`)

- `tauri` (`2.x`, features: `protocol-asset`): Core desktop application framework.
- `tauri-plugin-dialog` (`2.x`): Dialog plugin backend.
- `tauri-plugin-http` (`2.x`): HTTP client plugin backend.
- `tauri-plugin-opener` (`2.x`): Shell opener plugin backend.
- `serde` (`1.x`, features: `derive`): Serialization/deserialization framework for Rust types.
- `serde_json` (`1.x`): JSON parser and generator for IPC messages.
- `discord-sdk` (`0.4.0`): Rust implementation of the Discord Game SDK and local IPC protocol.
- `tokio` (`1.44.2`): Asynchronous runtime for managing Discord RPC tasks.
- `once_cell` (`1.21.3`): Lazy static initialization for the Discord client singleton.
- `reqwest` (`=0.11`, features: `json`, `multipart`, `brotli`, `gzip`, `blocking`): HTTP client used for fetching remote game lists.

### Build Dependencies (`src-tauri/build.rs`)
- `tauri-build` (`2.x`): Build helper generating Tauri resource contexts and application manifests.

---

## 4. Windows Native Runner Dependencies (`src-win/Cargo.toml`)

- `cc` (`1.0`): Build dependency invoking the MSVC C++ compiler (`cl.exe`) to compile `src-win/src/main.cpp`.
- Linked Windows System Libraries:
  - `kernel32.lib`: Core Win32 process management and memory allocation.
  - `user32.lib`: Window class registration, message loop (`GetMessageW`), and dialog creation.
  - `shell32.lib`: Shell integration and system tray notification icon (`Shell_NotifyIconW`, `ShellExecuteW`).
  - `gdi32.lib`: Font creation and GDI object management (`CreateFontW`, `DeleteObject`).
