import { describe, expect, it, vi } from 'vitest'
import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import { authLogin, getProfile, updatePassword, updateProfile } from './user'

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
