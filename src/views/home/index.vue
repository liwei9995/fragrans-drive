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
							:type="item.mimeType"
							:tap-action-item="handleTapCardActionItem"
						/>
					</div>
					<el-dialog class="dialog-folder" width="340px" v-model="dialogFormVisible" title="新建文件夹">
						<el-row justify="center">
							<el-icon :size="100" class="icon-folder">
								<Folder />
							</el-icon>
						</el-row>
						<el-row justify="center">
							<el-input v-model="folderName" autofocus="true" maxlength="30" />
						</el-row>
						<el-row justify="end">
							<div class="dialog-footer">
								<el-button type="primary" @click="handleCreateFolder" :disabled="!folderName.trim()">确定</el-button>
							</div>
						</el-row>
					</el-dialog>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts" name="home">
import { ref, onBeforeMount } from 'vue'
import { useRoute } from 'vue-router'
import { format } from 'date-fns'
import { ElMessage, UploadProps, ElNotification } from 'element-plus'
import Card from '@/components/StorageCard/index.vue'
import Header from './widgets/Header/index.vue'
import { createFolder, getFile, getFiles, deleteFile } from '@/api/modules/storage'

const defaultFolderName = '新建文件夹'
const dialogFormVisible = ref(false)
const folderName = ref(defaultFolderName)
const listData = ref<{ [key: string]: any }>()
const route = useRoute()

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
	const data = await getFiles()

	if (Array.isArray(data?.docs)) {
		data.docs = data.docs.map((item: Storage) => ({
			...item,
			desc: format(new Date(item.updatedAt), 'MM/dd HH:mm')
		}))
	}

	listData.value = data
}

onBeforeMount(async () => fetchFiles())

const breadcrumbItems = [
	{
		id: '1',
		text: '文件'
	},
	{
		id: '2',
		isOmit: true
	},
	{
		id: '3',
		isHighlight: true,
		text: 'NBA录像'
	}
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

const handleCreateFolder = () => {
	const parentId = (route.params.id || 'root') as string

	dialogFormVisible.value = false
	ElMessage.info({
		message: '正在创建文件夹...',
		duration: 0
	})

	createFolder({
		name: folderName.value,
		type: 'folder',
		parentId
	})
		.then(({ exist }) => {
			ElMessage.closeAll()

			if (exist) {
				ElMessage.error('此目录下已存在同名文件，请修改名称')
			} else {
				ElMessage.success('创建成功')
			}
		})
		.catch(() => ElMessage.error('创建失败，请重试'))

	folderName.value = defaultFolderName
}

const handleTapActionItem = (command: string | number | object) => {
	if (command === 'folder') {
		dialogFormVisible.value = true
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

const handleTapCardActionItem = async (command: string | number | object, id: string, name: string, type?: string) => {
	if (command === 'download') {
		download(id, name, type)
	} else if (command === 'delete') {
		await deleteFile(id)
		fetchFiles()
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
