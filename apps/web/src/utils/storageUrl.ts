/**
 * 将后端返回的完整 storage URL 转为走前端代理，避免 img/video 等直接请求后端时的 CORS
 * 开发环境下 VITE_API_URL 为 '/api'，请求会由 Vite 代理到后端
 */
export function toProxyStorageUrl(url: string | undefined): string {
  if (!url || typeof url !== 'string') return url ?? ''
  const base = import.meta.env.VITE_API_URL as string | undefined
  if (!base || !base.startsWith('/')) return url
  if (!url.startsWith('http://') && !url.startsWith('https://')) return url
  const path = url.replace(/^https?:\/\/[^/]+/, '')
  return `${base}${path}`
}
