<script setup lang="ts">
import {
  ArrowLeft,
  ArrowRight,
  Close,
  Document,
  Download,
  TopRight,
} from '@element-plus/icons-vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  formatFileSize,
  getPreviewBadge,
  getPreviewType,
} from './previewHelper'
import type { FilePreviewItem } from './types'
import AudioViewer from './viewers/AudioViewer.vue'
import CodeTextViewer from './viewers/CodeTextViewer.vue'
import CsvViewer from './viewers/CsvViewer.vue'
import ImageViewer from './viewers/ImageViewer.vue'
import MarkdownViewer from './viewers/MarkdownViewer.vue'
import PdfViewer from './viewers/PdfViewer.vue'
import UnsupportedViewer from './viewers/UnsupportedViewer.vue'
import VideoViewer from './viewers/VideoViewer.vue'

interface Props {
  visible: boolean
  file: FilePreviewItem | null
  fileList?: FilePreviewItem[]
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  file: null,
  fileList: () => [],
})

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'change-file', item: FilePreviewItem): void
  (e: 'download', item: FilePreviewItem): void
}>()

const activeFile = ref<FilePreviewItem | null>(props.file)

watch(
  () => props.file,
  (newVal) => {
    activeFile.value = newVal
  },
  { immediate: true },
)

const previewType = computed(() => {
  if (!activeFile.value) return 'unsupported'
  return getPreviewType(
    activeFile.value.extName,
    activeFile.value.mimeType,
    activeFile.value.name,
  )
})

const badge = computed(() => {
  if (!activeFile.value)
    return { label: 'FILE', color: '#6366f1', bg: 'rgba(99, 102, 241, 0.1)' }
  return getPreviewBadge(previewType.value, activeFile.value.extName)
})

const formattedSize = computed(() => {
  return formatFileSize(activeFile.value?.size)
})

// Navigation among file list (filter only files)
const onlyFiles = computed(() => {
  return props.fileList.filter(
    (item) => (item as unknown as { type?: string }).type !== 'folder',
  )
})

const currentIndex = computed(() => {
  if (!activeFile.value || onlyFiles.value.length === 0) return -1
  return onlyFiles.value.findIndex((f) => f.id === activeFile.value?.id)
})

const hasPrev = computed(() => currentIndex.value > 0)
const hasNext = computed(
  () =>
    currentIndex.value >= 0 && currentIndex.value < onlyFiles.value.length - 1,
)

const handlePrev = () => {
  if (hasPrev.value) {
    const prevItem = onlyFiles.value[currentIndex.value - 1]
    activeFile.value = prevItem
    emit('change-file', prevItem)
  }
}

const handleNext = () => {
  if (hasNext.value) {
    const nextItem = onlyFiles.value[currentIndex.value + 1]
    activeFile.value = nextItem
    emit('change-file', nextItem)
  }
}

const handleClose = () => {
  emit('close')
}

const handleDownload = () => {
  if (activeFile.value) {
    emit('download', activeFile.value)
  }
}

const handleOpenInNewTab = () => {
  if (activeFile.value?.url) {
    window.open(activeFile.value.url, '_blank')
  }
}

const handleGlobalKeydown = (e: KeyboardEvent) => {
  if (!props.visible) return

  // Don't trigger when user is typing inside an input/textarea
  const targetTag = (e.target as HTMLElement)?.tagName
  if (targetTag === 'INPUT' || targetTag === 'TEXTAREA') return

  if (e.key === 'Escape') {
    handleClose()
  } else if (e.key === 'ArrowLeft' && previewType.value !== 'video') {
    handlePrev()
  } else if (e.key === 'ArrowRight' && previewType.value !== 'video') {
    handleNext()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
})
</script>

<template>
  <teleport to="body">
    <transition name="preview-fade">
      <div v-if="visible && activeFile" class="file-preview-overlay" role="dialog" aria-modal="true">
        <div class="preview-backdrop" @click="handleClose" />

        <div class="preview-container">
        <!-- Top Header Navigation & Meta -->
        <header class="preview-header">
          <!-- Left: File identity -->
          <div class="file-meta">
            <img
              v-if="activeFile.thumb"
              :src="activeFile.thumb"
              class="header-thumb"
              alt="thumb"
            />
            <el-icon v-else class="header-fallback-icon" :size="20"><Document /></el-icon>

            <div class="title-wrapper">
              <span class="file-title" :title="activeFile.name">{{ activeFile.name }}</span>
            </div>

            <span
              class="type-badge"
              :style="{ color: badge.color, backgroundColor: badge.bg, borderColor: badge.color }"
            >
              {{ badge.label }}
            </span>

            <span class="size-tag">{{ formattedSize }}</span>
          </div>

          <!-- Center: Previous / Next Navigation -->
          <div v-if="onlyFiles.length > 1" class="file-navigation">
            <button
              class="nav-btn"
              :disabled="!hasPrev"
              title="上一个文件 (←)"
              @click="handlePrev"
            >
              <el-icon :size="16"><ArrowLeft /></el-icon>
            </button>
            <span class="nav-indicator">
              {{ currentIndex + 1 }} / {{ onlyFiles.length }}
            </span>
            <button
              class="nav-btn"
              :disabled="!hasNext"
              title="下一个文件 (→)"
              @click="handleNext"
            >
              <el-icon :size="16"><ArrowRight /></el-icon>
            </button>
          </div>

          <!-- Right: Actions & Close -->
          <div class="header-actions">
            <el-tooltip v-if="activeFile.url" content="在浏览器独立标签页打开" placement="bottom">
              <button class="action-btn" @click="handleOpenInNewTab">
                <el-icon :size="16"><TopRight /></el-icon>
              </button>
            </el-tooltip>

            <el-tooltip content="下载文件" placement="bottom">
              <button class="action-btn download-btn" @click="handleDownload">
                <el-icon :size="16"><Download /></el-icon>
                <span>下载</span>
              </button>
            </el-tooltip>

            <div class="divider"></div>

            <el-tooltip content="关闭预览 (Esc)" placement="bottom">
              <button class="action-btn close-btn" @click="handleClose">
                <el-icon :size="18"><Close /></el-icon>
              </button>
            </el-tooltip>
          </div>
        </header>

        <!-- Main Body: Viewer dispatch based on PreviewType -->
        <main class="preview-main">
          <!-- Image -->
          <ImageViewer
            v-if="previewType === 'image'"
            :src="activeFile.url || ''"
            :name="activeFile.name"
          />

          <!-- Video -->
          <VideoViewer
            v-else-if="previewType === 'video'"
            :src="activeFile.url || ''"
            :name="activeFile.name"
          />

          <!-- Audio -->
          <AudioViewer
            v-else-if="previewType === 'audio'"
            :src="activeFile.url || ''"
            :name="activeFile.name"
            :size-text="formattedSize"
          />

          <!-- PDF -->
          <PdfViewer
            v-else-if="previewType === 'pdf'"
            :src="activeFile.url || ''"
            :name="activeFile.name"
            @download="handleDownload"
          />

          <!-- Markdown -->
          <MarkdownViewer
            v-else-if="previewType === 'markdown'"
            :src="activeFile.url || ''"
            :name="activeFile.name"
            :size="activeFile.size"
          />

          <!-- CSV / TSV Table -->
          <CsvViewer
            v-else-if="previewType === 'csv'"
            :src="activeFile.url || ''"
            :name="activeFile.name"
            :ext-name="activeFile.extName"
            :size="activeFile.size"
          />

          <!-- Code / Text -->
          <CodeTextViewer
            v-else-if="previewType === 'code'"
            :src="activeFile.url || ''"
            :name="activeFile.name"
            :ext-name="activeFile.extName"
            :size="activeFile.size"
          />

          <!-- Unsupported format fallback -->
          <UnsupportedViewer
            v-else
            :name="activeFile.name"
            :size="activeFile.size"
            :ext-name="activeFile.extName"
            :mime-type="activeFile.mimeType"
            :updated-at="activeFile.updatedAt"
            :content-hash="activeFile.contentHash"
            :thumb="activeFile.thumb"
            :download-url="activeFile.url"
            :public-url="activeFile.publicUrl"
            @download="handleDownload"
          />
        </main>
      </div>
    </div>
  </transition>
  </teleport>
</template>

<style scoped lang="scss">
.file-preview-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(13, 15, 20, 0.94);

  .preview-backdrop {
    position: absolute;
    inset: 0;
    background: rgba(13, 15, 20, 0.85);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
  }

  .preview-container {
    position: relative;
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    z-index: 1;

    .preview-header {
      height: 56px;
      background: rgba(15, 23, 42, 0.85);
      border-bottom: 1px solid rgba(255, 255, 255, 0.08);
      backdrop-filter: blur(20px);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 20px;
      user-select: none;
      z-index: 10;
      gap: 10px;

      @media (max-width: 768px) {
        height: 50px;
        padding: 0 10px;
        gap: 6px;
      }

      .file-meta {
        display: flex;
        align-items: center;
        gap: 10px;
        min-width: 0;
        max-width: 45%;

        @media (max-width: 768px) {
          max-width: none;
          flex: 1 1 auto;
          gap: 6px;

          .size-tag {
            display: none;
          }
        }

        @media (max-width: 480px) {
          .type-badge {
            display: none;
          }
        }

        .header-thumb {
          width: 24px;
          height: 24px;
          object-fit: contain;
          flex-shrink: 0;
        }

        .header-fallback-icon {
          color: #94a3b8;
          flex-shrink: 0;
        }

        .title-wrapper {
          min-width: 0;
          flex: 1;

          .file-title {
            font-size: 14px;
            font-weight: 600;
            color: #f8fafc;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            display: block;

            @media (max-width: 768px) {
              font-size: 13px;
            }
          }
        }

        .type-badge {
          font-size: 10px;
          font-weight: 700;
          padding: 2px 6px;
          border-radius: 4px;
          border: 1px solid transparent;
          flex-shrink: 0;
          letter-spacing: 0.5px;
          white-space: nowrap;
        }

        .size-tag {
          font-size: 12px;
          color: #94a3b8;
          font-variant-numeric: tabular-nums;
          flex-shrink: 0;
          white-space: nowrap;
        }
      }

      .file-navigation {
        display: flex;
        align-items: center;
        gap: 8px;
        background: rgba(255, 255, 255, 0.06);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 9999px;
        padding: 4px 10px;
        flex-shrink: 0;
        white-space: nowrap;

        @media (max-width: 768px) {
          padding: 2px 6px;
          gap: 4px;

          .nav-btn {
            width: 22px;
            height: 22px;
          }

          .nav-indicator {
            font-size: 11px;
            padding: 0 2px;
          }
        }

        .nav-btn {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 26px;
          height: 26px;
          border-radius: 50%;
          background: transparent;
          border: none;
          color: #f1f5f9;
          cursor: pointer;
          transition: all 0.2s;
          flex-shrink: 0;

          &:hover:not(:disabled) {
            background: rgba(255, 255, 255, 0.15);
            color: var(--c-primary-light, #33a5fd);
          }

          &:disabled {
            opacity: 0.3;
            cursor: not-allowed;
          }
        }

        .nav-indicator {
          font-size: 12px;
          color: #cbd5e1;
          font-variant-numeric: tabular-nums;
          padding: 0 4px;
          white-space: nowrap;
        }
      }

      .header-actions {
        display: flex;
        align-items: center;
        gap: 8px;
        flex-shrink: 0;
        white-space: nowrap;

        @media (max-width: 768px) {
          gap: 4px;

          .action-btn {
            height: 28px;
            padding: 0 6px;
            font-size: 12px;

            &.download-btn {
              padding: 0 8px;

              span {
                display: none;
              }
            }
          }

          .divider {
            display: none;
          }
        }

        .action-btn {
          display: flex;
          align-items: center;
          gap: 6px;
          height: 32px;
          padding: 0 10px;
          border-radius: 6px;
          background: rgba(255, 255, 255, 0.06);
          border: 1px solid rgba(255, 255, 255, 0.1);
          color: #cbd5e1;
          font-size: 13px;
          cursor: pointer;
          transition: all 0.2s ease;
          white-space: nowrap;
          flex-shrink: 0;

          &:hover {
            background: rgba(255, 255, 255, 0.12);
            color: #f8fafc;
          }

          &.download-btn {
            background: rgba(0, 143, 253, 0.15);
            border-color: rgba(0, 143, 253, 0.35);
            color: var(--c-primary-light, #33a5fd);

            &:hover {
              background: var(--c-primary, #008ffd);
              border-color: var(--c-primary, #008ffd);
              color: #fff;
            }
          }

          &.close-btn {
            padding: 0 8px;
            background: transparent;
            border-color: transparent;

            &:hover {
              background: rgba(239, 68, 68, 0.2);
              color: #f87171;
            }
          }
        }

        .divider {
          width: 1px;
          height: 16px;
          background: rgba(255, 255, 255, 0.15);
          margin: 0 4px;
        }
      }
    }

    .preview-main {
      flex: 1;
      width: 100%;
      height: calc(100vh - 56px);
      overflow: hidden;
      position: relative;
    }
  }
}

@media (prefers-color-scheme: light) {
  .file-preview-modal {
    background: rgba(241, 245, 249, 0.92);

    .preview-backdrop {
      background: rgba(248, 250, 252, 0.85);
    }

    .preview-container {
      .preview-header {
        background: rgba(255, 255, 255, 0.88);
        border-bottom: 1px solid rgba(0, 0, 0, 0.08);

        .file-meta {
          .title-wrapper .file-title {
            color: #0f172a;
          }

          .header-fallback-icon,
          .size-tag {
            color: #64748b;
          }
        }

        .file-navigation {
          background: rgba(0, 0, 0, 0.04);
          border-color: rgba(0, 0, 0, 0.08);

          .nav-btn {
            color: #334155;

            &:hover:not(:disabled) {
              background: rgba(0, 0, 0, 0.06);
              color: var(--c-primary, #008ffd);
            }
          }

          .nav-indicator {
            color: #475569;
          }
        }

        .header-actions {
          .action-btn {
            background: rgba(0, 0, 0, 0.04);
            border-color: rgba(0, 0, 0, 0.08);
            color: #475569;

            &:hover {
              background: rgba(0, 0, 0, 0.08);
              color: #0f172a;
            }

            &.download-btn {
              background: rgba(0, 143, 253, 0.1);
              border-color: rgba(0, 143, 253, 0.25);
              color: var(--c-primary, #008ffd);

              &:hover {
                background: var(--c-primary, #008ffd);
                border-color: var(--c-primary, #008ffd);
                color: #ffffff;
              }
            }

            &.close-btn:hover {
              background: rgba(239, 68, 68, 0.1);
              color: #ef4444;
            }
          }

          .divider {
            background: rgba(0, 0, 0, 0.1);
          }
        }
      }
    }
  }
}

// Fade animation for modal
.preview-fade-enter-active,
.preview-fade-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}

.preview-fade-enter-from,
.preview-fade-leave-to {
  opacity: 0;
  transform: scale(0.98);
}
</style>
