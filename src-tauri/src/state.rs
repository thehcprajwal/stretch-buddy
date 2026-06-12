use chrono::Local;
use serde::{Deserialize, Serialize};

pub const TIMER_DEFAULT_SECS: i32 = 50 * 60; // 50 minutes
pub const PAUSE_DEFAULT_SECS: i32 = 120 * 60; // 2 hours

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PauseMode {
    #[default]
    None,
    Manual,
    Meeting,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppState {
    pub timer_secs: i32,
    #[serde(default)]
    pub pause_mode: PauseMode,
    pub pause_secs: i32,
    pub stretches_today: i32,
    pub last_date: String,
    pub notification_shown: bool,
    pub timer_running: bool,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            timer_secs: TIMER_DEFAULT_SECS,
            pause_mode: PauseMode::None,
            pause_secs: 0,
            stretches_today: 0,
            last_date: today_string(),
            notification_shown: false,
            timer_running: true,
        }
    }
}

impl AppState {
    pub fn is_paused(&self) -> bool {
        self.pause_mode != PauseMode::None
    }

    pub fn format_timer(&self) -> String {
        format_secs(self.timer_secs)
    }

    pub fn format_pause(&self) -> String {
        format_secs(self.pause_secs)
    }

    /// Called every second. Returns true if the timer just hit zero.
    pub fn tick(&mut self) -> bool {
        match self.pause_mode {
            PauseMode::Manual => return false,
            PauseMode::Meeting => {
                if self.pause_secs > 0 {
                    self.pause_secs -= 1;
                    if self.pause_secs == 0 {
                        // Meeting expired: reset timer to full interval
                        self.pause_mode = PauseMode::None;
                        self.timer_secs = TIMER_DEFAULT_SECS;
                        self.timer_running = true;
                    }
                }
                return false;
            }
            PauseMode::None => {}
        }

        if !self.timer_running || self.timer_secs <= 0 {
            return false;
        }

        self.timer_secs -= 1;

        if self.timer_secs == 0 {
            self.timer_running = false;
            return true;
        }

        false
    }

    pub fn log_stretch(&mut self) {
        if self.is_paused() {
            return;
        }
        self.stretches_today += 1;
        self.last_date = today_string();
        self.timer_secs = TIMER_DEFAULT_SECS;
        self.timer_running = true;
        self.notification_shown = false;
    }

    /// Toggle manual pause. Keeps timer position on resume.
    pub fn toggle_manual_pause(&mut self) -> &'static str {
        if self.pause_mode == PauseMode::Manual {
            self.pause_mode = PauseMode::None;
            self.timer_running = true;
            "pause_end"
        } else {
            self.pause_mode = PauseMode::Manual;
            self.pause_secs = 0;
            self.timer_running = false;
            "pause_start"
        }
    }

    /// Toggle meeting pause. Resetting timer to full on resume.
    pub fn toggle_meeting(&mut self) -> &'static str {
        if self.pause_mode == PauseMode::Meeting {
            self.pause_mode = PauseMode::None;
            self.pause_secs = 0;
            self.timer_secs = TIMER_DEFAULT_SECS;
            self.timer_running = true;
            self.notification_shown = false;
            "pause_end"
        } else {
            self.pause_mode = PauseMode::Meeting;
            self.pause_secs = PAUSE_DEFAULT_SECS;
            self.timer_running = false;
            "pause_start"
        }
    }

    pub fn reset_timer(&mut self) {
        self.timer_secs = TIMER_DEFAULT_SECS;
        self.timer_running = true;
        self.notification_shown = false;
    }

    pub fn snooze(&mut self, minutes: i32) {
        self.timer_secs = minutes * 60;
        self.timer_running = true;
        self.notification_shown = false;
    }

    pub fn reset_if_new_day(&mut self) {
        let today = today_string();
        if self.last_date != today {
            self.stretches_today = 0;
            self.last_date = today;
        }
    }

    pub fn tray_label(&self) -> String {
        match self.pause_mode {
            PauseMode::Meeting => format!("⏸ {}", self.format_pause()),
            PauseMode::Manual => "⏸ Paused".to_string(),
            PauseMode::None => {
                if self.timer_secs == 0 {
                    "🔴 Stretch!".to_string()
                } else {
                    self.format_timer()
                }
            }
        }
    }
}

pub fn today_string() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn format_secs(total: i32) -> String {
    let total = total.max(0);
    let mins = total / 60;
    let secs = total % 60;
    format!("{:02}:{:02}", mins, secs)
}
