<script setup lang="ts" name="storage-item">
import { ref } from 'vue'
import { CircleCloseFilled, SuccessFilled } from '@element-plus/icons-vue'
import type { Item } from '@/hooks/useCreateFolder'
import { useCreateFolder } from '@/hooks/useCreateFolder'

interface FolderCreationProps {
	parentId?: string
	thumbUrl?: string
	success?: (item: Item) => void
	close?: () => void
}

const props = withDefaults(defineProps<FolderCreationProps>(), {
	parentId: 'root',
	thumbUrl: 'https://img.alicdn.com/imgextra/i1/O1CN01rGJZac1Zn37NL70IT_!!6000000003238-2-tps-230-180.png'
})

const folderName = ref('新建文件夹')

const handleCreateFolder = () =>
	useCreateFolder(folderName.value, props.parentId, (item: Item) => props.success && props.success(item))

const handleClose = () => props.close && props.close()
</script>

<template>
	<div class="folder-creation-wrapper">
		<el-image class="icon" :src="thumbUrl" fit="contain" />
		<el-input v-model="folderName" class="folder-name" autofocus maxlength="30" />
		<div class="actions">
			<el-icon circle size="20" class="primary" @click="handleCreateFolder">
				<SuccessFilled />
			</el-icon>
			<el-icon circle size="20" class="info" @click="handleClose">
				<CircleCloseFilled />
			</el-icon>
		</div>
	</div>
</template>

<style scoped lang="scss">
@use './index';
</style>
