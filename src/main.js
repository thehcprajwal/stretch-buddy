import { createApp } from 'vue'
import { createVuetify } from 'vuetify'
import { VApp, VMain, VCard, VCardTitle, VBtn, VDivider, VIcon } from 'vuetify/components'
import { Ripple } from 'vuetify/directives'
import 'vuetify/styles'
import '@mdi/font/css/materialdesignicons.css'
import './style.css'
import App from './App.vue'

const vuetify = createVuetify({
  components: { VApp, VMain, VCard, VCardTitle, VBtn, VDivider, VIcon },
  directives: { Ripple },
  theme: {
    defaultTheme: 'light',
    themes: {
      light: {
        colors: {
          primary: '#4CAF50',
          success: '#4CAF50',
          warning: '#FF9800',
          error: '#f44336',
        }
      }
    }
  }
})

createApp(App).use(vuetify).mount('#app')
