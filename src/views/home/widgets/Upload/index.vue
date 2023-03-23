<template>
	<el-upload
		ref="uploadRef"
		class="upload-zone"
		:multiple="multiple"
		:action="storageAction"
		:data="uploadPayload"
		:headers="uploadHeaders"
		:show-file-list="showFileList"
		:limit="limit"
		:on-change="handleUploadChange"
		:on-exceed="handleUploadExceed"
		:on-progress="handleUploadProgress"
		:before-upload="handleBeforeUpload"
	>
		<template #trigger>
			<slot name="trigger" />
		</template>
	</el-upload>
</template>

<script setup lang="ts" name="upload">
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

interface UploaderProps {
	multiple?: boolean
	showFileList?: boolean
	limit?: number
	onChange?: UploadProps['onChange']
	onExceed?: UploadProps['onExceed']
	onProgress?: UploadProps['onProgress']
	beforeUpload?: UploadProps['beforeUpload']
}

const props = withDefaults(defineProps<UploaderProps>(), {
	multiple: () => true,
	showFileList: () => false,
	limit: () => 10
})

watch(
	() => router.currentRoute.value,
	() => {
		const parentId = (route.params.id as string) || 'root'

		uploadPayload.value = { parentId }
	}
)

const handleUploadChange: UploadProps['onChange'] = (uploadFile, uploadFiles) =>
	props.onChange && props.onChange(uploadFile, uploadFiles)

const handleUploadExceed: UploadProps['onExceed'] = (files, uploadFiles) => props.onExceed && props.onExceed(files, uploadFiles)

const handleUploadProgress: UploadProps['onProgress'] = (event, uploadFile, uploadFiles) =>
	props.onProgress && props.onProgress(event, uploadFile, uploadFiles)

const handleBeforeUpload: UploadProps['beforeUpload'] = rawFile => {
	props.beforeUpload && props.beforeUpload(rawFile)
}
</script>
