import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  toClipboardUrl,
  toDownloadHref,
  toProxyStorageUrl,
  withDownloadParam,
} from './storageUrl'

describe('toProxyStorageUrl', () => {
  const originalEnv = import.meta.env

  beforeEach(() => {
    vi.stubGlobal('import.meta', { env: { ...originalEnv } })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('returns empty string if url is undefined', () => {
    expect(toProxyStorageUrl(undefined)).toBe('')
  })

  it('returns the same string if not a string (edge case handled gracefully in JS)', () => {
    // @ts-expect-error
    expect(toProxyStorageUrl(123)).toBe(123)
  })

  it('returns original url if VITE_API_URL is missing', () => {
    import.meta.env.VITE_API_URL = undefined
    expect(toProxyStorageUrl('http://example.com/test.jpg')).toBe(
      'http://example.com/test.jpg',
    )
  })

  it('returns original url if VITE_API_URL does not start with /', () => {
    import.meta.env.VITE_API_URL = 'http://api.com'
    expect(toProxyStorageUrl('http://example.com/test.jpg')).toBe(
      'http://example.com/test.jpg',
    )
  })

  it('returns original url if url does not start with http', () => {
    import.meta.env.VITE_API_URL = '/api'
    expect(toProxyStorageUrl('/test.jpg')).toBe('/test.jpg')
  })

  it('replaces the origin with VITE_API_URL if valid', () => {
    import.meta.env.VITE_API_URL = '/api'
    expect(toProxyStorageUrl('http://example.com/test.jpg')).toBe(
      '/api/test.jpg',
    )
    expect(toProxyStorageUrl('https://example.com:8080/path/test.jpg')).toBe(
      '/api/path/test.jpg',
    )
  })
})

describe('toDownloadHref', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('rewrites absolute download URLs through the /api proxy', () => {
    import.meta.env.VITE_API_URL = '/api'
    expect(
      toDownloadHref('http://localhost:3821/v1/storage/abc?token=eyJ.abc.def'),
    ).toBe('/api/v1/storage/abc?token=eyJ.abc.def')
  })

  it('strips accidental JSON quotes from a plain-text URL body', () => {
    import.meta.env.VITE_API_URL = '/api'
    expect(
      toDownloadHref('"https://drive.example.com/v1/storage/abc?token=tok"'),
    ).toBe('/api/v1/storage/abc?token=tok')
  })
})

describe('withDownloadParam', () => {
  it('appends download=1 when missing', () => {
    expect(withDownloadParam('/api/v1/storage/abc?token=tok')).toBe(
      '/api/v1/storage/abc?token=tok&download=1',
    )
  })

  it('keeps existing download param', () => {
    expect(withDownloadParam('/api/v1/storage/abc?download=1&token=tok')).toBe(
      '/api/v1/storage/abc?download=1&token=tok',
    )
  })
})

describe('toClipboardUrl', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('prefixes origin for relative /api download links', () => {
    vi.stubGlobal('location', { origin: 'https://drive.oyiyio.com' })
    import.meta.env.VITE_API_URL = '/api'
    expect(
      toClipboardUrl(
        'http://localhost:3821/v1/storage/abc?download=1&token=tok',
      ),
    ).toBe('https://drive.oyiyio.com/api/v1/storage/abc?download=1&token=tok')
  })

  it('rewrites absolute API URLs to the current origin /api proxy', () => {
    vi.stubGlobal('location', { origin: 'https://drive.oyiyio.com' })
    import.meta.env.VITE_API_URL = '/api'
    expect(toClipboardUrl('https://cdn.example.com/v1/p/slug?download=1')).toBe(
      'https://drive.oyiyio.com/api/v1/p/slug?download=1',
    )
  })
})
