import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { useFetchFiles, convertItem, sortDocs } from './useFetchFiles'
import * as storageApi from '@/api/modules/storage'
import axios from 'axios'
import { format } from 'date-fns'

vi.mock('@/api/modules/storage', () => ({
  getFiles: vi.fn()
}))

vi.mock('@/utils/storageUrl', () => ({
  toProxyStorageUrl: vi.fn((url: string) => `proxy-${url}`)
}))

vi.mock('@/utils/thumb/index', () => ({
  getThumb: vi.fn((ext, type) => `thumb-${ext}-${type}`)
}))

describe('useFetchFiles hooks', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
    const date = new Date('2023-10-01T12:00:00Z')
    vi.setSystemTime(date)
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  describe('convertItem', () => {
    it('converts item correctly for video', () => {
      const item: any = {
        updatedAt: '2023-10-01T10:00:00Z',
        thumbnail: 'thumb-url',
        extName: 'mp4',
        type: 'file',
        mimeType: 'video/mp4',
        url: 'video-url'
      }
      const res = convertItem(item)
      expect(res.desc).toBeDefined()
      expect(res.thumb).toBe('proxy-thumb-url')
      expect(res.thumbPlaceholder).toBe('thumb-mp4-file')
      expect(res.previewSrcList).toEqual([])
      expect(res.videoUrl).toBe('proxy-video-url')
    })

    it('converts item correctly for image', () => {
      const item: any = {
        updatedAt: '2022-10-01T10:00:00Z', // Different year
        thumbnail: '',
        extName: 'png',
        type: 'file',
        mimeType: 'image/png',
        url: 'img-url'
      }
      const res = convertItem(item)
      expect(res.thumb).toBe('thumb-png-file')
      expect(res.previewSrcList).toEqual(['proxy-img-url'])
      expect(res.videoUrl).toBe('')
    })
    
    it('converts item correctly for different date', () => {
      const item: any = {
        updatedAt: '2023-09-01T10:00:00Z', // Different day, same year
      }
      const res = convertItem(item)
      expect(res.desc).toBeDefined()
    })
  })

  describe('sortDocs', () => {
    it('sorts folders before files and by updated desc', () => {
      const docs: any[] = [
        { type: 'file', updatedAt: '2023-10-01T10:00:00Z' },
        { type: 'folder', updatedAt: '2023-10-01T09:00:00Z' },
        { type: 'folder', updatedAt: '2023-10-01T11:00:00Z' },
        { type: 'file', updatedAt: '2023-10-01T12:00:00Z' }
      ]
      const res = sortDocs(docs)
      expect(res[0].updatedAt).toBe('2023-10-01T11:00:00Z')
      expect(res[0].type).toBe('folder')
      expect(res[1].updatedAt).toBe('2023-10-01T09:00:00Z')
      expect(res[2].updatedAt).toBe('2023-10-01T12:00:00Z')
      expect(res[2].type).toBe('file')
      expect(res[3].updatedAt).toBe('2023-10-01T10:00:00Z')
    })
  })

  describe('useFetchFiles', () => {
    it('fetches files successfully', async () => {
      const { fetchFiles, isFetching, listData, resetListData } = useFetchFiles()
      
      const mockDocs = [{ type: 'file', updatedAt: '2023-10-01T10:00:00Z' }]
      vi.spyOn(storageApi, 'getFiles').mockResolvedValue({ docs: mockDocs, page: 1, pages: 1 })
      
      const promise = fetchFiles('0')
      expect(isFetching.value).toBe(true)
      await promise
      
      expect(isFetching.value).toBe(false)
      expect(listData.value.docs.length).toBe(1)
      expect(listData.value.page).toBe(1)
      
      resetListData()
      expect(listData.value.docs.length).toBe(0)
    })
    
    it('fetches files successfully with append', async () => {
      const { fetchFiles, listData } = useFetchFiles()
      
      vi.spyOn(storageApi, 'getFiles').mockResolvedValue({ docs: [{ type: 'folder', updatedAt: '2023-10-01T10:00:00Z' }], page: 1, pages: 2 })
      await fetchFiles('0', true)
      
      expect(listData.value.docs.length).toBe(1)
      
      vi.spyOn(storageApi, 'getFiles').mockResolvedValue({ docs: [{ type: 'file', updatedAt: '2023-10-01T09:00:00Z' }], page: 2, pages: 2 })
      await fetchFiles('0', false)
      
      expect(listData.value.docs.length).toBe(2)
      expect(listData.value.docs[0].type).toBe('folder')
      expect(listData.value.docs[1].type).toBe('file')
    })

    it('handles api error', async () => {
      const { fetchFiles, isFetching } = useFetchFiles()
      vi.spyOn(storageApi, 'getFiles').mockRejectedValue(new Error('api err'))
      
      await expect(fetchFiles('0')).rejects.toThrow('api err')
      expect(isFetching.value).toBe(false)
    })

    it('handles cancel error without throwing', async () => {
      const { fetchFiles, isFetching } = useFetchFiles()
      const err = new axios.Cancel('canceled')
      vi.spyOn(storageApi, 'getFiles').mockRejectedValue(err)
      
      await fetchFiles('0') // Should not throw
      expect(isFetching.value).toBe(false)
    })
    
    it('handles non-array docs', async () => {
      const { fetchFiles } = useFetchFiles()
      vi.spyOn(storageApi, 'getFiles').mockResolvedValue({ docs: undefined as any })
      await expect(fetchFiles('0')).rejects.toThrow()
    })
  })
})
