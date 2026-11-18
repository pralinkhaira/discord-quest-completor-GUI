use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

static ACTIVE_SESSION: Mutex<Option<SessionState>> = Mutex::new(None);
static APP_SETTINGS: Mutex<Option<AppSettings>> = Mutex::new(None);
static CUSTOM_RPC_TIMESTAMP: Mutex<Option<i64>> = Mutex::new(None);
static LAST_TICK: AtomicI64 = AtomicI64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub app_id: i64,
    pub game_name: String,
    pub executable_name: String,
    pub start_timestamp: i64,
    pub is_active: bool,
    pub last_sync_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub minimize_to_tray: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            minimize_to_tray: true,
        }
    }
}

pub fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn storage_dir(app: &AppHandle) -> PathBuf {
    let app_data = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("data"));
    let _ = fs::create_dir_all(&app_data);
    app_data
}

fn session_file_path(app: &AppHandle) -> PathBuf {
    storage_dir(app).join("active_session.json")
}

fn settings_file_path(app: &AppHandle) -> PathBuf {
    storage_dir(app).join("settings.json")
}

pub fn load_settings(app: &AppHandle) -> AppSettings {
    let path = settings_file_path(app);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                let mut guard = APP_SETTINGS.lock().unwrap();
                *guard = Some(settings.clone());
                return settings;
            }
        }
    }
    let default = AppSettings::default();
    save_settings(app, default.clone());
    default
}

pub fn save_settings(app: &AppHandle, settings: AppSettings) {
    let path = settings_file_path(app);
    if let Ok(json) = serde_json::to_string_pretty(&settings) {
        let _ = fs::write(path, json);
    }
    let mut guard = APP_SETTINGS.lock().unwrap();
    *guard = Some(settings);
}

pub fn get_minimize_to_tray(app: &AppHandle) -> bool {
    let guard = APP_SETTINGS.lock().unwrap();
    if let Some(ref settings) = *guard {
        settings.minimize_to_tray
    } else {
        drop(guard);
        load_settings(app).minimize_to_tray
    }
}

pub fn load_persisted_session(app: &AppHandle) -> Option<SessionState> {
    let path = session_file_path(app);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(session) = serde_json::from_str::<SessionState>(&content) {
                if session.is_active {
                    let mut guard = ACTIVE_SESSION.lock().unwrap();
                    *guard = Some(session.clone());
                    return Some(session);
                }
            }
        }
    }
    None
}

pub fn persist_session(app: &AppHandle, session: &SessionState) {
    let path = session_file_path(app);
    if let Ok(json) = serde_json::to_string_pretty(session) {
        let _ = fs::write(path, json);
    }
}

pub fn clear_persisted_session(app: &AppHandle) {
    let path = session_file_path(app);
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

pub fn start_session_internal(
    app: &AppHandle,
    app_id: i64,
    game_name: String,
    executable_name: String,
) -> SessionState {
    let now = current_unix_timestamp();
    let session = SessionState {
        session_id: format!("{}-{}", app_id, now),
        app_id,
        game_name,
        executable_name,
        start_timestamp: now,
        is_active: true,
        last_sync_timestamp: now,
    };

    {
        let mut guard = ACTIVE_SESSION.lock().unwrap();
        *guard = Some(session.clone());
    }

    persist_session(app, &session);
    session
}

pub fn get_active_session_internal() -> Option<SessionState> {
    let guard = ACTIVE_SESSION.lock().unwrap();
    guard.clone()
}

pub fn end_session_internal(app: &AppHandle) {
    {
        let mut guard = ACTIVE_SESSION.lock().unwrap();
        *guard = None;
    }
    set_custom_rpc_timestamp(None);
    clear_persisted_session(app);
}

pub fn set_custom_rpc_timestamp(ts: Option<i64>) {
    let mut guard = CUSTOM_RPC_TIMESTAMP.lock().unwrap();
    *guard = ts;
}

pub fn get_custom_rpc_timestamp() -> Option<i64> {
    let guard = CUSTOM_RPC_TIMESTAMP.lock().unwrap();
    *guard
}

pub fn get_effective_rpc_timestamp(actual_start_timestamp: i64) -> i64 {
    get_custom_rpc_timestamp().unwrap_or(actual_start_timestamp)
}

pub fn calculate_elapsed(start_timestamp: i64) -> i64 {
    let now = current_unix_timestamp();
    if now >= start_timestamp {
        now - start_timestamp
    } else {
        0
    }
}

/// Spawns the background monitor that tracks power transitions (sleep/hibernation)
/// by monitoring system wall-clock discontinuities.
pub fn init_power_and_resume_monitor(app: AppHandle) {
    let initial_now = current_unix_timestamp();
    LAST_TICK.store(initial_now, Ordering::SeqCst);

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let now = current_unix_timestamp();
        let previous = LAST_TICK.swap(now, Ordering::SeqCst);

        // If the gap between 1-second ticks exceeds 3 seconds,
        // the host system was suspended, slept, or hibernated!
        let diff = now - previous;
        if diff > 3 {
            let gap_seconds = diff - 1;
            println!(
                "[POWER_EVENT] System resumed from sleep/hibernation! Time jump: +{} seconds",
                gap_seconds
            );

            // Fetch current active session if any
            if let Some(mut session) = get_active_session_internal() {
                let elapsed = calculate_elapsed(session.start_timestamp);
                session.last_sync_timestamp = now;
                persist_session(&app, &session);

                let payload = serde_json::json!({
                    "resumed_at": now,
                    "gap_seconds": gap_seconds,
                    "session_start_timestamp": session.start_timestamp,
                    "elapsed_seconds": elapsed,
                    "game_name": session.game_name,
                    "app_id": session.app_id,
                });

                let _ = app.emit("system_resumed", payload);

                // Auto-sync Discord RPC with effective start timestamp (preserving custom start time if set)
                let app_handle_clone = app.clone();
                let app_id_str = session.app_id.to_string();
                let start_ts = get_effective_rpc_timestamp(session.start_timestamp);
                let details_str = format!("Playing {}", session.game_name);

                tauri::async_runtime::spawn(async move {
                    let activity_json = serde_json::json!({
                        "app_id": app_id_str,
                        "details": details_str,
                        "state": "Quest in progress",
                        "timestamp": start_ts,
                    })
                    .to_string();

                    crate::reconnect_discord_rpc_preserve_timestamp(
                        app_handle_clone,
                        activity_json,
                    )
                    .await;
                });
            } else {
                let payload = serde_json::json!({
                    "resumed_at": now,
                    "gap_seconds": gap_seconds,
                });
                let _ = app.emit("system_resumed", payload);
            }
        }
    });
}

