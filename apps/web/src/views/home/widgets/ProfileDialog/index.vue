<script setup lang="ts" name="profile-dialog">
import {
  Camera,
  Check,
  CircleCheck,
  Delete,
  Folder,
  Key,
  PieChart,
  Upload,
  User,
  UserFilled,
} from '@element-plus/icons-vue'
import {
  browserSupportsWebAuthn,
  startRegistration,
} from '@simplewebauthn/browser'
import { ElMessage } from 'element-plus'
import { computed, onMounted, reactive, ref, watch } from 'vue'
import type { Storage, User as UserType } from '@/api/interface'
import { getStorageUsage } from '@/api/modules/storage'
import {
  deletePasskey,
  getPasskeys,
  getProfile,
  updatePassword,
  updateProfile,
  webauthnRegisterFinish,
  webauthnRegisterStart,
} from '@/api/modules/user'
import AvatarCropper from '@/components/AvatarCropper/index.vue'
import { GlobalStore } from '@/store'

interface ProfileDialogProps {
  visible: boolean
}

const props = defineProps<ProfileDialogProps>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'updated', profile: UserType.UserProfile): void
}>()

const globalStore = GlobalStore()
const activeTab = ref('storage')
const loading = ref(false)
const savingProfile = ref(false)
const savingPassword = ref(false)
const cropperVisible = ref(false)

const profile = ref<UserType.UserProfile | null>(null)
const usage = ref<Storage.StorageUsage>({
  usedBytes: 0,
  fileCount: 0,
  quotaBytes: 50 * 1024 * 1024 * 1024,
})

const profileForm = reactive({
  firstName: '',
  lastName: '',
  avatar: '',
  gender: 0,
  age: 0,
})

const passwordForm = reactive({
  oldPassword: '',
  password: '',
  confirmPassword: '',
})

const formatBytes = (bytes?: number) => {
  if (typeof bytes !== 'number' || bytes <= 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / k ** i).toFixed(i === 0 ? 0 : 2)} ${sizes[i]}`
}

const usedPercentage = computed(() => {
  if (!usage.value.quotaBytes) return 0
  const pct = (usage.value.usedBytes / usage.value.quotaBytes) * 100
  return Number(pct.toFixed(1))
})

const progressStatus = computed(() => {
  if (usedPercentage.value >= 90) return 'exception'
  if (usedPercentage.value >= 75) return 'warning'
  return 'success'
})

const userName = computed(() => {
  const first = profile.value?.firstName?.trim() || ''
  const last = profile.value?.lastName?.trim() || ''
  const full = [first, last].filter(Boolean).join(' ')
  return full || profile.value?.email || '未设置昵称'
})

const loadData = async () => {
  loading.value = true
  try {
    const [profileRes, usageRes] = await Promise.all([
      getProfile(),
      getStorageUsage(),
    ])
    profile.value = profileRes
    usage.value = usageRes

    profileForm.firstName = profileRes.firstName || ''
    profileForm.lastName = profileRes.lastName || ''
    profileForm.avatar = profileRes.avatar || ''
    profileForm.gender = profileRes.gender || 0
    profileForm.age = profileRes.age || 0
    globalStore.setUserInfo(profileRes)
  } catch (error) {
    console.error('Failed to load profile/usage:', error)
  } finally {
    loading.value = false
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      passwordForm.oldPassword = ''
      passwordForm.password = ''
      passwordForm.confirmPassword = ''
      loadData()
    }
  },
  { immediate: true },
)

const handleSaveProfile = async () => {
  savingProfile.value = true
  try {
    const updated = await updateProfile({
      firstName: profileForm.firstName.trim() || undefined,
      lastName: profileForm.lastName.trim() || undefined,
      avatar: profileForm.avatar.trim(),
      gender: profileForm.gender,
      age: profileForm.age > 0 ? profileForm.age : undefined,
    })
    profile.value = updated
    profileForm.avatar = updated.avatar || ''
    globalStore.setUserInfo(updated)
    ElMessage.success('个人资料保存成功')
    emit('updated', updated)
  } catch (error) {
    console.error('Update profile error:', error)
    ElMessage.error('保存失败，请稍后重试')
  } finally {
    savingProfile.value = false
  }
}

const handleAvatarCrop = async (dataUrl: string) => {
  profileForm.avatar = dataUrl
  await handleSaveProfile()
}

const handleClearAvatar = async () => {
  profileForm.avatar = ''
  await handleSaveProfile()
}

const handleChangePassword = async () => {
  if (!passwordForm.oldPassword) {
    ElMessage.warning('请输入当前密码')
    return
  }
  if (!passwordForm.password) {
    ElMessage.warning('请输入新密码')
    return
  }
  if (passwordForm.password.length < 6) {
    ElMessage.warning('新密码长度不能少于 6 位')
    return
  }
  if (passwordForm.password !== passwordForm.confirmPassword) {
    ElMessage.warning('两次输入的新密码不一致')
    return
  }

  savingPassword.value = true
  try {
    await updatePassword({
      oldPassword: passwordForm.oldPassword,
      password: passwordForm.password,
      changePassword: passwordForm.confirmPassword,
    })
    ElMessage.success('密码修改成功')
    passwordForm.oldPassword = ''
    passwordForm.password = ''
    passwordForm.confirmPassword = ''
  } catch (error: any) {
    const msg = error?.response?.data?.message || '密码修改失败'
    ElMessage.error(msg)
  } finally {
    savingPassword.value = false
  }
}

// Passkey / Touch ID state & actions
interface PasskeyItem {
  id: string
  name: string
  createdAt?: string
}
const supportsWebAuthn = ref(false)
const passkeys = ref<PasskeyItem[]>([])
const loadingPasskeys = ref(false)
const registeringPasskey = ref(false)

onMounted(() => {
  supportsWebAuthn.value = browserSupportsWebAuthn()
})

const loadPasskeys = async () => {
  loadingPasskeys.value = true
  try {
    const res = await getPasskeys()
    passkeys.value = res || []
  } catch (error) {
    console.error('Failed to load passkeys:', error)
  } finally {
    loadingPasskeys.value = false
  }
}

watch(
  () => activeTab.value,
  (tab) => {
    if (tab === 'security' && props.visible) {
      loadPasskeys()
    }
  },
)

const handleRegisterPasskey = async () => {
  registeringPasskey.value = true
  try {
    const { sessionId, challenge } = await webauthnRegisterStart()
    const credential = await startRegistration({ optionsJSON: challenge })
    const isMac = /Macintosh|Mac OS X/i.test(navigator.userAgent)
    const defaultName = isMac ? 'Mac Touch ID' : '设备通行密钥'
    await webauthnRegisterFinish({
      sessionId,
      credential,
      name: defaultName,
    })
    ElMessage.success(
      'Touch ID 凭据绑定成功！您现在可以使用 Touch ID 快速登录了。',
    )
    await loadPasskeys()
  } catch (error: any) {
    if (error?.name === 'NotAllowedError') {
      return
    }
    console.error('Passkey registration failed:', error)
    const msg =
      error?.response?.data?.message || error?.message || '绑定失败，请稍后重试'
    ElMessage.error(msg)
  } finally {
    registeringPasskey.value = false
  }
}

const handleDeletePasskey = async (id: string) => {
  try {
    await deletePasskey(id)
    ElMessage.success('已移除该凭据')
    await loadPasskeys()
  } catch (error: any) {
    console.error('Delete passkey error:', error)
    ElMessage.error('移除失败，请稍后重试')
  }
}

const formatDate = (dateStr?: string) => {
  if (!dateStr) return '未知时间'
  try {
    const d = new Date(dateStr)
    return d.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return dateStr
  }
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    title="个人中心"
    width="640px"
    destroy-on-close
    class="profile-dialog"
    @close="emit('close')"
  >
    <div v-loading="loading" class="profile-container">
      <!-- User Overview Header -->
      <div class="user-overview">
        <div
          class="user-avatar-wrapper"
          title="点击更换头像"
          @click="cropperVisible = true"
        >
          <el-avatar
            :size="68"
            :src="profileForm.avatar || profile?.avatar"
            class="user-avatar"
          >
            <el-icon :size="34"><UserFilled /></el-icon>
          </el-avatar>
          <div class="avatar-hover-overlay">
            <el-icon :size="18"><Camera /></el-icon>
            <span>更换头像</span>
          </div>
        </div>
        <div class="user-overview-info">
          <div class="user-name">
            {{ userName }}
          </div>
          <div class="user-email">{{ profile?.email }}</div>
        </div>
      </div>

      <el-tabs v-model="activeTab" class="profile-tabs">
        <!-- Storage Tab -->
        <el-tab-pane name="storage">
          <template #label>
            <span class="tab-label">
              <el-icon><PieChart /></el-icon>
              <span>存储配额</span>
            </span>
          </template>

          <div class="storage-section">
            <div class="quota-card">
              <div class="quota-header">
                <span class="quota-title">空间使用率</span>
                <span class="quota-ratio">
                  <strong>{{ formatBytes(usage.usedBytes) }}</strong> / {{ formatBytes(usage.quotaBytes) }}
                </span>
              </div>
              <el-progress
                :percentage="usedPercentage"
                :status="progressStatus"
                :stroke-width="12"
                striped
                striped-flow
              />
              <div class="quota-footer-tip">
                已使用 {{ usedPercentage }}% 的总存储空间
              </div>
            </div>

            <div class="stats-grid">
              <div class="stat-box">
                <span class="stat-label">已使用存储</span>
                <span class="stat-value">{{ formatBytes(usage.usedBytes) }}</span>
              </div>
              <div class="stat-box">
                <span class="stat-label">文件总数</span>
                <span class="stat-value">{{ usage.fileCount }} 个</span>
              </div>
              <div class="stat-box">
                <span class="stat-label">配额上限</span>
                <span class="stat-value">{{ formatBytes(usage.quotaBytes) }}</span>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- Profile Tab -->
        <el-tab-pane name="profile">
          <template #label>
            <span class="tab-label">
              <el-icon><User /></el-icon>
              <span>基本资料</span>
            </span>
          </template>

          <el-form :model="profileForm" label-position="top" class="profile-form">
            <div class="form-row">
              <el-form-item label="姓氏 (Last Name)">
                <el-input v-model="profileForm.lastName" placeholder="请输入姓氏" />
              </el-form-item>
              <el-form-item label="名字 (First Name)">
                <el-input v-model="profileForm.firstName" placeholder="请输入名字" />
              </el-form-item>
            </div>

            <el-form-item label="个人头像">
              <div class="avatar-setting-row">
                <div
                  class="avatar-thumbnail-wrapper"
                  title="点击更换头像"
                  @click="cropperVisible = true"
                >
                  <el-avatar
                    :size="56"
                    :src="profileForm.avatar || profile?.avatar"
                    class="thumbnail-avatar"
                  >
                    <el-icon :size="28"><UserFilled /></el-icon>
                  </el-avatar>
                  <div class="thumbnail-hover-overlay">
                    <el-icon :size="16"><Camera /></el-icon>
                  </div>
                </div>

                <div class="avatar-controls-col">
                  <div class="avatar-actions">
                    <el-button
                      type="primary"
                      plain
                      :icon="Upload"
                      @click="cropperVisible = true"
                    >
                      上传并裁剪头像
                    </el-button>
                    <el-button
                      v-if="profileForm.avatar"
                      text
                      type="danger"
                      :icon="Delete"
                      @click="handleClearAvatar"
                    >
                      清除头像
                    </el-button>
                  </div>
                  <div class="avatar-url-row">
                    <el-input
                      v-model="profileForm.avatar"
                      placeholder="或输入外部图片 URL (https://...)"
                      clearable
                      size="small"
                    />
                  </div>
                </div>
              </div>
            </el-form-item>

            <div class="form-row">
              <el-form-item label="性别">
                <el-select v-model="profileForm.gender" style="width: 100%">
                  <el-option :value="0" label="保密" />
                  <el-option :value="1" label="男" />
                  <el-option :value="2" label="女" />
                </el-select>
              </el-form-item>
              <el-form-item label="年龄">
                <el-input-number
                  v-model="profileForm.age"
                  :min="0"
                  :max="120"
                  style="width: 100%"
                />
              </el-form-item>
            </div>

            <div class="form-actions">
              <el-button
                type="primary"
                :icon="Check"
                :loading="savingProfile"
                @click="handleSaveProfile"
              >
                保存个人资料
              </el-button>
            </div>
          </el-form>
        </el-tab-pane>

        <!-- Security Tab -->
        <el-tab-pane name="security">
          <template #label>
            <span class="tab-label">
              <el-icon><Key /></el-icon>
              <span>安全设置</span>
            </span>
          </template>

          <div class="security-pane">
            <div class="security-sub-header">
              <span class="sub-title">修改登录密码</span>
            </div>

            <el-form :model="passwordForm" label-position="top" class="security-form">
              <el-form-item label="当前密码">
                <el-input
                  v-model="passwordForm.oldPassword"
                  type="password"
                  show-password
                  placeholder="请输入当前正在使用的密码"
                />
              </el-form-item>

              <div class="form-row">
                <el-form-item label="新密码">
                  <el-input
                    v-model="passwordForm.password"
                    type="password"
                    show-password
                    placeholder="请输入不少于 6 位的新密码"
                  />
                </el-form-item>

                <el-form-item label="确认新密码">
                  <el-input
                    v-model="passwordForm.confirmPassword"
                    type="password"
                    show-password
                    placeholder="请再次输入新密码"
                  />
                </el-form-item>
              </div>

              <div class="form-actions">
                <el-button
                  type="primary"
                  :icon="CircleCheck"
                  :loading="savingPassword"
                  @click="handleChangePassword"
                >
                  确认修改密码
                </el-button>
              </div>
            </el-form>

            <el-divider class="security-divider" />

            <!-- Touch ID / Passkey Section -->
            <div class="passkey-section">
              <div class="passkey-header">
                <div class="passkey-header-text">
                  <span class="sub-title">Touch ID / 通行密钥</span>
                  <span class="sub-desc">
                    绑定此设备的 Touch ID 或生物识别，下次登录时可一键指纹免密进入。
                  </span>
                </div>
                <el-button
                  v-if="supportsWebAuthn"
                  type="primary"
                  plain
                  :loading="registeringPasskey"
                  class="bind-passkey-btn"
                  @click="handleRegisterPasskey"
                >
                  <svg
                    class="touch-id-btn-icon"
                    viewBox="0 0 24 24"
                    width="16"
                    height="16"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="M12 2a10 10 0 0 0-10 10c0 3.5 1.8 6.6 4.6 8.4" />
                    <path d="M12 6a6 6 0 0 0-6 6c0 1.8.8 3.4 2 4.5" />
                    <path d="M12 10a2 2 0 0 0-2 2c0 .6.3 1.1.7 1.5" />
                    <path d="M12 14v.01" />
                    <path d="M16 12a4 4 0 0 0-1.2-2.8" />
                    <path d="M19.4 12a7.4 7.4 0 0 0-2.2-5.2" />
                    <path d="M22 12c0-2.8-1.1-5.3-3-7.1" />
                  </svg>
                  <span>绑定此设备</span>
                </el-button>
              </div>

              <div v-if="!supportsWebAuthn" class="passkey-unsupported">
                当前浏览器或环境暂不支持 WebAuthn / Touch ID。请在支持的浏览器（如 Safari, Chrome）并启用 HTTPS 或 localhost 环境下使用。
              </div>

              <div v-loading="loadingPasskeys" class="passkey-list">
                <div v-if="passkeys.length === 0" class="passkey-empty">
                  <span>暂无已绑定的 Touch ID / 设备凭据</span>
                </div>
                <div
                  v-for="item in passkeys"
                  :key="item.id"
                  class="passkey-item"
                >
                  <div class="passkey-item-left">
                    <div class="passkey-icon-badge">
                      <svg
                        viewBox="0 0 24 24"
                        width="20"
                        height="20"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      >
                        <path d="M12 2a10 10 0 0 0-10 10c0 3.5 1.8 6.6 4.6 8.4" />
                        <path d="M12 6a6 6 0 0 0-6 6c0 1.8.8 3.4 2 4.5" />
                        <path d="M12 10a2 2 0 0 0-2 2c0 .6.3 1.1.7 1.5" />
                        <path d="M12 14v.01" />
                        <path d="M16 12a4 4 0 0 0-1.2-2.8" />
                        <path d="M19.4 12a7.4 7.4 0 0 0-2.2-5.2" />
                        <path d="M22 12c0-2.8-1.1-5.3-3-7.1" />
                      </svg>
                    </div>
                    <div class="passkey-info">
                      <div class="passkey-name">{{ item.name || 'Touch ID 凭据' }}</div>
                      <div class="passkey-date">绑定时间: {{ formatDate(item.createdAt) }}</div>
                    </div>
                  </div>
                  <el-popconfirm
                    title="确定移除该 Touch ID 凭据吗？移除后将无法使用该设备指纹登录。"
                    confirm-button-text="确定"
                    cancel-button-text="取消"
                    @confirm="handleDeletePasskey(item.id)"
                  >
                    <template #reference>
                      <el-button
                        type="danger"
                        text
                        :icon="Delete"
                        size="small"
                      >
                        移除
                      </el-button>
                    </template>
                  </el-popconfirm>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <!-- Avatar Cropper Modal -->
    <AvatarCropper
      v-model="cropperVisible"
      :initial-image="profileForm.avatar || profile?.avatar"
      @crop="handleAvatarCrop"
    />
  </el-dialog>
</template>

<style lang="scss">
.profile-dialog {
  border-radius: 16px;
  overflow: hidden;
  max-width: calc(100vw - 20px);
  margin: 16px auto !important;
  text-align: left;

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

    @media (max-width: 540px) {
      padding: 14px 16px;
    }

    .el-dialog__title {
      font-size: 17px;
      font-weight: 600;
    }
  }

  .el-dialog__body {
    padding: 20px 24px 28px;
    max-height: calc(88vh - 60px);
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;

    @media (max-width: 540px) {
      padding: 14px 16px 20px;
    }
  }
}
</style>

<style scoped lang="scss">
.user-overview {
  display: flex;
  align-items: center;
  text-align: left;
  gap: 16px;
  padding-bottom: 20px;
  margin-bottom: 8px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));

  @media (max-width: 540px) {
    gap: 12px;
    padding-bottom: 14px;
  }

  .user-avatar-wrapper {
    position: relative;
    border-radius: 50%;
    cursor: pointer;
    overflow: hidden;
    display: inline-flex;
    flex-shrink: 0;

    .user-avatar {
      border: 2px solid var(--c-primary, #008ffd);
      box-shadow: 0 4px 12px rgba(0, 143, 253, 0.2);
      transition: transform 0.2s ease;
    }

    .avatar-hover-overlay {
      position: absolute;
      inset: 0;
      background: rgba(15, 23, 42, 0.62);
      backdrop-filter: blur(2px);
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      color: #fff;
      font-size: 11px;
      gap: 2px;
      opacity: 0;
      border-radius: 50%;
      transition: opacity 0.2s ease;
      user-select: none;
    }

    &:hover {
      .user-avatar {
        transform: scale(1.02);
      }
      .avatar-hover-overlay {
        opacity: 1;
      }
    }
  }

  .user-overview-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    text-align: left;

    .user-name {
      font-size: 18px;
      font-weight: 700;
      color: var(--text-color, #111);
      font-family: 'Outfit', sans-serif;
      text-align: left;
    }

    .user-email {
      font-size: 13px;
      color: #888;
      text-align: left;
    }
  }
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;

  @media (max-width: 480px) {
    font-size: 13px;
    gap: 4px;
  }
}

.storage-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 8px;

  .quota-card {
    background-color: rgba(0, 143, 253, 0.04);
    border: 1px solid rgba(0, 143, 253, 0.12);
    border-radius: 12px;
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;

    .quota-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .quota-title {
        font-size: 14px;
        font-weight: 600;
        color: var(--text-color, #333);
      }

      .quota-ratio {
        font-size: 13px;
        color: #666;

        strong {
          color: var(--c-primary, #008ffd);
          font-weight: 700;
        }
      }
    }

    .quota-footer-tip {
      font-size: 12px;
      color: #888;
    }
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;

    @media (max-width: 540px) {
      gap: 8px;
    }

    @media (max-width: 480px) {
      grid-template-columns: repeat(3, 1fr);
      gap: 6px;

      .stat-box {
        padding: 10px 6px;
        text-align: center;

        .stat-label {
          font-size: 11px;
        }

        .stat-value {
          font-size: 13px;
        }
      }
    }

    .stat-box {
      background-color: rgba(0, 0, 0, 0.02);
      border: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));
      border-radius: 10px;
      padding: 14px 16px;
      display: flex;
      flex-direction: column;
      gap: 6px;

      .stat-label {
        font-size: 12px;
        color: #888;
      }

      .stat-value {
        font-size: 16px;
        font-weight: 600;
        color: var(--text-color, #333);
      }
    }
  }
}

.profile-form,
.security-form {
  padding-top: 12px;

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;

    @media (max-width: 540px) {
      grid-template-columns: 1fr;
      gap: 0;
    }
  }

  .avatar-setting-row {
    display: flex;
    align-items: center;
    gap: 16px;
    width: 100%;
    padding: 10px 14px;
    background-color: rgba(0, 0, 0, 0.02);
    border: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));
    border-radius: 12px;

    @media (max-width: 540px) {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .avatar-thumbnail-wrapper {
      position: relative;
      border-radius: 50%;
      cursor: pointer;
      overflow: hidden;
      display: inline-flex;
      flex-shrink: 0;

      .thumbnail-avatar {
        border: 2px solid var(--c-primary, #008ffd);
        transition: transform 0.2s ease;
      }

      .thumbnail-hover-overlay {
        position: absolute;
        inset: 0;
        background: rgba(15, 23, 42, 0.6);
        display: flex;
        align-items: center;
        justify-content: center;
        color: #fff;
        opacity: 0;
        border-radius: 50%;
        transition: opacity 0.2s ease;
      }

      &:hover {
        .thumbnail-avatar {
          transform: scale(1.04);
        }
        .thumbnail-hover-overlay {
          opacity: 1;
        }
      }
    }

    .avatar-controls-col {
      display: flex;
      flex-direction: column;
      gap: 8px;
      flex: 1;
      width: 100%;

      .avatar-actions {
        display: flex;
        align-items: center;
        gap: 10px;
        flex-wrap: wrap;

        @media (max-width: 480px) {
          width: 100%;
          .el-button {
            flex: 1;
          }
        }
      }

      .avatar-url-row {
        width: 100%;
      }
    }
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    padding-top: 12px;
  }
}

.security-pane {
  padding-top: 12px;

  .security-sub-header {
    margin-bottom: 12px;
  }

  .sub-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-color, #1e293b);
  }

  .sub-desc {
    font-size: 12px;
    color: #888;
    margin-top: 4px;
    line-height: 1.5;
  }

  .security-divider {
    margin: 24px 0;
    border-color: var(--border-color, rgba(0, 0, 0, 0.08));
  }

  .passkey-section {
    display: flex;
    flex-direction: column;
    gap: 14px;

    .passkey-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 16px;

      @media (max-width: 540px) {
        flex-direction: column;
        align-items: flex-start;
      }

      .passkey-header-text {
        display: flex;
        flex-direction: column;
      }

      .bind-passkey-btn {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        flex-shrink: 0;

        .touch-id-btn-icon {
          color: var(--c-primary, #008ffd);
        }
      }
    }

    .passkey-unsupported {
      font-size: 12px;
      color: #e6a23c;
      background: rgba(230, 162, 60, 0.08);
      border: 1px solid rgba(230, 162, 60, 0.2);
      border-radius: 8px;
      padding: 10px 12px;
    }

    .passkey-list {
      display: flex;
      flex-direction: column;
      gap: 10px;
      min-height: 48px;

      .passkey-empty {
        text-align: center;
        padding: 24px 0;
        color: #999;
        font-size: 13px;
        background: rgba(0, 0, 0, 0.02);
        border: 1px dashed var(--border-color, rgba(0, 0, 0, 0.1));
        border-radius: 10px;
      }

      .passkey-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 12px 16px;
        background: rgba(0, 0, 0, 0.02);
        border: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));
        border-radius: 10px;
        transition: background-color 0.2s ease;

        &:hover {
          background: rgba(0, 0, 0, 0.04);
        }

        .passkey-item-left {
          display: flex;
          align-items: center;
          gap: 12px;

          .passkey-icon-badge {
            width: 36px;
            height: 36px;
            border-radius: 8px;
            background: rgba(0, 143, 253, 0.1);
            color: var(--c-primary, #008ffd);
            display: flex;
            align-items: center;
            justify-content: center;
            flex-shrink: 0;
          }

          .passkey-info {
            display: flex;
            flex-direction: column;
            gap: 3px;

            .passkey-name {
              font-size: 14px;
              font-weight: 500;
              color: var(--text-color, #1e293b);
            }

            .passkey-date {
              font-size: 12px;
              color: #888;
            }
          }
        }
      }
    }
  }
}
</style>
