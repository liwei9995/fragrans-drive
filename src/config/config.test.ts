import { describe, it, expect } from 'vitest'
import { HOME_URL, ADMIN_URL, LOGIN_URL, ALLOW_LIST } from './config'

describe('config', () => {
  it('has correct constants', () => {
    expect(HOME_URL).toBe('/home')
    expect(ADMIN_URL).toBe('/admin')
    expect(LOGIN_URL).toBe('/login')
    expect(ALLOW_LIST).toEqual(['/login', '/register'])
  })
})
