<template>
	<el-dialog class="move-dialog-wrapper" :title="title" v-model="dialogFormVisible" @close="handleClose">
		<Breadcrumb :breadcrumb-items="breadcrumbItems" :autoNav="false" :on-click-breadcrumb-item="handleClickBreadcrumbItem" />
		<div class="list" v-infinite-scroll="load">
			<FolderCreation v-if="createFolderItemVisible" :parentId="id" :close="handleCloseFolderCreationItem" />
			<StorageItem
				v-for="item in listData?.docs"
				:key="item.id"
				:id="item.id"
				:disabled="item.type !== 'folder'"
				:name="item.name"
				:thumbUrl="item.thumb"
				:thumb-placeholder="item.thumbPlaceholder"
				:tap="handleTapItem"
			/>
			<div class="empty" v-if="listData?.docs.length === 0 && !isFetching && !createFolderItemVisible">
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
				<el-button type="primary" @click="handleMove">移动到此处</el-button>
			</div>
		</div>
	</el-dialog>
</template>

<script setup lang="ts" name="move">
import { ref, onBeforeMount, watch } from 'vue'
import { ElMessage } from 'element-plus'
import StorageItem from '@/components/StorageItem/index.vue'
import Breadcrumb, { BreadcrumbItem } from '../Breadcrumb/index.vue'
import FolderCreation from '../FolderCreation/index.vue'
import { getPath, moveFile } from '@/api/modules/storage'
import { useFetchFiles } from '@/hooks/useFetchFiles'

interface MoveProps {
	id?: string
	parentId?: string
	title: string
	onClose?: () => void
	onMoved?: () => void
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
			props.onMoved && props.onMoved()
		})
		.catch(() => {
			ElMessage.closeAll()
			ElMessage.error('移动失败，请重试')
		})
}
</script>

<style lang="scss">
@import './index.scss';
</style>
