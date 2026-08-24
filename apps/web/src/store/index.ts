import { createPinia, defineStore } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import piniaPersistConfig from '@/config/piniaPersist'
import type { GlobalState } from './interface'

// defineStore 调用后返回一个函数，调用该函数获得 Store 实体
export const GlobalStore = defineStore('GlobalState', {
  // state: 返回对象的函数
  state: (): GlobalState => ({
    accessToken: '',
    refreshToken: '',
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
  },
  persist: piniaPersistConfig('GlobalState'),
})

// piniaPersist(持久化)
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

export default pinia
