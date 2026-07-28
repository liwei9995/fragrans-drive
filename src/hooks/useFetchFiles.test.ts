import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { StorageListResponse, StorageNode } from '@/api/interface'
import * as storageApi from '@/api/modules/storage'
import { convertItem, sortDocs, useFetchFiles } from './useFetchFiles'

const storageNode = (overrides: Partial<StorageNode> = {}): StorageNode => ({
  id: '1',
  name: 'file',
  parentId: 'root',
  type: 'file',
  createdAt: '2023-10-01T09:00:00Z',
  updatedAt: '2023-10-01T10:00:00Z',
  ...overrides,
})

vi.mock('@/api/modules/storage', () => ({
  getFiles: vi.fn(),
}))

vi.mock('@/utils/storageUrl', () => ({
  toProxyStorageUrl: vi.fn((url: string) => `proxy-${url}`),
}))

vi.mock('@/utils/thumb/index', () => ({
  getThumb: vi.fn((ext, type) => `thumb-${ext}-${type}`),
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
      const item = storageNode({
        thumbnail: 'thumb-url',
        extName: 'mp4',
        mimeType: 'video/mp4',
        url: 'video-url',
      })
      const res = convertItem(item)
      expect(res.desc).toBeDefined()
      expect(res.thumb).toBe('proxy-thumb-url')
      expect(res.thumbPlaceholder).toBe('thumb-mp4-file')
      expect(res.previewSrcList).toEqual([])
      expect(res.videoUrl).toBe('proxy-video-url')
    })

    it('converts item correctly for image', () => {
      const item = storageNode({
        updatedAt: '2022-10-01T10:00:00Z', // Different year
        thumbnail: '',
        extName: 'png',
        mimeType: 'image/png',
        url: 'img-url',
      })
      const res = convertItem(item)
      expect(res.thumb).toBe('thumb-png-file')
      expect(res.previewSrcList).toEqual(['proxy-img-url'])
      expect(res.videoUrl).toBe('')
    })

    it('converts item correctly for different date', () => {
      const item = storageNode({
        updatedAt: '2023-09-01T10:00:00Z', // Different day, same year
      })
      const res = convertItem(item)
      expect(res.desc).toBeDefined()
      expect(convertItem(storageNode({ updatedAt: 'invalid' })).desc).toBe('')
    })
  })

  describe('sortDocs', () => {
    it('sorts folders before files and by updated desc', () => {
      const docs = [
        convertItem(storageNode()),
        convertItem(
          storageNode({
            id: '2',
            type: 'folder',
            updatedAt: '2023-10-01T09:00:00Z',
          }),
        ),
        convertItem(
          storageNode({
            id: '3',
            type: 'folder',
            updatedAt: '2023-10-01T11:00:00Z',
          }),
        ),
        convertItem(
          storageNode({ id: '4', updatedAt: '2023-10-01T12:00:00Z' }),
        ),
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
      const { fetchFiles, isFetching, listData, resetListData } =
        useFetchFiles()

      const mockDocs = [storageNode()]
      vi.spyOn(storageApi, 'getFiles').mockResolvedValue({
        docs: mockDocs,
        limit: 100,
        page: 1,
        pages: 1,
      })

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

      vi.spyOn(storageApi, 'getFiles').mockResolvedValue({
        docs: [storageNode({ type: 'folder' })],
        limit: 100,
        page: 1,
        pages: 2,
      })
      await fetchFiles('0', true)

      expect(listData.value.docs.length).toBe(1)

      vi.spyOn(storageApi, 'getFiles').mockResolvedValue({
        docs: [storageNode({ id: '2', updatedAt: '2023-10-01T09:00:00Z' })],
        limit: 100,
        page: 2,
        pages: 2,
      })
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

    it('handles non-array docs', async () => {
      const { fetchFiles, listData } = useFetchFiles()
      vi.spyOn(storageApi, 'getFiles').mockResolvedValue({
        docs: undefined,
        limit: 100,
        page: 1,
        pages: 1,
      } as unknown as StorageListResponse)
      await fetchFiles('0')
      expect(listData.value.docs).toEqual([])
    })

    it('ignores an older response that finishes last', async () => {
      const { fetchFiles, listData } = useFetchFiles()
      let resolveOld!: (value: StorageListResponse) => void
      let resolveNew!: (value: StorageListResponse) => void
      vi.spyOn(storageApi, 'getFiles')
        .mockReturnValueOnce(
          new Promise((resolve) => {
            resolveOld = resolve
          }),
        )
        .mockReturnValueOnce(
          new Promise((resolve) => {
            resolveNew = resolve
          }),
        )

      const oldRequest = fetchFiles('old')
      const newRequest = fetchFiles('new')
      resolveNew({
        docs: [storageNode({ id: 'new' })],
        limit: 100,
        page: 1,
        pages: 1,
      })
      await newRequest
      resolveOld({
        docs: [storageNode({ id: 'old' })],
        limit: 100,
        page: 1,
        pages: 1,
      })
      await oldRequest

      expect(listData.value.docs.map(({ id }) => id)).toEqual(['new'])
    })
  })
})
