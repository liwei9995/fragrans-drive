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
						:on-upload-progress="handleUploadProgress"
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
						<Empty
							v-if="!isFetching && listData?.docs.length === 0"
							:on-upload-change="handleUploadChange"
							:on-upload-exceed="handleUploadExceed"
							:on-upload-progress="handleUploadProgress"
							:before-upload="handelBeforeUpload"
							:tap-item="handleTapActionItem"
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
					<Move v-if="moveDialogFormVisible" title="移动到" :on-close="handleCloseMoveDialog" />
					<UploadStatus
						ref="uploadStatusRef"
						:percentage="uploadPercentage"
						:title="notificationTitle"
						:type="notificationType"
						:on-close="handleCloseUploadStatus"
					/>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts" name="home">
import { ref, onBeforeMount, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox, UploadProps, UploadFiles, ElNotification } from 'element-plus'
import emitter from '@/utils/emitter'
import Card from '@/components/StorageCard/index.vue'
import Dialog from './widgets/Dialog/index.vue'
import UploadStatus from './widgets/UploadStatus/index.vue'
import Header from './widgets/Header/index.vue'
import Breadcrumb from './widgets/Breadcrumb/index.vue'
import Empty from './widgets/Empty/index.vue'
import Move from './widgets/Move/index.vue'
import { LOGIN_URL } from '@/config/config'
import { GlobalStore } from '@/store'
import { UploadEventEnum } from '@/enums/events'
import { createFolder, getDownloadUrl, deleteFile, updateFile, getPath } from '@/api/modules/storage'
import { useFetchFiles } from '@/hooks/useFetchFiles'

type BreadcrumbItem = {
	id: string
	name: string
}

const globalStore = GlobalStore()
const defaultFolderName = '新建文件夹'
const folderDialogFormVisible = ref(false)
const renameDialogFormVisible = ref(false)
const moveDialogFormVisible = ref(false)
const moveToFolderId = ref('root')
const uploadStatusRef = ref()
const uploadPercentage = ref(0)
const uploadedFiles = ref([] as UploadFiles)
const notificationTitle = ref('')
const notificationType = ref('info')
const folderName = ref(defaultFolderName)
const needToRenameThumb = ref('')
const needToRenameFileId = ref('')
const needToRenameFileName = ref('')
const breadcrumbItems = ref([] as BreadcrumbItem[])
const uploadFileLimit = 10
const route = useRoute()
const router = useRouter()
const { fetchFiles, listData, resetListData, isFetching } = useFetchFiles()
const parentId = ref((route.params.id as string) || 'root')
const basicActionItems = [
	{
		id: 'rename',
		name: '重命名',
		divided: false
	},
	{
		id: 'move',
		name: '移动',
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

const load = () => {
	if (isFetching.value || listData.value.page + 1 > listData.value.pages) return

	fetchFiles(parentId.value, false)
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
		parentId.value = (route.params.id as string) || 'root'
		resetListData()
		fetchFiles(parentId.value)
		fetchPath()
	}
)

const handleCloseFolderDialog = () => (folderDialogFormVisible.value = false)

const handleCloseRenameDialog = () => (renameDialogFormVisible.value = false)

const handleCloseMoveDialog = () => (moveDialogFormVisible.value = false)

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
				fetchFiles(parentId)
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
		fetchFiles(parentId.value)
	})
}

const handleCloseUploadStatus = () => (uploadedFiles.value = [])

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

	// * 在当前页面弹出下载窗口
	window.open(url, '_self')
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
				fetchFiles(parentId.value)
				ElMessage.success('文件删除成功')
			})
			.catch(() => {})
	} else if (command === 'rename') {
		renameDialogFormVisible.value = true
		needToRenameThumb.value = thumb || ''
		needToRenameFileId.value = id
		needToRenameFileName.value = name
	} else if (command === 'move') {
		moveDialogFormVisible.value = true
		moveToFolderId.value = id
	}
}

const handleUploadChange: UploadProps['onChange'] = (uploadFile, uploadFiles) => {
	const isUploadingFiles = uploadFiles.filter(file => ['uploading', 'ready'].includes(file.status))
	const isSuccessFiles = uploadFiles.filter(file => file.status === 'success')
	const isFailFiles = uploadFiles.filter(file => file.status === 'fail')

	if (isUploadingFiles.length === 0 && isSuccessFiles.length === 0 && isFailFiles.length === 0) return

	const totalFiles = [...uploadedFiles.value, ...isSuccessFiles]
	const title =
		isUploadingFiles.length > 0
			? `正在上传 ∙ 剩余${isUploadingFiles.length}项`
			: isFailFiles.length > 0 || uploadedFiles.value.filter(file => file.status === 'fail').length > 0
			? `上传完成 ∙ 成功${isSuccessFiles.length}项 失败${isFailFiles.length}项`
			: `上传完成 ∙ 共${totalFiles.length}项`
	const type = isUploadingFiles.length > 0 ? 'uploading' : 'success'

	ElNotification.closeAll()
	uploadStatusRef.value!.show()
	notificationTitle.value = title
	notificationType.value = type

	if (type === 'success') {
		uploadPercentage.value = 0
	}

	// refetch the file list
	if (isUploadingFiles.length === 0) {
		uploadedFiles.value = totalFiles
		emitter.emit(UploadEventEnum.CLEAR_FILES)
		fetchFiles(parentId.value)
	}
}

const handleUploadExceed: UploadProps['onExceed'] = files => {
	ElMessage.warning(`一次最多允许上传${uploadFileLimit}个文件，你这次选择了${files.length}个`)
}

const handleUploadProgress: UploadProps['onProgress'] = (event, uploadFile, uploadFiles) => {
	const totalSize = uploadFiles.reduce((accumulator, current) => accumulator + (current?.size || 0), 0)
	const uploadedSize = uploadFiles.reduce((accumulator, current) => {
		const cur = ['uploading', 'success'].includes(current?.status) ? ((current?.size || 0) * (current?.percentage || 0)) / 100 : 0

		return accumulator + cur
	}, 0)
	const percentage = (uploadedSize / totalSize) * 100

	uploadPercentage.value = percentage
}

const handelBeforeUpload: UploadProps['beforeUpload'] = rawFile => {
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
