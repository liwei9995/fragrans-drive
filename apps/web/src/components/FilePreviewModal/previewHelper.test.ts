import { describe, expect, it } from 'vitest'
import {
  formatDate,
  formatFileSize,
  getLanguageFromExt,
  getPreviewBadge,
  getPreviewType,
  normalizeExt,
} from './previewHelper'

describe('previewHelper', () => {
  describe('normalizeExt', () => {
    it('normalizes extension with and without dot', () => {
      expect(normalizeExt('.PNG')).toBe('png')
      expect(normalizeExt('pdf')).toBe('pdf')
      expect(normalizeExt('', 'test.tar.gz')).toBe('gz')
    })
  })

  describe('getPreviewType', () => {
    it('identifies image types correctly', () => {
      expect(getPreviewType('jpg', '')).toBe('image')
      expect(getPreviewType('png', '')).toBe('image')
      expect(getPreviewType('', 'image/webp')).toBe('image')
      expect(getPreviewType('', '', 'photo.svg')).toBe('image')
    })

    it('identifies video types correctly', () => {
      expect(getPreviewType('mp4', '')).toBe('video')
      expect(getPreviewType('mkv', '')).toBe('video')
      expect(getPreviewType('', 'video/webm')).toBe('video')
    })

    it('identifies audio types correctly', () => {
      expect(getPreviewType('mp3', '')).toBe('audio')
      expect(getPreviewType('wav', '')).toBe('audio')
      expect(getPreviewType('flac', '')).toBe('audio')
      expect(getPreviewType('', 'audio/mpeg')).toBe('audio')
    })

    it('identifies pdf documents', () => {
      expect(getPreviewType('pdf', '')).toBe('pdf')
      expect(getPreviewType('', 'application/pdf')).toBe('pdf')
    })

    it('identifies markdown documents', () => {
      expect(getPreviewType('md', '')).toBe('markdown')
      expect(getPreviewType('markdown', '')).toBe('markdown')
      expect(getPreviewType('', 'text/markdown')).toBe('markdown')
    })

    it('identifies csv and tsv tabular data', () => {
      expect(getPreviewType('csv', '')).toBe('csv')
      expect(getPreviewType('tsv', '')).toBe('csv')
      expect(getPreviewType('', 'text/csv')).toBe('csv')
      expect(getPreviewType('', 'text/tab-separated-values')).toBe('csv')
    })

    it('identifies code and text files', () => {
      expect(getPreviewType('ts', '')).toBe('code')
      expect(getPreviewType('vue', '')).toBe('code')
      expect(getPreviewType('rs', '')).toBe('code')
      expect(getPreviewType('py', '')).toBe('code')
      expect(getPreviewType('json', '')).toBe('code')
      expect(getPreviewType('txt', '')).toBe('code')
      expect(getPreviewType('', 'application/json')).toBe('code')
      expect(getPreviewType('', 'text/plain')).toBe('code')
    })

    it('falls back to unsupported for binaries and office docs', () => {
      expect(getPreviewType('docx', '')).toBe('unsupported')
      expect(getPreviewType('xlsx', '')).toBe('unsupported')
      expect(getPreviewType('zip', '')).toBe('unsupported')
      expect(getPreviewType('exe', '')).toBe('unsupported')
    })
  })

  describe('formatFileSize', () => {
    it('formats bytes correctly', () => {
      expect(formatFileSize(0)).toBe('0 B')
      expect(formatFileSize(1024)).toBe('1.00 KB')
      expect(formatFileSize(1024 * 1024 * 15)).toBe('15.00 MB')
      expect(formatFileSize(undefined)).toBe('0 B')
    })
  })

  describe('formatDate', () => {
    it('formats date correctly', () => {
      expect(formatDate('')).toBe('-')
      expect(formatDate('2026-09-10T12:00:00Z')).toContain('2026-')
    })
  })

  describe('getLanguageFromExt', () => {
    it('returns readable language names', () => {
      expect(getLanguageFromExt('ts')).toBe('TypeScript')
      expect(getLanguageFromExt('rs')).toBe('Rust')
      expect(getLanguageFromExt('vue')).toBe('Vue Template')
      expect(getLanguageFromExt('json')).toBe('JSON')
    })
  })

  describe('getPreviewBadge', () => {
    it('returns appropriate badge config', () => {
      const badge = getPreviewBadge('pdf', 'pdf')
      expect(badge.label).toBe('PDF')
      expect(badge.color).toBe('#ef4444')

      const csvBadge = getPreviewBadge('csv', 'csv')
      expect(csvBadge.label).toBe('CSV')
      expect(csvBadge.color).toBe('#10b981')
    })
  })
})
