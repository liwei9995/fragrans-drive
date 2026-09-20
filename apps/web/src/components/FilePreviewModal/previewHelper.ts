import { formatDateTime } from '@/utils/date'
import type { PreviewType } from './types'

const IMAGE_EXTS = new Set([
  'jpg',
  'jpeg',
  'png',
  'gif',
  'webp',
  'svg',
  'bmp',
  'ico',
  'avif',
  'tif',
  'tiff',
])

const VIDEO_EXTS = new Set([
  'mp4',
  'webm',
  'ogg',
  'mov',
  'm4v',
  'mkv',
  'avi',
  'flv',
  'wmv',
])

const AUDIO_EXTS = new Set([
  'mp3',
  'wav',
  'ogg',
  'aac',
  'flac',
  'm4a',
  'wma',
  'ape',
  'mid',
  'midi',
])

const MARKDOWN_EXTS = new Set(['md', 'markdown'])

const CSV_EXTS = new Set(['csv', 'tsv'])

const CODE_EXTS = new Set([
  'js',
  'mjs',
  'cjs',
  'ts',
  'mts',
  'cts',
  'jsx',
  'tsx',
  'vue',
  'json',
  'html',
  'htm',
  'css',
  'scss',
  'sass',
  'less',
  'xml',
  'py',
  'rs',
  'go',
  'java',
  'c',
  'cpp',
  'h',
  'hpp',
  'cs',
  'php',
  'rb',
  'sh',
  'bash',
  'zsh',
  'fish',
  'sql',
  'yaml',
  'yml',
  'toml',
  'env',
  'ini',
  'conf',
  'dockerfile',
  'txt',
  'log',
  'diff',
  'patch',
  'gitignore',
  'gitattributes',
  'editorconfig',
  'properties',
  'swift',
  'kt',
  'dart',
  'lua',
  'r',
  'scala',
  'groovy',
  'pl',
  'perl',
  'bat',
  'cmd',
  'ps1',
  'tex',
  'latex',
  'proto',
  'graphql',
  'gql',
  'prisma',
  'rst',
  'lock',
])

export function normalizeExt(ext = '', fileName = ''): string {
  let clean = ext.trim().toLowerCase()
  if (clean.startsWith('.')) clean = clean.slice(1)
  if (!clean && fileName) {
    const parts = fileName.split('.')
    if (parts.length > 1) {
      clean = parts.pop()?.toLowerCase() || ''
    }
  }
  return clean
}

export function getPreviewType(
  extName = '',
  mimeType = '',
  fileName = '',
): PreviewType {
  const ext = normalizeExt(extName, fileName)
  const mime = mimeType.trim().toLowerCase()

  if (IMAGE_EXTS.has(ext) || mime.startsWith('image/')) {
    return 'image'
  }

  if (VIDEO_EXTS.has(ext) || mime.startsWith('video/')) {
    return 'video'
  }

  if (AUDIO_EXTS.has(ext) || mime.startsWith('audio/')) {
    return 'audio'
  }

  if (ext === 'pdf' || mime === 'application/pdf') {
    return 'pdf'
  }

  if (MARKDOWN_EXTS.has(ext) || mime === 'text/markdown') {
    return 'markdown'
  }

  if (
    CSV_EXTS.has(ext) ||
    mime === 'text/csv' ||
    mime === 'text/tab-separated-values'
  ) {
    return 'csv'
  }

  if (
    CODE_EXTS.has(ext) ||
    mime.startsWith('text/') ||
    mime === 'application/json' ||
    mime === 'application/xml' ||
    mime === 'application/javascript' ||
    mime === 'application/x-javascript'
  ) {
    return 'code'
  }

  return 'unsupported'
}

export function formatFileSize(bytes?: number): string {
  if (typeof bytes !== 'number' || bytes < 0 || Number.isNaN(bytes)) {
    return '0 B'
  }
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / k ** i).toFixed(i === 0 ? 0 : 2)} ${sizes[i]}`
}

export function formatDate(dateStr?: string): string {
  return formatDateTime(dateStr, '-')
}

export function getLanguageFromExt(extName = '', fileName = ''): string {
  const ext = normalizeExt(extName, fileName)
  const map: Record<string, string> = {
    js: 'JavaScript',
    mjs: 'JavaScript',
    cjs: 'JavaScript',
    ts: 'TypeScript',
    mts: 'TypeScript',
    cts: 'TypeScript',
    jsx: 'React JSX',
    tsx: 'React TSX',
    vue: 'Vue Template',
    json: 'JSON',
    html: 'HTML',
    htm: 'HTML',
    css: 'CSS',
    scss: 'SCSS',
    sass: 'SASS',
    less: 'LESS',
    xml: 'XML',
    py: 'Python',
    rs: 'Rust',
    go: 'Go',
    java: 'Java',
    c: 'C',
    cpp: 'C++',
    h: 'C Header',
    hpp: 'C++ Header',
    cs: 'C#',
    php: 'PHP',
    rb: 'Ruby',
    sh: 'Shell Script',
    bash: 'Bash',
    zsh: 'Zsh',
    sql: 'SQL',
    yaml: 'YAML',
    yml: 'YAML',
    toml: 'TOML',
    env: 'Env Config',
    ini: 'INI Config',
    conf: 'Config',
    dockerfile: 'Dockerfile',
    txt: 'Plain Text',
    log: 'Log File',
    csv: 'CSV',
    tsv: 'TSV',
    md: 'Markdown',
    markdown: 'Markdown',
    pdf: 'PDF Document',
  }
  return map[ext] || ext.toUpperCase() || 'Text'
}

export function getPreviewBadge(previewType: PreviewType, extName = '') {
  const ext = extName ? extName.toUpperCase() : ''
  switch (previewType) {
    case 'image':
      return {
        label: ext || 'IMAGE',
        color: '#10b981',
        bg: 'rgba(16, 185, 129, 0.1)',
      }
    case 'video':
      return {
        label: ext || 'VIDEO',
        color: '#6366f1',
        bg: 'rgba(99, 102, 241, 0.1)',
      }
    case 'audio':
      return {
        label: ext || 'AUDIO',
        color: '#ec4899',
        bg: 'rgba(236, 72, 153, 0.1)',
      }
    case 'pdf':
      return { label: 'PDF', color: '#ef4444', bg: 'rgba(239, 68, 68, 0.1)' }
    case 'markdown':
      return {
        label: 'MARKDOWN',
        color: '#06b6d4',
        bg: 'rgba(6, 182, 212, 0.1)',
      }
    case 'csv':
      return {
        label: ext || 'CSV',
        color: '#10b981',
        bg: 'rgba(16, 185, 129, 0.1)',
      }
    case 'code':
      return {
        label: ext || 'CODE',
        color: '#f59e0b',
        bg: 'rgba(245, 158, 11, 0.1)',
      }
    default:
      return {
        label: ext || 'FILE',
        color: '#8b5cf6',
        bg: 'rgba(139, 92, 246, 0.1)',
      }
  }
}
