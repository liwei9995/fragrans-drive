<script setup lang="ts" name="trash-dialog">
import {
  Delete,
  DeleteFilled,
  Loading,
  Refresh,
  RefreshLeft,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { computed, onMounted, ref, watch } from 'vue'
import type { StorageNode } from '@/api/interface'
import {
  emptyTrash,
  getTrashList,
  restoreFile,
  restoreTrash,
} from '@/api/modules/storage'
import { getThumb } from '@/utils/thumb'

interface TrashDialogProps {
  visible: boolean
}

const props = defineProps<TrashDialogProps>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'restored'): void
}>()

const loading = ref(false)
const actionLoading = ref(false)
const trashItems = ref<StorageNode[]>([])
const total = ref(0)
const page = ref(1)
const limit = ref(50)
const selectedIds = ref(new Set<string>())

const formatBytes = (bytes?: number) => {
  if (typeof bytes !== 'number' || bytes <= 0) return '-'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / k ** i).toFixed(i === 0 ? 0 : 1)} ${sizes[i]}`
}

const formatDate = (dateStr?: string) => {
  if (!dateStr) return '-'
  const d = new Date(dateStr)
  if (Number.isNaN(d.getTime())) return '-'
  return `${d.getFullYear()}/${String(d.getMonth() + 1).padStart(2, '0')}/${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}

const fetchTrash = async () => {
  loading.value = true
  try {
    const res = await getTrashList({
      page: page.value,
      limit: limit.value,
      viewMode: 'all',
    })
    trashItems.value = res.docs || []
    total.value = res.total ?? res.docs.length
    // Clear selections that no longer exist
    const currentIds = new Set(trashItems.value.map((i) => i.id))
    selectedIds.value = new Set(
      Array.from(selectedIds.value).filter((id) => currentIds.has(id)),
    )
  } catch (error) {
    console.error('Fetch trash error:', error)
    ElMessage.error('获取回收站文件失败')
  } finally {
    loading.value = false
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      selectedIds.value = new Set()
      page.value = 1
      fetchTrash()
    }
  },
  { immediate: true },
)

const isAllSelected = computed(
  () =>
    trashItems.value.length > 0 &&
    trashItems.value.every((i) => selectedIds.value.has(i.id)),
)

const isIndeterminate = computed(
  () =>
    selectedIds.value.size > 0 &&
    selectedIds.value.size < trashItems.value.length,
)

const handleToggleSelectAll = () => {
  if (isAllSelected.value) {
    selectedIds.value = new Set()
  } else {
    selectedIds.value = new Set(trashItems.value.map((i) => i.id))
  }
}

const handleToggleSelect = (id: string) => {
  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id)
  } else {
    selectedIds.value.add(id)
  }
  selectedIds.value = new Set(selectedIds.value)
}

// 还原单项
const handleRestoreSingle = async (item: StorageNode) => {
  actionLoading.value = true
  try {
    await restoreFile(item.id)
    ElMessage.success(`"${item.name}" 已成功还原`)
    emit('restored')
    await fetchTrash()
  } catch (error: any) {
    const msg =
      error?.response?.data?.message || '还原失败，上级文件夹可能已被删除'
    ElMessage.error(msg)
  } finally {
    actionLoading.value = false
  }
}

// 还原选中项
const handleRestoreSelected = async () => {
  const count = selectedIds.value.size
  if (count === 0) return

  actionLoading.value = true
  try {
    const res = await restoreTrash({
      fileIds: Array.from(selectedIds.value),
    })
    ElMessage.success(`已成功还原 ${res.restoredDocs} 项`)
    selectedIds.value = new Set()
    emit('restored')
    await fetchTrash()
  } catch (error: any) {
    const msg = error?.response?.data?.message || '部分文件还原失败'
    ElMessage.error(msg)
  } finally {
    actionLoading.value = false
  }
}

// 还原全部
const handleRestoreAll = () => {
  if (trashItems.value.length === 0) return

  ElMessageBox.confirm(
    `确定要还原回收站中的全部 ${trashItems.value.length} 个项目吗？`,
    '还原全部',
    {
      confirmButtonText: '确定还原',
      cancelButtonText: '取消',
      type: 'info',
    },
  )
    .then(async () => {
      actionLoading.value = true
      try {
        const res = await restoreTrash({ restoreAll: true })
        ElMessage.success(`已成功还原全部 ${res.restoredDocs} 项`)
        selectedIds.value = new Set()
        emit('restored')
        await fetchTrash()
      } catch (error: any) {
        const msg = error?.response?.data?.message || '还原失败'
        ElMessage.error(msg)
      } finally {
        actionLoading.value = false
      }
    })
    .catch(() => {})
}

// 清空回收站
const handleEmptyTrash = () => {
  if (trashItems.value.length === 0) return

  ElMessageBox.confirm(
    '清空回收站将彻底删除所有项目，释放磁盘空间，且无法恢复。确定继续吗？',
    '彻底清空回收站',
    {
      confirmButtonText: '清空并永久删除',
      cancelButtonText: '取消',
      confirmButtonClass: 'el-button--danger',
      type: 'warning',
    },
  )
    .then(async () => {
      actionLoading.value = true
      try {
        const res = await emptyTrash()
        ElMessage.success(`回收站已清空，彻底删除 ${res.deletedDocs} 项数据`)
        selectedIds.value = new Set()
        emit('restored')
        await fetchTrash()
      } catch (error) {
        console.error('Empty trash error:', error)
        ElMessage.error('清空回收站失败')
      } finally {
        actionLoading.value = false
      }
    })
    .catch(() => {})
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    title="回收站"
    width="780px"
    destroy-on-close
    class="trash-dialog"
    @close="emit('close')"
  >
    <div class="trash-container">
      <div class="trash-toolbar">
        <div class="toolbar-left">
          <span class="trash-count-badge">
            共 {{ total }} 项已删除文件
          </span>
          <el-button
            :icon="Refresh"
            circle
            size="small"
            :loading="loading"
            @click="fetchTrash"
          />
        </div>

        <div class="toolbar-right">
          <el-button
            v-if="selectedIds.size > 0"
            type="primary"
            size="small"
            :icon="RefreshLeft"
            :loading="actionLoading"
            @click="handleRestoreSelected"
          >
            还原所选 ({{ selectedIds.size }})
          </el-button>

          <el-button
            v-if="trashItems.length > 0"
            type="default"
            size="small"
            :icon="RefreshLeft"
            :disabled="actionLoading"
            @click="handleRestoreAll"
          >
            一键全部还原
          </el-button>

          <el-button
            v-if="trashItems.length > 0"
            type="danger"
            size="small"
            plain
            :icon="DeleteFilled"
            :disabled="actionLoading"
            @click="handleEmptyTrash"
          >
            清空回收站
          </el-button>
        </div>
      </div>

      <div v-if="loading && trashItems.length === 0" class="trash-empty-state">
        <el-icon class="is-loading" :size="28"><Loading /></el-icon>
        <span>正在载入回收站...</span>
      </div>

      <div v-else-if="trashItems.length === 0" class="trash-empty-state">
        <div class="empty-icon-box">
          <el-icon :size="48" color="#a0aec0"><Delete /></el-icon>
        </div>
        <span class="empty-title">回收站空空如也</span>
        <span class="empty-subtitle">删除的文件和文件夹会安全地暂存在这里</span>
      </div>

      <div v-else class="trash-list-wrapper">
        <div class="trash-table-header">
          <div class="col-check">
            <el-checkbox
              :model-value="isAllSelected"
              :indeterminate="isIndeterminate"
              @change="handleToggleSelectAll"
            />
          </div>
          <div class="col-name">名称</div>
          <div class="col-size">大小</div>
          <div class="col-date">删除时间</div>
          <div class="col-action">操作</div>
        </div>

        <el-scrollbar max-height="420px">
          <div class="trash-table-body">
            <div
              v-for="item in trashItems"
              :key="item.id"
              class="trash-row"
              :class="{ selected: selectedIds.has(item.id) }"
              @click="handleToggleSelect(item.id)"
            >
              <div class="col-check" @click.stop>
                <el-checkbox
                  :model-value="selectedIds.has(item.id)"
                  @change="handleToggleSelect(item.id)"
                />
              </div>

              <div class="col-name">
                <img
                  :src="getThumb(item.extName, item.type)"
                  :alt="item.name"
                  class="row-icon"
                />
                <span class="row-name" :title="item.name">{{ item.name }}</span>
              </div>

              <div class="col-size">
                {{ item.type === 'folder' ? '-' : formatBytes(item.size) }}
              </div>

              <div class="col-date">
                {{ formatDate(item.updatedAt) }}
              </div>

              <div class="col-action" @click.stop>
                <el-tooltip content="还原此项目" placement="top">
                  <el-button
                    type="primary"
                    link
                    size="small"
                    :icon="RefreshLeft"
                    :disabled="actionLoading"
                    @click="handleRestoreSingle(item)"
                  >
                    还原
                  </el-button>
                </el-tooltip>
              </div>
            </div>
          </div>
        </el-scrollbar>
      </div>
    </div>
  </el-dialog>
</template>

<style lang="scss">
.trash-dialog {
  border-radius: 16px;
  overflow: hidden;
  max-width: calc(100vw - 20px);
  margin: 16px auto !important;

  @media (max-width: 768px) {
    --el-dialog-width: calc(100vw - 20px) !important;
    width: calc(100vw - 20px) !important;
    max-width: calc(100vw - 20px) !important;
    margin: 12px auto !important;
  }

  .el-dialog__header {
    margin-right: 0;
    padding: 18px 24px;
    border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));

    @media (max-width: 640px) {
      padding: 14px 16px;
    }

    .el-dialog__title {
      font-size: 17px;
      font-weight: 600;
    }
  }

  .el-dialog__body {
    padding: 0 !important;
  }
}
</style>

<style scoped lang="scss">
.trash-container {
  display: flex;
  flex-direction: column;
}

.trash-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background-color: rgba(0, 0, 0, 0.02);
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));

  @media (max-width: 640px) {
    flex-direction: column;
    align-items: stretch;
    gap: 10px;
    padding: 10px 14px;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 10px;

    @media (max-width: 640px) {
      width: 100%;
      justify-content: space-between;
    }

    .trash-count-badge {
      font-size: 13px;
      color: #666;
      font-weight: 500;
      white-space: nowrap;
    }
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 8px;

    @media (max-width: 640px) {
      width: 100%;
      justify-content: flex-end;
      flex-wrap: wrap;
      gap: 6px;
    }
  }
}

.trash-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  gap: 10px;
  color: #888;
  text-align: center;

  @media (max-width: 640px) {
    padding: 40px 16px;
  }

  .empty-icon-box {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background-color: rgba(0, 0, 0, 0.04);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 6px;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-color, #444);
  }

  .empty-subtitle {
    font-size: 13px;
    color: #999;
  }
}

.trash-list-wrapper {
  display: flex;
  flex-direction: column;
}

.trash-table-header {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  font-size: 12px;
  font-weight: 600;
  color: #888;
  background-color: rgba(0, 0, 0, 0.015);
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.05));

  @media (max-width: 640px) {
    padding: 8px 10px;
  }
}

.trash-table-body {
  display: flex;
  flex-direction: column;
}

.trash-row {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.04));
  cursor: pointer;
  transition: background-color 0.15s ease;

  @media (max-width: 640px) {
    padding: 8px 10px;
  }

  &:hover {
    background-color: rgba(0, 143, 253, 0.04);
  }

  &.selected {
    background-color: rgba(0, 143, 253, 0.08);
  }
}

.col-check {
  width: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.col-name {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;

  .row-icon {
    width: 28px;
    height: 28px;
    object-fit: contain;
    flex-shrink: 0;
  }

  .row-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-color, #333);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.col-size {
  width: 90px;
  font-size: 12px;
  color: #777;
  text-align: right;
  padding-right: 16px;
  flex-shrink: 0;

  @media (max-width: 640px) {
    width: 65px;
    padding-right: 8px;
  }
}

.col-date {
  width: 140px;
  font-size: 12px;
  color: #777;
  flex-shrink: 0;

  @media (max-width: 640px) {
    display: none;
  }
}

.col-action {
  width: 70px;
  text-align: right;
  flex-shrink: 0;

  @media (max-width: 640px) {
    width: 50px;
  }
}
</style>
