# Project Structure

## Directory Map

```text
discord-quest-completer/
│
├── .github/
│   ├── ISSUE_TEMPLATE/
│   │   └── bug_report.md               # GitHub issue template for reporting bugs
│   └── workflows/
│       ├── build-release.yaml          # Multi-stage CI workflow for compiling release bundles
│       ├── pages.yaml                  # Workflow for hosting detectable games mirror on Pages
│       └── rust-check.yml              # Scheduled Rust compilation and linter sanity checks
│
├── public/
│   ├── logo.svg                        # Application logo vector asset
│   ├── tauri.svg                       # Tauri logo asset
│   └── vite.svg                        # Vite logo asset
│
├── scripts/
│   └── sync-runner.mjs                 # Cross-platform runner build and asset copy automation
│
├── src/                                # Frontend Source Code (Vue 3 + Vite + TypeScript)
│   ├── assets/
│   │   ├── gamelist.json               # Bundled static fallback of detectable Discord games
│   │   └── vue.svg                     # Vue logo asset
│   ├── components/
│   │   ├── GameCard.vue                # Display card with cover image and launch action
│   │   ├── GameExecutables.vue         # Executable path selector, breadcrumb viewer & launch controller
│   │   ├── IconRustLang.vue            # SVG icon component for Rust
│   │   ├── IconVerified.vue            # SVG badge for Discord Verified application status
│   │   ├── IconVueJs.vue               # SVG icon component for Vue.js
│   │   ├── MainLayout.vue              # Top navigation bar, main container & footer layout
│   │   └── TimedNotification.vue       # Auto-dismissing status pill for network operations
│   ├── composables/
│   │   ├── app-state.ts                # Global reactive state (active page, runtime logs)
│   │   └── fetch-gamelist.ts           # Game list fetching with multi-tier fallback & validation
│   ├── constants/
│   │   └── constants.ts                # Operating system constants and dependency injection keys
│   ├── pages/
│   │   ├── HomeView.vue                # Main application interface: search, game management & launcher
│   │   └── Playground.vue              # Diagnostic testing page for Discord RPC connection and log stream
│   ├── services/
│   │   └── fake-games.ts               # Mock service utility for slicing demo game entries
│   ├── theme/
│   │   └── style.css                   # Global stylesheet importing Tailwind CSS v4 rules
│   ├── types/
│   │   └── types.d.ts                  # TypeScript definitions for Game, GameExecutable, and actions
│   ├── utils/
│   │   └── random-string.ts            # Unique UID generator for tracking selected game instances
│   ├── App.vue                         # Root Vue component switching between pages
│   ├── main.ts                         # Frontend application bootstrap
│   └── vite-env.d.ts                   # Vite client TypeScript declarations
│
├── src-darwin/                         # macOS runner stub and project files
│   ├── Cargo.toml
│   └── src/main.rs
│
├── src-linux/                          # Linux runner stub and project files
│   ├── Cargo.toml
│   └── src/main.rs
│
├── src-linux-rust-sleeper/             # Experimental headless Linux runner
│   ├── Cargo.toml
│   └── src/main.rs
│
├── src-tauri/                          # Tauri v2 Backend (Rust)
│   ├── capabilities/
│   │   └── default.json                # Security capabilities & network/window permissions
│   ├── icons/                          # Application icon assets for various resolutions & formats
│   ├── resources/
│   │   └── .gitkeep                    # Target folder for bundled runner executables (src-win.exe)
│   ├── src/
│   │   ├── lib.rs                      # Tauri setup, command registration & process execution logic
│   │   ├── main.rs                     # Windows GUI entry point (`windows_subsystem = "windows"`)
│   │   ├── rpc.rs                      # Discord SDK IPC socket client initialization and lifecycle
│   │   └── runner.rs                   # ActivityBuilder payload deserialization and status builder
│   ├── build.rs                        # Tauri build script verifying runner resource existence
│   ├── Cargo.toml                      # Rust dependencies for Tauri core
│   ├── tauri.conf.json                 # Tauri configuration (bundle, CSP, window dimensions)
│   └── tauri.windows.conf.json         # Windows-specific bundle resource mappings
│
├── src-template/                       # Reference template for dummy runners
│   ├── Cargo.toml
│   └── src/main.rs
│
├── src-win/                            # Windows C++ Dummy Game Runner
│   ├── src/
│   │   ├── main.cpp                    # Minimal Win32 window implementation with tray icon
│   │   └── main.rs                     # Rust entry point forwarding to mainEntryPoint
│   ├── build.rs                        # Compiles main.cpp with MSVC C++ compiler and flags
│   ├── Cargo.toml                      # Runner manifest configuring release optimizations
│   ├── embed_icon.rc                   # Windows resource script for embedding application icon
│   └── icon.ico                        # Embedded icon file
│
├── docs/                               # Architecture and Technical Documentation
├── index.html                          # Single-page application HTML entry
├── package.json                        # Node.js project manifest & scripts
├── pnpm-lock.yaml                      # Lockfile for frontend dependencies
├── README.md                           # Original project README
├── tsconfig.json                       # TypeScript compiler configuration
├── tsconfig.node.json                  # TypeScript configuration for build tooling
└── vite.config.ts                      # Vite build and dev server configuration
```
