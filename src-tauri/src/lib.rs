// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use once_cell::sync::OnceCell;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Emitter, Listener, Manager};

pub mod rpc;
pub mod runner;
pub mod session;
pub mod tray;

// Global static instance of the Discord client
static DISCORD_CLIENT: OnceCell<Mutex<Option<rpc::Client>>> = OnceCell::new();

fn get_discord_client() -> &'static Mutex<Option<rpc::Client>> {
    DISCORD_CLIENT.get_or_init(|| Mutex::new(None))
}

fn runner_resource_name() -> &'static str {
    #[cfg(target_os = "windows")]
    let runner_name = "data/src-win.exe";

    #[cfg(target_os = "linux")]
    let runner_name = "data/src-linux";

    #[cfg(target_os = "macos")]
    let runner_name = "data/src-darwin";

    runner_name
}

fn is_app_bundle(executable_name: &str) -> bool {
    cfg!(target_os = "macos") && executable_name.ends_with(".app")
}

fn bundle_binary_name(app_name: &str) -> String {
    app_name
        .strip_suffix(".app")
        .unwrap_or(app_name)
        .to_string()
}

fn game_folder_path(exe_dir: &Path, path: &str, app_id: i64) -> PathBuf {
    let normalized_path = Path::new(path).to_string_lossy().to_string();

    exe_dir
        .join("games")
        .join(app_id.to_string())
        .join(normalized_path)
}

fn resolve_runner_template(handle: &AppHandle) -> Result<PathBuf, String> {
    handle
        .path()
        .resolve(runner_resource_name(), BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve runner template: {}", e))
}

fn make_macos_app_bundle(
    app_bundle_path: &Path,
    app_name: &str,
    display_name: &str,
    runner_template: &Path,
) -> Result<PathBuf, String> {
    let binary_name = bundle_binary_name(app_name);
    let macos_dir = app_bundle_path.join("Contents/MacOS");
    fs::create_dir_all(&macos_dir)
        .map_err(|e| format!("Failed to create app bundle directories: {}", e))?;

    let target_executable_path = macos_dir.join(&binary_name);
    fs::copy(runner_template, &target_executable_path)
        .map_err(|e| format!("Failed to copy dummy executable: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&target_executable_path)
            .map_err(|e| format!("Failed to read executable permissions: {}", e))?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&target_executable_path, permissions)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
    }

    let info_plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>{display_name}</string>
    <key>CFBundleExecutable</key>
    <string>{binary_name}</string>
    <key>CFBundleIdentifier</key>
    <string>me.markterence.discordquestcompleter.dummy</string>
    <key>CFBundleName</key>
    <string>{display_name}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
</dict>
</plist>
"#
    );

    fs::write(app_bundle_path.join("Contents/Info.plist"), info_plist)
        .map_err(|e| format!("Failed to write Info.plist: {}", e))?;

    Ok(target_executable_path)
}

fn launch_executable_path(game_folder_path: &Path, executable_name: &str) -> PathBuf {
    if is_app_bundle(executable_name) {
        game_folder_path
            .join(executable_name)
            .join("Contents/MacOS")
            .join(bundle_binary_name(executable_name))
    } else {
        game_folder_path.join(executable_name)
    }
}

fn stop_process_name(exec_name: &str) -> String {
    if is_app_bundle(exec_name) {
        bundle_binary_name(exec_name)
    } else {
        exec_name.to_string()
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command(rename_all = "snake_case")]
async fn create_fake_game(
    handle: tauri::AppHandle,
    path: &str,
    executable_name: &str,
    path_len: i64,
    app_id: i64,
    display_name: Option<String>,
) -> Result<String, String> {
    let _ = path_len;
    // Must create in the same directory as the executable to avoid permission issues
    // Get the executable directory to look for config file
    let exe_path: std::path::PathBuf = env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new(""));

    let game_folder_path = game_folder_path(exe_dir, path, app_id);

    println!("Game folder path: {:?}", game_folder_path);
    println!(
        "Game full path: {:?}",
        game_folder_path.join(executable_name)
    );

    match fs::create_dir_all(&game_folder_path) {
        Ok(_) => {
            println!("Successfully created directory: {:?}", game_folder_path);
        }
        Err(e) => return Err(format!("Failed to create game folder: {}", e)),
    };

    let resource_path = resolve_runner_template(&handle)?;
    println!("Creating dummy game executable from: {:?}", resource_path);

    if is_app_bundle(executable_name) {
        let app_bundle_path = game_folder_path.join(executable_name);
        let bundle_display_name = display_name
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| bundle_binary_name(executable_name));
        let target_executable_path = make_macos_app_bundle(
            &app_bundle_path,
            executable_name,
            &bundle_display_name,
            &resource_path,
        )?;
        return Ok(format!(
            "Dummy app bundle created at: {:?}",
            target_executable_path
        ));
    }

    let target_executable_path = game_folder_path.join(executable_name);
    match fs::copy(&resource_path, &target_executable_path) {
        Ok(_) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&target_executable_path)
                    .map_err(|e| format!("Failed to get file metadata: {}", e))?
                    .permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&target_executable_path, perms)
                    .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
            }

            Ok(format!(
                "Dummy executable copied to: {:?}",
                target_executable_path
            ))
        }
        Err(e) => Err(format!("Failed to copy dummy executable: {}", e)),
    }
}

#[cfg(target_os = "macos")]
fn launch_macos_app_bundle(app_bundle_path: &Path, title: &str) -> Result<(), String> {
    let mut command = std::process::Command::new("open");
    command
        .arg("-n")
        .arg("-g")
        .arg("-a")
        .arg(app_bundle_path)
        .arg("--args")
        .arg("--title")
        .arg(title);

    command
        .spawn()
        .map_err(|e| format!("Failed to launch app bundle with open: {}", e))?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
async fn run_background_process(
    name: &str,
    path: &str,
    executable_name: &str,
    path_len: i64,
    app_id: i64,
) -> Result<String, String> {
    let _ = path_len;
    let exe_path = env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new(""));

    let game_folder_path = game_folder_path(exe_dir, path, app_id);

    if is_app_bundle(executable_name) {
        #[cfg(target_os = "macos")]
        {
            let bundle_path = game_folder_path.join(executable_name);
            launch_macos_app_bundle(&bundle_path, name)?;
            return Ok("App bundle launched successfully".to_string());
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = name;
            return Err("App bundle launches are only supported on macOS".to_string());
        }
    }

    let executable_path = launch_executable_path(&game_folder_path, executable_name);

    let mut cmd = std::process::Command::new(&executable_path);
    cmd.args(["--title", name]).current_dir(game_folder_path);

    // Platform-specific process spawning
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0); // Create new process group on Unix
    }

    match cmd.spawn() {
        Ok(_) => Ok("Process started successfully".to_string()),
        Err(e) => Err(format!("Failed to start process: {}", e)),
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn stop_process(exec_name: String) -> Result<(), String> {
    let process_name = stop_process_name(&exec_name);

    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("taskkill")
            .arg("/F")
            .arg("/IM")
            .arg(&process_name)
            .output()
            .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to stop process: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    #[cfg(target_os = "macos")]
    {
        if is_app_bundle(&exec_name) {
            let bundle_pattern = format!("{}/Contents/MacOS", exec_name);
            let output = std::process::Command::new("pkill")
                .arg("-f")
                .arg(&bundle_pattern)
                .output()
                .map_err(|e| format!("Failed to execute pkill: {}", e))?;

            if output.status.success() {
                return Ok(());
            }
        }

        let output = std::process::Command::new("pkill")
            .arg("-x")
            .arg(&process_name)
            .output()
            .map_err(|e| format!("Failed to execute pkill: {}", e))?;

        if output.status.success() || output.status.code() == Some(1) {
            Ok(())
        } else {
            Err(format!(
                "Failed to stop process: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("pkill")
            .arg("-f")
            .arg(&exec_name)
            .output()
            .map_err(|e| format!("Failed to execute pkill: {}", e))?;

        if output.status.success() || output.status.code() == Some(1) {
            // pkill returns 1 if no processes were killed, which is fine
            Ok(())
        } else {
            Err(format!(
                "Failed to stop process: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

pub async fn reconnect_discord_rpc_preserve_timestamp(
    handle: AppHandle,
    activity_json: String,
) {
    println!("[RPC] Reconnecting to Discord RPC after resume / wake...");
    // Disconnect any existing client cleanly
    let existing_client = {
        let mut client_guard = get_discord_client().lock().unwrap();
        client_guard.take()
    };
    if let Some(client) = existing_client {
        let _ = client.discord.disconnect().await;
    }

    match runner::set_activity(activity_json).await {
        Ok(client) => {
            println!("[RPC] Successfully reconnected to Discord RPC!");
            let mut client_guard = get_discord_client().lock().unwrap();
            *client_guard = Some(client);
            let _ = handle.emit("rpc_reconnected", serde_json::json!({ "status": "ok" }));
        }
        Err(e) => {
            eprintln!("[RPC] Reconnection to Discord RPC failed: {}", e);
            let _ = handle.emit("rpc_error", serde_json::json!({ "error": e }));
        }
    }
}

/// Usage: Calling from JS:
/// ```javascript
/// await invoke('connect_to_discord_rpc_3', json, 'connect' | 'disconnect');
/// ```
#[tauri::command(rename_all = "snake_case")]
fn connect_to_discord_rpc_3(handle: AppHandle, activity_json: String, _action: String) {
    let app = handle.clone();

    let event_connecting = "client_connecting";
    let event_connected = "client_connected";
    let event_disconnect = "event_disconnect";

    let activity = match runner::parse_activity_json(&activity_json) {
        Ok(act) => act,
        Err(e) => {
            eprintln!("Invalid activity JSON: {}", e);
            let _ = handle.emit("rpc_error", serde_json::json!({ "error": e }));
            return;
        }
    };

    let connecting_payload = serde_json::json!({
        "app_id": activity.app_id,
    });

    let _ = {
        let mut client_guard = get_discord_client().lock().unwrap();
        client_guard.take()
    };

    let task = tauri::async_runtime::spawn(async move {
        handle
            .emit(event_connecting, connecting_payload)
            .unwrap_or_else(|e| eprintln!("Failed to emit event: {}", e));

        match runner::set_activity(activity_json).await {
            Ok(client) => {
                let connected_payload = serde_json::json!({
                    "app_id": activity.app_id,
                });

                {
                    let mut client_guard = get_discord_client().lock().unwrap();
                    *client_guard = Some(client);
                }

                handle
                    .emit(event_connected, connected_payload)
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to emit event: {}", e);
                    });
            }
            Err(e) => {
                println!("Failed to set activity: {}", e);
                let _ = handle.emit("rpc_error", serde_json::json!({ "error": e }));
            }
        }

        handle.listen(event_disconnect, move |_| {
            println!("Disconnecting from Discord RPC inner");
            tauri::async_runtime::spawn(async move {
                let client_option = {
                    let mut client_guard = get_discord_client().lock().unwrap();
                    client_guard.take()
                };
                if let Some(client) = client_option {
                    let _ = client.discord.disconnect().await;
                    println!("Disconnected from Discord RPC inner");
                }
            });
        });
    });

    app.listen(event_disconnect, move |_| {
        println!("Disconnecting from Discord RPC...");
        task.abort();
    });
}

#[tauri::command(rename_all = "snake_case")]
async fn fetch_gamelist_gh_mirror() -> tauri::ipc::Response {
    let res = tauri_plugin_http::reqwest::get(
        "https://markterence.github.io/discord-quest-completer/detectable.json",
    )
    .await;
    tauri::ipc::Response::new(res.unwrap().text().await.unwrap())
}

#[tauri::command(rename_all = "snake_case")]
async fn fetch_gamelist_from_discord() -> tauri::ipc::Response {
    let res =
        tauri_plugin_http::reqwest::get("https://discord.com/api/applications/detectable").await;
    tauri::ipc::Response::new(res.unwrap().text().await.unwrap())
}

#[tauri::command(rename_all = "snake_case")]
fn get_tray_settings(handle: AppHandle) -> session::AppSettings {
    session::load_settings(&handle)
}

#[tauri::command(rename_all = "snake_case")]
fn set_minimize_to_tray(handle: AppHandle, enabled: bool) -> session::AppSettings {
    let settings = session::AppSettings {
        minimize_to_tray: enabled,
    };
    session::save_settings(&handle, settings.clone());
    settings
}

#[tauri::command(rename_all = "snake_case")]
fn get_active_session(handle: AppHandle) -> Option<session::SessionState> {
    session::load_persisted_session(&handle)
}

#[tauri::command(rename_all = "snake_case")]
fn start_game_session(
    handle: AppHandle,
    app_id: i64,
    game_name: String,
    executable_name: String,
) -> session::SessionState {
    session::start_session_internal(&handle, app_id, game_name, executable_name)
}

#[tauri::command(rename_all = "snake_case")]
fn stop_game_session(handle: AppHandle) {
    session::end_session_internal(&handle);
}

#[tauri::command(rename_all = "snake_case")]
fn get_session_elapsed() -> Option<i64> {
    session::get_active_session_internal().map(|s| session::calculate_elapsed(s.start_timestamp))
}

#[tauri::command(rename_all = "snake_case")]
fn restore_window(handle: AppHandle) {
    if let Some(window) = handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[tauri::command(rename_all = "snake_case")]
fn hide_window(handle: AppHandle) {
    if let Some(window) = handle.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn set_custom_start_time(
    handle: AppHandle,
    custom_timestamp: i64,
) -> Result<(), String> {
    session::set_custom_rpc_timestamp(Some(custom_timestamp));
    println!(
        "[RPC] Custom start timestamp applied: {} (approx {}s ago)",
        custom_timestamp,
        session::current_unix_timestamp() - custom_timestamp
    );

    // If an active session exists, update Discord RPC immediately
    if let Some(session) = session::get_active_session_internal() {
        let activity_json = serde_json::json!({
            "app_id": session.app_id.to_string(),
            "details": format!("Playing {}", session.game_name),
            "state": "Quest in progress",
            "timestamp": custom_timestamp,
        })
        .to_string();

        reconnect_discord_rpc_preserve_timestamp(handle.clone(), activity_json).await;
    }

    let _ = handle.emit(
        "custom_start_time_changed",
        serde_json::json!({
            "custom_timestamp": custom_timestamp,
        }),
    );

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
async fn reset_custom_start_time(handle: AppHandle) -> Result<(), String> {
    session::set_custom_rpc_timestamp(None);
    println!("[RPC] Custom start timestamp reset to actual session start timestamp");

    // If an active session exists, update Discord RPC with the actual session start timestamp
    if let Some(session) = session::get_active_session_internal() {
        let activity_json = serde_json::json!({
            "app_id": session.app_id.to_string(),
            "details": format!("Playing {}", session.game_name),
            "state": "Quest in progress",
            "timestamp": session.start_timestamp,
        })
        .to_string();

        reconnect_discord_rpc_preserve_timestamp(handle.clone(), activity_json).await;
    }

    let _ = handle.emit(
        "custom_start_time_changed",
        serde_json::json!({
            "custom_timestamp": null,
        }),
    );

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
fn get_custom_start_time() -> Option<i64> {
    session::get_custom_rpc_timestamp()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();
            let _ = session::load_settings(handle);

            if let Err(err) = tray::create_tray(handle) {
                eprintln!("[TRAY] Failed to create system tray: {}", err);
            } else {
                println!("[TRAY] System tray initialized successfully.");
            }

            session::init_power_and_resume_monitor(handle.clone());

            if let Some(session) = session::load_persisted_session(handle) {
                println!(
                    "[SESSION] Restored persisted active game session: {} (app_id: {}, started: {})",
                    session.game_name, session.app_id, session.start_timestamp
                );
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                if session::get_minimize_to_tray(app) {
                    println!("[WINDOW] Close requested: minimizing to system tray.");
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            create_fake_game,
            stop_process,
            connect_to_discord_rpc_3,
            run_background_process,
            fetch_gamelist_gh_mirror,
            fetch_gamelist_from_discord,
            get_tray_settings,
            set_minimize_to_tray,
            get_active_session,
            start_game_session,
            stop_game_session,
            get_session_elapsed,
            restore_window,
            hide_window,
            set_custom_start_time,
            reset_custom_start_time,
            get_custom_start_time,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
