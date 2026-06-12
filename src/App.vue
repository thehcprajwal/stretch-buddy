<template>
  <v-app class="popover-app">
    <v-main>
      <StretchReminder />
    </v-main>
  </v-app>
</template>

<script setup>
import StretchReminder from './components/StretchReminder.vue'
</script>

<style>
/* Make the app flush with the frameless window edges */
html, body, #app {
  width: 380px;
  height: 480px;
  overflow: hidden;
  background: transparent;
}

.popover-app {
  width: 380px !important;
  min-height: unset !important;
}
</style>
