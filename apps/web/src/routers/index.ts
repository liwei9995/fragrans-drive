import { LOGIN_URL } from '@/config/config'
import router from '@/routers/router'
import { GlobalStore } from '@/store'

/**
 * @description Navigation guard beforeEach
 * */
router.beforeEach((to) => {
  // Check if current route requires authentication
  if (!to.matched.some((record) => record.meta.requiresAuth)) return true

  const globalStore = GlobalStore()
  if (globalStore.accessToken) return true

  return {
    path: LOGIN_URL,
    query: { redirect: to.fullPath },
  }
})

export default router
