import { describe, it, expect, vi } from 'vitest'
import { authLogin, authProfile } from './user'
import http from '@/api'
import { PORT } from '@/api/config/servicePort'

vi.mock('@/api', () => ({
  default: {
    post: vi.fn(),
    get: vi.fn(),
  }
}))

describe('user module api', () => {
  it('authLogin', () => {
    const params = { username: 'test', password: 'pwd' } as any
    authLogin(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/auth/login`, params)
  })

  it('authProfile', () => {
    authProfile()
    expect(http.get).toHaveBeenCalledWith(`${PORT}/profile`)
  })
})
