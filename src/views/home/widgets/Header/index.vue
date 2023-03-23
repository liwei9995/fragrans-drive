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
					<div class="action">
						<ActionButton
							:action-items="actionItems"
							:upload-file-limit="uploadFileLimit"
							:tap-action-item="handleCommand"
							:on-upload-change="onUploadChange"
							:on-upload-exceed="onUploadExceed"
							:on-upload-progress="onUploadProgress"
							:on-upload-success="onUploadSuccess"
							:on-upload-error="onUploadError"
							:before-upload="beforeUpload"
						/>
					</div>
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
	<div class="float-action">
		<ActionButton
			:action-items="actionItems"
			:upload-file-limit="uploadFileLimit"
			:icon-size="64"
			:tap-action-item="handleCommand"
			:on-upload-change="onUploadChange"
			:on-upload-exceed="onUploadExceed"
			:on-upload-progress="onUploadProgress"
			:on-upload-success="onUploadSuccess"
			:on-upload-error="onUploadError"
			:before-upload="beforeUpload"
		/>
	</div>
</template>

<script setup lang="ts" name="header">
import { useRouter } from 'vue-router'
import { UploadProps } from 'element-plus'
import { UserFilled } from '@element-plus/icons-vue'
import { HOME_URL } from '@/config/config'
import ActionButton from '../ActionButton/index.vue'
import Breadcrumb, { BreadcrumbItem } from '../Breadcrumb/index.vue'
import logo from '@/assets/logo.svg'

const router = useRouter()

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
	onUploadProgress?: UploadProps['onProgress']
	onUploadSuccess?: UploadProps['onSuccess']
	onUploadError?: UploadProps['onError']
	beforeUpload?: UploadProps['beforeUpload']
}

const props = withDefaults(defineProps<HeaderProps>(), {
	avatar: () => 'https://wpimg.wallstcn.com/f778738c-e4f8-4870-b634-56703b4acafe.gif?imageView2/1/w/80/h/80',
	uploadFileLimit: () => 10,
	breadcrumbItems: () => [],
	actionItems: () => [],
	avatarActionItems: () => []
})

const handleCommand = (command: string | number | object) => props.tapActionItem && props.tapActionItem(command)

const handleClickGoHome = () => router.push(HOME_URL)

const errorHandler = () => true
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
