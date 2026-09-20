import { createPinia, defineStore } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import piniaPersistConfig from '@/config/piniaPersist'
import i18n, { getStoredLanguage, LANGUAGE_KEY } from '@/languages'
import type { GlobalState } from './interface'

// defineStore returns a function that can be called to obtain the store instance
export const GlobalStore = defineStore('GlobalState', {
  // state: function that returns the state object
  state: (): GlobalState => ({
    accessToken: '',
    userInfo: null,
    language: getStoredLanguage(),
  }),
  getters: {},
  actions: {
    setAccessToken(token: string) {
      this.accessToken = token
    },
    setUserInfo(userInfo: GlobalState['userInfo']) {
      this.userInfo = userInfo
    },
    setAvatar(avatar: string) {
      if (this.userInfo) {
        this.userInfo.avatar = avatar
      }
    },
    setLanguage(language: 'zh' | 'en') {
      this.language = language
      try {
        window.localStorage.setItem(LANGUAGE_KEY, language)
      } catch {
        // ignore
      }
      ;(i18n.global.locale as any).value = language
    },
    logout() {
      this.accessToken = ''
      this.userInfo = null
    },
  },
  persist: piniaPersistConfig('GlobalState'),
})

// piniaPersist (persistence)
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

export default pinia
