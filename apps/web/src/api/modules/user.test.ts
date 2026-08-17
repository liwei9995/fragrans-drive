import { describe, expect, it, vi } from 'vitest'
import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import { authLogin } from './user'

vi.mock('@/api', () => ({
  default: {
    post: vi.fn(),
  },
}))

describe('user module api', () => {
  it('authLogin', () => {
    const params = { email: 'test@example.com', password: 'pwd' }
    authLogin(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/auth/login`, params)
  })
})
