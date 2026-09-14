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

/** Ensure a download-forcing query flag is present. */
export function withDownloadParam(url: string): string {
  if (!url || /[?&]download=/.test(url)) return url
  return url.includes('?') ? `${url}&download=1` : `${url}?download=1`
}

/** Absolute URL suitable for clipboard paste outside the SPA. */
export function toClipboardUrl(url: unknown): string {
  const href = toDownloadHref(url)
  if (!href) return ''
  if (href.startsWith('http://') || href.startsWith('https://')) return href
  if (href.startsWith('/') && typeof window !== 'undefined') {
    return `${window.location.origin}${href}`
  }
  return href
}
