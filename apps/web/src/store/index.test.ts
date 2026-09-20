import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { GlobalStore } from './index'

describe('GlobalStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initial state', () => {
    const store = GlobalStore()
    expect(store.accessToken).toBe('')
    expect(store.refreshToken).toBe('')
  })

  it('setTokens updates both tokens', () => {
    const store = GlobalStore()
    store.setTokens('access', 'refresh')
    expect(store.accessToken).toBe('access')
    expect(store.refreshToken).toBe('refresh')
  })

  it('setAccessToken updates only access token', () => {
    const store = GlobalStore()
    store.setTokens('access', 'refresh')
    store.setAccessToken('next-access')
    expect(store.accessToken).toBe('next-access')
    expect(store.refreshToken).toBe('refresh')
  })

  it('setUserInfo and setAvatar update userInfo state', () => {
    const store = GlobalStore()
    expect(store.userInfo).toBeNull()
    store.setUserInfo({
      id: 'u1',
      email: 'alex@example.com',
      firstName: 'Alex',
      lastName: 'Li',
      avatar: 'https://example.com/avatar.png',
    })
    expect(store.userInfo?.firstName).toBe('Alex')
    expect(store.userInfo?.avatar).toBe('https://example.com/avatar.png')

    store.setAvatar('https://example.com/new-avatar.png')
    expect(store.userInfo?.avatar).toBe('https://example.com/new-avatar.png')
  })

  it('setLanguage updates language state and logout preserves language', () => {
    const store = GlobalStore()
    store.setTokens('access', 'refresh')
    store.setUserInfo({
      id: 'u1',
      email: 'alex@example.com',
      firstName: 'Alex',
      lastName: 'Li',
      avatar: '',
    })
    store.setLanguage('en')
    expect(store.language).toBe('en')

    store.logout()
    expect(store.accessToken).toBe('')
    expect(store.refreshToken).toBe('')
    expect(store.userInfo).toBeNull()
    expect(store.language).toBe('en')
  })
})
