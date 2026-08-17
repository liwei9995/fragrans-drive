import type { UploadFiles, UploadProps } from 'element-plus'
import { ref } from 'vue'

interface UploadQueueOptions {
  onChange: () => void
  onComplete: () => void
  showStatus: () => void
}

export const useUploadQueue = ({
  onChange,
  onComplete,
  showStatus,
}: UploadQueueOptions) => {
  const uploadedFiles = ref<UploadFiles>([])
  const uploadPercentage = ref(0)
  const notificationTitle = ref('')
  const notificationType = ref('info')
  let cleanedUp = false

  const reset = () => {
    uploadedFiles.value = []
  }

  const handleUploadChange: UploadProps['onChange'] = (
    uploadFile,
    uploadFiles,
  ) => {
    onChange()

    if (uploadFiles.length === 0 && uploadedFiles.value.length === 0) {
      if (!uploadFile || !['success', 'fail'].includes(uploadFile.status))
        return
    }

    const activeFiles = uploadFiles.filter((file) =>
      ['uploading', 'ready'].includes(file.status),
    )
    const finishedInBatch = uploadFiles.filter((file) =>
      ['success', 'fail'].includes(file.status),
    )

    if (
      uploadFile &&
      ['success', 'fail'].includes(uploadFile.status) &&
      !finishedInBatch.some((file) => file.uid === uploadFile.uid)
    ) {
      finishedInBatch.push(uploadFile)
    }

    const combined = [...uploadedFiles.value]
    for (const file of finishedInBatch) {
      const index = combined.findIndex((item) => item.uid === file.uid)
      if (index > -1) combined[index] = file
      else combined.push(file)
    }
    uploadedFiles.value = combined

    const pendingCount = activeFiles.filter(
      (file) => !finishedInBatch.some((done) => done.uid === file.uid),
    ).length
    const isAllDone = pendingCount === 0
    const successCount = combined.filter(
      (file) => file.status === 'success',
    ).length
    const failCount = combined.filter((file) => file.status === 'fail').length

    if (!isAllDone) {
      notificationTitle.value = `正在上传 ∙ 剩余${pendingCount}项`
      notificationType.value = 'uploading'
      cleanedUp = false
    } else if (combined.length > 0) {
      notificationTitle.value =
        failCount > 0
          ? `上传完成 ∙ 成功${successCount}项 失败${failCount}项`
          : `上传完成 ∙ 共${successCount}项`
      notificationType.value = failCount > 0 ? 'error' : 'success'
      uploadPercentage.value = 0
    }

    showStatus()

    if (isAllDone && combined.length > 0 && !cleanedUp) {
      cleanedUp = true
      reset()
      onComplete()
    }
  }

  const handleUploadProgress: UploadProps['onProgress'] = (
    _event,
    _uploadFile,
    uploadFiles,
  ) => {
    const totalSize = uploadFiles.reduce(
      (total, file) => total + (file.size || 0),
      0,
    )
    const uploadedSize = uploadFiles.reduce(
      (total, file) =>
        total +
        (['uploading', 'success'].includes(file.status)
          ? ((file.size || 0) * (file.percentage || 0)) / 100
          : 0),
      0,
    )

    uploadPercentage.value =
      totalSize === 0 ? 0 : (uploadedSize / totalSize) * 100
  }

  return {
    handleUploadChange,
    handleUploadProgress,
    notificationTitle,
    notificationType,
    reset,
    uploadedFiles,
    uploadPercentage,
  }
}
