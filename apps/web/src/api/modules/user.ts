import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import type { Auth, Login, User } from '@/api/interface/index'

/**
 * @name 用户与认证模块
 */
// 用户登录接口
export const authLogin = (params: Login.ReqLoginForm) => {
  return http.post<Login.ResLogin>(`${PORT}/auth/login`, params)
}

// 获取认证配置（是否允许注册、邮箱验证、图形验证码等）
export const getAuthConfig = () => {
  return http.get<Auth.AuthConfig>(`${PORT}/auth/config`)
}

// 获取图形验证码
export const getCaptcha = () => {
  return http.get<Auth.CaptchaData>(`${PORT}/auth/captcha`)
}

// 发送邮箱验证码（需先通过图形验证码）
export const sendEmailCode = (params: Auth.ReqSendEmailCode) => {
  return http.post<{ message: string }>(`${PORT}/auth/send-code`, params)
}

// 用户注册接口
export const registerUser = (params: Auth.ReqRegister) => {
  return http.post<{ id: string }>(`${PORT}/users`, params)
}

// 找回密码/重置密码
export const resetPassword = (params: Auth.ReqResetPassword) => {
  return http.post<{ message: string }>(`${PORT}/auth/reset-password`, params)
}

// 获取当前用户个人资料
export const getProfile = () => {
  return http.get<User.UserProfile>(`${PORT}/profile`)
}

// 修改当前用户个人资料
export const updateProfile = (params: User.UpdateProfileParams) => {
  return http.patch<User.UserProfile>(`${PORT}/profile`, params)
}

// 修改密码（已登录状态）
export const updatePassword = (params: User.UpdatePasswordParams) => {
  return http.post<void>(`${PORT}/users/password`, params)
}
