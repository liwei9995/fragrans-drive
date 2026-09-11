<script setup lang="ts" name="public-link-dialog">
import {
  Check,
  Clock,
  CopyDocument,
  DataLine,
  Link,
  Refresh,
  TopRight,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { computed, ref, watch } from 'vue'
import { setPublicStatus } from '@/api/modules/storage'
import type { StorageViewItem } from '@/hooks/useFetchFiles'

interface PublicLinkDialogProps {
  file: StorageViewItem | null
}

const props = defineProps<PublicLinkDialogProps>()
const emit = defineEmits<{
  (e: 'close'): void
  (
    e: 'updated',
    data: {
      id: string
      isPublic: boolean
      publicSlug?: string
      publicUrl?: string
      publicExpiresAt?: string
      publicAccessCount?: number
      lastPublicAccessedAt?: string
    },
  ): void
}>()

const dialogVisible = ref(true)
const loading = ref(false)
const isPublic = ref(props.file?.isPublic ?? false)
const publicSlug = ref(props.file?.publicSlug ?? '')
const rawPublicUrl = ref(props.file?.publicUrl ?? '')
const publicExpiresAt = ref(props.file?.publicExpiresAt)
const publicAccessCount = ref(props.file?.publicAccessCount ?? 0)
const lastPublicAccessedAt = ref(props.file?.lastPublicAccessedAt)
const selectedExpiresIn = ref<number>(0)
const copiedField = ref<string | null>(null)

watch(
  () => props.file,
  (newFile) => {
    if (newFile) {
      isPublic.value = newFile.isPublic ?? false
      publicSlug.value = newFile.publicSlug ?? ''
      rawPublicUrl.value = newFile.publicUrl ?? ''
      publicExpiresAt.value = newFile.publicExpiresAt
      publicAccessCount.value = newFile.publicAccessCount ?? 0
      lastPublicAccessedAt.value = newFile.lastPublicAccessedAt
    }
  },
  { immediate: true },
)

const isExpired = computed(() => {
  if (!publicExpiresAt.value) return false
  return new Date(publicExpiresAt.value).getTime() < Date.now()
})

const formatDateTime = (dateStr?: string) => {
  if (!dateStr) return '-'
  const d = new Date(dateStr)
  if (Number.isNaN(d.getTime())) return '-'
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

const publicUrl = computed(() => {
  if (!isPublic.value || !publicSlug.value) return ''
  if (rawPublicUrl.value) {
    try {
      const parsed = new URL(rawPublicUrl.value)
      const currentHost = window.location.hostname
      const isCurrentLocal =
        currentHost === 'localhost' || currentHost === '127.0.0.1'
      const isParsedLocal =
        parsed.hostname === 'localhost' || parsed.hostname === '127.0.0.1'
      if (isParsedLocal && !isCurrentLocal) {
        return `${window.location.origin}${parsed.pathname}`
      }
      return rawPublicUrl.value
    } catch {
      return rawPublicUrl.value
    }
  }
  const origin = window.location.origin
  return `${origin}/v1/p/${publicSlug.value}`
})

const namedPublicUrl = computed(() => {
  if (!publicUrl.value || !props.file?.name) return publicUrl.value
  return `${publicUrl.value}/${encodeURIComponent(props.file.name)}`
})

const downloadUrl = computed(() => {
  if (!publicUrl.value) return ''
  return `${publicUrl.value}?download=1`
})

const isImage = computed(() => props.file?.mimeType?.startsWith('image/'))

const handleTogglePublic = async (val: boolean | string | number) => {
  if (!props.file) return
  const nextVal = Boolean(val)
  loading.value = true
  try {
    const res = await setPublicStatus(props.file.id, {
      isPublic: nextVal,
      expiresIn:
        nextVal && selectedExpiresIn.value > 0
          ? selectedExpiresIn.value
          : undefined,
    })
    isPublic.value = res.isPublic
    publicSlug.value = res.publicSlug || ''
    rawPublicUrl.value = res.publicUrl || ''
    publicExpiresAt.value = res.publicExpiresAt
    publicAccessCount.value = res.publicAccessCount ?? 0
    lastPublicAccessedAt.value = res.lastPublicAccessedAt
    emit('updated', {
      id: props.file.id,
      isPublic: res.isPublic,
      publicSlug: res.publicSlug,
      publicUrl: res.publicUrl,
      publicExpiresAt: res.publicExpiresAt,
      publicAccessCount: res.publicAccessCount,
      lastPublicAccessedAt: res.lastPublicAccessedAt,
    })
    ElMessage.success(nextVal ? '已开启公开直链访问' : '已关闭公开直链访问')
  } catch (err) {
    isPublic.value = !nextVal
    ElMessage.error('更新公开状态失败，请重试')
    console.error('Failed to set public status:', err)
  } finally {
    loading.value = false
  }
}

const handleExpirationChange = async (val: any) => {
  const currentFile = props.file
  if (!currentFile || !isPublic.value) return
  loading.value = true
  try {
    const res = await setPublicStatus(currentFile.id, {
      isPublic: true,
      expiresIn: Number(val),
    })
    isPublic.value = res.isPublic
    publicSlug.value = res.publicSlug || ''
    rawPublicUrl.value = res.publicUrl || ''
    publicExpiresAt.value = res.publicExpiresAt
    publicAccessCount.value = res.publicAccessCount ?? 0
    lastPublicAccessedAt.value = res.lastPublicAccessedAt
    emit('updated', {
      id: currentFile.id,
      isPublic: res.isPublic,
      publicSlug: res.publicSlug,
      publicUrl: res.publicUrl,
      publicExpiresAt: res.publicExpiresAt,
      publicAccessCount: res.publicAccessCount,
      lastPublicAccessedAt: res.lastPublicAccessedAt,
    })
    ElMessage.success(Number(val) > 0 ? '直链有效期已更新' : '已设置为永久有效')
  } catch (err) {
    ElMessage.error('更新有效期失败，请重试')
    console.error('Failed to update expiration:', err)
  } finally {
    loading.value = false
  }
}

const handleRefreshSlug = () => {
  const currentFile = props.file
  if (!currentFile) return
  ElMessageBox.confirm(
    '重置后原公开直链将永久失效，并立即生成全新直链，是否继续？',
    '重置直链警告',
    {
      confirmButtonText: '确定重置',
      cancelButtonText: '取消',
      type: 'warning',
    },
  )
    .then(async () => {
      loading.value = true
      try {
        const res = await setPublicStatus(currentFile.id, {
          isPublic: true,
          refresh: true,
          expiresIn:
            selectedExpiresIn.value > 0 ? selectedExpiresIn.value : undefined,
        })
        isPublic.value = res.isPublic
        publicSlug.value = res.publicSlug || ''
        rawPublicUrl.value = res.publicUrl || ''
        publicExpiresAt.value = res.publicExpiresAt
        publicAccessCount.value = res.publicAccessCount ?? 0
        lastPublicAccessedAt.value = res.lastPublicAccessedAt
        emit('updated', {
          id: currentFile.id,
          isPublic: res.isPublic,
          publicSlug: res.publicSlug,
          publicUrl: res.publicUrl,
          publicExpiresAt: res.publicExpiresAt,
          publicAccessCount: res.publicAccessCount,
          lastPublicAccessedAt: res.lastPublicAccessedAt,
        })
        ElMessage.success('已生成全新直链，原直链已失效')
      } catch (err) {
        ElMessage.error('重置直链失败，请重试')
        console.error('Failed to refresh slug:', err)
      } finally {
        loading.value = false
      }
    })
    .catch(() => {})
}

const copyToClipboard = async (
  text: string,
  fieldKey: string,
  label: string,
) => {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text)
    } else {
      const input = document.createElement('textarea')
      input.value = text
      input.style.position = 'absolute'
      input.style.left = '-9999px'
      document.body.appendChild(input)
      input.select()
      document.execCommand('copy')
      document.body.removeChild(input)
    }
    copiedField.value = fieldKey
    setTimeout(() => {
      if (copiedField.value === fieldKey) {
        copiedField.value = null
      }
    }, 1800)
    ElMessage.success(`${label}已复制到剪贴板`)
  } catch {
    ElMessage.error('复制失败，请手动选择复制')
  }
}

const handleClose = () => emit('close')
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    class="public-link-dialog"
    width="540px"
    title="公开直链管理"
    :close-on-click-modal="false"
    append-to-body
    @close="handleClose"
  >
    <div v-loading="loading" class="dialog-body">
      <div class="file-summary">
        <div class="thumb-box">
          <el-image :src="file?.thumb" class="thumb-img" fit="cover" />
        </div>
        <div class="file-meta">
          <div class="name" :title="file?.name">{{ file?.name }}</div>
          <div class="sub-info">
            <span class="mime">{{ file?.mimeType || '未知类型' }}</span>
            <el-tag
              :type="isPublic ? (isExpired ? 'danger' : 'success') : 'info'"
              size="small"
              effect="plain"
              class="status-tag"
            >
              {{ isPublic ? (isExpired ? '● 直链已过期' : '● 公开访问已生效') : '○ 仅私有访问' }}
            </el-tag>
          </div>
        </div>
      </div>

      <div class="toggle-card">
        <div class="toggle-info">
          <div class="toggle-title">
            <el-icon class="icon-link"><Link /></el-icon>
            <span>允许公开直链访问</span>
          </div>
          <div class="toggle-desc">
            无需账号登录凭据即可访问（类似于 OSS / S3 公开读），可直接嵌入网页、Markdown、图片展示或外部下载。
          </div>
        </div>
        <el-switch
          v-model="isPublic"
          :loading="loading"
          active-color="#13ce66"
          @change="handleTogglePublic"
        />
      </div>

      <transition name="el-fade-in-linear">
        <div v-if="isPublic" class="links-section">
          <!-- 有效期设置 -->
          <div class="config-card">
            <div class="config-header">
              <div class="config-title">
                <el-icon class="section-icon"><Clock /></el-icon>
                <span>直链有效期</span>
              </div>
              <el-tag
                v-if="publicExpiresAt"
                :type="isExpired ? 'danger' : 'warning'"
                size="small"
                effect="light"
              >
                {{ isExpired ? '已过期' : `截止: ${formatDateTime(publicExpiresAt)}` }}
              </el-tag>
              <el-tag v-else size="small" type="info" effect="light">永久有效</el-tag>
            </div>
            <div class="expiration-radios">
              <el-radio-group
                v-model="selectedExpiresIn"
                size="small"
                :disabled="loading"
                @change="handleExpirationChange"
              >
                <el-radio-button :value="0">永久有效</el-radio-button>
                <el-radio-button :value="3600">1 小时</el-radio-button>
                <el-radio-button :value="86400">1 天</el-radio-button>
                <el-radio-button :value="604800">7 天</el-radio-button>
                <el-radio-button :value="2592000">30 天</el-radio-button>
              </el-radio-group>
            </div>
          </div>

          <!-- 访问热度统计 -->
          <div class="analytics-card">
            <div class="analytics-item">
              <div class="item-title">
                <el-icon class="analytics-icon"><DataLine /></el-icon>
                <span>累计访问</span>
              </div>
              <div class="item-value highlight">
                {{ publicAccessCount }} <span class="unit">次</span>
              </div>
            </div>
            <div class="analytics-divider" />
            <div class="analytics-item">
              <div class="item-title">
                <el-icon class="analytics-icon"><Clock /></el-icon>
                <span>最近访问</span>
              </div>
              <div class="item-value">
                {{ lastPublicAccessedAt ? formatDateTime(lastPublicAccessedAt) : '尚未被访问' }}
              </div>
            </div>
          </div>

          <!-- 链接列表 -->
          <div v-if="publicUrl" class="link-item">
            <div class="link-header">
              <span class="label">内联引用直链（推荐用于网页/图片嵌入）</span>
              <a :href="namedPublicUrl" target="_blank" rel="noreferrer" class="open-link">
                新标签打开 <el-icon><TopRight /></el-icon>
              </a>
            </div>
            <el-input :model-value="namedPublicUrl" readonly>
              <template #append>
                <el-button
                  :type="copiedField === 'inline' ? 'success' : 'default'"
                  :icon="copiedField === 'inline' ? Check : CopyDocument"
                  @click="copyToClipboard(namedPublicUrl, 'inline', '内联直链')"
                >
                  {{ copiedField === 'inline' ? '已复制' : '复制' }}
                </el-button>
              </template>
            </el-input>
          </div>

          <div v-if="isImage && publicUrl" class="link-item">
            <div class="link-header">
              <span class="label">Markdown 引用代码</span>
            </div>
            <el-input :model-value="`![${file?.name}](${namedPublicUrl})`" readonly>
              <template #append>
                <el-button
                  :type="copiedField === 'markdown' ? 'success' : 'default'"
                  :icon="copiedField === 'markdown' ? Check : CopyDocument"
                  @click="copyToClipboard(`![${file?.name}](${namedPublicUrl})`, 'markdown', 'Markdown 代码')"
                >
                  {{ copiedField === 'markdown' ? '已复制' : '复制' }}
                </el-button>
              </template>
            </el-input>
          </div>

          <div v-if="publicUrl" class="link-item">
            <div class="link-header">
              <span class="label">强制下载直链（触发浏览器直接保存）</span>
            </div>
            <el-input :model-value="downloadUrl" readonly>
              <template #append>
                <el-button
                  :type="copiedField === 'download' ? 'success' : 'default'"
                  :icon="copiedField === 'download' ? Check : CopyDocument"
                  @click="copyToClipboard(downloadUrl, 'download', '下载直链')"
                >
                  {{ copiedField === 'download' ? '已复制' : '复制' }}
                </el-button>
              </template>
            </el-input>
          </div>

          <div class="security-tips">
            <div class="tip-text">
              🛡️ 安全保障：对外使用高熵Slug隔离对象ID；支持自定义有效期与零IO快速HEAD探测；流式防重放防遍历。
            </div>
            <el-button
              type="danger"
              link
              size="small"
              :icon="Refresh"
              @click="handleRefreshSlug"
            >
              重置直链（使旧链接失效）
            </el-button>
          </div>
        </div>
      </transition>
    </div>
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">关闭</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.public-link-dialog {
  :deep(.el-dialog__body) {
    padding: 16px 24px;
  }

  .dialog-body {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .file-summary {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px;
    background: #f8fafc;
    border-radius: 8px;
    border: 1px solid #edf2f7;

    .thumb-box {
      width: 48px;
      height: 48px;
      flex-shrink: 0;
      border-radius: 6px;
      overflow: hidden;
      background: #fff;
      display: flex;
      align-items: center;
      justify-content: center;
      border: 1px solid #e2e8f0;

      .thumb-img {
        width: 100%;
        height: 100%;
      }
    }

    .file-meta {
      flex: 1;
      min-width: 0;

      .name {
        font-size: 14px;
        font-weight: 600;
        color: #1e293b;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .sub-info {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-top: 4px;

        .mime {
          font-size: 12px;
          color: #64748b;
        }

        .status-tag {
          font-size: 11px;
        }
      }
    }
  }

  .toggle-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px;
    border-radius: 8px;
    background: #f1f5f9;

    .toggle-info {
      flex: 1;

      .toggle-title {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 14px;
        font-weight: 600;
        color: #0f172a;

        .icon-link {
          font-size: 16px;
          color: var(--c-primary, #008ffd);
        }
      }

      .toggle-desc {
        font-size: 12px;
        color: #64748b;
        margin-top: 4px;
        line-height: 1.5;
      }
    }
  }

  .links-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 4px;

    .config-card {
      padding: 12px;
      background: #f8fafc;
      border-radius: 8px;
      border: 1px solid #edf2f7;

      .config-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 8px;

        .config-title {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 13px;
          font-weight: 600;
          color: #334155;

          .section-icon {
            font-size: 15px;
            color: var(--c-primary, #008ffd);
          }
        }
      }

      .expiration-radios {
        :deep(.el-radio-group) {
          display: flex;
          width: 100%;

          .el-radio-button {
            flex: 1;

            .el-radio-button__inner {
              width: 100%;
              padding: 6px 0;
              font-size: 12px;
            }
          }
        }
      }
    }

    .analytics-card {
      display: flex;
      align-items: center;
      justify-content: space-around;
      padding: 10px 14px;
      background: #fdfefe;
      border-radius: 8px;
      border: 1px solid #e2e8f0;

      .analytics-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 2px;

        .item-title {
          display: flex;
          align-items: center;
          gap: 4px;
          font-size: 11px;
          color: #64748b;

          .analytics-icon {
            font-size: 13px;
            color: #94a3b8;
          }
        }

        .item-value {
          font-size: 13px;
          font-weight: 600;
          color: #1e293b;

          &.highlight {
            font-size: 15px;
            color: var(--c-primary, #008ffd);
          }

          .unit {
            font-size: 11px;
            font-weight: normal;
            color: #64748b;
          }
        }
      }

      .analytics-divider {
        width: 1px;
        height: 28px;
        background: #e2e8f0;
      }
    }

    .link-item {
      .link-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 6px;

        .label {
          font-size: 12px;
          font-weight: 500;
          color: #475569;
        }

        .open-link {
          display: inline-flex;
          align-items: center;
          gap: 2px;
          font-size: 12px;
          color: var(--c-primary, #008ffd);
          text-decoration: none;

          &:hover {
            text-decoration: underline;
            color: var(--c-primary-hover, #007ceb);
          }
        }
      }
    }

    .security-tips {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 10px 12px;
      background: rgba(0, 143, 253, 0.06);
      border-radius: 6px;
      border: 1px dashed rgba(0, 143, 253, 0.25);
      margin-top: 6px;

      .tip-text {
        font-size: 11px;
        color: var(--c-primary-hover, #007ceb);
        line-height: 1.4;
        flex: 1;
      }
    }
  }
}
</style>
