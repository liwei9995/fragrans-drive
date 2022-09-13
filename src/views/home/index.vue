<template>
	<div class="home flx-center">
		<div class="content">
			<div class="file-drag-zone">
				<div class="page-content">
					<Header :breadcrumb-items="breadcrumbItems" :action-items="actionItems" :tapActionItem="handleTapActionItem" />
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
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import Header from './widgets/Header/index.vue'
import { createFolder } from '@/api/modules/storage'

const defaultFolderName = '新建文件夹'
const dialogFormVisible = ref(false)
const folderName = ref(defaultFolderName)
const route = useRoute()

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
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
