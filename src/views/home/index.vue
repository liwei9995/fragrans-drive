<template>
	<div class="home flx-center">
		<div class="content">
			<div class="file-drag-zone">
				<div class="page-content">
					<Header
						:breadcrumb-items="breadcrumbItems"
						:action-items="actionItems"
						:avatar-action-items="avatarActionItems"
						:upload-file-limit="uploadFileLimit"
						:tap-action-item="handleTapActionItem"
						:on-upload-change="handleUploadChange"
						:on-upload-exceed="handleUploadExceed"
						:before-upload="handelBeforeUpload"
					/>
					<div class="sub-nav-wrapper">
						<Breadcrumb :breadcrumb-items="breadcrumbItems" />
					</div>
					<div class="items-wrapper">
						<div v-infinite-scroll="load" class="items">
							<Card
								v-for="item in listData?.docs"
								:key="item.id"
								:id="item.id"
								:title="item.name"
								:desc="item.desc"
								:mimeType="item.mimeType"
								:type="item.type"
								:thumbUrl="item.thumb"
								:thumbPlaceholder="item.thumbPlaceholder"
								:preview-src-list="item.previewSrcList"
								:action-items="item.type === 'file' ? fullActionItems : basicActionItems"
								:tap-action-item="handleTapCardActionItem"
							/>
							<Card v-for="item in 10" :id="'empty-item-id' + item" :key="item" isEmpty />
						</div>
						<el-empty v-if="!isFetching && listData?.docs.length === 0" description="该目录下没有文件" />
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
			</div>
		</div>
	</div>
</template>

<script setup lang="ts" name="home">
import { ref, onBeforeMount, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { format } from 'date-fns'
import { ElMessage, ElMessageBox, UploadProps, ElNotification } from 'element-plus'
import Card from '@/components/StorageCard/index.vue'
import Dialog from './widgets/Dialog/index.vue'
import { LOGIN_URL } from '@/config/config'
import { GlobalStore } from '@/store'
import { getThumb } from '@/utils/thumb/index'
import Header from './widgets/Header/index.vue'
import Breadcrumb from './widgets/Breadcrumb/index.vue'
import { createFolder, getDownloadUrl, getFiles, deleteFile, updateFile, getPath } from '@/api/modules/storage'

type BreadcrumbItem = {
	id: string
	name: string
}

const globalStore = GlobalStore()
const defaultFolderName = '新建文件夹'
const folderDialogFormVisible = ref(false)
const renameDialogFormVisible = ref(false)
const isFetching = ref(false)
const folderName = ref(defaultFolderName)
const needToRenameThumb = ref('')
const needToRenameFileId = ref('')
const needToRenameFileName = ref('')
const breadcrumbItems = ref([] as BreadcrumbItem[])
const initialData = {
	docs: [],
	limit: 100,
	page: 0,
	pages: 1
}
const listData = ref<{ [key: string]: any }>(initialData)
const uploadFileLimit = 10
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

const avatarActionItems = [
	{
		id: 'logout',
		name: '退出登录'
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
	thumbnail?: string
	url?: string
	createdAt: string
	updatedAt: string
}

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

const fetchFiles = async (init = true) => {
	const parentId = (route.params.id as string) || 'root'

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
		data.docs = data.docs.map((item: Storage) => ({
			...item,
			desc: getDesc(item.updatedAt),
			thumb: item.thumbnail ? item.thumbnail : getThumb(item.extName, item.type),
			thumbPlaceholder: getThumb(item.extName, item.type),
			previewSrcList: item.url ? [item.url] : []
		}))
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

const load = () => {
	if (isFetching.value || listData.value.page + 1 > listData.value.pages) return

	fetchFiles(false)
}

const fetchPath = async () => {
	const fileId = route.params.id as string

	if (fileId) {
		const pathItems = await getPath(fileId)

		breadcrumbItems.value = [
			...pathItems.map((path: { id: string; name: string }) => ({
				id: path.id,
				text: path.name
			}))
		]
	} else {
		breadcrumbItems.value = []
	}
}

onBeforeMount(() => fetchPath())

watch(
	() => router.currentRoute.value,
	() => {
		console.log('watch')
		listData.value = initialData
		fetchFiles()
		fetchPath()
	}
)

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
	} else if (command === 'logout') {
		// * 清空 token
		globalStore.setToken('')
		// * 跳转到登录页面
		router.push(LOGIN_URL)
	}
}

const download = async (id: string) => {
	ElMessage.info('文件下载准备中...')
	const { url } = await getDownloadUrl(id)

	window.open(url)
}

const handleTapCardActionItem = async (
	command: string | number | object,
	id: string,
	name: string,
	type?: string,
	thumb?: string
) => {
	if (command === 'download') {
		download(id)
	} else if (command === 'delete') {
		ElMessageBox.confirm('文件删除后将无法恢复，确定要删除么？', '删除文件', {
			confirmButtonText: '确定删除',
			cancelButtonText: '取消',
			type: 'warning'
		})
			.then(async () => {
				await deleteFile(id)
				fetchFiles()
				ElMessage.success('文件删除成功')
			})
			.catch(() => {})
	} else if (command === 'rename') {
		renameDialogFormVisible.value = true
		needToRenameThumb.value = thumb || ''
		needToRenameFileId.value = id
		needToRenameFileName.value = name
	}
}

const handleUploadChange: UploadProps['onChange'] = (uploadFile, uploadFiles) => {
	const isReadyFIles = uploadFiles.filter(file => file.status === 'ready')
	const isUploadingFiles = uploadFiles.filter(file => file.status === 'uploading')
	const isSuccessFiles = uploadFiles.filter(file => file.status === 'success')
	const isFailFiles = uploadFiles.filter(file => file.status === 'fail')

	if (isUploadingFiles.length === 0 && isSuccessFiles.length === 0 && isFailFiles.length === 0) return

	const title =
		isUploadingFiles.length > 0
			? `正在上传 ∙ 剩余${isUploadingFiles.length}项`
			: isFailFiles.length > 0
			? `上传完成 ∙ 成功${isSuccessFiles.length}项 失败${isFailFiles.length}项`
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
	if (isUploadingFiles.length === 0 && isReadyFIles.length === 0) {
		fetchFiles()
	}
}

const handleUploadExceed: UploadProps['onExceed'] = files => {
	ElMessage.warning(`一次最多允许上传${uploadFileLimit}个文件，你这次选择了${files.length}个`)
}

const handelBeforeUpload: UploadProps['beforeUpload'] = rawFile => {
	console.log(`rawFile :>> ${JSON.stringify(rawFile, null, 2)}`)
	if (rawFile.size / 1024 / 1024 > 512) {
		ElMessage.error('上传文件的大小不能超过512MB')

		return false
	}

	return true
}
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
