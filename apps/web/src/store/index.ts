import { createPinia, defineStore } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import piniaPersistConfig from '@/config/piniaPersist'
import type { GlobalState } from './interface'

// defineStore returns a function that can be called to obtain the store instance
export const GlobalStore = defineStore('GlobalState', {
  // state: function that returns the state object
  state: (): GlobalState => ({
    accessToken: '',
    refreshToken: '',
    userInfo: null,
  }),
  getters: {},
  actions: {
    setTokens(accessToken: string, refreshToken: string) {
      this.accessToken = accessToken
      this.refreshToken = refreshToken
    },
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
  },
  persist: piniaPersistConfig('GlobalState'),
})

// piniaPersist (persistence)
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

export default pinia
