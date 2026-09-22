<script setup lang="ts" name="upload">
import type {
  UploadInstance,
  UploadProps,
  UploadRequestOptions,
} from 'element-plus'
import { ElMessage } from 'element-plus'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { uploadChunk, uploadComplete, uploadInit } from '@/api/modules/storage'
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
  nextTick(() => {
    clearFiles(['success'])
  })
}

const handleError: UploadProps['onError'] = (
  error,
  uploadFile,
  uploadFiles,
) => {
  props.onUploadError?.(error, uploadFile, uploadFiles)
  const err = error as any
  if (err?.response?.status === 413 || err?.status === 413) {
    ElMessage.error(
      '文件过大，超出服务器上传限制 (413 Request Entity Too Large)',
    )
  } else if (err?.response?.data?.message || err?.response?.data?.error) {
    ElMessage.error(err.response.data.message || err.response.data.error)
  }
  nextTick(() => {
    clearFiles(['fail'])
  })
}

const customUploadRequest = async (options: UploadRequestOptions) => {
  const { file, onProgress, onSuccess, onError } = options

  try {
    const fileHash = await calculateFileHash(file, (percent) => {
      onProgress({ percent: percent * 0.1 } as any)
    })

    const CHUNK_SIZE = 5 * 1024 * 1024
    const totalChunks = Math.ceil(file.size / CHUNK_SIZE) || 1

    const initRes = await uploadInit({
      parentId: uploadPayload.value.parentId,
      name: file.name,
      hash: fileHash,
      size: file.size,
      chunkSize: CHUNK_SIZE,
      totalChunks,
    })

    if (initRes.completed) {
      onProgress({ percent: 100 } as any)
      onSuccess(initRes)
      return
    }

    const { uploadId, uploadedChunks } = initRes
    const uploadedSet = new Set(uploadedChunks || [])

    for (let i = 0; i < totalChunks; i++) {
      if (uploadedSet.has(i)) {
        const percent = 10 + Math.round(((i + 1) / totalChunks) * 85)
        onProgress({ percent } as any)
        continue
      }

      const start = i * CHUNK_SIZE
      const end = Math.min(file.size, (i + 1) * CHUNK_SIZE)
      const chunkBlob = file.slice(start, end)

      const chunkFormData = new FormData()
      chunkFormData.append('uploadId', uploadId)
      chunkFormData.append('chunkIndex', i.toString())
      chunkFormData.append('chunk', chunkBlob, `${i}.part`)

      await uploadChunk(chunkFormData, (progressEvent) => {
        const { loaded, total } = progressEvent
        if (total) {
          const chunkFraction = loaded / total
          const overallProgress = (i + chunkFraction) / totalChunks
          const percent = 10 + Math.round(overallProgress * 85)
          onProgress({ percent } as any)
        }
      })
    }

    const completeRes = await uploadComplete({ uploadId })
    onProgress({ percent: 100 } as any)
    onSuccess(completeRes)
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
