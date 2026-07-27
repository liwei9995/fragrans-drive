import { describe, it, expect, vi } from 'vitest'
import emitter from './index'

describe('emitter', () => {
  it('is a mitt instance', () => {
    expect(emitter).toBeDefined()
    expect(typeof emitter.on).toBe('function')
    expect(typeof emitter.emit).toBe('function')
    expect(typeof emitter.off).toBe('function')
  })

  it('can emit and receive events', () => {
    const mockCallback = vi.fn()
    emitter.on('test-event', mockCallback)
    
    emitter.emit('test-event', { payload: 123 })
    expect(mockCallback).toHaveBeenCalledWith({ payload: 123 })
    
    emitter.off('test-event', mockCallback)
    emitter.emit('test-event', { payload: 456 })
    expect(mockCallback).toHaveBeenCalledTimes(1)
  })
})
