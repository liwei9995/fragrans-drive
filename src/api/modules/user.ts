import type { Login } from '@/api/interface/index'
import { PORT } from '@/api/config/servicePort'

import http from '@/api'

/**
 * @name 登录模块
 */
// 用户登录接口
export const authLogin = (params: Login.ReqLoginForm) => {
	return http.post(`${PORT}/auth/login`, params) // 正常 post json 请求  ==>  application/json
}

export const authProfile = () => {
	return http.get(`${PORT}/profile`) // 正常 post json 请求  ==>  application/json
}
