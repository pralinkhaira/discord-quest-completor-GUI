# Execution Flow

## 1. Application Startup Flow

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant OS as Windows OS
    participant Main as Tauri Core (main.rs / lib.rs)
    participant Web as WebView2 (Vue 3 App)
    participant Fetcher as useFetchGameList Composable
    participant API as Discord / GitHub API

    User->>OS: Launch Executable
    OS->>Main: Run tauri_app_lib::run()
    Main->>Main: Register Plugins (Dialog, Http, Opener) & Invoke Handlers
    Main->>Web: Initialize WebView2 & Render index.html
    Web->>Fetcher: tryOnMounted -> fetchGameList()
    Fetcher->>Main: invoke('fetch_gamelist_gh_mirror')
    Main->>API: HTTP GET /detectable.json
    alt Mirror Responds Successfully
        API-->>Main: Return JSON list
        Main-->>Fetcher: Game[] list
    else Mirror Fails
        Fetcher->>Main: invoke('fetch_gamelist_from_discord')
        Main->>API: HTTP GET /api/applications/detectable
        alt Discord API Success
            API-->>Main: Return JSON list
            Main-->>Fetcher: Game[] list
        else Discord Fails
            Fetcher->>Fetcher: Load bundled src/assets/gamelist.json
        end
    end
    Fetcher-->>Web: Populate gameDB reactive store
```

---

## 2. Game Selection and Dummy Process Launch Flow

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant UI as HomeView.vue / GameExecutables.vue
    participant Rust as Tauri Backend (lib.rs)
    participant FS as File System
    participant Runner as Dummy Process (src-win.exe)
    participant Discord as Discord Desktop Client

    User->>UI: Types query in search bar (Fuse.js filter)
    UI-->>User: Displays matching verified games
    User->>UI: Clicks "Add game to list"
    User->>UI: Selects game & clicks "Play" on executable
    
    UI->>Rust: invoke('create_fake_game', { path, executable_name, app_id, display_name })
    Rust->>FS: fs::create_dir_all("<exe_dir>/games/<app_id>/<path>")
    Rust->>FS: fs::copy("resources/src-win.exe", "<target_path>/<executable_name>")
    Rust-->>UI: Return creation status
    
    UI->>Rust: invoke('run_background_process', { name, path, executable_name, app_id })
    Rust->>FS: Resolve launch path
    Rust->>Runner: std::process::Command::spawn("--title", game.name)
    Runner->>Runner: Win32 message loop & tray creation (DQCTray)
    Rust-->>UI: "Process started successfully"
    UI->>UI: Set game.is_running = true & currentlyPlaying = game.id
    
    Note over Discord,Runner: Discord process scanner periodically inspects active processes
    Discord->>Discord: Detects running executable matching verified database entry
    Discord->>Discord: Updates user status to "Playing <Game>" & ticks quest progress
```

---

## 3. Game Process Termination Flow

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant UI as HomeView.vue / GameExecutables.vue
    participant Rust as Tauri Backend (lib.rs)
    participant OS as Windows OS (taskkill)
    participant Runner as Dummy Process (src-win.exe)
    participant Discord as Discord Desktop Client

    User->>UI: Clicks "Stop" on executable
    UI->>Rust: invoke('stop_process', { exec_name: executable.filename })
    Rust->>OS: std::process::Command("taskkill /F /IM <exec_name>")
    OS->>Runner: Terminate process instances
    OS-->>Rust: Exit code 0
    Rust-->>UI: Ok(())
    UI->>UI: Set game.is_running = false & currentlyPlaying = null
    Discord->>Discord: Detects process no longer running
    Discord->>Discord: Clears playing activity
```

---

## 4. Discord Direct RPC Flow (Experimental Feature)

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant UI as HomeView.vue
    participant Rust as Tauri Backend (lib.rs / runner.rs / rpc.rs)
    participant SDK as discord-sdk (IPC Socket)
    participant Discord as Discord Client

    User->>UI: Clicks "Test RPC"
    UI->>UI: Shows risk disclaimer dialog
    User->>UI: Clicks "Accept risk and continue"
    UI->>Rust: invoke('connect_to_discord_rpc_3', { activity_json, action: "connect" })
    Rust->>Rust: Emit "client_connecting" event
    Rust->>SDK: rpc::make_client(app_id, Subscriptions::ACTIVITY)
    SDK->>Discord: Connect to IPC socket
    Discord-->>SDK: User connected handshake
    Rust->>SDK: discord.update_activity(ActivityBuilder)
    SDK->>Discord: Set activity state, details, start_timestamp
    Rust->>Rust: Emit "client_connected" event
    Rust->>UI: Update status to connected
    
    Note over UI,Rust: When disconnecting
    User->>UI: Clicks "Disconnect to Discord Gateway"
    UI->>Rust: emit('event_disconnect')
    Rust->>SDK: client.discord.disconnect()
    Rust->>UI: Update status to disconnected
```
