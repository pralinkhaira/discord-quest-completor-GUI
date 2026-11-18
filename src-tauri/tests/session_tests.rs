use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionState {
    pub session_id: String,
    pub app_id: i64,
    pub game_name: String,
    pub executable_name: String,
    pub start_timestamp: i64,
    pub is_active: bool,
    pub last_sync_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettings {
    pub minimize_to_tray: bool,
}

pub fn calculate_elapsed(start_timestamp: i64, current_timestamp: i64) -> i64 {
    if current_timestamp >= start_timestamp {
        current_timestamp - start_timestamp
    } else {
        0
    }
}

pub fn detect_power_resume_gap(previous_tick: i64, current_tick: i64) -> Option<i64> {
    let diff = current_tick - previous_tick;
    if diff > 3 {
        Some(diff - 1)
    } else {
        None
    }
}

#[test]
fn test_calculate_elapsed_normal() {
    let start = 1700000000;
    let now = 1700003600; // 1 hour later
    assert_eq!(calculate_elapsed(start, now), 3600);
}

#[test]
fn test_calculate_elapsed_with_simulated_hibernation() {
    // 2 hours before hibernation + 30 minutes hibernated = 2 hours 30 minutes after resume
    let session_start = 1700000000;
    let before_hibernate = session_start + 7200; // 2 hours active
    let hibernate_duration = 1800; // 30 minutes hibernated
    let after_resume_wall_clock = before_hibernate + hibernate_duration; // T0 + 9000

    let elapsed = calculate_elapsed(session_start, after_resume_wall_clock);
    assert_eq!(elapsed, 9000); // 2h 30m = 9000s
}

#[test]
fn test_session_state_serialization_roundtrip() {
    let session = SessionState {
        session_id: "test-session-123".to_string(),
        app_id: 123456789,
        game_name: "Discord Quest Game".to_string(),
        executable_name: "game.exe".to_string(),
        start_timestamp: 1700000000,
        is_active: true,
        last_sync_timestamp: 1700001000,
    };

    let serialized = serde_json::to_string_pretty(&session).expect("serialize");
    let deserialized: SessionState = serde_json::from_str(&serialized).expect("deserialize");

    assert_eq!(deserialized, session);
}

#[test]
fn test_app_settings_serialization() {
    let settings = AppSettings {
        minimize_to_tray: true,
    };
    let json = serde_json::to_string(&settings).expect("serialize");
    let deserialized: AppSettings = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized, settings);
}

#[test]
fn test_power_gap_detection() {
    // Normal 1-second interval
    assert_eq!(detect_power_resume_gap(1000, 1001), None);
    assert_eq!(detect_power_resume_gap(1000, 1002), None);

    // Sleep/hibernation of 45 seconds (tick gap of 46 seconds)
    let gap = detect_power_resume_gap(1000, 1046);
    assert_eq!(gap, Some(45));
}

#[test]
fn test_custom_rpc_start_timestamp_calculation() {
    // User enters 03:00:00 -> duration is 3 * 3600 = 10800 seconds
    let now = 1700010800;
    let duration_seconds = 10800; // 03:00:00
    let custom_start_ts = now - duration_seconds;

    // Discord Rich Presence displays elapsed = now - custom_start_ts
    let elapsed = calculate_elapsed(custom_start_ts, now);
    assert_eq!(elapsed, 10800); // exactly 3 hours (03:00:00)

    // And continues counting: 1 minute later
    let now_plus_1m = now + 60;
    let elapsed_plus_1m = calculate_elapsed(custom_start_ts, now_plus_1m);
    assert_eq!(elapsed_plus_1m, 10860); // 3 hours and 1 minute
}

#[test]
fn test_custom_start_time_separated_from_persisted_session() {
    // Actual session was started 30 minutes ago (1800s)
    let actual_session_start = 1700000000;
    let now = actual_session_start + 1800; // 30 mins active

    let session = SessionState {
        session_id: "test-session".to_string(),
        app_id: 12345,
        game_name: "Test Game".to_string(),
        executable_name: "test.exe".to_string(),
        start_timestamp: actual_session_start,
        is_active: true,
        last_sync_timestamp: now,
    };

    // User applies custom start time of 03:00:00 (10800s ago)
    let custom_duration = 10800;
    let custom_start_ts = now - custom_duration;

    // Ensure session.start_timestamp is COMPLETELY SEPARATE and untouched
    assert_ne!(session.start_timestamp, custom_start_ts);
    assert_eq!(session.start_timestamp, actual_session_start);

    // When custom is applied:
    let mut custom_override: Option<i64> = Some(custom_start_ts);
    let effective_rpc_with_custom = custom_override.unwrap_or(session.start_timestamp);
    assert_eq!(effective_rpc_with_custom, custom_start_ts);

    // When reset is clicked, restores actual session start timestamp:
    custom_override = None;
    let effective_rpc_after_reset = custom_override.unwrap_or(session.start_timestamp);
    assert_eq!(effective_rpc_after_reset, session.start_timestamp);
}
