import { describe, it, expect, vi, beforeEach } from 'vitest'
import { AxiosCanceler, getPendingUrl } from './axiosCancel'
import axios from 'axios'
import qs from 'qs'

describe('axiosCancel', () => {
  let canceler: AxiosCanceler

  beforeEach(() => {
    canceler = new AxiosCanceler()
    canceler.reset()
  })

  it('getPendingUrl returns formatted url', () => {
    const config = {
      method: 'GET',
      url: '/api',
      data: { id: 1 },
      params: { q: 'search' }
    }
    const url = getPendingUrl(config)
    expect(url).toBe(`GET&/api&${qs.stringify({ id: 1 })}&${qs.stringify({ q: 'search' })}`)
  })

  it('addPending sets cancel token and adds to map', () => {
    const config: any = { url: '/test', method: 'GET' }
    canceler.addPending(config)
    expect(config.cancelToken).toBeDefined()
  })

  it('removePending cancels specific request', () => {
    const cancelFn = vi.fn()
    // @ts-ignore
    axios.CancelToken = class {
      constructor(cb: any) {
        cb(cancelFn)
      }
    }
    
    const config: any = { url: '/test-remove', method: 'GET' }
    canceler.addPending(config)
    
    canceler.removePending(config)
    expect(cancelFn).toHaveBeenCalled()
  })

  it('removeAllPending cancels all requests', () => {
    const cancelFn1 = vi.fn()
    const cancelFn2 = vi.fn()
    let calls = 0
    // @ts-ignore
    axios.CancelToken = class {
      constructor(cb: any) {
        calls++
        cb(calls === 1 ? cancelFn1 : cancelFn2)
      }
    }

    const config1: any = { url: '/test1', method: 'GET' }
    const config2: any = { url: '/test2', method: 'POST' }

    canceler.addPending(config1)
    canceler.addPending(config2)

    canceler.removeAllPending()
    
    expect(cancelFn1).toHaveBeenCalled()
    expect(cancelFn2).toHaveBeenCalled()
  })
})
