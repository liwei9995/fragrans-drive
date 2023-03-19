<template>
	<div class="action-button-wrapper">
		<el-dropdown trigger="click" @command="handleCommand">
			<div class="action">
				<el-icon :size="iconSize">
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
								:before-upload="handleBeforeUpload"
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
</template>

<script setup lang="ts" name="action-button">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { UploadInstance, UploadProps } from 'element-plus'
import { GlobalStore } from '@/store'

const route = useRoute()
const router = useRouter()

const parentId = (route.params.id as string) || 'root'
const uploadPayload = ref({ parentId })

const uploadRef = ref<UploadInstance>()
const storageAction = computed(() => `${import.meta.env.VITE_API_URL}/v1/storage/upload`)
const globalStore = GlobalStore()
const uploadHeaders = {
	Authorization: `Bearer ${globalStore.token}`
}

type ActionItem = {
	id?: string
	name: string
	isUpload?: boolean
}

interface ActionButtonProps {
	actionItems?: Partial<ActionItem>[]
	uploadFileLimit?: number
	iconSize?: number
	tapActionItem?: (command: string | number | object) => void
	onUploadChange?: UploadProps['onChange']
	onUploadExceed?: UploadProps['onExceed']
	beforeUpload?: UploadProps['beforeUpload']
}

const props = withDefaults(defineProps<ActionButtonProps>(), {
	uploadFileLimit: () => 10,
	actionItems: () => [],
	iconSize: () => 32
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

const handleBeforeUpload: UploadProps['beforeUpload'] = rawFile => {
	console.log(`rawFile - handleBeforeUpload :>> ${JSON.stringify(rawFile, null, 2)}`)
	props.beforeUpload && props.beforeUpload(rawFile)
}
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
