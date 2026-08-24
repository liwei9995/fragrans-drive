import type {
  AxiosError,
  AxiosInstance,
  AxiosRequestConfig,
  AxiosResponse,
  InternalAxiosRequestConfig,
} from 'axios'
import axios from 'axios'
import { ElMessage } from 'element-plus'
import { ResultEnum } from '@/enums/httpEnum'
import router from '@/routers'
import { GlobalStore } from '@/store'
import { checkStatus } from './helper/checkStatus'

type RetryConfig = InternalAxiosRequestConfig & { _retry?: boolean }

let isRefreshing = false
let requestsQueue: Array<(token: string | null) => void> = []

const flushQueue = (token: string | null) => {
  const queued = requestsQueue
  requestsQueue = []
  for (const resume of queued) resume(token)
}

const redirectToLogin = async () => {
  GlobalStore().$reset()
  ElMessage.error('登录失效！请您重新登录')
  await router.replace({
    path: '/login',
    query: { redirect: router.currentRoute.value.fullPath },
  })
}

const config = {
  // 默认地址请求地址，可在 .env 开头文件中修改
  baseURL: import.meta.env.VITE_API_URL as string,
  // 设置超时时间（10s）
  timeout: ResultEnum.TIMEOUT as number,
  // 跨域时候允许携带凭证
  // withCredentials: true
}

class RequestHttp {
  service: AxiosInstance
  public constructor(config: AxiosRequestConfig) {
    // 实例化axios
    this.service = axios.create(config)

    /**
     * @description 请求拦截器
     * 客户端发送请求 -> [请求拦截器] -> 服务器
     * token校验(JWT) : 接受服务器返回的token,存储到vuex/pinia/本地储存当中
     */
    this.service.interceptors.request.use(
      (config: InternalAxiosRequestConfig) => {
        const globalStore = GlobalStore()
        const token: string = globalStore.accessToken

        if (token) {
          config.headers.set('Authorization', `Bearer ${token}`)
        }
        return config
      },
      (error: AxiosError) => {
        return Promise.reject(error)
      },
    )

    /**
     * @description 响应拦截器
     *  服务器换返回信息 -> [拦截统一处理] -> 客户端JS获取到信息
     */
    this.service.interceptors.response.use(
      (response: AxiosResponse) => {
        return response.data
      },
      async (error: AxiosError) => {
        const { response } = error

        // 请求超时单独判断，因为请求超时没有 response
        if (error.message.indexOf('timeout') !== -1)
          ElMessage.error('请求超时！请您稍后重试')

        if (response?.status === ResultEnum.UNAUTHORIZED) {
          const retryConfig = error.config as RetryConfig | undefined
          const requestUrl = String(retryConfig?.url ?? '')
          const isRefreshRequest = requestUrl.includes('/auth/refresh')
          const isLoginRequest = requestUrl.includes('/auth/login')

          if (isLoginRequest) {
            const message =
              (response.data as { error?: string } | undefined)?.error ??
              '登录失败，请检查邮箱和密码'
            ElMessage.error(message)
            return Promise.reject(error)
          }

          if (!retryConfig || retryConfig._retry || isRefreshRequest) {
            await redirectToLogin()
            return Promise.reject(error)
          }

          const globalStore = GlobalStore()
          if (!globalStore.refreshToken) {
            await redirectToLogin()
            return Promise.reject(error)
          }

          if (isRefreshing) {
            return new Promise((resolve, reject) => {
              requestsQueue.push((token) => {
                if (!token) {
                  reject(error)
                  return
                }
                retryConfig._retry = true
                resolve(this.service(retryConfig))
              })
            })
          }

          isRefreshing = true
          try {
            const { data } = await axios.post<{
              access_token: string
              refresh_token: string
            }>(
              `${config.baseURL}/v1/auth/refresh`,
              {
                refresh_token: globalStore.refreshToken,
              },
              { timeout: config.timeout },
            )
            globalStore.setTokens(data.access_token, data.refresh_token)
            isRefreshing = false
            flushQueue(data.access_token)
            retryConfig._retry = true
            return this.service(retryConfig)
          } catch {
            isRefreshing = false
            flushQueue(null)
            await redirectToLogin()
            return Promise.reject(error)
          }
        }

        if (response) checkStatus(response.status)
        // 服务器结果都没有返回(可能服务器错误可能客户端断网)，断网处理:可以跳转到断网页面
        if (!window.navigator.onLine) router.replace({ path: '/500' })

        return Promise.reject(error)
      },
    )
  }

  // * 常用请求方法封装
  get<T = unknown>(url: string, params?: object, _object = {}): Promise<T> {
    return this.service.get<T, T>(url, { params, ..._object })
  }
  post<T = unknown>(url: string, params?: object, _object = {}): Promise<T> {
    return this.service.post<T, T>(url, params, _object)
  }
  put<T = unknown>(url: string, params?: object, _object = {}): Promise<T> {
    return this.service.put<T, T>(url, params, _object)
  }
  delete<T = unknown>(url: string, params?: unknown, _object = {}): Promise<T> {
    return this.service.delete<T, T>(url, { params, ..._object })
  }
  download(url: string, params?: object, _object = {}): Promise<Blob> {
    return this.service.get<Blob, Blob>(url, {
      ...params,
      ..._object,
      responseType: 'blob',
    })
  }
}

export default new RequestHttp(config)
