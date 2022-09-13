<template>
	<header class="header">
		<div class="page-title">
			<el-breadcrumb separator="›">
				<template v-for="item in breadcrumbItems" :key="item.id">
					<el-breadcrumb-item v-if="!item.isOmit && !item.isHighlight">
						<div class="breadcrumb-item-content">{{ item.text }}</div>
					</el-breadcrumb-item>
					<el-breadcrumb-item v-else-if="item.isOmit">
						<el-icon :size="12">
							<MoreFilled />
						</el-icon>
					</el-breadcrumb-item>
					<div class="breadcrumb-item-content highlight" v-else-if="item.isHighlight">
						{{ item.text }}
					</div>
				</template>
			</el-breadcrumb>
		</div>
		<div class="actions">
			<div class="search">
				<el-icon :size="24">
					<Search />
				</el-icon>
			</div>
			<el-dropdown trigger="click" @command="handleCommand">
				<div class="action">
					<el-icon :size="32">
						<CirclePlusFilled />
					</el-icon>
				</div>
				<template #dropdown>
					<el-dropdown-menu>
						<el-dropdown-item v-for="item in actionItems" :key="item.id" :command="item.id">
							<template v-if="item.isUpload">
								<el-upload
									ref="uploadRef"
									class="upload-zone"
									multiple
									:action="storageAction"
									:headers="uploadHeaders"
									:show-file-list="false"
									:limit="10"
								>
									<template #trigger>{{ item.name }}</template>
								</el-upload>
							</template>
							<template v-else>{{ item.name }}</template>
						</el-dropdown-item>
					</el-dropdown-menu>
				</template>
			</el-dropdown>
		</div>
	</header>
</template>

<script setup lang="ts" name="header">
import { ref, computed } from 'vue'
import { UploadInstance } from 'element-plus'
import { GlobalStore } from '@/store'

const uploadRef = ref<UploadInstance>()
const storageAction = computed(() => `${import.meta.env.VITE_API_URL}/v1/storage/upload`)
const globalStore = GlobalStore()
const uploadHeaders = {
	Authorization: `Bearer ${globalStore.token}`
}

type BreadcrumbItem = {
	id?: string
	text?: string
	isHighlight?: boolean
	isOmit?: boolean
}

type ActionItem = {
	id?: string
	name: string
	isUpload?: boolean
}

interface HeaderProps {
	breadcrumbItems?: Partial<BreadcrumbItem>[]
	actionItems?: Partial<ActionItem>[]
	tapActionItem?: (command: string | number | object) => void
}

const props = withDefaults(defineProps<HeaderProps>(), {
	breadcrumbItems: () => [],
	actionItems: () => []
})

const handleCommand = (command: string | number | object) => props.tapActionItem && props.tapActionItem(command)
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
