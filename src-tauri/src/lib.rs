mod commands;
mod db;
mod state;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

pub type SharedState = Arc<Mutex<state::AppState>>;
pub type SharedDb = Arc<Mutex<rusqlite::Connection>>;

#[derive(Clone, serde::Serialize)]
struct TimerEvent {
    timer_secs: i32,
    pause_mode: String, // "none" | "manual" | "meeting"
    pause_secs: i32,
    stretches_today: i32,
    timer_running: bool,
    notification_shown: bool,
}

fn spawn_timer(
    app: AppHandle,
    shared_state: SharedState,
    shared_db: SharedDb,
    today_item: MenuItem<tauri::Wry>,
    pause_item: MenuItem<tauri::Wry>,
    meeting_item: MenuItem<tauri::Wry>,
) {
    thread::spawn(move || {
        let mut tick_count: u32 = 0;
        loop {
            thread::sleep(Duration::from_secs(1));
            tick_count = tick_count.wrapping_add(1);

            let just_expired = {
                let mut s = match shared_state.lock() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                s.reset_if_new_day();
                s.tick()
            };

            if just_expired {
                if let Ok(conn) = shared_db.lock() {
                    let _ = db::log_event(&conn, "timer_expired", "auto");
                }
                show_notification(&app, shared_state.clone(), shared_db.clone());
                if let Ok(mut s) = shared_state.lock() {
                    s.notification_shown = true;
                }
            }

            let (event, tray_label, today_label, pause_label, meeting_label) = {
                let s = match shared_state.lock() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let count = s.stretches_today;
                let today = format!(
                    "Today: {} {}",
                    count,
                    if count == 1 { "stretch" } else { "stretches" }
                );
                use state::PauseMode;
                let pause = match s.pause_mode {
                    PauseMode::Manual => "Resume",
                    _ => "Pause",
                };
                let meeting = match s.pause_mode {
                    PauseMode::Meeting => "End Meeting",
                    _ => "In a Meeting",
                };
                let mode_str = match s.pause_mode {
                    PauseMode::None => "none",
                    PauseMode::Manual => "manual",
                    PauseMode::Meeting => "meeting",
                };
                (
                    TimerEvent {
                        timer_secs: s.timer_secs,
                        pause_mode: mode_str.to_string(),
                        pause_secs: s.pause_secs,
                        stretches_today: s.stretches_today,
                        timer_running: s.timer_running,
                        notification_shown: s.notification_shown,
                    },
                    s.tray_label(),
                    today,
                    pause,
                    meeting,
                )
            };

            if let Some(tray) = app.tray_by_id("sb-tray") {
                let _ = tray.set_title(Some(&tray_label));
            }
            let _ = today_item.set_text(today_label);
            let _ = pause_item.set_text(pause_label);
            let _ = meeting_item.set_text(meeting_label);

            let _ = app.emit("timer-state", &event);

            if tick_count % 10 == 0 {
                if let (Ok(s), Ok(conn)) = (shared_state.lock(), shared_db.lock()) {
                    let _ = db::save_state(&conn, &s);
                }
            }
        }
    });
}

fn show_notification(app: &AppHandle, shared_state: SharedState, shared_db: SharedDb) {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        thread::spawn(move || {
            // display alert gives us real action buttons and blocks until the user responds.
            // Runs in its own thread so the timer thread keeps ticking.
            // Gives up after 2 minutes if ignored.
            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(r#"set dlg to display alert "Time to Stretch!" message "You've been working for 50 minutes. Stand up and stretch!" as informational buttons {"Snooze 5 min", "I Stretched!"} default button "I Stretched!" giving up after 120
if gave up of dlg then
  return ""
end if
return button returned of dlg"#)
                .output();

            if let Ok(out) = output {
                let choice = String::from_utf8_lossy(&out.stdout).trim().to_string();
                match choice.as_str() {
                    "I Stretched!" => {
                        shared_state.lock().unwrap().log_stretch();
                        let conn = shared_db.lock().unwrap();
                        let _ = db::log_event(&conn, "stretched", "notification");
                    }
                    "Snooze 5 min" => {
                        shared_state.lock().unwrap().snooze(5);
                    }
                    _ => {}
                }
            }
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (shared_state, shared_db);
        use tauri_plugin_notification::NotificationExt;
        let _ = app
            .notification()
            .builder()
            .title("Time to Stretch!")
            .body("You've been working for 50 minutes. Stand up and stretch!")
            .show();
    }
}

fn toggle_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let visible = win.is_visible().unwrap_or(false);
        if visible {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            // macOS: remove from Dock, live only in menu bar
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Database
            let db_path = app
                .path()
                .app_data_dir()
                .expect("app data dir")
                .join("stretch-buddy.db");
            std::fs::create_dir_all(db_path.parent().unwrap())?;
            let conn = db::init(db_path.to_str().unwrap())?;

            // Request notification permission on first run
            {
                use tauri_plugin_notification::NotificationExt;
                let _ = app.notification().request_permission();
            }

            // Load persisted state or start fresh
            let app_state = db::load_state(&conn).unwrap_or_default();
            let shared_state: SharedState = Arc::new(Mutex::new(app_state));
            let shared_db: SharedDb = Arc::new(Mutex::new(conn));

            app.manage(shared_state.clone());
            app.manage(shared_db.clone());

            // System tray right-click menu
            let open_item    = MenuItem::with_id(app, "open",    "Open Stretch Buddy",  true,  None::<&str>)?;
            let reset_item   = MenuItem::with_id(app, "reset",   "Reset Timer",         true,  None::<&str>)?;
            let pause_item   = MenuItem::with_id(app, "pause",   "Pause",               true,  None::<&str>)?;
            let meeting_item = MenuItem::with_id(app, "meeting", "In a Meeting",        true,  None::<&str>)?;
            let today_item   = MenuItem::with_id(app, "today",   "Today: 0 stretches",  false, None::<&str>)?;
            let sep1         = PredefinedMenuItem::separator(app)?;
            let sep2         = PredefinedMenuItem::separator(app)?;
            let sep3         = PredefinedMenuItem::separator(app)?;
            let quit_item    = MenuItem::with_id(app, "quit",    "Quit Stretch Buddy",  true,  None::<&str>)?;

            let tray_menu = Menu::with_items(app, &[
                &open_item, &sep1,
                &reset_item, &pause_item, &meeting_item, &sep2,
                &today_item, &sep3,
                &quit_item,
            ])?;

            TrayIconBuilder::with_id("sb-tray")
                .tooltip("Stretch Buddy — click to open")
                .title("40:00")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => toggle_window(app),
                    "reset" => {
                        let state = app.state::<SharedState>();
                        state.lock().unwrap().reset_timer();
                    }
                    "pause" => {
                        let state = app.state::<SharedState>();
                        let db = app.state::<SharedDb>();
                        let action = state.lock().unwrap().toggle_manual_pause();
                        let conn = db.lock().unwrap();
                        let _ = db::log_event(&conn, action, "tray");
                    }
                    "meeting" => {
                        let state = app.state::<SharedState>();
                        let db = app.state::<SharedDb>();
                        let action = state.lock().unwrap().toggle_meeting();
                        let conn = db.lock().unwrap();
                        let _ = db::log_event(&conn, action, "tray");
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Global shortcut: CMD+SHIFT+S → log stretch
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };
                let state_ref = shared_state.clone();
                let db_ref = shared_db.clone();
                let shortcut =
                    Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyS);
                app.handle().global_shortcut().on_shortcut(
                    shortcut,
                    move |_app, _sc, event| {
                        if event.state == ShortcutState::Pressed {
                            let mut s = state_ref.lock().unwrap();
                            s.log_stretch();
                            drop(s);
                            let conn = db_ref.lock().unwrap();
                            let _ = db::log_event(&conn, "stretched", "hotkey");
                        }
                    },
                )?;
            }

            // Auto-start at login (enable by default)
            {
                use tauri_plugin_autostart::ManagerExt;
                let _ = app.autolaunch().enable();
            }

            // Start timer background thread
            spawn_timer(app.handle().clone(), shared_state, shared_db, today_item, pause_item, meeting_item);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::log_stretch,
            commands::reset_timer,
            commands::toggle_pause,
            commands::toggle_meeting,
            commands::snooze,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
