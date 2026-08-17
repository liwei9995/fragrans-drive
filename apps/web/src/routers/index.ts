import { LOGIN_URL } from '@/config/config'
import router from '@/routers/router'
import { GlobalStore } from '@/store'

/**
 * @description 路由拦截 beforeEach（路由配置无数种方法，个人觉得最简便）
 * */
router.beforeEach((to) => {
  // 判断当前路由是否需要访问权限
  if (!to.matched.some((record) => record.meta.requiresAuth)) return true

  const globalStore = GlobalStore()
  if (globalStore.token) return true

  return {
    path: LOGIN_URL,
    query: { redirect: to.fullPath },
  }
})

export default router
