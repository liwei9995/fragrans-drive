<script setup lang="ts">
import {
  Check,
  Document,
  Download,
  Link,
  TopRight,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { computed, ref } from 'vue'
import { formatDate, formatFileSize } from '../previewHelper'

interface Props {
  name: string
  size?: number
  extName?: string
  mimeType?: string
  updatedAt?: string
  contentHash?: string
  thumb?: string
  downloadUrl?: string
  publicUrl?: string
}

const props = withDefaults(defineProps<Props>(), {
  name: '',
  size: 0,
  extName: '',
  mimeType: '',
  updatedAt: '',
  contentHash: '',
  thumb: '',
  downloadUrl: '',
  publicUrl: '',
})

const emit = defineEmits<(e: 'download') => void>()

const copied = ref(false)

const OFFICE_EXTS = new Set(['docx', 'doc', 'xlsx', 'xls', 'pptx', 'ppt'])
const isOfficeDoc = computed(() => {
  const ext = (props.extName || '').toLowerCase().replace(/^\./, '')
  return OFFICE_EXTS.has(ext)
})

const officeOnlineUrl = computed(() => {
  if (!props.publicUrl || !props.publicUrl.startsWith('http')) return ''
  return `https://view.officeapps.live.com/op/view.aspx?src=${encodeURIComponent(props.publicUrl)}`
})

const handleOpenOfficeOnline = () => {
  if (officeOnlineUrl.value) {
    window.open(officeOnlineUrl.value, '_blank')
  }
}

const formattedSize = computed(() => formatFileSize(props.size))
const formattedDate = computed(() => formatDate(props.updatedAt))

const handleCopyLink = async () => {
  const targetUrl = props.publicUrl || props.downloadUrl
  if (!targetUrl) return
  try {
    await navigator.clipboard.writeText(targetUrl)
    copied.value = true
    ElMessage.success('已复制下载链接')
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch {
    ElMessage.error('复制链接失败')
  }
}
</script>

<template>
  <div class="unsupported-viewer">
    <div class="info-card">
      <div class="icon-section">
        <img v-if="thumb" :src="thumb" class="thumb-img" alt="file-thumb" />
        <el-icon v-else class="fallback-icon" :size="72"><Document /></el-icon>
        <span class="ext-badge">{{ (extName || 'FILE').toUpperCase() }}</span>
      </div>

      <h3 class="file-name" :title="name">{{ name }}</h3>

      <p class="notice-text">
        该文件类型暂不支持在浏览器内直接预览，建议下载至本地设备后使用相应的客户端软件打开。
      </p>

      <!-- Metadata Table -->
      <div class="meta-grid">
        <div class="meta-row">
          <span class="meta-label">文件大小</span>
          <span class="meta-value">{{ formattedSize }}</span>
        </div>
        <div v-if="mimeType" class="meta-row">
          <span class="meta-label">MIME 类型</span>
          <span class="meta-value">{{ mimeType }}</span>
        </div>
        <div v-if="updatedAt" class="meta-row">
          <span class="meta-label">修改时间</span>
          <span class="meta-value">{{ formattedDate }}</span>
        </div>
        <div v-if="contentHash" class="meta-row">
          <span class="meta-label">SHA-256</span>
          <span class="meta-value hash-code" :title="contentHash">
            {{ contentHash.slice(0, 16) }}...
          </span>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="action-buttons">
        <el-button
          v-if="isOfficeDoc && officeOnlineUrl"
          type="success"
          size="large"
          class="office-btn"
          @click="handleOpenOfficeOnline"
        >
          <el-icon><TopRight /></el-icon>
          <span>使用微软 Office 在线预览</span>
        </el-button>
        <div v-else-if="isOfficeDoc" class="office-tip">
          💡 该文件为 Office 文档，在文件菜单中开启【公开直链】后可一键使用微软 Office 在线预览。
        </div>

        <el-button type="primary" size="large" :icon="Download" class="dl-btn" @click="emit('download')">
          立即下载文件
        </el-button>
        <el-button
          v-if="downloadUrl || publicUrl"
          size="large"
          class="link-btn"
          @click="handleCopyLink"
        >
          <el-icon>
            <Check v-if="copied" />
            <Link v-else />
          </el-icon>
          <span>{{ copied ? '已复制链接' : '复制下载链接' }}</span>
        </el-button>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.unsupported-viewer {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: radial-gradient(circle at 50% 50%, #1e293b 0%, #0f172a 100%);
  user-select: none;

  .info-card {
    width: 100%;
    max-width: 480px;
    background: rgba(30, 41, 59, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(20px);
    border-radius: 20px;
    padding: 36px 32px 30px;
    display: flex;
    flex-direction: column;
    align-items: center;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);

    .icon-section {
      position: relative;
      margin-bottom: 20px;

      .thumb-img {
        width: 80px;
        height: 80px;
        object-fit: contain;
      }

      .fallback-icon {
        color: #94a3b8;
      }

      .ext-badge {
        position: absolute;
        bottom: -6px;
        right: -8px;
        background: #6366f1;
        color: #fff;
        font-size: 10px;
        font-weight: 700;
        padding: 2px 6px;
        border-radius: 4px;
        letter-spacing: 0.5px;
      }
    }

    .file-name {
      font-size: 18px;
      font-weight: 600;
      color: #f8fafc;
      margin: 0 0 10px;
      text-align: center;
      max-width: 100%;
      word-break: break-all;
    }

    .notice-text {
      font-size: 13px;
      color: #94a3b8;
      text-align: center;
      line-height: 1.6;
      margin: 0 0 24px;
    }

    .meta-grid {
      width: 100%;
      background: rgba(15, 23, 42, 0.5);
      border-radius: 10px;
      padding: 12px 16px;
      display: flex;
      flex-direction: column;
      gap: 10px;
      margin-bottom: 24px;

      .meta-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 13px;

        .meta-label {
          color: #64748b;
        }

        .meta-value {
          color: #cbd5e1;
          font-variant-numeric: tabular-nums;

          &.hash-code {
            font-family: monospace;
            font-size: 12px;
            color: #94a3b8;
          }
        }
      }
    }

    .action-buttons {
      width: 100%;
      display: flex;
      flex-direction: column;
      gap: 10px;

      .office-btn {
        width: 100%;
        background: linear-gradient(135deg, #10b981 0%, #059669 100%);
        border: none;
        color: #fff;
      }

      .office-tip {
        font-size: 12px;
        color: #94a3b8;
        line-height: 1.5;
        text-align: center;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        padding: 8px 12px;
      }

      .dl-btn {
        width: 100%;
        background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
        border: none;
      }

      .link-btn {
        width: 100%;
        margin-left: 0 !important;
        background: rgba(255, 255, 255, 0.05);
        border-color: rgba(255, 255, 255, 0.15);
        color: #e2e8f0;

        &:hover {
          background: rgba(255, 255, 255, 0.1);
          color: var(--c-primary-light, #33a5fd);
          border-color: var(--c-primary-light, #33a5fd);
        }
      }
    }
  }
}

@media (prefers-color-scheme: light) {
  .unsupported-viewer {
    background: radial-gradient(circle at 50% 50%, rgba(248, 250, 252, 0.8) 0%, rgba(226, 232, 240, 0.9) 100%);

    .info-card {
      background: rgba(255, 255, 255, 0.88);
      border: 1px solid rgba(0, 0, 0, 0.08);
      box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.12);

      .file-name {
        color: #0f172a;
      }

      .notice-text {
        color: #64748b;
      }

      .meta-grid {
        background: #f8fafc;
        border: 1px solid rgba(0, 0, 0, 0.06);

        .meta-row {
          .meta-label {
            color: #64748b;
          }

          .meta-value {
            color: #0f172a;

            &.hash-code {
              color: #64748b;
            }
          }
        }
      }

      .action-buttons {
        .office-tip {
          background: #f8fafc;
          border-color: rgba(0, 0, 0, 0.08);
          color: #64748b;
        }

        .link-btn {
          background: rgba(0, 0, 0, 0.04);
          border-color: rgba(0, 0, 0, 0.1);
          color: #334155;

          &:hover {
            background: rgba(0, 0, 0, 0.08);
            color: var(--c-primary, #008ffd);
            border-color: var(--c-primary, #008ffd);
          }
        }
      }
    }
  }
}
</style>
