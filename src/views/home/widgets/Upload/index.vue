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
		:on-change="onUploadChange"
		:on-exceed="onUploadExceed"
		:on-progress="onUploadProgress"
		:on-success="onUploadSuccess"
		:on-error="onUploadError"
		:before-upload="beforeUpload"
	>
		<template #trigger>
			<slot name="trigger" />
		</template>
	</el-upload>
</template>

<script setup lang="ts" name="upload">
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { UploadInstance, UploadProps } from 'element-plus'
import { GlobalStore } from '@/store'
import emitter from '@/utils/emitter'

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
	onUploadChange?: UploadProps['onChange']
	onUploadExceed?: UploadProps['onExceed']
	onUploadProgress?: UploadProps['onProgress']
	onUploadSuccess?: UploadProps['onSuccess']
	onUploadError?: UploadProps['onError']
	beforeUpload?: UploadProps['beforeUpload']
}

withDefaults(defineProps<UploaderProps>(), {
	multiple: () => true,
	showFileList: () => false,
	limit: () => 10
})

const clearFiles = (status?: Array<'ready' | 'uploading' | 'success' | 'fail'>) => uploadRef.value!.clearFiles(status)

emitter.on('clearFiles', clearFiles)

watch(
	() => router.currentRoute.value,
	() => {
		const parentId = (route.params.id as string) || 'root'

		uploadPayload.value = { parentId }
	}
)

onBeforeUnmount(() => {
	emitter.off('clearFiles')
})
</script>
