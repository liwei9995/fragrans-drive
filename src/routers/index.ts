import { AxiosCanceler } from '@/api/helper/axiosCancel'
import { authProfile } from '@/api/modules/user'
import { ADMIN_URL, HOME_URL } from '@/config/config'
import router from '@/routers/router'
import { GlobalStore } from '@/store'

const axiosCancel = new AxiosCanceler()

/**
 * @description 路由拦截 beforeEach（路由配置无数种方法，个人觉得最简便）
 * */
router.beforeEach(async (to, _from, next) => {
  // 在跳转路由之前，清除所有的请求
  axiosCancel.removeAllPending()

  // 判断当前路由是否需要访问权限
  if (!to.matched.some((record) => record.meta.requiresAuth)) return next()

  // 判断是否有 Token
  const globalStore = GlobalStore()
  const token = globalStore.token

  if (token) {
    if (to.path === '/') {
      next({ path: HOME_URL })
    } else if (to.path === '/login') {
      // if is logged in, redirect to the home page
      next({ path: HOME_URL })
    } else {
      try {
        const { roles } = (await authProfile()) as { roles: string[] }
        const staticRouter = [HOME_URL]
        const roleList = ([] as string[]).concat(roles)
        const routerList = roleList.includes('admin')
          ? staticRouter.concat(ADMIN_URL)
          : staticRouter

        if (routerList.indexOf(to.path) !== -1 || to.matched.length > 0)
          return next()

        next({ path: '/403' })
      } catch (error) {
        console.log('permission error', error)
        next(`/login?redirect=${to.path}`)
      }
    }
  } else {
    next(`/login?redirect=${to.path}`)
  }
})

export default router
