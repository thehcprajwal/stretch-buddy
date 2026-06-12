# Stretch Buddy PWA - Build Specification

## Project Overview
A Progressive Web App (PWA) that reminds users to stretch every 40 minutes during their workday. Users can pause reminders during meetings. Built with Vue 3, Vuetify, and Vite.

**Target**: Prototype/exploration phase. All state is client-side initially (LocalStorage). No backend yet.

---

## Technology Stack
- **Frontend Framework**: Vue 3 (Composition API)
- **UI Component Library**: Vuetify 3
- **Build Tool**: Vite
- **PWA Support**: vite-plugin-pwa
- **State Management**: Vue Composition API (reactive refs)
- **Storage**: LocalStorage (for persisting stretch count across sessions)
- **Notifications**: Browser Notification API

---

## Core Features

### 1. Timer System
- **Default interval**: 40 minutes (2400 seconds)
- **Display format**: MM:SS (e.g., "23:45")
- **Behavior when countdown reaches 0:00**:
  - Timer pauses at 0:00
  - Timer display turns red (warning color)
  - "I Stretched!" button pulses (animation)
  - Browser notification fires with title "Time to Stretch!"
  - Timer remains at 0:00 until user clicks "I Stretched!"

### 2. Stretch Logging
- **Action**: User clicks "I Stretched!" button
- **Behavior**:
  - Timer resets to 40:00
  - Timer color returns to normal
  - Pulsing animation stops
  - Stretch count increments by 1
  - Notification dismisses
  - Event synced to LocalStorage (timestamp + today's date)

### 3. Meeting Pause
- **Action**: User clicks "In a Meeting" button
- **Behavior**:
  - Timer pauses (stops counting down)
  - Pause duration: 2 hours (7200 seconds)
  - "In a Meeting" button shows countdown of remaining pause time (e.g., "In a Meeting (1h 58m remaining)")
  - "I Stretched!" button is disabled/hidden while paused
  - Pause countdown is visible and updates every second
  - When pause expires, timer resumes from wherever it was

### 4. Daily Stretch Count
- **Display**: "Today: X stretches" shown below action buttons
- **Reset**: Count resets at midnight (local timezone)
- **Persistence**: Stored in LocalStorage with date key

### 5. Status Indicator
- **Display**: Shows "Active" (when timer running) or "Paused" (when in meeting)
- **Location**: Bottom of main card
- **Color coding**: Green for Active, Orange/Yellow for Paused

---

## UI/UX Design

### Wireframe Layout
```
┌─────────────────────────────────┐
│     Stretch Buddy               │  (Header - Title)
├─────────────────────────────────┤
│                                 │
│         45:23                   │  (Timer - Large, monospace font)
│     (time remaining)            │
│                                 │
├─────────────────────────────────┤
│                                 │
│    ┌─────────────────────────┐  │
│    │   ✓ I Stretched!        │  │  (Primary button - Green)
│    └─────────────────────────┘  │  (Pulses when timer = 0:00)
│                                 │
│    ┌─────────────────────────┐  │
│    │  🔇 In a Meeting        │  │  (Secondary button - Orange)
│    │  (1h 58m remaining)     │  │
│    └─────────────────────────┘  │
│                                 │
├─────────────────────────────────┤
│  Today: 7 stretches ✓           │  (Stat line)
├─────────────────────────────────┤
│  Status: Active                 │  (Status - Green or Orange)
└─────────────────────────────────┘
```

### Visual States

**Normal State (Timer Running)**
- Timer: Black text on white background
- Timer size: Large (maybe 3-4rem font)
- "I Stretched!" button: Green (v-btn color="success")
- "In a Meeting" button: Orange/Warning (v-btn color="warning")
- Both buttons: Enabled

**Timer at 0:00 (Stretch Overdue)**
- Timer: Red text (color="error")
- "I Stretched!" button: Green with pulsing animation (pulse every 1 second, scale 1.0 to 1.1)
- Browser notification: Fires automatically
- All other elements unchanged

**In Meeting State (Paused)**
- Timer: Grayed out (still visible but not updating)
- Timer display: Shows remaining pause time
- "I Stretched!" button: Disabled (opacity 0.5, no click)
- "In a Meeting" button: Orange with pause countdown displayed
- Status: Shows "Paused"
- Timer does NOT count down

---

## Component Structure

### Single Main Component: `StretchReminder.vue`

**Reactive State (Composition API)**
```javascript
// Timer state
const timerMinutes = ref(40)
const timerSeconds = ref(0)
const timerRunning = ref(true)

// Meeting pause state
const isPaused = ref(false)
const pauseMinutes = ref(0)
const pauseSeconds = ref(0)

// Stretch tracking
const stretchestoday = ref(0)
const lastStretchDate = ref(null) // Track date for reset at midnight

// Notification state
const notificationShown = ref(false)
```

**Key Methods**
- `startTimerInterval()` - Runs every 1 second, decrements timer
- `startPauseInterval()` - Runs every 1 second when paused, decrements pause countdown
- `handleStretched()` - Called when "I Stretched!" clicked
- `handleMeeting()` - Called when "In a Meeting" clicked
- `resetDailyCountIfNeeded()` - Check if date changed, reset if so
- `requestNotificationPermission()` - Asks browser for notification access on mount
- `showNotification()` - Triggers browser notification when timer hits 0
- `saveToLocalStorage()` - Persists state
- `loadFromLocalStorage()` - Restores state on mount
- `formatTime(min, sec)` - Returns "MM:SS" string
- `computeTimerColor()` - Returns color based on state (normal/error/warning)
- `isTimerExpired()` - Returns true if minutes === 0 && seconds === 0

**Watchers**
- Watch `timerSeconds` to trigger notification when timer hits 0
- Watch date to reset `stretchestoday` at midnight

---

## Styling with Vuetify

### Color Scheme
- **Primary action (I Stretched!)**: Green (`color="success"`)
- **Secondary action (In Meeting)**: Orange (`color="warning"`)
- **Timer at 0:00**: Red (`color="error"`)
- **Status Active**: Green
- **Status Paused**: Orange

### Component Styling
- **Card**: v-card with elevation and padding
- **Buttons**: v-btn with size="large", full width
- **Typography**: 
  - Timer: `text-h3` or larger, monospace font, centered
  - "Today" stat: `text-body1`, centered
  - Status: `text-subtitle2`, centered, color-coded

### Animation - Pulse Effect
When timer reaches 0:00, "I Stretched!" button should pulse:
```css
@keyframes pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.1); }
}

.pulse-animation {
  animation: pulse 1s infinite;
}
```

---

## Browser Notifications

**When to show**: Timer reaches 0:00
**Title**: "Time to Stretch!"
**Body**: "You've been working for 40 minutes. Stand up and stretch!"
**Icon**: Use app icon from PWA manifest
**Auto-dismiss**: After user clicks "I Stretched!" (notification closes)
**Permission**: Request on app first load

---

## LocalStorage Schema

```javascript
// Key: "stretch-buddy-state"
{
  "stretchestoday": 7,
  "lastStretchDate": "2025-02-24", // YYYY-MM-DD format
  "timerMinutes": 23,
  "timerSeconds": 45,
  "isPaused": false,
  "pauseMinutes": 0,
  "pauseSeconds": 0
}

// Key: "stretch-buddy-history" (for future analytics)
[
  { "timestamp": 1708776543210, "action": "stretched" },
  { "timestamp": 1708776542100, "action": "pause_start" },
  { "timestamp": 1708793742100, "action": "pause_end" }
]
```

---

## PWA Configuration (vite-plugin-pwa)

**manifest.json fields**
```json
{
  "name": "Stretch Buddy",
  "short_name": "Stretch",
  "description": "Health reminder to stretch every 40 minutes",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#ffffff",
  "theme_color": "#4CAF50",
  "icons": [
    {
      "src": "/icon-192.png",
      "sizes": "192x192",
      "type": "image/png"
    },
    {
      "src": "/icon-512.png",
      "sizes": "512x512",
      "type": "image/png"
    }
  ]
}
```

**Service Worker**: Auto-generated by vite-plugin-pwa, handles:
- Offline caching of app shell
- Background sync (optional, not needed for prototype)

---

## Data Flow

1. **App Mount**:
   - Load from LocalStorage
   - Request notification permission
   - Start timer interval
   - Start pause interval (if paused)

2. **Every second**:
   - If not paused: decrement timer
   - If paused: decrement pause countdown
   - Save state to LocalStorage

3. **Timer reaches 0:00**:
   - Stop timer interval
   - Show notification
   - Turn timer red
   - Pulse "I Stretched!" button

4. **User clicks "I Stretched!"**:
   - Reset timer to 40:00
   - Increment `stretchestoday`
   - Stop pulsing animation
   - Restore normal colors
   - Restart timer interval
   - Dismiss notification
   - Save to LocalStorage

5. **User clicks "In a Meeting"**:
   - Set `isPaused = true`
   - Set `pauseMinutes = 120`, `pauseSeconds = 0`
   - Disable "I Stretched!" button
   - Start pause countdown interval
   - Save to LocalStorage

6. **Pause countdown reaches 0:00**:
   - Set `isPaused = false`
   - Resume timer countdown
   - Enable "I Stretched!" button
   - Save to LocalStorage

---

## Edge Cases to Handle

1. **Midnight reset**: If user keeps app open past midnight, `stretchestoday` should reset
2. **Browser notification permission denied**: App still works, just no notifications
3. **Tab loses focus**: Timer still runs (important for PWA)
4. **User closes/refreshes app**: State persists via LocalStorage
5. **Multiple stretches while timer at 0:00**: Only count once (one click = one stretch)
6. **User clicks "In a Meeting" while timer at 0:00**: Pause takes priority, timer stays red until unpause

---

## Testing Checklist (Manual)

- [ ] Timer counts down every second correctly
- [ ] Timer resets to 40:00 after "I Stretched!" click
- [ ] Stretch count increments correctly
- [ ] Notification fires when timer hits 0:00
- [ ] Timer turns red at 0:00
- [ ] "I Stretched!" button pulses at 0:00
- [ ] "In a Meeting" button pauses timer
- [ ] Pause countdown shows and decrements
- [ ] "I Stretched!" button disabled during pause
- [ ] Pause expires and timer resumes
- [ ] State persists after page refresh
- [ ] App works offline (no errors)
- [ ] Notification permission request shows on first load
- [ ] Can add to home screen (PWA install prompt)

---

## Future Phases (Not for this prototype)

1. **Backend sync**: Node.js + SQLite to persist stretch history
2. **Analytics dashboard**: Weekly/monthly stretch statistics
3. **Android app**: Companion notification on phone
4. **Calendar integration**: Auto-detect meetings and auto-pause
5. **Customizable interval**: Allow users to change 40-min default
6. **Themes**: Dark mode, custom colors

---

## File Structure (Once built)

```
stretch-buddy/
├── src/
│   ├── components/
│   │   └── StretchReminder.vue
│   ├── App.vue
│   ├── main.js
│   └── style.css
├── public/
│   ├── icon-192.png
│   └── icon-512.png
├── vite.config.js
├── index.html
└── package.json
```

---

## Questions for Claude Code Build

When building, ask yourself:
- Is the timer updating every second smoothly?
- Does the pulsing animation feel natural (not too fast/slow)?
- Is the LocalStorage persisting correctly across refreshes?
- Are the button states (enabled/disabled) working as expected?
- Does the notification appear at the right time?
- Is the UI responsive on mobile browser view?