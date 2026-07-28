import { ref } from 'vue'
import type { StorageListResponse, StorageNode } from '@/api/interface'
import { getFiles } from '@/api/modules/storage'
import { toProxyStorageUrl } from '@/utils/storageUrl'
import { getThumb } from '@/utils/thumb/index'

const pad = (value: number) => String(value).padStart(2, '0')

const getDesc = (dateTime: string) => {
  const dt = new Date(dateTime)
  const now = new Date()
  if (Number.isNaN(dt.getTime())) return ''

  const sameYear = dt.getFullYear() === now.getFullYear()
  const sameDay =
    sameYear &&
    dt.getMonth() === now.getMonth() &&
    dt.getDate() === now.getDate()
  const time = `${pad(dt.getHours())}:${pad(dt.getMinutes())}`
  const day = `${pad(dt.getMonth() + 1)}/${pad(dt.getDate())}`

  if (sameDay) return `今天 ${time}`
  if (sameYear) return `${day} ${time}`
  return `${dt.getFullYear()}/${day} ${time}`
}

export interface StorageViewItem extends StorageNode {
  desc: string
  thumb: string
  thumbPlaceholder: string
  previewSrcList: string[]
  videoUrl: string
}

type StorageListView = Omit<StorageListResponse, 'docs'> & {
  docs: StorageViewItem[]
}

export const convertItem = (item: StorageNode): StorageViewItem => ({
  ...item,
  desc: getDesc(item.updatedAt),
  thumb: item.thumbnail
    ? toProxyStorageUrl(item.thumbnail)
    : item.mimeType?.startsWith('image/') &&
        item.url &&
        (item.mimeType === 'image/svg+xml' ||
          (typeof item.size === 'number' && item.size < 500 * 1024))
      ? toProxyStorageUrl(item.url)
      : getThumb(item.extName, item.type),
  thumbPlaceholder: getThumb(item.extName, item.type),
  previewSrcList:
    !item.mimeType?.startsWith('video/') && item.url
      ? [toProxyStorageUrl(item.url)]
      : [],
  videoUrl: item.mimeType?.startsWith('video/')
    ? toProxyStorageUrl(item.url)
    : '',
})

const dateToNumber = (date: string) => +new Date(date)

export const sortDocs = (docs: StorageViewItem[]) => {
  const folderItems: StorageViewItem[] = []
  const fileItems: StorageViewItem[] = []

  docs
    .sort((a, b) => dateToNumber(b.updatedAt) - dateToNumber(a.updatedAt))
    .forEach((doc) => {
      if (doc.type === 'folder') {
        folderItems.push(doc)
      } else {
        fileItems.push(doc)
      }
    })

  const sortedDocs = [...folderItems, ...fileItems]

  return sortedDocs
}

/**
 * 获取当前目录下文件列表
 */
export const useFetchFiles = () => {
  const initialData = (): StorageListView => ({
    docs: [],
    limit: 100,
    page: 0,
    pages: 1,
  })
  const listData = ref(initialData())
  const isFetching = ref(false)
  const showSkeleton = ref(false)
  let skeletonTimer: ReturnType<typeof setTimeout> | null = null
  let latestRequest = 0

  const resetListData = () => (listData.value = initialData())

  const fetchFiles = async (parentId: string, init = true) => {
    const request = ++latestRequest
    isFetching.value = true

    if (init) {
      listData.value = initialData()
      if (skeletonTimer) clearTimeout(skeletonTimer)
      showSkeleton.value = false
      skeletonTimer = setTimeout(() => {
        if (isFetching.value) showSkeleton.value = true
      }, 200)
    }

    let data: StorageListResponse
    try {
      data = await getFiles({
        query: { parentId },
        pagination: {
          page: listData.value.page + 1,
          limit: listData.value.limit,
          sort: {
            updatedAt: -1,
          },
        },
      })
    } catch (err) {
      if (request === latestRequest) {
        isFetching.value = false
        if (skeletonTimer) clearTimeout(skeletonTimer)
        showSkeleton.value = false
      }
      throw err
    }

    // ponytail: stale responses are ignored; add AbortController only if
    // canceled network traffic becomes material.
    if (request !== latestRequest) return

    isFetching.value = false
    if (init) {
      if (skeletonTimer) clearTimeout(skeletonTimer)
      showSkeleton.value = false
    }

    const docs = [
      ...listData.value.docs,
      ...(Array.isArray(data?.docs) ? data.docs.map(convertItem) : []),
    ]

    listData.value = {
      ...data,
      limit: data.limit ?? listData.value.limit,
      page: data.page ?? listData.value.page,
      pages: data.pages ?? listData.value.pages,
      docs: sortDocs(docs),
    }
  }

  return {
    fetchFiles,
    isFetching,
    showSkeleton,
    listData,
    resetListData,
  }
}
