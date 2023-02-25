import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

const staticRouter: RouteRecordRaw[] = [
	{
		path: '/',
		redirect: { name: 'home' }
	},
	{
		path: '/home/:id?',
		name: 'home',
		component: () => import('@/views/home/index.vue'),
		meta: {
			requiresAuth: true,
			title: '首页',
			key: 'home'
		}
	},
	{
		path: '/login',
		name: 'login',
		component: () => import('@/views/login/index.vue'),
		meta: {
			requiresAuth: false,
			title: '登录',
			key: 'login'
		}
	}
]

/**
 * errorRouter(错误页面路由)
 */
export const errorRouter = [
	{
		path: '/403',
		name: '403',
		component: () => import('@/components/ErrorMessage/403.vue'),
		meta: {
			title: '403页面'
		}
	},
	{
		path: '/404',
		name: '404',
		component: () => import('@/components/ErrorMessage/404.vue'),
		meta: {
			title: '404页面'
		}
	},
	{
		path: '/500',
		name: '500',
		component: () => import('@/components/ErrorMessage/500.vue'),
		meta: {
			title: '500页面'
		}
	},
	// 解决刷新页面，路由警告
	{
		path: '/:pathMatch(.*)*',
		component: () => import('@/components/ErrorMessage/404.vue')
	}
]
const router = createRouter({
	history: createWebHashHistory(),
	routes: [...staticRouter, ...errorRouter],
	strict: false,
	// 切换页面，滚动到最顶部
	scrollBehavior: () => ({ left: 0, top: 0 })
})

export default router
