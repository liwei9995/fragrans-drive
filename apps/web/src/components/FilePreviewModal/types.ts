export type PreviewType =
  | 'image'
  | 'video'
  | 'audio'
  | 'pdf'
  | 'markdown'
  | 'code'
  | 'csv'
  | 'unsupported'

export interface FilePreviewItem {
  id: string
  name: string
  size?: number
  extName?: string
  mimeType?: string
  url?: string
  thumb?: string
  thumbnail?: string
  isPublic?: boolean
  publicSlug?: string
  publicUrl?: string
  updatedAt?: string
  createdAt?: string
  contentHash?: string
  md5Hash?: string
  parentId?: string
}
