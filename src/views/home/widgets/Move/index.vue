<template>
	<el-dialog class="move-dialog-wrapper" width="340px" :title="title" v-model="dialogFormVisible" @close="handleClose">
		<Breadcrumb :breadcrumb-items="breadcrumbItems" :autoNav="false" :on-click-breadcrumb-item="handleClickBreadcrumbItem" />
	</el-dialog>
</template>

<script setup lang="ts" name="move">
import { ref, onBeforeMount, watch } from 'vue'
import Breadcrumb, { BreadcrumbItem } from '../Breadcrumb/index.vue'
import { getPath } from '@/api/modules/storage'

interface MoveProps {
	id?: string
	title: string
	onClose?: () => void
}

const props = withDefaults(defineProps<MoveProps>(), {
	id: () => '64352d45f913e41c75a8303c',
	title: () => ''
})
const id = ref(props.id)
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

onBeforeMount(() => fetchPath())

watch(
	() => props.id,
	() => (id.value = props.id)
)

const handleClose = () => props.onClose && props.onClose()

const handleClickBreadcrumbItem = (item: BreadcrumbItem) => console.log(`item :>> ${JSON.stringify(item, null, 2)}`)
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
