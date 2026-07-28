import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import type { Login } from '@/api/interface/index'

/**
 * @name 登录模块
 */
// 用户登录接口
export const authLogin = (params: Login.ReqLoginForm) => {
  return http.post<Login.ResLogin>(`${PORT}/auth/login`, params)
}
