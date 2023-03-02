<template>
	<header class="header">
		<div class="page-title">
			<el-breadcrumb separator="›">
				<span class="el-breadcrumb__item" :class="{ current: breadcrumbItems.length === 0 }" @click="handleClickGoHome">
					<span class="el-breadcrumb__inner is-link" role="link">
						<div class="breadcrumb-item-content">文件</div>
					</span>
					<span v-if="breadcrumbItems.length > 0" class="el-breadcrumb__separator" role="presentation">›</span>
				</span>
				<span class="el-breadcrumb breadcrumb-items-wrapper">
					<template v-for="item in breadcrumbItems" :key="item.id">
						<el-breadcrumb-item v-if="!item.isOmit && !item.isHighlight" :to="{ path: '/home/' + item.id }">
							<div class="breadcrumb-item-content">{{ item.text }}</div>
						</el-breadcrumb-item>
						<el-breadcrumb-item v-else-if="item.isOmit">
							<el-icon :size="12">
								<MoreFilled />
							</el-icon>
						</el-breadcrumb-item>
						<div class="breadcrumb-item-content highlight" v-else-if="item.isHighlight" :to="{ path: '/home/' + item.id }">
							{{ item.text }}
						</div>
					</template>
				</span>
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
									:data="data"
									:headers="uploadHeaders"
									:show-file-list="false"
									:limit="uploadFileLimit"
									:on-change="handleUploadChange"
									:on-exceed="handleUploadExceed"
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
import { useRoute, useRouter } from 'vue-router'
import { UploadInstance, UploadProps } from 'element-plus'
import { GlobalStore } from '@/store'

const route = useRoute()
const router = useRouter()
const uploadRef = ref<UploadInstance>()
const storageAction = computed(() => `${import.meta.env.VITE_API_URL}/v1/storage/upload`)
const globalStore = GlobalStore()
const uploadHeaders = {
	Authorization: `Bearer ${globalStore.token}`
}
const parentId = (route.params.id as string) || 'root'
const data = { parentId }

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
	uploadFileLimit?: number
	tapActionItem?: (command: string | number | object) => void
	onUploadChange?: UploadProps['onChange']
	onUploadExceed?: UploadProps['onExceed']
}

const props = withDefaults(defineProps<HeaderProps>(), {
	uploadFileLimit: () => 10,
	breadcrumbItems: () => [],
	actionItems: () => []
})

const handleCommand = (command: string | number | object) => props.tapActionItem && props.tapActionItem(command)

const handleUploadChange: UploadProps['onChange'] = (uploadFile, uploadFiles) =>
	props.onUploadChange && props.onUploadChange(uploadFile, uploadFiles)

const handleUploadExceed: UploadProps['onExceed'] = (files, uploadFiles) =>
	props.onUploadExceed && props.onUploadExceed(files, uploadFiles)

const handleClickGoHome = () => router.push('/home')
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
