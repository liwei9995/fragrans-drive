import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import type { Login, User } from '@/api/interface/index'

/**
 * @name 用户模块
 */
// 用户登录接口
export const authLogin = (params: Login.ReqLoginForm) => {
  return http.post<Login.ResLogin>(`${PORT}/auth/login`, params)
}

// 获取当前用户个人资料
export const getProfile = () => {
  return http.get<User.UserProfile>(`${PORT}/profile`)
}

// 修改当前用户个人资料
export const updateProfile = (params: User.UpdateProfileParams) => {
  return http.patch<User.UserProfile>(`${PORT}/profile`, params)
}

// 修改密码
export const updatePassword = (params: User.UpdatePasswordParams) => {
  return http.post<void>(`${PORT}/users/password`, params)
}
