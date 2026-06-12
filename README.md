# Stretch Buddy

A macOS menu bar daemon that reminds you to stretch every 50 minutes. Lives in your menu bar, runs at login, never needs a browser.

Built with **Tauri 2** (Rust backend + Vue 3 frontend).

---

## What it does

- Shows a live countdown in your menu bar (`50:00 → 00:00`)
- Fires a native macOS notification when it's time to stretch
- Click the menu bar icon to open/close the control panel
- `⌘⇧S` anywhere to log a stretch without opening the app
- Two distinct pause modes — quick pause and meeting mode (see below)
- Snooze 5 min when the timer expires (shown only at that moment)
- Reset Timer button to restart the countdown early
- Tracks your daily stretch count and logs everything to a local SQLite database
- Starts at login, no Dock icon

---

## Pause modes

| Mode | Trigger | Timer on resume |
|---|---|---|
| **Pause** | Click "Pause" button or tray → Pause | Resumes from where it was |
| **In a Meeting** | Click "In a Meeting" button or tray → In a Meeting | Resets to 50:00 |

Meeting mode has a 2-hour countdown and auto-expires. Pause mode is indefinite until you click Resume. Both are mutually exclusive — switching to one cancels the other.

---

## Stack

| Layer | Tech |
|---|---|
| UI | Vue 3 + Vuetify 3 (tree-shaken — only used components imported) |
| Backend | Rust (Tauri 2) |
| Persistence | SQLite via `rusqlite` (bundled) |
| Notifications | `osascript` on macOS (works for unsigned builds) |
| Global hotkeys | `tauri-plugin-global-shortcut` |
| Auto-start | `tauri-plugin-autostart` → LaunchAgent |

---

## Project structure

```
stretch-buddy/
├── src/                          # Vue frontend
│   ├── components/
│   │   └── StretchReminder.vue   # Main UI — driven by backend events
│   ├── App.vue                   # Popover shell
│   ├── main.js                   # Vue + Vuetify bootstrap
│   └── style.css
├── src-tauri/                    # Rust backend (the daemon)
│   ├── src/
│   │   ├── lib.rs                # App setup: tray, timer thread, shortcuts
│   │   ├── main.rs               # Entry point
│   │   ├── state.rs              # Timer state machine
│   │   ├── db.rs                 # SQLite: save/load state, event log
│   │   └── commands.rs           # Tauri commands exposed to Vue
│   ├── capabilities/
│   │   └── default.json          # Plugin permissions
│   ├── Cargo.toml
│   └── tauri.conf.json           # Window config, plugins
├── vite.config.js
└── package.json
```

---

## Getting started

### Prerequisites

- macOS 12+
- Node.js 18+
- Rust (installed automatically or via `rustup`)
- Xcode Command Line Tools (`xcode-select --install`)

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Install Tauri CLI

```bash
cargo install tauri-cli --version "^2"
```

### Run in development

```bash
npm install
cargo tauri dev
```

The menu bar icon appears with a live countdown. The popover opens when you click it.

### Build for production

```bash
source ~/.cargo/env && cargo tauri build
```

Output `.app` is in `src-tauri/target/release/bundle/macos/`.

### Install / reinstall

```bash
# Stop the running instance first
killall "Stretch Buddy"

# Copy new build to Applications
cp -r "src-tauri/target/release/bundle/macos/Stretch Buddy.app" /Applications/

# Launch
open "/Applications/Stretch Buddy.app"
```

If `killall` reports no process found, the app wasn't running — safe to ignore.

---

## Architecture

The app has no `setInterval` in JavaScript. All timer logic lives in Rust:

```
Rust timer thread (every 1s)
  → updates AppState (Arc<Mutex<AppState>>)
  → emits `timer-state` event via AppHandle
  → updates tray icon title + menu item labels
  → logs `timer_expired` to SQLite when countdown hits zero

Vue frontend
  → listens to `timer-state` events via Tauri's listen()
  → calls invoke('log_stretch') / invoke('toggle_pause') / invoke('toggle_meeting') / invoke('reset_timer') / invoke('snooze')
  → Rust handles all state mutations
```

State is persisted to SQLite every 10 seconds and on every user action. On restart, the daemon resumes exactly where it left off.

### Pause state machine

```
PauseMode::None    — timer running normally
PauseMode::Manual  — timer frozen at current position; resumes from same position
PauseMode::Meeting — 2-hour countdown; on expiry or manual end, timer resets to 50:00
```

`toggle_pause()` and `toggle_meeting()` are mutually exclusive — switching to one clears the other.

### SQLite schema

```sql
-- Persisted app state
CREATE TABLE app_state (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL       -- JSON blob of AppState
);

-- Event history (for Phase 4 analytics)
CREATE TABLE events (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp     INTEGER NOT NULL,   -- unix ms
  action        TEXT NOT NULL,      -- 'stretched', 'pause_start', 'pause_end', 'timer_expired'
  reminder_type TEXT DEFAULT 'stretch',
  source        TEXT DEFAULT 'manual'  -- 'manual', 'hotkey', 'calendar', 'idle'
);
```

Database lives at: `~/Library/Application Support/com.stretchbuddy.app/stretch-buddy.db`

---

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `⌘⇧S` | Log a stretch (works from any app) |
| Click tray icon | Toggle popover open/close |
| Right-click tray icon | Open menu: Pause, In a Meeting, Today count, Quit |

---

## Configuration

Default intervals (in `src-tauri/src/state.rs`):

```rust
pub const TIMER_DEFAULT_SECS: i32 = 50 * 60;   // 50 minutes
pub const PAUSE_DEFAULT_SECS: i32 = 120 * 60;  // 2 hours (meeting pause)
```

A settings UI is planned for Phase 2 — these will become user-configurable.

---

## Roadmap

See [PRD.md](PRD.md) for the full product roadmap.

| Phase | Status |
|---|---|
| Phase 1 — Core daemon | ✅ Done |
| Phase 2 — Smart auto-pause (idle, sleep, calendar) | Planned |
| Phase 3 — Multi-stream health (water, eyes, walk) | Planned |
| Phase 4 — Analytics & streaks | Planned |
