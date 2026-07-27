import { describe, it, expect, vi, beforeEach } from 'vitest'
import RequestHttp from './index'
import { GlobalStore } from '@/store'
import router from '@/routers'
import { ElMessage } from 'element-plus'
import { checkStatus } from './helper/checkStatus'
import { setActivePinia, createPinia } from 'pinia'

vi.mock('element-plus', () => ({
  ElMessage: {
    error: vi.fn()
  }
}))

vi.mock('@/routers', () => ({
  default: {
    replace: vi.fn()
  }
}))

vi.mock('./helper/checkStatus', () => ({
  checkStatus: vi.fn()
}))

vi.mock('./helper/axiosCancel', () => {
  return {
    AxiosCanceler: class {
      addPending = vi.fn()
      removePending = vi.fn()
      removeAllPending = vi.fn()
      reset = vi.fn()
    }
  }
})

describe('api index', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    Object.defineProperty(window.navigator, 'onLine', {
      value: true,
      writable: true
    })
  })

  it('request methods', () => {
    vi.spyOn(RequestHttp.service, 'get').mockResolvedValue('get')
    vi.spyOn(RequestHttp.service, 'post').mockResolvedValue('post')
    vi.spyOn(RequestHttp.service, 'put').mockResolvedValue('put')
    vi.spyOn(RequestHttp.service, 'delete').mockResolvedValue('delete')

    RequestHttp.get('/url')
    expect(RequestHttp.service.get).toHaveBeenCalledWith('/url', { params: undefined })

    RequestHttp.post('/url', { a: 1 })
    expect(RequestHttp.service.post).toHaveBeenCalledWith('/url', { a: 1 }, {})

    RequestHttp.put('/url', { a: 1 })
    expect(RequestHttp.service.put).toHaveBeenCalledWith('/url', { a: 1 }, {})

    RequestHttp.delete('/url')
    expect(RequestHttp.service.delete).toHaveBeenCalledWith('/url', { params: undefined })

    RequestHttp.download('/url', { a: 1 })
    expect(RequestHttp.service.get).toHaveBeenCalledWith('/url', { a: 1, responseType: 'blob' })
  })

  it('request interceptor adds token', async () => {
    const store = GlobalStore()
    store.setToken('test-token')
    
    // @ts-ignore
    const requestInterceptor = RequestHttp.service.interceptors.request.handlers[0]
    
    const config = { headers: new Map() }
    config.headers.set = vi.fn()
    
    const res = await requestInterceptor.fulfilled(config)
    expect(config.headers.set).toHaveBeenCalledWith('Authorization', 'Bearer test-token')
  })

  it('request interceptor handles error', async () => {
    // @ts-ignore
    const requestInterceptor = RequestHttp.service.interceptors.request.handlers[0]
    await expect(requestInterceptor.rejected(new Error('req err'))).rejects.toThrow('req err')
  })

  it('response interceptor handles 401', async () => {
    // @ts-ignore
    const responseInterceptor = RequestHttp.service.interceptors.response.handlers[0]
    const store = GlobalStore()
    store.setToken('old-token')

    const response = {
      status: 401,
      data: { message: 'unauthorized' },
      config: {}
    }

    await expect(responseInterceptor.fulfilled(response)).rejects.toEqual(response.data)
    expect(ElMessage.error).toHaveBeenCalledWith('unauthorized')
    expect(store.token).toBe('')
    expect(router.replace).toHaveBeenCalledWith({ path: '/login' })
  })

  it('response interceptor handles other errors', async () => {
    // @ts-ignore
    const responseInterceptor = RequestHttp.service.interceptors.response.handlers[0]

    const response = {
      status: 400,
      data: { message: 'bad request' },
      config: {}
    }

    await expect(responseInterceptor.fulfilled(response)).rejects.toEqual(response.data)
    expect(ElMessage.error).toHaveBeenCalledWith('bad request')
  })

  it('response interceptor handles success', async () => {
    // @ts-ignore
    const responseInterceptor = RequestHttp.service.interceptors.response.handlers[0]

    const response = {
      status: 200,
      data: { result: 'ok' },
      config: {}
    }

    const data = await responseInterceptor.fulfilled(response)
    expect(data).toEqual(response.data)
  })

  it('response interceptor handles reject timeout', async () => {
    // @ts-ignore
    const responseInterceptor = RequestHttp.service.interceptors.response.handlers[0]

    const error = {
      message: 'timeout of 10000ms exceeded'
    }

    await expect(responseInterceptor.rejected(error)).rejects.toEqual(error)
    expect(ElMessage.error).toHaveBeenCalledWith('请求超时！请您稍后重试')
  })

  it('response interceptor handles reject with response', async () => {
    // @ts-ignore
    const responseInterceptor = RequestHttp.service.interceptors.response.handlers[0]

    const error = {
      message: 'network err',
      response: { status: 500 }
    }

    await expect(responseInterceptor.rejected(error)).rejects.toEqual(error)
    expect(checkStatus).toHaveBeenCalledWith(500)
  })

  it('response interceptor handles reject offline', async () => {
    // @ts-ignore
    const responseInterceptor = RequestHttp.service.interceptors.response.handlers[0]

    Object.defineProperty(window.navigator, 'onLine', {
      value: false,
      writable: true
    })

    const error = {
      message: 'network err'
    }

    await expect(responseInterceptor.rejected(error)).rejects.toEqual(error)
    expect(router.replace).toHaveBeenCalledWith({ path: '/500' })
  })
})
