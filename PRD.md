# Stretch Buddy — Product Requirements Document

**Owner**: Prajwal HC  
**Status**: Phase 1 shipped  
**Stack**: Tauri 2 (Rust + Vue 3), SQLite  
**Audience**: Personal use

---

## Problem

Sitting at a desk for long stretches without moving causes back pain, eye strain, and fatigue. Browser-based reminder apps require a tab open and stop working if the browser is killed. A native daemon solves this properly — it runs at login, lives in the menu bar, and has access to macOS system APIs that browsers can't touch (idle detection, calendar, camera, screen lock).

---

## Goal

A macOS menu bar daemon that:
1. Runs silently in the background at all times
2. Knows when you're actually working (vs idle, locked, in a meeting)
3. Reminds you to stretch at the right moment — not during a presentation, not when you stepped away
4. Costs zero cognitive overhead: one click or one hotkey to log a stretch and move on

---

## Non-goals

- No cloud sync (personal tool, all data stays local)
- No mobile app (Mac only for now)
- No team features (solo use)
- No web dashboard (analytics live in the app)

---

## Phases

---

### Phase 1 — Core Daemon ✅ Done

**What was built:**

| Feature | Detail |
|---|---|
| Menu bar countdown | Live `MM:SS` in menu bar, color-coded (normal / red when expired / ⏸ when paused) |
| Native notifications | `osascript` on macOS — works for unsigned builds, always appears on top |
| Control panel | Frameless Vue popover: timer, buttons, daily count, status |
| "I Stretched!" | Resets timer, increments daily count, logs to SQLite |
| **Pause** | Indefinite manual pause; timer resumes from same position |
| **In a Meeting** | 2-hour auto-expiring pause; timer **resets to 50:00** on resume or expiry |
| Snooze 5 min | Delays next reminder by 5 minutes (shown only when timer expired) |
| Global hotkey | `⌘⇧S` logs a stretch from any app |
| Tray right-click menu | Pause, In a Meeting, Today count, Quit — all with live labels |
| SQLite persistence | State saved every 10s; resumes exactly where you left off after restart |
| Auto-start | LaunchAgent installed at first run via `tauri-plugin-autostart` |
| No Dock icon | `ActivationPolicy::Accessory` — lives only in menu bar |

**Pause state machine (`PauseMode` enum in `state.rs`):**

| Mode | Timer on resume | Auto-expires |
|---|---|---|
| `None` | — (running) | — |
| `Manual` | Continues from paused position | No |
| `Meeting` | Resets to 50:00 | Yes, after 2 hours |

**Architecture:**
- Rust timer thread emits `timer-state` events every second (includes `pause_mode: "none" | "manual" | "meeting"`)
- Vue frontend is purely reactive — zero `setInterval`, all state from Rust; only used Vuetify components imported (tree-shaken)
- `timer_expired` logged to SQLite on every expiry — powers Phase 4 compliance analytics
- SQLite at `~/Library/Application Support/com.stretchbuddy.app/stretch-buddy.db`

**Build and install:**
```bash
source ~/.cargo/env && cargo tauri build
killall "Stretch Buddy"
cp -r "src-tauri/target/release/bundle/macos/Stretch Buddy.app" /Applications/
open "/Applications/Stretch Buddy.app"
```

---

### Phase 2 — Smart Auto-Pause

**Goal**: The timer pauses itself. You never need to tap "In a Meeting" for common cases.

| Feature | Implementation |
|---|---|
| **Idle detection** | `CGEventSourceSecondsSinceLastEventType` via IOKit — pause after N min of no input, resume on activity |
| **Screen sleep / lid close** | `NSWorkspace` `didScreenSleepNotification` / `didWakeNotification` via `objc` crate |
| **Foreground process detection** | `osascript` or `active-win` — if Zoom/Teams/Meet/Webex/FaceTime/Discord is frontmost, auto-pause |
| **macOS Focus mode** | Detect active Focus profile; configurable: "pause during DND" |
| **Calendar.app** | `EventKit` via Swift helper or AppleScript — read current events, auto-pause if "busy" event is active |

**Settings window** (first real preferences UI):
- Idle threshold (default: 5 min)
- Quiet hours (default: before 9am / after 7pm)
- Weekend mode on/off
- Timer interval (20 / 30 / 40 / 60 min or custom)
- Meeting pause duration
- Enabled detection methods (toggle each)

**UX change**: "In a Meeting" button becomes a **manual override**, not the primary mechanism. The button label changes to "Force Meeting Pause" to reflect this.

---

### Phase 3 — Multi-Stream Health

**Goal**: One daemon, multiple independent health reminder streams.

| Stream | Default Interval | Notification |
|---|---|---|
| Stretch | 50 min | "Time to Stretch!" |
| Eyes (20-20-20) | 20 min | "Look 20 feet away for 20 seconds" |
| Water | 60 min | "Drink a glass of water" |
| Walk | 90 min | "Take a short walk" |
| Posture check | 30 min | "Check your posture" |

Each stream:
- Independent countdown timer in Rust
- Can be enabled/disabled per stream in settings
- Own daily count shown in the popover
- Own SQLite `reminder_type` column in the events table (already schema'd)
- Own icon in the notification

**Guided stretch panel**: Clicking the stretch notification opens a small floating panel with:
- Name of a specific stretch (e.g., "Neck rolls", "Shoulder shrug", "Hip flexor")
- Diagram or description
- 30-second countdown — press done to log

Stretch exercises rotate through a curated list of 10-12 exercises.

**Notification action buttons** (Phase 3):
- "I Stretched!" directly on the macOS notification (no need to open app)
- "Snooze 5 min" on the notification
- These require registering `UNNotificationCategory` with action identifiers

---

### Phase 4 — Analytics & Streaks

**Goal**: Make the data you've been collecting meaningful.

**Streak tracking**:
- Consecutive days with at least 1 stretch
- Shown in menu bar tooltip: "🔥 14-day streak"
- Broken if 0 stretches logged on any calendar day

**Compliance rate**:
- (stretches logged / stretch opportunities that day) × 100
- "Today: 78% compliance"
- Opportunity = every time the timer hit 0:00

**Response time**:
- How long after the timer expired did you actually stretch?
- Average response time per day/week

**SQLite queries that power this** (events table already captures everything needed):

```sql
-- Daily streak
SELECT date(timestamp/1000, 'unixepoch', 'localtime') as day,
       COUNT(*) as stretches
FROM events WHERE action = 'stretched'
GROUP BY day ORDER BY day DESC;

-- Compliance rate today
SELECT
  (SELECT COUNT(*) FROM events WHERE action = 'stretched' AND date(timestamp/1000,'unixepoch','localtime') = date('now','localtime')) * 100.0 /
  NULLIF((SELECT COUNT(*) FROM events WHERE action = 'timer_expired' AND date(timestamp/1000,'unixepoch','localtime') = date('now','localtime')), 0)
  AS compliance_pct;
```

**Analytics window** (opens from tray right-click menu):
- GitHub-style contribution heatmap (12 weeks)
- Weekly bar chart
- Streak counter with flame icon
- Personal records: best day, best week, longest streak
- Per-stream breakdown (stretch vs water vs eyes)

**Achievement system**:
| Achievement | Trigger |
|---|---|
| First stretch | First ever stretch logged |
| 7-day streak | 7 consecutive days |
| 30-day streak | 30 consecutive days |
| Century | 100 lifetime stretches |
| Perfect week | >90% compliance 5 days in a row |
| Early bird | Stretch before 9am 3 days in a row |

Achievements shown as a brief notification when unlocked.

---

## Technical decisions

### Why Tauri over Swift
- Keep Vue 3 frontend (existing knowledge)
- Rust handles all macOS system calls via FFI
- ~15MB app vs 120MB Electron
- Tauri's plugin ecosystem covers all Phase 1-2 needs out of the box

### Why SQLite over UserDefaults / JSON
- Event log is append-only and grows indefinitely → SQLite handles this better than a single JSON file
- Phase 4 analytics queries are trivial SQL, not JavaScript array operations
- Single file, easy to inspect with any SQLite browser

### Why no cloud sync
- This is a personal health tool — data is private
- Adds complexity (auth, server, schema migrations) without adding value for solo use
- If team features are added later, iCloud CloudKit is the right sync layer for a macOS app

---

## Data model

```
AppState (persisted as JSON blob in app_state table)
├── timer_secs: i32         # seconds remaining on the stretch timer
├── pause_mode: PauseMode   # "none" | "manual" | "meeting"
├── pause_secs: i32         # seconds remaining on pause
├── stretches_today: i32    # resets at midnight
├── last_date: String       # YYYY-MM-DD — used to detect midnight
├── notification_shown: bool
└── timer_running: bool

events (append-only log)
├── id: INTEGER PRIMARY KEY
├── timestamp: INTEGER      # unix ms
├── action: TEXT            # 'stretched' | 'pause_start' | 'pause_end' | 'timer_expired' (logged) | 'idle_start' | 'idle_end' (Phase 2)
├── reminder_type: TEXT     # 'stretch' | 'water' | 'eyes' | 'walk' | 'posture'
└── source: TEXT            # 'manual' | 'hotkey' | 'calendar' | 'idle' | 'auto'
```

---

## Testing checklist

### Phase 1
- [ ] App appears in menu bar on launch, not in Dock
- [ ] Menu bar shows live countdown, updates every second
- [ ] Clicking tray icon opens/closes popover
- [ ] Timer counts down to 0:00, turns red, "I Stretched!" pulses
- [ ] "I Stretched!" resets timer and increments daily count
- [ ] **Pause** stops timer; tray shows `⏸ Paused`; Resume continues from same position
- [ ] **In a Meeting** starts 2-hour countdown; tray shows `⏸ MM:SS`; End Meeting resets timer to 50:00
- [ ] Meeting pause auto-expires after 2 hours and resets timer to 50:00
- [ ] Tray right-click menu: Pause/Resume and In a Meeting/End Meeting labels update live
- [ ] Pause and meeting modes are mutually exclusive
- [ ] `⌘⇧S` logs a stretch from another app (e.g., while in Xcode)
- [ ] State survives app restart (open popover, wait 30s, quit, reopen — timer should resume)
- [ ] App launches automatically on login
- [ ] SQLite file exists at `~/Library/Application Support/com.stretchbuddy.app/stretch-buddy.db`
- [ ] Daily count resets to 0 the next day (or test by setting `last_date` to yesterday in DB)

### Phase 2 (planned)
- [ ] Timer auto-pauses after 5 min of no keyboard/mouse input
- [ ] Timer auto-resumes when activity detected
- [ ] Timer pauses when Mac sleeps, resumes on wake
- [ ] Timer auto-pauses when Zoom is fullscreen
- [ ] Timer auto-pauses when Calendar event marked "busy" is active
- [ ] Settings window opens from tray right-click menu
- [ ] Quiet hours respected (no notification before 9am / after 7pm)
