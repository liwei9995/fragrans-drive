<script setup lang="ts" name="profile-dialog">
import {
  Check,
  CircleCheck,
  Folder,
  Key,
  PieChart,
  User,
  UserFilled,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { computed, onMounted, reactive, ref, watch } from 'vue'
import type { Storage, User as UserType } from '@/api/interface'
import { getStorageUsage } from '@/api/modules/storage'
import { getProfile, updatePassword, updateProfile } from '@/api/modules/user'
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
      avatar: profileForm.avatar.trim() || undefined,
      gender: profileForm.gender,
      age: profileForm.age > 0 ? profileForm.age : undefined,
    })
    profile.value = updated
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
        <el-avatar
          :size="64"
          :src="profileForm.avatar || profile?.avatar"
          class="user-avatar"
        >
          <el-icon :size="32"><UserFilled /></el-icon>
        </el-avatar>
        <div class="user-overview-info">
          <div class="user-name">
            {{ (profile?.firstName || '') + ' ' + (profile?.lastName || '') || '未设置昵称' }}
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

            <el-form-item label="头像图片 URL">
              <el-input
                v-model="profileForm.avatar"
                placeholder="https://example.com/avatar.png"
                clearable
              />
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
              <span>修改密码</span>
            </span>
          </template>

          <el-form :model="passwordForm" label-position="top" class="security-form">
            <el-form-item label="当前密码">
              <el-input
                v-model="passwordForm.oldPassword"
                type="password"
                show-password
                placeholder="请输入当前正在使用的密码"
              />
            </el-form-item>

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
        </el-tab-pane>
      </el-tabs>
    </div>
  </el-dialog>
</template>

<style scoped lang="scss">
:deep(.profile-dialog) {
  border-radius: 16px;
  overflow: hidden;

  .el-dialog__header {
    margin-right: 0;
    padding: 18px 24px;
    border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));

    .el-dialog__title {
      font-size: 17px;
      font-weight: 600;
    }
  }

  .el-dialog__body {
    padding: 20px 24px 28px;
  }
}

.user-overview {
  display: flex;
  align-items: center;
  gap: 16px;
  padding-bottom: 20px;
  margin-bottom: 8px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));

  .user-avatar {
    border: 2px solid var(--c-primary, #409eff);
    box-shadow: 0 4px 12px rgba(64, 158, 255, 0.2);
  }

  .user-overview-info {
    display: flex;
    flex-direction: column;
    gap: 4px;

    .user-name {
      font-size: 18px;
      font-weight: 600;
      color: #222;
    }

    .user-email {
      font-size: 13px;
      color: #888;
    }
  }
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
}

.storage-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 8px;

  .quota-card {
    background-color: rgba(64, 158, 255, 0.04);
    border: 1px solid rgba(64, 158, 255, 0.12);
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
        color: #333;
      }

      .quota-ratio {
        font-size: 13px;
        color: #666;

        strong {
          color: var(--c-primary, #409eff);
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
        color: #333;
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
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    padding-top: 12px;
  }
}
</style>
