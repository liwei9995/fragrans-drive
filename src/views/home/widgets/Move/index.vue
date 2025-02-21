<script setup lang="ts" name="move">
import { onBeforeMount, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import StorageItem from '@/components/StorageItem/index.vue'
import type { BreadcrumbItem } from '../Breadcrumb/index.vue'
import Breadcrumb from '../Breadcrumb/index.vue'
import FolderCreation from '../FolderCreation/index.vue'
import { getPath, moveFile } from '@/api/modules/storage'
import { useFetchFiles } from '@/hooks/useFetchFiles'
import type { Item } from '@/hooks/useCreateFolder'

interface MoveProps {
	id?: string
	parentId?: string
	title: string
	onClose?: () => void
	onMoved?: (id: string, parentId: string) => void
	onFolderCreated?: (parentId: string) => void
}

const props = withDefaults(defineProps<MoveProps>(), {
	id: () => '',
	parentId: () => 'root',
	title: () => ''
})
const id = ref(props.parentId)
const createFolderItemVisible = ref(false)
const { fetchFiles, listData, isFetching } = useFetchFiles()
const breadcrumbItems = ref([] as BreadcrumbItem[])
const fetchPath = async () => {
	if (!id.value) return

	if (id.value === 'root') {
		breadcrumbItems.value = []

		return
	}

	const pathItems = await getPath(id.value)

	breadcrumbItems.value = [
		...pathItems.map((path: { id: string; name: string }) => ({
			id: path.id,
			text: path.name
		}))
	]
}

const dialogFormVisible = ref(true)

onBeforeMount(() => {
	fetchPath()
	fetchFiles(id.value)
})

const load = () => {
	if (isFetching.value || listData.value.page + 1 > listData.value.pages) return

	fetchFiles(props.parentId, false)
}

watch(
	() => props.id,
	() => (id.value = props.id)
)

const handleClose = () => props.onClose && props.onClose()

const handleClickBreadcrumbItem = (item: BreadcrumbItem) => {
	if (item?.id && item?.id !== id.value) {
		createFolderItemVisible.value = false
		id.value = item.id
		fetchPath()
		fetchFiles(id.value)
	}
}

const handleCloseFolderCreationItem = () => (createFolderItemVisible.value = false)

const handleCreateFolderSuccess = (item: Item) => {
	createFolderItemVisible.value = false
	id.value = item.id
	fetchPath()
	fetchFiles(id.value)
	props.onFolderCreated && props.onFolderCreated(item.parentId)
}

const handleTapItem = (itemId: string) => {
	createFolderItemVisible.value = false
	id.value = itemId
	fetchPath()
	fetchFiles(id.value)
}

const handleCreateFolder = () => (createFolderItemVisible.value = true)

const handleCancel = () => (dialogFormVisible.value = false)

const handleMove = () => {
	ElMessage.info({
		message: '正在移动文件...',
		duration: 0
	})

	moveFile({
		fileId: props.id,
		parentId: id.value
	})
		.then(() => {
			dialogFormVisible.value = false
			ElMessage.closeAll()
			ElMessage.success('移动成功')
			props.onMoved && props.onMoved(props.id, id.value)
		})
		.catch(() => {
			ElMessage.closeAll()
			ElMessage.error('移动失败，请重试')
		})
}
</script>

<template>
	<el-dialog v-model="dialogFormVisible" class="move-dialog-wrapper" :title="title" @close="handleClose">
		<Breadcrumb :breadcrumb-items="breadcrumbItems" :auto-nav="false" :on-click-breadcrumb-item="handleClickBreadcrumbItem" />
		<div v-infinite-scroll="load" class="list" :infinite-scroll-disabled="!dialogFormVisible">
			<FolderCreation
				v-if="createFolderItemVisible"
				:parent-id="id"
				:close="handleCloseFolderCreationItem"
				:success="handleCreateFolderSuccess"
			/>
			<StorageItem
				v-for="item in listData?.docs"
				:id="item.id"
				:key="item.id"
				:disabled="item.type !== 'folder'"
				:name="item.name"
				:thumb-url="item.thumb"
				:thumb-placeholder="item.thumbPlaceholder"
				:tap="handleTapItem"
			/>
			<div v-if="listData?.docs.length === 0 && !isFetching && !createFolderItemVisible" class="empty">
				<el-image
					class="icon"
					src="https://img.alicdn.com/imgextra/i2/O1CN018yXBXY1caApf7qUew_!!6000000003616-2-tps-224-224.png"
				/>
				<div>文件夹为空</div>
			</div>
		</div>
		<div class="action">
			<div class="create" @click="handleCreateFolder">新建文件夹</div>
			<div class="buttons">
				<el-button @click="handleCancel">取消</el-button>
				<el-button type="primary" @click="handleMove"> 移动到此处 </el-button>
			</div>
		</div>
	</el-dialog>
</template>

<style lang="scss">
@use './index';
</style>
