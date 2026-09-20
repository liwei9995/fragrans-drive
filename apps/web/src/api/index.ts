import type {
  AxiosError,
  AxiosInstance,
  AxiosRequestConfig,
  AxiosResponse,
  InternalAxiosRequestConfig,
} from 'axios'
import axios from 'axios'
import { ElMessage } from 'element-plus'
import { LOGIN_URL } from '@/config/config'
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
  GlobalStore().logout()
  ElMessage.error('登录失效！请您重新登录')
  const currentPath = router.currentRoute?.value?.fullPath
  const shouldRedirect =
    currentPath &&
    currentPath !== LOGIN_URL &&
    !currentPath.startsWith(`${LOGIN_URL}?`) &&
    !currentPath.startsWith(`${LOGIN_URL}#`)
  await router.replace({
    path: LOGIN_URL,
    query: shouldRedirect ? { redirect: currentPath } : undefined,
  })
}

const config = {
  // Default base request URL, can be configured in .env files
  baseURL: import.meta.env.VITE_API_URL as string,
  // Request timeout (10s)
  timeout: ResultEnum.TIMEOUT as number,
  // Allow credentials for cross-origin requests
  // withCredentials: true
}

class RequestHttp {
  service: AxiosInstance
  public constructor(config: AxiosRequestConfig) {
    // Instantiate axios
    this.service = axios.create(config)

    /**
     * @description Request interceptor
     * Client request -> [Request Interceptor] -> Server
     * Token validation (JWT): receive token from server and store in pinia/local storage
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
     * @description Response interceptor
     * Server response -> [Response Interceptor] -> Client JS
     */
    this.service.interceptors.response.use(
      (response: AxiosResponse) => {
        return response.data
      },
      async (error: AxiosError) => {
        const { response } = error

        // Check timeout separately, since timeout error has no response
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

        const backendMessage =
          (response?.data as { error?: string; message?: string } | undefined)
            ?.error ||
          (response?.data as { error?: string; message?: string } | undefined)
            ?.message

        if (backendMessage) {
          ElMessage.error(backendMessage)
        } else if (response) {
          checkStatus(response.status)
        }
        // If no response returned (server error or client offline), handle offline navigation
        if (!window.navigator.onLine) router.replace({ path: '/500' })

        return Promise.reject(error)
      },
    )
  }

  // * Common request method wrappers
  // ponytail: axios 1.19's AxiosResponseResult does not collapse when R is a
  // free generic; interceptor already unwraps response.data to T.
  get<T = unknown>(url: string, params?: object, _object = {}): Promise<T> {
    return this.service.get(url, { params, ..._object }) as Promise<T>
  }
  post<T = unknown>(url: string, params?: object, _object = {}): Promise<T> {
    return this.service.post(url, params, _object) as Promise<T>
  }
  put<T = unknown>(url: string, params?: object, _object = {}): Promise<T> {
    return this.service.put(url, params, _object) as Promise<T>
  }
  patch<T = unknown>(url: string, params?: object, _object = {}): Promise<T> {
    return this.service.patch(url, params, _object) as Promise<T>
  }
  delete<T = unknown>(url: string, params?: unknown, _object = {}): Promise<T> {
    return this.service.delete(url, { params, ..._object }) as Promise<T>
  }
  download(url: string, params?: object, _object = {}): Promise<Blob> {
    return this.service.get(url, {
      ...params,
      ..._object,
      responseType: 'blob',
    }) as Promise<Blob>
  }
}

export default new RequestHttp(config)
