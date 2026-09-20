// element plus

import axios from 'axios'
import { createApp } from 'vue'
import App from './App.vue'
// reset style sheet
import '@/styles/reset.scss'
// app style sheet
import './style.css'
// element css
import 'element-plus/theme-chalk/el-message-box.css'
import 'element-plus/theme-chalk/el-message.css'
import 'element-plus/theme-chalk/el-notification.css'
import 'element-plus/theme-chalk/el-overlay.css'
// element dark (built-in dark mode)
import 'element-plus/theme-chalk/dark/css-vars.css'
// element breakpoint-based display helper classes
import 'element-plus/theme-chalk/display.css'
// i18n
import i18n from '@/languages/index'
// vue Router
import router from '@/routers/index'
// pinia store
import pinia, { GlobalStore } from '@/store/index'

const app = createApp(App)
app.use(pinia)
const globalStore = GlobalStore()
try {
  // Remove tokens persisted by older releases while retaining the language.
  window.localStorage.setItem(
    'GlobalState',
    JSON.stringify({ language: globalStore.language }),
  )
} catch {
  /* storage unavailable */
}
;(i18n.global.locale as any).value = globalStore.language

async function bootstrap() {
  try {
    const { data } = await axios.post<{ access_token: string }>(
      `${import.meta.env.VITE_API_URL}/v1/auth/refresh`,
      {},
      { withCredentials: true, timeout: 10000 },
    )
    globalStore.setAccessToken(data.access_token)
  } catch {
    /* No active session. */
  }
  app.use(router).use(i18n)
  app.mount('#app')
}
void bootstrap()
