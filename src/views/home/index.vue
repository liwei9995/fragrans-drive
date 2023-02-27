<template>
	<div class="home flx-center">
		<div class="content">
			<div class="file-drag-zone">
				<div class="page-content">
					<Header
						:breadcrumb-items="breadcrumbItems"
						:action-items="actionItems"
						:tap-action-item="handleTapActionItem"
						:on-upload-change="handleUploadChange"
					/>
					<div class="items-wrapper">
						<Card
							v-for="item in listData?.docs"
							:key="item.id"
							:id="item.id"
							:title="item.name"
							:desc="item.desc"
							:mimeType="item.mimeType"
							:type="item.type"
							:thumbUrl="item.thumb"
							:action-items="item.type === 'file' ? fullActionItems : basicActionItems"
							:tap-action-item="handleTapCardActionItem"
						/>
					</div>
					<Dialog
						v-if="folderDialogFormVisible"
						title="新建文件夹"
						:name="folderName"
						:on-close="handleCloseFolderDialog"
						:on-confirm="handleCreateFolder"
					/>
					<Dialog
						v-if="renameDialogFormVisible"
						title="重命名"
						:thumbUrl="needToRenameThumb"
						:name="needToRenameFileName"
						:on-close="handleCloseRenameDialog"
						:on-confirm="handleRenameFile"
					/>
				</div>
				<el-empty v-if="listData?.docs.length === 0" description="No Data" />
			</div>
		</div>
	</div>
</template>

<script setup lang="ts" name="home">
import { ref, onBeforeMount, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { format } from 'date-fns'
import { ElMessage, UploadProps, ElNotification } from 'element-plus'
import Card from '@/components/StorageCard/index.vue'
import Dialog from './widgets/Dialog/index.vue'
import { getThumb } from '@/utils/thumb/index'
import Header from './widgets/Header/index.vue'
import { createFolder, getFile, getFiles, deleteFile, updateFile, getPath } from '@/api/modules/storage'

const defaultFolderName = '新建文件夹'
const defaultBreadcrumbItem = {
	id: 'root',
	text: '文件'
}
const folderDialogFormVisible = ref(false)
const renameDialogFormVisible = ref(false)
const folderName = ref(defaultFolderName)
const needToRenameThumb = ref('')
const needToRenameFileId = ref('')
const needToRenameFileName = ref('')
const breadcrumbItems = ref([defaultBreadcrumbItem])
const listData = ref<{ [key: string]: any }>()
const route = useRoute()
const router = useRouter()
const basicActionItems = [
	{
		id: 'rename',
		name: '重命名',
		divided: false
	},
	{
		id: 'delete',
		name: '删除',
		divided: true
	}
]
const fullActionItems = [
	{
		id: 'download',
		name: '下载',
		divided: false
	},
	...basicActionItems
]

const actionItems = [
	{
		id: 'folder',
		name: '新建文件夹'
	},
	{
		id: 'file',
		name: '上传文件',
		isUpload: true
	}
]

interface Storage {
	id: string
	name: string
	baseName?: string
	extName?: string
	mimeType?: string
	encoding?: string
	size?: string
	parentId: string
	type: string
	createdAt: string
	updatedAt: string
}

const fetchFiles = async () => {
	const parentId = (route.params.id as string) || 'root'
	const data = await getFiles({
		query: { parentId }
	})

	if (Array.isArray(data?.docs)) {
		data.docs = data.docs.map((item: Storage) => ({
			...item,
			desc: format(new Date(item.updatedAt), 'MM/dd HH:mm'),
			thumb: getThumb(item.extName, item.type)
		}))
		const folderItems: Storage[] = []
		const fileItems: Storage[] = []

		data.docs.forEach((doc: Storage) => {
			if (doc.type === 'folder') {
				folderItems.push(doc)
			} else {
				fileItems.push(doc)
			}
		})
		data.docs = [
			...folderItems.sort((a, b) => new Date(b.updatedAt).valueOf() - new Date(a.updatedAt).valueOf()),
			...fileItems.sort((a, b) => new Date(b.updatedAt).valueOf() - new Date(a.updatedAt).valueOf())
		]
	}

	listData.value = data
}

const fetchPath = async () => {
	const fileId = route.params.id as string

	if (fileId) {
		const pathItems = await getPath(fileId)

		breadcrumbItems.value = [
			defaultBreadcrumbItem,
			...pathItems.map((path: { id: string; name: string }) => ({
				id: path.id,
				text: path.name
			}))
		]
	}
}

onBeforeMount(() => {
	fetchFiles()
	fetchPath()
})

watch(
	() => router.currentRoute.value,
	() => {
		fetchFiles()
		fetchPath()
	}
)

// const breadcrumbItems = [
// 	{
// 		id: 'root',
// 		text: '文件'
// 	},
// 	{
// 		id: '2',
// 		isOmit: true
// 	},
// 	{
// 		id: '3',
// 		isHighlight: true,
// 		text: 'NBA录像'
// 	}
// ]

const handleCloseFolderDialog = () => (folderDialogFormVisible.value = false)

const handleCloseRenameDialog = () => (renameDialogFormVisible.value = false)

const handleCreateFolder = (name: string) => {
	const parentId = (route.params.id || 'root') as string

	folderDialogFormVisible.value = false
	ElMessage.info({
		message: '正在创建文件夹...',
		duration: 0
	})

	createFolder({
		name,
		type: 'folder',
		parentId
	})
		.then(({ exist }) => {
			ElMessage.closeAll()

			if (exist) {
				ElMessage.error('此目录下已存在同名文件，请修改名称')
			} else {
				ElMessage.success('创建成功')
				fetchFiles()
			}
		})
		.catch(() => ElMessage.error('创建失败，请重试'))

	folderName.value = defaultFolderName
}

const handleRenameFile = (name: string) => {
	const fileId = needToRenameFileId.value
	const doc = (listData?.value?.docs as Storage[])?.find(item => item.id === fileId)

	if (!doc) return

	updateFile(fileId, {
		name,
		parentId: doc?.parentId || 'root',
		type: doc?.type
	}).then(({ exist }) => {
		if (exist) {
			ElMessage.error('已存在同名文件，请修改名称')
		} else {
			renameDialogFormVisible.value = false
		}
		fetchFiles()
	})
}

const handleTapActionItem = (command: string | number | object) => {
	if (command === 'folder') {
		folderDialogFormVisible.value = true
	}
}

const download = async (id: string, name: string, type?: string) => {
	const response = await getFile(id)
	const blob = new Blob([response], { type })
	const url = window.URL.createObjectURL(blob)
	const a = document.createElement('a')

	a.href = url
	a.download = name
	a.style.display = 'none'
	document.body.appendChild(a)
	a.click()
	window.URL.revokeObjectURL(url)
	document.body.removeChild(a)
}

const handleTapCardActionItem = async (
	command: string | number | object,
	id: string,
	name: string,
	type?: string,
	thumb?: string
) => {
	if (command === 'download') {
		download(id, name, type)
	} else if (command === 'delete') {
		await deleteFile(id)
		fetchFiles()
	} else if (command === 'rename') {
		renameDialogFormVisible.value = true
		needToRenameThumb.value = thumb || ''
		needToRenameFileId.value = id
		needToRenameFileName.value = name
	}
}

const handleUploadChange: UploadProps['onChange'] = (uploadFile, uploadFiles) => {
	const isUploadingFiles = uploadFiles.filter(file => file.status === 'uploading')
	const isSuccessFiles = uploadFiles.filter(file => file.status === 'success')
	const isFailFiles = uploadFiles.filter(file => file.status === 'fail')
	const title =
		isUploadingFiles.length > 0
			? `正在上传 ∙ 共${isUploadingFiles.length}项`
			: isFailFiles.length > 0
			? `上传完成 ∙ 成功${isSuccessFiles.length}项 失败${isFailFiles.length}项目`
			: `上传完成 ∙ 共${isSuccessFiles.length}项`
	const type = isUploadingFiles.length > 0 ? 'info' : 'success'

	ElNotification.closeAll()
	ElNotification({
		title,
		type,
		position: 'bottom-right',
		duration: 0
	})

	// refetch the file list
	if (isUploadingFiles.length === 0) {
		fetchFiles()
	}
}
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
