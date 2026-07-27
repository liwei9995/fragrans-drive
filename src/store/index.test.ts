import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { GlobalStore } from './index'

describe('GlobalStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initial state', () => {
    const store = GlobalStore()
    expect(store.token).toBe('')
    expect(store.userInfo).toEqual({})
    expect(store.assemblySize).toBe('default')
    expect(store.isMobile).toBe(false)
  })

  it('setToken updates token', () => {
    const store = GlobalStore()
    store.setToken('new-token')
    expect(store.token).toBe('new-token')
  })

  it('setUserInfo updates userInfo', () => {
    const store = GlobalStore()
    store.setUserInfo({ name: 'test' })
    expect(store.userInfo).toEqual({ name: 'test' })
  })

  it('setIsMobile updates isMobile', () => {
    const store = GlobalStore()
    store.setIsMobile(true)
    expect(store.isMobile).toBe(true)
  })
})
