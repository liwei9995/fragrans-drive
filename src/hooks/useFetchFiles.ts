import { ref } from 'vue'
import { format } from 'date-fns'
import { getFiles } from '@/api/modules/storage'
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
	thumb: item.thumbnail ? item.thumbnail : getThumb(item.extName, item.type),
	thumbPlaceholder: getThumb(item.extName, item.type),
	previewSrcList: item.url ? [item.url] : []
})

/**
 * 获取当前目录下文件列表
 */
export const useFetchFiles = () => {
	const initialData = {
		docs: [],
		limit: 100,
		page: 0,
		pages: 1
	}
	const listData = ref<{ [key: string]: any }>(initialData)
	const isFetching = ref(false)

	const resetListData = () => (listData.value = initialData)

	const fetchFiles = async (parentId: string, init = true) => {
		isFetching.value = true

		if (init) {
			listData.value = initialData
		}

		const data = await getFiles({
			query: { parentId },
			pagination: {
				page: listData.value.page + 1,
				limit: listData.value.limit,
				sort: {
					updatedAt: -1
				}
			}
		})

		isFetching.value = false

		if (Array.isArray(data?.docs)) {
			data.docs = data.docs.map((item: Storage) => convertItem(item))
		}

		const docs = [...listData.value.docs, ...data.docs]
		const folderItems: Storage[] = []
		const fileItems: Storage[] = []

		docs.forEach((doc: Storage) => {
			if (doc.type === 'folder') {
				folderItems.push(doc)
			} else {
				fileItems.push(doc)
			}
		})

		const sortedDocs = [...folderItems, ...fileItems]

		listData.value = {
			...data,
			docs: sortedDocs
		}
	}

	return {
		fetchFiles,
		isFetching,
		listData,
		resetListData
	}
}
