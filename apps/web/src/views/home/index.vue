<script setup lang="ts" name="home">
import type { UploadProps } from 'element-plus'
import { ElMessage, ElMessageBox } from 'element-plus'
import { onBeforeMount, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { deleteFile, getFile, getPath, updateFile } from '@/api/modules/storage'
import Card from '@/components/StorageCard/index.vue'
import VideoPlayer from '@/components/VideoPlayer/index.vue'
import { LOGIN_URL } from '@/config/config'
import { useCreateFolder } from '@/hooks/useCreateFolder'
import { convertItem, sortDocs, useFetchFiles } from '@/hooks/useFetchFiles'
import { useUploadQueue } from '@/hooks/useUploadQueue'
import { GlobalStore } from '@/store'
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
const folderName = ref(defaultFolderName)
const needToRenameThumb = ref('')
const needToRenameFileId = ref('')
const needToRenameFileName = ref('')
const breadcrumbItems = ref([] as BreadcrumbItem[])
const isDragging = ref(false)
const selectedIds = ref(new Set<string>())
let dragCounter = 0
let latestPathRequest = 0
const uploadFileLimit = 10
const route = useRoute()
const router = useRouter()
const { fetchFiles, listData, isFetching, showSkeleton } = useFetchFiles()
const parentId = ref((route.params.id as string) || 'root')
const {
  handleUploadChange,
  handleUploadProgress,
  notificationTitle,
  notificationType,
  reset: resetUploads,
  uploadPercentage,
} = useUploadQueue({
  onChange: () => {
    isDragging.value = false
    dragCounter = 0
  },
  onComplete: () => fetchFiles(parentId.value),
  showStatus: () => uploadStatusRef.value?.show(),
})
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

const load = () => {
  if (isFetching.value || listData.value.page + 1 > listData.value.pages) return

  fetchFiles(parentId.value, false)
}

const fetchPath = async () => {
  const request = ++latestPathRequest
  const fileId = route.params.id as string

  if (fileId) {
    const pathItems = await getPath(fileId)
    if (request !== latestPathRequest) return
    if (pathItems && Array.isArray(pathItems)) {
      breadcrumbItems.value = [
        ...pathItems.map((path) => ({
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
  () => route.params.id,
  () => {
    parentId.value = (route.params.id as string) || 'root'
    handleClearSelection()
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

  listData.value.docs = listData.value.docs.filter((doc) => doc.id !== id)
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
  const doc = listData.value.docs.find((item) => item.id === fileId)

  if (!doc) return

  const suffix = doc.extName
    ? doc.extName.startsWith('.')
      ? doc.extName
      : `.${doc.extName}`
    : ''
  const fullName = `${name}${suffix}`

  updateFile(fileId, {
    name: fullName,
    parentId: doc?.parentId || 'root',
    type: doc?.type,
  }).then((res) => {
    const { exist, id, name, baseName, extName, createdAt, updatedAt } = res
    if (exist) {
      ElMessage.error('已存在同名文件，请修改名称')
    } else {
      renameDialogFormVisible.value = false

      const docs = listData.value.docs
      const index = docs.findIndex((doc) => doc.id === id)

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

        listData.value.docs = sortDocs(docs)
      }
    }
  })
}

const handleCloseUploadStatus = () => {
  uploadStatusRef.value?.close()
  resetUploads()
}

const handleCloseVideoPlayer = () => (videoPlayerVisible.value = false)

const handleTapActionItem = (command: string | number | object) => {
  if (command === 'folder') {
    folderDialogFormVisible.value = true
  } else if (command === 'logout') {
    globalStore.$reset()
    router.push(LOGIN_URL)
  }
}

/** 通过带 Authorization 的直连接口下载，不依赖 URL 中的 token */
const download = async (id: string, filename?: string) => {
  ElMessage.info('文件下载准备中...')
  try {
    const blob = (await getFile(id)) as Blob
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = filename || id
    a.style.display = 'none'
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    // 不在此处提示「下载成功」：点击后仅唤起保存对话框，用户选择保存位置并确认后才真正完成，无法可靠检测
  } catch {
    ElMessage.error('下载失败，请重试')
  }
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
    download(id, name)
  } else if (command === 'delete') {
    ElMessageBox.confirm('文件删除后将无法恢复，确定要删除么？', '删除文件', {
      confirmButtonText: '确定删除',
      cancelButtonText: '取消',
      type: 'warning',
    })
      .then(async () => {
        try {
          await deleteFile(id)
          listData.value.docs = listData.value.docs.filter(
            (doc) => doc.id !== id,
          )
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
    const suffix = extName
      ? extName.startsWith('.')
        ? extName
        : `.${extName}`
      : ''
    needToRenameFileName.value =
      suffix && name.endsWith(suffix) ? name.slice(0, -suffix.length) : name
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
  )
    .then(async () => {
      const ids = Array.from(selectedIds.value)
      let failed = 0

      for (const id of ids) {
        try {
          await deleteFile(id)
        } catch {
          failed++
        }
      }

      handleClearSelection()
      await fetchFiles(parentId.value)

      if (failed === 0) {
        ElMessage.success(`成功删除 ${count} 项`)
      } else {
        ElMessage.warning(
          `删除完成：成功 ${count - failed} 项，失败 ${failed} 项`,
        )
      }
    })
    .catch(() => {})
}

const handlePreviewVideo = (videoUrl: string) => {
  videoPlayerVisible.value = true
  videoSrc.value = videoUrl
}

const handleUploadExceed: UploadProps['onExceed'] = (files) => {
  ElMessage.warning(
    `一次最多允许上传${uploadFileLimit}个文件，你这次选择了${files.length}个`,
  )
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

const onDragOver = (e: DragEvent) => e.preventDefault()

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    isDragging.value = false
    dragCounter = 0
  }
}

onMounted(() => {
  window.addEventListener('dragenter', onDragEnter)
  window.addEventListener('dragleave', onDragLeave)
  window.addEventListener('dragover', onDragOver)
  window.addEventListener('drop', onDrop)
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('dragenter', onDragEnter)
  window.removeEventListener('dragleave', onDragLeave)
  window.removeEventListener('dragover', onDragOver)
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
            <transition name="el-fade-in-linear">
              <div v-if="showSkeleton" class="items skeleton-container">
                <FileSkeleton v-for="i in 10" :key="'skeleton-' + i" />
              </div>
            </transition>
            <div class="items">
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
            </div>
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
