<script setup lang="ts" name="upload">
import axios from 'axios'
import type {
  UploadInstance,
  UploadProps,
  UploadRequestOptions,
} from 'element-plus'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { GlobalStore } from '@/store'
import { calculateFileHash } from '@/utils/fileHash'

const route = useRoute()
const router = useRouter()

const parentId = (route.params.id as string) || 'root'
const uploadPayload = ref({ parentId })

const uploadRef = ref<UploadInstance>()
const storageAction = computed(
  () => `${import.meta.env.VITE_API_URL}/v1/storage/upload`,
)
const globalStore = GlobalStore()
const uploadHeaders = computed(() => ({
  Authorization: `Bearer ${globalStore.accessToken}`,
}))

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

const props = withDefaults(defineProps<UploaderProps>(), {
  multiple: () => true,
  showFileList: () => false,
  limit: () => 10,
})

const handleSuccess: UploadProps['onSuccess'] = (
  response,
  uploadFile,
  uploadFiles,
) => {
  props.onUploadSuccess?.(response, uploadFile, uploadFiles)
  props.onUploadChange?.(uploadFile, uploadFiles)
  clearFiles(['success'])
}

const handleError: UploadProps['onError'] = (
  error,
  uploadFile,
  uploadFiles,
) => {
  props.onUploadError?.(error, uploadFile, uploadFiles)
  props.onUploadChange?.(uploadFile, uploadFiles)
  clearFiles(['fail'])
}

const customUploadRequest = async (options: UploadRequestOptions) => {
  const { file, onProgress, onSuccess, onError } = options

  try {
    const fileHash = await calculateFileHash(file, (percent) => {
      onProgress({ percent: percent * 0.1 } as any)
    })

    const formData = new FormData()
    formData.append('parentId', uploadPayload.value.parentId)
    formData.append('hash', fileHash)
    formData.append('size', file.size.toString())
    formData.append(options.filename || 'file', file)

    const response = await axios.post(storageAction.value, formData, {
      headers: uploadHeaders.value,
      onUploadProgress: (progressEvent) => {
        const { loaded, total } = progressEvent
        if (total) {
          const percent = 10 + Math.round((loaded / total) * 90)
          onProgress({ percent } as any)
        }
      },
    })

    onSuccess(response.data)
  } catch (error) {
    onError(error as any)
  }
}

const clearFiles = (
  status?: Array<'ready' | 'uploading' | 'success' | 'fail'>,
) => {
  uploadRef.value?.clearFiles(status)
}

watch(
  () => router.currentRoute.value,
  () => {
    const parentId = (route.params.id as string) || 'root'

    uploadPayload.value = { parentId }
  },
)

onBeforeUnmount(() => {
  clearFiles()
})
</script>

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
    :on-success="handleSuccess"
    :on-error="handleError"
    :before-upload="beforeUpload"
    :http-request="customUploadRequest"
  >
    <template #trigger>
      <slot name="trigger" />
    </template>
  </el-upload>
</template>
