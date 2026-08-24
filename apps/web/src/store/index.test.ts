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
})
