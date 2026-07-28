import { describe, expect, it } from 'vitest'
import piniaPersistConfig from './piniaPersist'

describe('piniaPersistConfig', () => {
  it('returns correctly configured persistence options', () => {
    const config = piniaPersistConfig('test-key')
    expect(config.key).toBe('test-key')
    expect(config.storage).toBe(window.localStorage)
  })
})
