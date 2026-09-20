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
  return full || profile.value?.email || 'No name set'
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
    ElMessage.success('Profile saved successfully')
    emit('updated', updated)
  } catch (error) {
    console.error('Update profile error:', error)
    ElMessage.error('Failed to save profile. Please try again later.')
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
    ElMessage.warning('Please enter your current password')
    return
  }
  if (!passwordForm.password) {
    ElMessage.warning('Please enter a new password')
    return
  }
  if (passwordForm.password.length < 6) {
    ElMessage.warning('Password must be at least 6 characters')
    return
  }
  if (passwordForm.password !== passwordForm.confirmPassword) {
    ElMessage.warning('Passwords do not match')
    return
  }

  savingPassword.value = true
  try {
    await updatePassword({
      oldPassword: passwordForm.oldPassword,
      password: passwordForm.password,
      changePassword: passwordForm.confirmPassword,
    })
    ElMessage.success('Password changed successfully')
    passwordForm.oldPassword = ''
    passwordForm.password = ''
    passwordForm.confirmPassword = ''
  } catch (error: any) {
    const msg = error?.response?.data?.message || 'Failed to change password'
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
    const options = (challenge as any).publicKey || challenge
    const credential = await startRegistration({ optionsJSON: options })
    const isMac = /Macintosh|Mac OS X/i.test(navigator.userAgent)
    const defaultName = isMac ? 'Mac Touch ID' : 'Passkey'
    await webauthnRegisterFinish({
      sessionId,
      credential,
      name: defaultName,
    })
    ElMessage.success(
      'Touch ID passkey registered successfully! You can now sign in using Touch ID.',
    )
    await loadPasskeys()
  } catch (error: any) {
    if (error?.name === 'NotAllowedError') {
      return
    }
    console.error('Passkey registration failed:', error)
    const msg =
      error?.response?.data?.message ||
      error?.message ||
      'Registration failed. Please try again later.'
    ElMessage.error(msg)
  } finally {
    registeringPasskey.value = false
  }
}

const handleDeletePasskey = async (id: string) => {
  try {
    await deletePasskey(id)
    ElMessage.success('Passkey removed successfully')
    await loadPasskeys()
  } catch (error: any) {
    console.error('Delete passkey error:', error)
    ElMessage.error('Failed to remove passkey. Please try again later.')
  }
}

const formatDate = (dateStr?: string) => {
  if (!dateStr) return 'Unknown'
  try {
    const d = new Date(dateStr)
    return d.toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
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
    title="Profile & Settings"
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
          title="Click to change avatar"
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
            <span>Change</span>
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
              <span>Storage</span>
            </span>
          </template>

          <div class="storage-section">
            <div class="quota-card">
              <div class="quota-header">
                <span class="quota-title">Storage Usage</span>
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
                {{ usedPercentage }}% of total storage used
              </div>
            </div>

            <div class="stats-grid">
              <div class="stat-box">
                <span class="stat-label">Used Storage</span>
                <span class="stat-value">{{ formatBytes(usage.usedBytes) }}</span>
              </div>
              <div class="stat-box">
                <span class="stat-label">Total Files</span>
                <span class="stat-value">{{ usage.fileCount }} files</span>
              </div>
              <div class="stat-box">
                <span class="stat-label">Quota Limit</span>
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
              <span>Profile</span>
            </span>
          </template>

          <el-form :model="profileForm" label-position="top" class="profile-form">
            <div class="form-row">
              <el-form-item label="Last Name">
                <el-input v-model="profileForm.lastName" placeholder="Enter last name" />
              </el-form-item>
              <el-form-item label="First Name">
                <el-input v-model="profileForm.firstName" placeholder="Enter first name" />
              </el-form-item>
            </div>

            <el-form-item label="Avatar">
              <div class="avatar-setting-row">
                <div
                  class="avatar-thumbnail-wrapper"
                  title="Click to change avatar"
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
                      Upload & Crop
                    </el-button>
                    <el-button
                      v-if="profileForm.avatar"
                      text
                      type="danger"
                      :icon="Delete"
                      @click="handleClearAvatar"
                    >
                      Remove
                    </el-button>
                  </div>
                  <div class="avatar-url-row">
                    <el-input
                      v-model="profileForm.avatar"
                      placeholder="Or enter image URL (https://...)"
                      clearable
                      size="small"
                    />
                  </div>
                </div>
              </div>
            </el-form-item>

            <div class="form-row">
              <el-form-item label="Gender">
                <el-select v-model="profileForm.gender" style="width: 100%">
                  <el-option :value="0" label="Private" />
                  <el-option :value="1" label="Male" />
                  <el-option :value="2" label="Female" />
                </el-select>
              </el-form-item>
              <el-form-item label="Age">
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
                Save Profile
              </el-button>
            </div>
          </el-form>
        </el-tab-pane>

        <!-- Security Tab -->
        <el-tab-pane name="security">
          <template #label>
            <span class="tab-label">
              <el-icon><Key /></el-icon>
              <span>Security</span>
            </span>
          </template>

          <div class="security-pane">
            <div class="security-sub-header">
              <span class="sub-title">Change Password</span>
            </div>

            <el-form :model="passwordForm" label-position="top" class="security-form">
              <el-form-item label="Current Password">
                <el-input
                  v-model="passwordForm.oldPassword"
                  type="password"
                  show-password
                  placeholder="Enter current password"
                />
              </el-form-item>

              <div class="form-row">
                <el-form-item label="New Password">
                  <el-input
                    v-model="passwordForm.password"
                    type="password"
                    show-password
                    placeholder="Enter at least 6 characters"
                  />
                </el-form-item>

                <el-form-item label="Confirm New Password">
                  <el-input
                    v-model="passwordForm.confirmPassword"
                    type="password"
                    show-password
                    placeholder="Confirm new password"
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
                  Update Password
                </el-button>
              </div>
            </el-form>

            <el-divider class="security-divider" />

            <!-- Touch ID / Passkey Section -->
            <div class="passkey-section">
              <div class="passkey-header">
                <div class="passkey-header-text">
                  <span class="sub-title">Touch ID / Passkeys</span>
                  <span class="sub-desc">
                    Sign in quickly and securely using your device's Touch ID or biometric passkey.
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
                  <span>Register this device</span>
                </el-button>
              </div>

              <div v-if="!supportsWebAuthn" class="passkey-unsupported">
                WebAuthn / Touch ID is not supported in this browser or environment. Please use a modern browser (e.g. Safari, Chrome) over HTTPS or localhost.
              </div>

              <div v-loading="loadingPasskeys" class="passkey-list">
                <div v-if="passkeys.length === 0" class="passkey-empty">
                  <span>No passkeys or Touch ID credentials registered yet</span>
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
                      <div class="passkey-name">{{ item.name || 'Touch ID Credential' }}</div>
                      <div class="passkey-date">Added on: {{ formatDate(item.createdAt) }}</div>
                    </div>
                  </div>
                  <el-popconfirm
                    title="Are you sure you want to remove this Touch ID passkey? You will no longer be able to use it to sign in."
                    confirm-button-text="Confirm"
                    cancel-button-text="Cancel"
                    @confirm="handleDeletePasskey(item.id)"
                  >
                    <template #reference>
                      <el-button
                        type="danger"
                        text
                        :icon="Delete"
                        size="small"
                      >
                        Remove
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
