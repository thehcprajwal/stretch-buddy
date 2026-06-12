use tauri::State;
use crate::{SharedState, SharedDb};

#[tauri::command]
pub fn get_state(state: State<SharedState>) -> serde_json::Value {
    let s = state.lock().unwrap();
    serde_json::json!({
        "timer_secs": s.timer_secs,
        "pause_mode": s.pause_mode,
        "pause_secs": s.pause_secs,
        "stretches_today": s.stretches_today,
        "timer_running": s.timer_running,
        "notification_shown": s.notification_shown,
    })
}

#[tauri::command]
pub fn log_stretch(state: State<SharedState>, db: State<SharedDb>) -> Result<(), String> {
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        if s.is_paused() {
            return Ok(());
        }
        s.log_stretch();
    }
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        crate::db::log_event(&conn, "stretched", "manual").map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn reset_timer(state: State<SharedState>) -> Result<(), String> {
    state.lock().map_err(|e| e.to_string())?.reset_timer();
    Ok(())
}

#[tauri::command]
pub fn toggle_pause(state: State<SharedState>, db: State<SharedDb>) -> Result<(), String> {
    let action = {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.toggle_manual_pause()
    };
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        crate::db::log_event(&conn, action, "manual").map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_meeting(state: State<SharedState>, db: State<SharedDb>) -> Result<(), String> {
    let action = {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.toggle_meeting()
    };
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        crate::db::log_event(&conn, action, "manual").map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn snooze(state: State<SharedState>, minutes: i32) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.snooze(minutes);
    Ok(())
}
