import {
  AxiosHeaders,
  type AxiosResponse,
  type InternalAxiosRequestConfig,
} from 'axios'
import { ElMessage } from 'element-plus'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import router from '@/routers'
import { GlobalStore } from '@/store'
import { checkStatus } from './helper/checkStatus'
import RequestHttp from './index'

type RequestInterceptor = {
  fulfilled: (
    config: InternalAxiosRequestConfig,
  ) => InternalAxiosRequestConfig | Promise<InternalAxiosRequestConfig>
  rejected: (error: unknown) => Promise<never>
}

type ResponseInterceptor = {
  fulfilled: (response: AxiosResponse) => unknown
  rejected: (error: unknown) => Promise<never>
}

const requestInterceptor = () =>
  (
    RequestHttp.service.interceptors.request as unknown as {
      handlers: RequestInterceptor[]
    }
  ).handlers[0]

const responseInterceptor = () =>
  (
    RequestHttp.service.interceptors.response as unknown as {
      handlers: ResponseInterceptor[]
    }
  ).handlers[0]

const response = (status: number, data: unknown): AxiosResponse => ({
  status,
  data,
  statusText: '',
  headers: new AxiosHeaders(),
  config: {
    headers: new AxiosHeaders(),
  },
})

vi.mock('element-plus', () => ({
  ElMessage: {
    error: vi.fn(),
  },
}))

vi.mock('@/routers', () => ({
  default: {
    replace: vi.fn(),
    currentRoute: {
      value: { fullPath: '/home/folder' },
    },
  },
}))

vi.mock('./helper/checkStatus', () => ({
  checkStatus: vi.fn(),
}))

describe('api index', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    Object.defineProperty(window.navigator, 'onLine', {
      value: true,
      writable: true,
    })
  })

  it('request methods', () => {
    vi.spyOn(RequestHttp.service, 'get').mockResolvedValue('get')
    vi.spyOn(RequestHttp.service, 'post').mockResolvedValue('post')
    vi.spyOn(RequestHttp.service, 'put').mockResolvedValue('put')
    vi.spyOn(RequestHttp.service, 'delete').mockResolvedValue('delete')

    RequestHttp.get('/url')
    expect(RequestHttp.service.get).toHaveBeenCalledWith('/url', {
      params: undefined,
    })

    RequestHttp.post('/url', { a: 1 })
    expect(RequestHttp.service.post).toHaveBeenCalledWith('/url', { a: 1 }, {})

    RequestHttp.put('/url', { a: 1 })
    expect(RequestHttp.service.put).toHaveBeenCalledWith('/url', { a: 1 }, {})

    RequestHttp.delete('/url')
    expect(RequestHttp.service.delete).toHaveBeenCalledWith('/url', {
      params: undefined,
    })

    RequestHttp.download('/url', { a: 1 })
    expect(RequestHttp.service.get).toHaveBeenCalledWith('/url', {
      a: 1,
      responseType: 'blob',
    })
  })

  it('request interceptor adds token', async () => {
    const store = GlobalStore()
    store.setToken('test-token')

    const config = {
      headers: new AxiosHeaders(),
    } as InternalAxiosRequestConfig
    const setHeader = vi.spyOn(config.headers, 'set')

    await requestInterceptor().fulfilled(config)
    expect(setHeader).toHaveBeenCalledWith('Authorization', 'Bearer test-token')
  })

  it('request interceptor handles error', async () => {
    await expect(
      requestInterceptor().rejected(new Error('req err')),
    ).rejects.toThrow('req err')
  })

  it('response interceptor handles 401', async () => {
    const store = GlobalStore()
    store.setToken('old-token')

    await expect(
      responseInterceptor().rejected({
        message: 'unauthorized',
        config: { headers: new AxiosHeaders() },
        response: { status: 401 },
      }),
    ).rejects.toMatchObject({ message: 'unauthorized' })
    expect(ElMessage.error).toHaveBeenCalledWith('登录失效！请您重新登录')
    expect(store.token).toBe('')
    expect(router.replace).toHaveBeenCalledWith({
      path: '/login',
      query: { redirect: '/home/folder' },
    })
  })

  it('response interceptor handles success', async () => {
    const result = response(200, { result: 'ok' })

    const data = await responseInterceptor().fulfilled(result)
    expect(data).toEqual(result.data)
  })

  it('response interceptor handles reject timeout', async () => {
    const error = {
      message: 'timeout of 10000ms exceeded',
    }

    await expect(responseInterceptor().rejected(error)).rejects.toEqual(error)
    expect(ElMessage.error).toHaveBeenCalledWith('请求超时！请您稍后重试')
  })

  it('response interceptor handles reject with response', async () => {
    const error = {
      message: 'network err',
      response: { status: 500 },
    }

    await expect(responseInterceptor().rejected(error)).rejects.toEqual(error)
    expect(checkStatus).toHaveBeenCalledWith(500)
  })

  it('response interceptor handles reject offline', async () => {
    Object.defineProperty(window.navigator, 'onLine', {
      value: false,
      writable: true,
    })

    const error = {
      message: 'network err',
    }

    await expect(responseInterceptor().rejected(error)).rejects.toEqual(error)
    expect(router.replace).toHaveBeenCalledWith({ path: '/500' })
  })
})
