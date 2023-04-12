<template>
	<el-dialog class="move-dialog-wrapper" width="340px" :title="title" v-model="dialogFormVisible" @close="handleClose">
		<Breadcrumb :breadcrumb-items="breadcrumbItems" :autoNav="false" :on-click-breadcrumb-item="handleClickBreadcrumbItem" />
		<div class="list">
			<StorageItem
				v-for="item in listData?.docs"
				:key="item.id"
				:name="item.name"
				:thumbUrl="item.thumb"
				:thumb-placeholder="item.thumbPlaceholder"
			/>
		</div>
		<div class="action"></div>
	</el-dialog>
</template>

<script setup lang="ts" name="move">
import { ref, onBeforeMount, watch } from 'vue'
import StorageItem from '@/components/StorageItem/index.vue'
import Breadcrumb, { BreadcrumbItem } from '../Breadcrumb/index.vue'
import { getPath } from '@/api/modules/storage'
import { useFetchFiles } from '@/hooks/useFetchFiles'

interface MoveProps {
	id?: string
	title: string
	onClose?: () => void
}

const props = withDefaults(defineProps<MoveProps>(), {
	id: () => 'root',
	title: () => ''
})
const id = ref(props.id)
const { fetchFiles, listData } = useFetchFiles()
const breadcrumbItems = ref([] as BreadcrumbItem[])
const fetchPath = async () => {
	if (!id.value || id.value === 'root') return

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

watch(
	() => props.id,
	() => (id.value = props.id)
)

const handleClose = () => props.onClose && props.onClose()

const handleClickBreadcrumbItem = (item: BreadcrumbItem) => {
	if (item?.id && item?.id !== id.value) {
		id.value = item.id
		fetchFiles(id.value)
	}
}
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
