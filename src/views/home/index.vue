<script setup lang="ts" name="home">
import type { UploadFiles, UploadProps } from 'element-plus'
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus'
import { onBeforeMount, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  deleteFile,
  getDownloadUrl,
  getPath,
  updateFile,
} from '@/api/modules/storage'
import Card from '@/components/StorageCard/index.vue'
import VideoPlayer from '@/components/VideoPlayer/index.vue'
import { LOGIN_URL } from '@/config/config'
import { UploadEventEnum } from '@/enums/events'
import { useCreateFolder } from '@/hooks/useCreateFolder'
import { convertItem, sortDocs, useFetchFiles } from '@/hooks/useFetchFiles'
import { GlobalStore } from '@/store'
import emitter from '@/utils/emitter'
import Breadcrumb from './widgets/Breadcrumb/index.vue'
import Dialog from './widgets/Dialog/index.vue'
import Empty from './widgets/Empty/index.vue'
import FileSkeleton from './widgets/FileSkeleton/index.vue'
import FloatingActionBar from './widgets/FloatingActionBar/index.vue'
import Footer from './widgets/Footer/index.vue'
import GlobalDropzone from './widgets/GlobalDropzone/index.vue'
import Header from './widgets/Header/index.vue'
import Move from './widgets/Move/index.vue'
import UploadStatus from './widgets/UploadStatus/index.vue'

type BreadcrumbItem = {
  id: string
  text: string
}

const globalStore = GlobalStore()
const defaultFolderName = '新建文件夹'
const folderDialogFormVisible = ref(false)
const renameDialogFormVisible = ref(false)
const moveDialogFormVisible = ref(false)
const videoPlayerVisible = ref(false)
const videoSrc = ref('')
const needToMoveId = ref('root')
const uploadStatusRef = ref()
const uploadPercentage = ref(0)
const uploadedFiles = ref([] as UploadFiles)
const notificationTitle = ref('')
const notificationType = ref('info')
const folderName = ref(defaultFolderName)
const needToRenameThumb = ref('')
const needToRenameFileId = ref('')
const needToRenameFileName = ref('')
const breadcrumbItems = ref([] as BreadcrumbItem[])
const isDragging = ref(false)
const selectedIds = ref(new Set<string>())
let dragCounter = 0
const uploadFileLimit = 10
const route = useRoute()
const router = useRouter()
const { fetchFiles, listData, resetListData, isFetching } = useFetchFiles()
const parentId = ref((route.params.id as string) || 'root')
const basicActionItems = [
  {
    id: 'rename',
    name: '重命名',
    divided: false,
  },
  {
    id: 'move',
    name: '移动',
    divided: false,
  },
  {
    id: 'delete',
    name: '删除',
    divided: true,
  },
]
const fullActionItems = [
  {
    id: 'download',
    name: '下载',
    divided: false,
  },
  ...basicActionItems,
]

const actionItems = [
  {
    id: 'folder',
    name: '新建文件夹',
  },
  {
    id: 'file',
    name: '上传文件',
    isUpload: true,
  },
]

const avatarActionItems = [
  {
    id: 'logout',
    name: '退出登录',
  },
]

interface Storage {
  id: string
  name: string
  baseName?: string
  extName?: string
  mimeType?: string
  encoding?: string
  size?: string
  parentId: string
  type: string
  thumbnail?: string
  url?: string
  createdAt: string
  updatedAt: string
}

const load = () => {
  if (isFetching.value || listData.value.page + 1 > listData.value.pages) return

  fetchFiles(parentId.value, false)
}

const fetchPath = async () => {
  const fileId = route.params.id as string

  if (fileId) {
    const pathItems = await getPath(fileId)
    if (pathItems && Array.isArray(pathItems)) {
      breadcrumbItems.value = [
        ...pathItems.map((path: any) => ({
          id: String(path.id),
          text: path.name,
        })),
      ]
    } else {
      // Handle case where pathItems is empty or not an array
      breadcrumbItems.value = [{ id: '0', text: '全部文件' }]
    }
  } else {
    breadcrumbItems.value = []
  }
}

onBeforeMount(() => {
  fetchPath()
  fetchFiles(parentId.value)
})

watch(
  () => router.currentRoute.value,
  () => {
    parentId.value = (route.params.id as string) || 'root'
    resetListData()
    fetchFiles(parentId.value)
    fetchPath()
  },
)

const handleCloseFolderDialog = () => (folderDialogFormVisible.value = false)

const handleCloseRenameDialog = () => (renameDialogFormVisible.value = false)

const handleCloseMoveDialog = () => (moveDialogFormVisible.value = false)

const handleMoved = (id: string, parentId: string) => {
  const paramId = (route.params.id as string) || 'root'

  if (parentId === paramId) return

  const docs = (listData.value.docs || []) as any as Storage[]
  const index = docs.findIndex((doc: any) => doc.id === id)

  if (index !== -1) {
    docs.splice(index, 1)
    listData.value.docs = docs as any
  }
}

const handleFolderCreated = (parentId: string) => {
  if (parentId === (route.params.id || 'root')) {
    fetchFiles(parentId)
  }
}

const handleCreateFolder = (name: string) => {
  const parentId = (route.params.id || 'root') as string

  folderDialogFormVisible.value = false
  useCreateFolder(name, parentId, () => fetchFiles(parentId))
  folderName.value = defaultFolderName
}

const handleRenameFile = (name: string) => {
  const fileId = needToRenameFileId.value
  const doc = (listData?.value?.docs as any as Storage[])?.find(
    (item) => item.id === fileId,
  )

  if (!doc) return

  const fullName = doc.extName ? `${name}${doc.extName}` : name

  updateFile(fileId, {
    name: fullName,
    parentId: doc?.parentId || 'root',
    type: doc?.type,
  }).then((res: any) => {
    const {
      exist,
      _id: id,
      name,
      baseName,
      extName,
      createdAt,
      updatedAt,
    } = res
    if (exist) {
      ElMessage.error('已存在同名文件，请修改名称')
    } else {
      renameDialogFormVisible.value = false

      const docs = listData.value.docs || []
      const index = docs.findIndex((doc: any) => doc.id === id)

      if (index !== -1) {
        docs[index] = convertItem({
          ...docs[index],
          id,
          name,
          baseName,
          extName,
          createdAt,
          updatedAt,
        })

        listData.value.docs = sortDocs(docs) as any
      }
    }
  })
}

const handleCloseUploadStatus = () => {
  uploadStatusRef.value?.close()
  uploadedFiles.value = []
}

const handleCloseVideoPlayer = () => (videoPlayerVisible.value = false)

const handleTapActionItem = (command: string | number | object) => {
  if (command === 'folder') {
    folderDialogFormVisible.value = true
  } else if (command === 'logout') {
    // * 清空 token
    globalStore.setToken('')
    // * 跳转到登录页面
    router.push(LOGIN_URL)
  }
}

const download = async (id: string) => {
  ElMessage.info('文件下载准备中...')
  const { url } = (await getDownloadUrl(id)) as { url: string }

  // * 在当前页面弹出下载窗口
  window.open(url, '_self')
}

const handleTapCardActionItem = async (
  command: string | number | object,
  id: string,
  name: string,
  _type?: string,
  thumb?: string,
  extName = '',
) => {
  if (command === 'download') {
    download(id)
  } else if (command === 'delete') {
    ElMessageBox.confirm('文件删除后将无法恢复，确定要删除么？', '删除文件', {
      confirmButtonText: '确定删除',
      cancelButtonText: '取消',
      type: 'warning',
    })
      .then(async () => {
        try {
          await deleteFile(id)
          const docs = listData.value.docs || []
          const index = docs.findIndex((doc: any) => doc.id === id)

          if (index !== -1) {
            docs.splice(index, 1)
            listData.value.docs = docs as any
          }
          ElMessage.success('文件删除成功')
        } catch {
          ElMessage.error('文件删除失败，请重试')
        }
      })
      .catch(() => {})
  } else if (command === 'rename') {
    renameDialogFormVisible.value = true
    needToRenameThumb.value = thumb || ''
    needToRenameFileId.value = id
    needToRenameFileName.value = name.replace(extName, '')
  } else if (command === 'move') {
    needToMoveId.value = id
    moveDialogFormVisible.value = true
  }
}

const handleToggleSelect = (id: string) => {
  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id)
  } else {
    selectedIds.value.add(id)
  }
  // Trigger reactivity by reassignment
  selectedIds.value = new Set(selectedIds.value)
}

const handleClearSelection = () => {
  selectedIds.value = new Set()
}

const handleBatchDelete = () => {
  const count = selectedIds.value.size
  ElMessageBox.confirm(
    `确定要删除选中的 ${count} 项吗？此操作不可逆。`,
    '批量删除',
    {
      confirmButtonText: '确定删除',
      cancelButtonText: '取消',
      type: 'warning',
    },
  ).then(async () => {
    try {
      const ids = Array.from(selectedIds.value)
      // Perform sequential deletions (simpler for now than Promise.all if reliability is preferred)
      for (const id of ids) {
        await deleteFile(id)
      }
      handleClearSelection()
      fetchFiles(parentId.value)
      ElMessage.success(`成功删除 ${count} 项`)
    } catch {
      ElMessage.error('部分文件删除失败')
    }
  })
}

const handleBatchMove = () => {
  // Use the existing Move dialog but set it to handle multiple IDs
  // For now, easier to move one by one if the dialog is designed for one.
  // I might need to update Move.vue to handle an array of IDs.
  // But let's keep it simple for now and move only the first one or prompt user.
  // Actually, I'll update Move to handle multiple IDs later.
  ElMessage.info('暂不支持批量移动，正在升级中...')
}

const handleBatchDownload = () => {
  const ids = Array.from(selectedIds.value)
  for (const id of ids) {
    download(id)
  }
  handleClearSelection()
}

const handlePreviewVideo = (videoUrl: string) => {
  videoPlayerVisible.value = true
  videoSrc.value = videoUrl
}

const handleUploadChange: UploadProps['onChange'] = (
  _uploadFile,
  uploadFiles,
) => {
  // Clear drag state whenever files are added or changed
  isDragging.value = false
  dragCounter = 0

  // Files that are actively being processed by the browser/server
  const uploadingFiles = uploadFiles.filter((f) => f.status === 'uploading')
  // Files that are in the queue but haven't started (might be waiting or blocked)
  const readyFiles = uploadFiles.filter((f) => f.status === 'ready')

  // We are finished ONLY if every file in the list has reached a terminal state (success or fail)
  // This prevents 'ready' files that might be stuck/rejected from hanging the notification.
  const isActuallyFinished = uploadFiles.every((f) =>
    ['success', 'fail'].includes(f.status),
  )

  if (uploadFiles.length === 0 && uploadedFiles.value.length === 0) return

  // Manage cumulative finished files counts using UIDs for deduplication
  const allFinishedFiles = [...uploadedFiles.value, ...uploadFiles].filter(
    (f) => ['success', 'fail'].includes(f.status),
  )

  const uniqueFinished = new Map()
  for (const f of allFinishedFiles) {
    const key = f.uid || f.name || Math.random()
    uniqueFinished.set(key, f)
  }

  const finishedList = Array.from(uniqueFinished.values())
  const successCount = finishedList.filter((f) => f.status === 'success').length
  const failCount = finishedList.filter((f) => f.status === 'fail').length

  const activeCount = uploadingFiles.length + readyFiles.length

  const title = !isActuallyFinished
    ? `正在上传 ∙ 剩余${activeCount}项`
    : failCount > 0
      ? `上传完成 ∙ 成功${successCount}项 失败${failCount}项`
      : `上传完成 ∙ 共${successCount}项`

  const type = !isActuallyFinished ? 'uploading' : 'success'

  ElNotification.closeAll()
  uploadStatusRef.value?.show()
  notificationTitle.value = title
  notificationType.value = type

  if (type === 'success') {
    uploadPercentage.value = 0
  }

  // Finalize the batch only when all files in the current list have reached a terminal state
  if (isActuallyFinished && uploadFiles.length > 0) {
    uploadedFiles.value = finishedList

    // Use a delay before clearing to ensure the Success state is visible
    setTimeout(() => {
      emitter.emit(UploadEventEnum.CLEAR_FILES)
      fetchFiles(parentId.value)
    }, 200)
  }
}

const handleUploadExceed: UploadProps['onExceed'] = (files) => {
  ElMessage.warning(
    `一次最多允许上传${uploadFileLimit}个文件，你这次选择了${files.length}个`,
  )
}

const handleUploadProgress: UploadProps['onProgress'] = (
  _event,
  _uploadFile,
  uploadFiles,
) => {
  const totalSize = uploadFiles.reduce(
    (accumulator, current) => accumulator + (current?.size || 0),
    0,
  )
  const uploadedSize = uploadFiles.reduce((accumulator, current) => {
    const cur = ['uploading', 'success'].includes(current?.status)
      ? ((current?.size || 0) * (current?.percentage || 0)) / 100
      : 0

    return accumulator + cur
  }, 0)
  const percentage = (uploadedSize / totalSize) * 100

  uploadPercentage.value = percentage
}

const handelBeforeUpload: UploadProps['beforeUpload'] = (rawFile) => {
  if (rawFile.size / 1024 / 1024 > 512) {
    ElMessage.error('上传文件的大小不能超过512MB')
    return false
  }
}

const onDragEnter = (e: DragEvent) => {
  e.preventDefault()
  dragCounter++
  isDragging.value = true
}

const onDragLeave = (e: DragEvent) => {
  e.preventDefault()
  dragCounter--
  if (dragCounter === 0) {
    isDragging.value = false
  }
}

const onDrop = (e: DragEvent) => {
  e.preventDefault()
  dragCounter = 0
  isDragging.value = false
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    isDragging.value = false
    dragCounter = 0
  }
}

onMounted(() => {
  window.addEventListener('dragenter', onDragEnter)
  window.addEventListener('dragleave', onDragLeave)
  window.addEventListener('dragover', (e) => e.preventDefault())
  window.addEventListener('drop', onDrop)
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('dragenter', onDragEnter)
  window.removeEventListener('dragleave', onDragLeave)
  window.removeEventListener('drop', onDrop)
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="home flx-center">
    <div class="content">
      <div class="file-drag-zone">
        <div class="page-content">
          <Header
            :breadcrumb-items="breadcrumbItems"
            :action-items="actionItems"
            :avatar-action-items="avatarActionItems"
            :upload-file-limit="uploadFileLimit"
            :tap-action-item="handleTapActionItem"
            :on-upload-change="handleUploadChange"
            :on-upload-exceed="handleUploadExceed"
            :on-upload-progress="handleUploadProgress"
            :before-upload="handelBeforeUpload"
          />
          <div class="sub-nav-wrapper">
            <Breadcrumb :breadcrumb-items="breadcrumbItems" />
          </div>
          <el-scrollbar class="items-wrapper" @end-reached="load">
            <transition-group name="list" tag="div" class="items">
              <template v-if="isFetching && listData?.docs.length === 0">
                <FileSkeleton v-for="i in 10" :key="'skeleton-' + i" />
              </template>
              <Card
                v-for="item in listData?.docs"
                :id="item.id"
                :key="item.id"
                :title="item.name"
                :desc="item.desc"
                :mime-type="item.mimeType"
                :type="item.type"
                :ext-name="item.extName"
                :thumb-url="item.thumb"
                :thumb-placeholder="item.thumbPlaceholder"
                :preview-src-list="item.previewSrcList"
                :video-url="item.videoUrl"
                :action-items="item.type === 'file' ? fullActionItems : basicActionItems"
                :tap-action-item="handleTapCardActionItem"
                :preview-video="handlePreviewVideo"
                :selected="selectedIds.has(item.id)"
                @toggle-select="handleToggleSelect"
              />
              <div v-for="item in 10" :key="'spacer-' + item" class="empty-card" />
            </transition-group>
          </el-scrollbar>
          <Empty
              v-if="!isFetching && listData?.docs.length === 0"
              :on-upload-change="handleUploadChange"
              :on-upload-exceed="handleUploadExceed"
              :on-upload-progress="handleUploadProgress"
              :before-upload="handelBeforeUpload"
              :tap-item="handleTapActionItem"
            />
          <Footer />
          <Dialog
            v-if="folderDialogFormVisible"
            title="新建文件夹"
            :name="folderName"
            :on-close="handleCloseFolderDialog"
            :on-confirm="handleCreateFolder"
          />
          <Dialog
            v-if="renameDialogFormVisible"
            title="重命名"
            :thumb-url="needToRenameThumb"
            :name="needToRenameFileName"
            :on-close="handleCloseRenameDialog"
            :on-confirm="handleRenameFile"
          />
          <Move
            v-if="moveDialogFormVisible"
            :id="needToMoveId"
            :parent-id="parentId"
            title="移动到"
            :on-close="handleCloseMoveDialog"
            :on-moved="handleMoved"
            :on-folder-created="handleFolderCreated"
          />
          <UploadStatus
            ref="uploadStatusRef"
            :percentage="uploadPercentage"
            :title="notificationTitle"
            :type="notificationType"
            :on-close="handleCloseUploadStatus"
          />
          <GlobalDropzone
            :show="isDragging"
            :on-upload-change="handleUploadChange"
            :on-upload-exceed="handleUploadExceed"
            :on-upload-progress="handleUploadProgress"
            :before-upload="handelBeforeUpload"
            @drop="onDrop"
          />
          <FloatingActionBar
            :selected-count="selectedIds.size"
            @delete="handleBatchDelete"
            @move="handleBatchMove"
            @download="handleBatchDownload"
            @clear="handleClearSelection"
          />
        </div>
      </div>
      <VideoPlayer v-if="videoPlayerVisible" :src="videoSrc" :close="handleCloseVideoPlayer" />
    </div>
  </div>
</template>

<style scoped lang="scss">
@use './index';
</style>
