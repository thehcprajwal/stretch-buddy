<template>
  <v-card elevation="4" rounded="lg" class="pa-6">
    <!-- Header -->
    <v-card-title class="text-center text-h5 font-weight-bold pb-2">
      <v-icon color="success" size="28" class="mr-2">mdi-human</v-icon>
      Stretch Buddy
    </v-card-title>

    <v-divider class="mb-4" />

    <!-- Timer Display -->
    <div class="text-center mb-6">
      <div class="timer-display font-weight-bold" :class="timerColorClass">
        {{ displayTime }}
      </div>
      <div class="text-body-2 text-medium-emphasis mt-1">
        {{ timerSubLabel }}
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="d-flex flex-column gap-3 mb-4">
      <v-btn
        color="success"
        size="large"
        block
        :disabled="isAnyPause"
        :class="{ 'pulse-animation': isExpired && !isAnyPause }"
        prepend-icon="mdi-check-circle"
        @click="handleStretched"
      >
        I Stretched!
      </v-btn>

      <!-- Reset timer back to 50:00 -->
      <v-btn
        color="blue-grey"
        size="large"
        block
        variant="outlined"
        prepend-icon="mdi-timer-refresh-outline"
        @click="handleReset"
      >
        Reset Timer
      </v-btn>

      <!-- Pause / Resume (manual, no countdown) -->
      <v-btn
        :color="isManuallyPaused ? 'grey-darken-1' : 'blue-grey'"
        size="large"
        block
        variant="flat"
        :prepend-icon="isManuallyPaused ? 'mdi-play-circle' : 'mdi-pause-circle'"
        @click="handlePause"
      >
        {{ isManuallyPaused ? 'Resume' : 'Pause' }}
      </v-btn>

      <!-- In a Meeting (2-hour auto-resume) -->
      <v-btn
        :color="isMeetingPaused ? 'deep-orange' : 'warning'"
        size="large"
        block
        variant="flat"
        prepend-icon="mdi-microphone-off"
        @click="handleMeeting"
      >
        <template v-if="isMeetingPaused">
          End Meeting ({{ meetingLabel }})
        </template>
        <template v-else>
          In a Meeting
        </template>
      </v-btn>

      <v-btn
        v-if="isExpired && !isAnyPause"
        color="grey"
        size="small"
        block
        variant="outlined"
        prepend-icon="mdi-alarm-snooze"
        @click="handleSnooze"
      >
        Snooze 5 min
      </v-btn>
    </div>

    <v-divider class="mb-3" />

    <!-- Daily Count -->
    <div class="text-center text-body-1 mb-3">
      <v-icon color="success" size="20" class="mr-1">mdi-check</v-icon>
      Today: <strong>{{ stretchesToday }}</strong>
      {{ stretchesToday === 1 ? 'stretch' : 'stretches' }}
    </div>

    <v-divider class="mb-3" />

    <!-- Status -->
    <div class="text-center text-subtitle-2">
      Status:
      <span :class="statusColor" class="font-weight-bold">
        <v-icon :color="statusIconColor" size="16" class="mr-1">{{ statusIcon }}</v-icon>
        {{ statusLabel }}
      </span>
    </div>

    <!-- Hotkey hint -->
    <div class="text-center text-caption text-disabled mt-3">
      ⌘⇧S to log a stretch from anywhere
    </div>
  </v-card>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// ---- State (driven by Tauri backend events) ----
const timerSecs = ref(2400)
const pauseMode = ref('none') // 'none' | 'manual' | 'meeting'
const pauseSecs = ref(0)
const stretchesToday = ref(0)
const timerRunning = ref(true)

let unlisten = null

// ---- Computed ----
const isManuallyPaused = computed(() => pauseMode.value === 'manual')
const isMeetingPaused  = computed(() => pauseMode.value === 'meeting')
const isAnyPause       = computed(() => pauseMode.value !== 'none')

const isExpired = computed(() => timerSecs.value === 0 && !timerRunning.value)

const displayTime = computed(() => {
  if (isMeetingPaused.value) return formatSecs(pauseSecs.value)
  return formatSecs(timerSecs.value)
})

const timerSubLabel = computed(() => {
  if (isMeetingPaused.value) return 'meeting pause remaining'
  if (isManuallyPaused.value) return 'paused'
  return 'time remaining'
})

const timerColorClass = computed(() => {
  if (isAnyPause.value) return 'text-medium-emphasis'
  if (isExpired.value) return 'timer-expired'
  return 'timer-normal'
})

const meetingLabel = computed(() => {
  const total = pauseSecs.value
  const h = Math.floor(total / 3600)
  const m = Math.floor((total % 3600) / 60)
  const s = total % 60
  if (h > 0) return `${h}h ${m}m remaining`
  return `${m}m ${s}s remaining`
})

const statusLabel = computed(() => {
  if (isMeetingPaused.value) return 'In a Meeting'
  if (isManuallyPaused.value) return 'Paused'
  return 'Active'
})

const statusColor = computed(() => {
  if (isMeetingPaused.value) return 'text-deep-orange'
  if (isManuallyPaused.value) return 'text-warning'
  return 'text-success'
})

const statusIconColor = computed(() => {
  if (isMeetingPaused.value) return 'deep-orange'
  if (isManuallyPaused.value) return 'warning'
  return 'success'
})

const statusIcon = computed(() => {
  if (isAnyPause.value) return 'mdi-pause-circle'
  return 'mdi-play-circle'
})

// ---- Helpers ----
function formatSecs(total) {
  const t = Math.max(0, total)
  const m = Math.floor(t / 60)
  const s = t % 60
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

// ---- Lifecycle ----
onMounted(async () => {
  const s = await invoke('get_state')
  applyState(s)

  unlisten = await listen('timer-state', (event) => {
    applyState(event.payload)
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})

function applyState(s) {
  timerSecs.value = s.timer_secs
  pauseMode.value = s.pause_mode
  pauseSecs.value = s.pause_secs
  stretchesToday.value = s.stretches_today
  timerRunning.value = s.timer_running
}

// ---- Actions ----
async function handleStretched() {
  if (isAnyPause.value) return
  await invoke('log_stretch')
}

async function handleReset() {
  await invoke('reset_timer')
}

async function handlePause() {
  await invoke('toggle_pause')
}

async function handleMeeting() {
  await invoke('toggle_meeting')
}

async function handleSnooze() {
  await invoke('snooze', { minutes: 5 })
}
</script>

<style scoped>
.timer-display {
  font-family: 'Roboto Mono', 'Courier New', monospace;
  font-size: 4rem;
  line-height: 1;
  letter-spacing: 0.05em;
  transition: color 0.3s ease;
}

.timer-normal {
  color: #212121;
}

.timer-expired {
  color: #f44336;
}

.text-medium-emphasis {
  color: #757575 !important;
}

.gap-3 {
  gap: 12px;
}

.pulse-animation {
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); box-shadow: 0 0 0 0 rgba(76, 175, 80, 0.4); }
  50% { transform: scale(1.05); box-shadow: 0 0 0 8px rgba(76, 175, 80, 0); }
}
</style>
