import { defineStore } from 'pinia'
import piniaPersistConfig from '@/config/piniaPersist'
import type { AuthState } from '../interface'

// AuthStore
export const AuthStore = defineStore('AuthState', {
  state: (): AuthState => ({
    // 路由权限列表
    authRouter: [],
  }),
  getters: {
    // 后台返回的菜单数据，用于方便控制路由跳转时权限（这里已经处理成一维数组了）
    dynamicRouter: (state): string[] => {
      return state.authRouter
    },
  },
  actions: {
    // setAuthRouter
    async setAuthRouter(dynamicRouter: string[]) {
      this.authRouter = dynamicRouter
    },
  },
  persist: piniaPersistConfig('AuthState'),
})
