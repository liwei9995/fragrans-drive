import { authRefresh } from '@/api/modules/user'
import { LOGIN_URL } from '@/config/config'
import router from '@/routers/router'
import { GlobalStore } from '@/store'

let isInitialAuthChecked = false

/**
 * @description Navigation guard beforeEach
 * */
router.beforeEach(async (to) => {
  // Check if current route requires authentication
  if (!to.matched.some((record) => record.meta.requiresAuth)) return true

  const globalStore = GlobalStore()
  if (globalStore.accessToken) return true

  // Attempt silent refresh once on initial load / F5 refresh
  if (!isInitialAuthChecked) {
    isInitialAuthChecked = true
    try {
      const res = await authRefresh()
      if (res?.data?.access_token) {
        globalStore.setAccessToken(res.data.access_token)
        return true
      }
    } catch {
      // Refresh failed, proceed to login redirect
    }
  }

  return {
    path: LOGIN_URL,
    query: { redirect: to.fullPath },
  }
})

export default router
