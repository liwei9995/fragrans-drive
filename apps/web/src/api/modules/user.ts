import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import type { Auth, Login, User } from '@/api/interface/index'

/**
 * @name User and authentication module
 */
// User login endpoint
export const authLogin = (params: Login.ReqLoginForm) => {
  return http.post<Login.ResLogin>(`${PORT}/auth/login`, params)
}

// Get auth configuration (registration allowed, email verification, captcha required, etc.)
export const getAuthConfig = () => {
  return http.get<Auth.AuthConfig>(`${PORT}/auth/config`)
}

// Get captcha SVG endpoint
export const getCaptcha = () => {
  return http.get<Auth.CaptchaData>(`${PORT}/auth/captcha`)
}

// Send email verification code (requires captcha solution first)
export const sendEmailCode = (params: Auth.ReqSendEmailCode) => {
  return http.post<{ message: string }>(`${PORT}/auth/send-code`, params)
}

// User registration endpoint
export const registerUser = (params: Auth.ReqRegister) => {
  return http.post<{ id: string }>(`${PORT}/users`, params)
}

// Password recovery/reset endpoint
export const resetPassword = (params: Auth.ReqResetPassword) => {
  return http.post<{ message: string }>(`${PORT}/auth/reset-password`, params)
}

// Get current user profile endpoint
export const getProfile = () => {
  return http.get<User.UserProfile>(`${PORT}/profile`)
}

// Update current user profile endpoint
export const updateProfile = (params: User.UpdateProfileParams) => {
  return http.patch<User.UserProfile>(`${PORT}/profile`, params)
}

// Update password (authenticated state)
export const updatePassword = (params: User.UpdatePasswordParams) => {
  return http.post<void>(`${PORT}/users/password`, params)
}
