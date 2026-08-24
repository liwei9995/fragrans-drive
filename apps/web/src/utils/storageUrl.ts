/**
 * 将后端返回的完整 storage URL 转为走前端代理，避免 img/video/download
 * 直接请求后端。开发环境和生产 Nginx 都只把 `/api/` 转到 API。
 */
export function toProxyStorageUrl(url: string | undefined): string {
  if (!url || typeof url !== 'string') return url ?? ''
  const base = import.meta.env.VITE_API_URL as string | undefined
  if (!base || !base.startsWith('/')) return url
  if (!url.startsWith('http://') && !url.startsWith('https://')) return url
  const path = url.replace(/^https?:\/\/[^/]+/, '')
  return `${base}${path}`
}

export function toDownloadHref(url: unknown): string {
  return toProxyStorageUrl(String(url ?? '').replace(/^"|"$/g, ''))
}
