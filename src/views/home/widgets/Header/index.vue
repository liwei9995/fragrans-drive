<template>
	<header class="header">
		<div class="navbar-wrapper">
			<div class="header-container">
				<div class="header-title">
					<div class="logo-container" @click="handleClickGoHome">
						<el-image :src="logo" style="width: 24px; height: 24px" />
						<div class="logo-name">Fragrans</div>
					</div>
					<Breadcrumb :breadcrumb-items="breadcrumbItems" />
				</div>
				<div class="content">
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
											:data="uploadPayload"
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
					<el-dropdown trigger="click" @command="handleCommand">
						<div class="avatar">
							<el-avatar :src="avatar" :size="32" @error="errorHandler">
								<el-avatar :icon="UserFilled" />
							</el-avatar>
						</div>
						<template #dropdown>
							<el-dropdown-menu>
								<el-dropdown-item v-for="item in avatarActionItems" :key="item.id" :command="item.id">
									{{ item.name }}
								</el-dropdown-item>
							</el-dropdown-menu>
						</template>
					</el-dropdown>
				</div>
			</div>
		</div>
	</header>
</template>

<script setup lang="ts" name="header">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { UploadInstance, UploadProps } from 'element-plus'
import { UserFilled } from '@element-plus/icons-vue'
import { GlobalStore } from '@/store'
import Breadcrumb, { BreadcrumbItem } from '../Breadcrumb/index.vue'
import logo from '@/assets/logo.svg'

const route = useRoute()
const router = useRouter()
const uploadRef = ref<UploadInstance>()
const storageAction = computed(() => `${import.meta.env.VITE_API_URL}/v1/storage/upload`)
const globalStore = GlobalStore()
const uploadHeaders = {
	Authorization: `Bearer ${globalStore.token}`
}
const parentId = (route.params.id as string) || 'root'
const uploadPayload = ref({ parentId })

type ActionItem = {
	id?: string
	name: string
	isUpload?: boolean
}

interface HeaderProps {
	avatar?: string
	breadcrumbItems?: Partial<BreadcrumbItem>[]
	actionItems?: Partial<ActionItem>[]
	avatarActionItems?: Partial<ActionItem>[]
	uploadFileLimit?: number
	tapActionItem?: (command: string | number | object) => void
	onUploadChange?: UploadProps['onChange']
	onUploadExceed?: UploadProps['onExceed']
}

const props = withDefaults(defineProps<HeaderProps>(), {
	avatar: () => 'https://wpimg.wallstcn.com/f778738c-e4f8-4870-b634-56703b4acafe.gif?imageView2/1/w/80/h/80',
	uploadFileLimit: () => 10,
	breadcrumbItems: () => [],
	actionItems: () => [],
	avatarActionItems: () => []
})

watch(
	() => router.currentRoute.value,
	() => {
		const parentId = (route.params.id as string) || 'root'

		uploadPayload.value = { parentId }
	}
)

const handleCommand = (command: string | number | object) => props.tapActionItem && props.tapActionItem(command)

const handleUploadChange: UploadProps['onChange'] = (uploadFile, uploadFiles) =>
	props.onUploadChange && props.onUploadChange(uploadFile, uploadFiles)

const handleUploadExceed: UploadProps['onExceed'] = (files, uploadFiles) =>
	props.onUploadExceed && props.onUploadExceed(files, uploadFiles)

const handleClickGoHome = () => router.push('/home')

const errorHandler = () => true
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
