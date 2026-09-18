import { describe, expect, it, vi } from 'vitest'
import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import {
  authLogin,
  getAuthConfig,
  getCaptcha,
  getProfile,
  registerUser,
  resetPassword,
  sendEmailCode,
  updatePassword,
  updateProfile,
} from './user'

vi.mock('@/api', () => ({
  default: {
    get: vi.fn(),
    post: vi.fn(),
    patch: vi.fn(),
  },
}))

describe('user module api', () => {
  it('authLogin', () => {
    const params = { email: 'test@example.com', password: 'pwd' }
    authLogin(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/auth/login`, params)
  })

  it('getAuthConfig', () => {
    getAuthConfig()
    expect(http.get).toHaveBeenCalledWith(`${PORT}/auth/config`)
  })

  it('getCaptcha', () => {
    getCaptcha()
    expect(http.get).toHaveBeenCalledWith(`${PORT}/auth/captcha`)
  })

  it('sendEmailCode', () => {
    const params = {
      email: 'user@example.com',
      purpose: 'register' as const,
      captchaId: 'cid-1',
      captchaCode: '1234',
    }
    sendEmailCode(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/auth/send-code`, params)
  })

  it('registerUser', () => {
    const params = {
      email: 'user@example.com',
      password: 'pwd',
      firstName: 'A',
      lastName: 'B',
      captchaId: 'cid-1',
      captchaCode: '1234',
    }
    registerUser(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/users`, params)
  })

  it('resetPassword', () => {
    const params = {
      email: 'user@example.com',
      code: '123456',
      password: 'new_password',
      changePassword: 'new_password',
    }
    resetPassword(params)
    expect(http.post).toHaveBeenCalledWith(
      `${PORT}/auth/reset-password`,
      params,
    )
  })

  it('getProfile', () => {
    getProfile()
    expect(http.get).toHaveBeenCalledWith(`${PORT}/profile`)
  })

  it('updateProfile', () => {
    const params = { firstName: 'Alex', lastName: 'Li' }
    updateProfile(params)
    expect(http.patch).toHaveBeenCalledWith(`${PORT}/profile`, params)
  })

  it('updatePassword', () => {
    const params = {
      oldPassword: 'currentSecret',
      password: 'newSecret123',
      changePassword: 'newSecret123',
    }
    updatePassword(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/users/password`, params)
  })
})
