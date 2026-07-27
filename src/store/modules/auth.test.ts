import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { AuthStore } from './auth'

describe('AuthStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initial state', () => {
    const store = AuthStore()
    expect(store.authRouter).toEqual([])
  })

  it('getters: dynamicRouter', () => {
    const store = AuthStore()
    store.authRouter = ['/home', '/about']
    expect(store.dynamicRouter).toEqual(['/home', '/about'])
  })

  it('actions: setAuthRouter', async () => {
    const store = AuthStore()
    await store.setAuthRouter(['/dashboard'])
    expect(store.authRouter).toEqual(['/dashboard'])
  })
})
