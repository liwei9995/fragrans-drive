import axios from 'axios'
import { format } from 'date-fns'
import { ref } from 'vue'
import { getFiles } from '@/api/modules/storage'
import { toProxyStorageUrl } from '@/utils/storageUrl'
import { getThumb } from '@/utils/thumb/index'

const getDesc = (dateTime: string) => {
  const dt = new Date(dateTime)
  const now = new Date()
  const dtYear = format(dt, 'yyyy')
  const year = format(now, 'yyyy')
  const dtDay = format(dt, 'yyyy/MM/dd')
  const today = format(now, 'yyyy/MM/dd')

  return dtDay === today
    ? `今天 ${format(dt, 'HH:mm')}`
    : dtYear === year
      ? format(dt, 'MM/dd HH:mm')
      : format(dt, 'yyyy/MM/dd HH:mm')
}

export const convertItem = (item: Storage) => ({
  ...item,
  desc: getDesc(item.updatedAt),
  thumb: item.thumbnail
    ? toProxyStorageUrl(item.thumbnail)
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

export const sortDocs = (docs: Storage[]) => {
  const folderItems: Storage[] = []
  const fileItems: Storage[] = []

  docs
    .sort((a, b) => dateToNumber(b.updatedAt) - dateToNumber(a.updatedAt))
    .forEach((doc: Storage) => {
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
  const initialData = {
    docs: [] as Storage[],
    limit: 100,
    page: 0,
    pages: 1,
  }
  const listData = ref(initialData)
  const isFetching = ref(false)

  const resetListData = () => (listData.value = initialData)

  const fetchFiles = async (parentId: string, init = true) => {
    isFetching.value = true

    if (init) {
      listData.value = initialData
    }

    let data: { docs: Storage[]; [key: string]: unknown }
    try {
      data = (await getFiles({
        query: { parentId },
        pagination: {
          page: listData.value.page + 1,
          limit: listData.value.limit,
          sort: {
            updatedAt: -1,
          },
        },
      })) as { docs: Storage[]; [key: string]: unknown }
    } catch (err) {
      isFetching.value = false
      if (axios.isCancel(err)) return
      throw err
    }

    isFetching.value = false

    if (Array.isArray(data?.docs)) {
      data.docs = data.docs.map((item: Storage) => convertItem(item))
    }

    const docs = [...listData.value.docs, ...data.docs]

    listData.value = {
      ...data,
      limit: listData.value.limit,
      page: (data.page as number) ?? listData.value.page,
      pages: (data.pages as number) ?? listData.value.pages,
      docs: sortDocs(docs),
    }
  }

  return {
    fetchFiles,
    isFetching,
    listData,
    resetListData,
  }
}
