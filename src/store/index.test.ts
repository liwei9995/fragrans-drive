import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { GlobalStore } from './index'

describe('GlobalStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initial state', () => {
    const store = GlobalStore()
    expect(store.token).toBe('')
  })

  it('setToken updates token', () => {
    const store = GlobalStore()
    store.setToken('new-token')
    expect(store.token).toBe('new-token')
  })
})
