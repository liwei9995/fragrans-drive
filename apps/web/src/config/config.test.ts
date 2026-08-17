import { describe, expect, it } from 'vitest'
import { HOME_URL, LOGIN_URL } from './config'

describe('config', () => {
  it('has correct constants', () => {
    expect(HOME_URL).toBe('/home')
    expect(LOGIN_URL).toBe('/login')
  })
})
