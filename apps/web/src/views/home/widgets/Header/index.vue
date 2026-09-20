<script setup lang="ts" name="header">
import { Delete, Link, Search, UserFilled } from '@element-plus/icons-vue'
import type { UploadInstance, UploadProps } from 'element-plus'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import logo from '@/assets/logo.svg'
import { HOME_URL } from '@/config/config'
import { GlobalStore } from '@/store'
import ActionButton from '../ActionButton/index.vue'
import type { BreadcrumbItem } from '../Breadcrumb/index.vue'
import Breadcrumb from '../Breadcrumb/index.vue'

const { t } = useI18n()
const globalStore = GlobalStore()
const router = useRouter()
const uploadRef = ref<UploadInstance>()

type ActionItem = {
  id?: string
  name: string
  isUpload?: boolean
}

interface HeaderProps {
  avatar?: string
  breadcrumbItems?: Partial<BreadcrumbItem>[]
  actionItems?: Partial<ActionItem>[]
  avatarActionItems?: Partial<ActionItem>[]
  uploadFileLimit?: number
  onlyPublic?: boolean
  tapActionItem?: (command: string | number | object) => void
  onUploadChange?: UploadProps['onChange']
  onUploadExceed?: UploadProps['onExceed']
  onUploadProgress?: UploadProps['onProgress']
  onUploadSuccess?: UploadProps['onSuccess']
  onUploadError?: UploadProps['onError']
  beforeUpload?: UploadProps['beforeUpload']
}

const props = withDefaults(defineProps<HeaderProps>(), {
  avatar: () =>
    'https://wpimg.wallstcn.com/f778738c-e4f8-4870-b634-56703b4acafe.gif?imageView2/1/w/80/h/80',
  uploadFileLimit: () => 10,
  breadcrumbItems: () => [],
  actionItems: () => [],
  avatarActionItems: () => [],
  onlyPublic: false,
})

const emit = defineEmits<{
  (e: 'toggle-only-public'): void
  (e: 'open-search'): void
  (e: 'open-trash'): void
}>()

const handleCommand = (command: string | number | object) =>
  props.tapActionItem && props.tapActionItem(command)

const handleClickGoHome = () => router.push(HOME_URL)

const toggleLanguage = () => {
  globalStore.setLanguage(globalStore.language === 'zh' ? 'en' : 'zh')
}

const errorHandler = () => true
</script>

<template>
  <header class="header">
    <div class="navbar-wrapper">
      <div class="header-container">
        <div class="header-title">
          <button
            type="button"
            class="logo-container"
            :aria-label="t('home.backHome')"
            @click="handleClickGoHome"
          >
            <el-image :src="logo" style="width: 24px; height: 24px" />
            <div class="logo-name">Fragrans</div>
          </button>
          <Breadcrumb :breadcrumb-items="breadcrumbItems" />
        </div>
        <div class="content">
          <div class="filter-item">
            <el-tooltip
              :content="onlyPublic ? t('home.clickShowAll') : t('home.onlyPublicHint')"
              placement="bottom"
            >
              <el-button
                class="public-filter-btn"
                :type="onlyPublic ? 'primary' : 'default'"
                :plain="!onlyPublic"
                size="small"
                round
                @click="emit('toggle-only-public')"
              >
                <el-icon><Link /></el-icon>
                <span>{{ onlyPublic ? t('home.onlyPublic') : t('home.publicFiles') }}</span>
              </el-button>
            </el-tooltip>
          </div>
          <div class="search" @click="emit('open-search')">
            <el-tooltip :content="t('home.searchPlaceholder')" placement="bottom">
              <el-icon :size="18">
                <Search />
              </el-icon>
            </el-tooltip>
          </div>
          <div class="trash-btn" @click="emit('open-trash')">
            <el-tooltip :content="t('home.trash')" placement="bottom">
              <el-icon :size="18">
                <Delete />
              </el-icon>
            </el-tooltip>
          </div>
          <div class="lang-btn" @click="toggleLanguage">
            <el-tooltip :content="t('home.switchLangTip')" placement="bottom">
              <span class="lang-code">{{ globalStore.language === 'zh' ? 'EN' : '中' }}</span>
            </el-tooltip>
          </div>
          <div class="action">
            <ActionButton
              ref="uploadRef"
              :action-items="actionItems"
              :upload-file-limit="uploadFileLimit"
              :tap-action-item="handleCommand"
              :on-upload-change="onUploadChange"
              :on-upload-exceed="onUploadExceed"
              :on-upload-progress="onUploadProgress"
              :on-upload-success="onUploadSuccess"
              :on-upload-error="onUploadError"
              :before-upload="beforeUpload"
            />
          </div>
          <el-dropdown trigger="click" @command="handleCommand">
            <div class="avatar">
              <el-avatar :src="avatar" :size="34" @error="errorHandler">
                <el-avatar :icon="UserFilled" :size="34" />
              </el-avatar>
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item v-for="item in avatarActionItems" :key="item.id" :command="item.id">
                  {{ item.name }}
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
    </div>
  </header>
  <div class="float-action">
    <ActionButton
      :action-items="actionItems"
      :upload-file-limit="uploadFileLimit"
      :icon-size="24"
      :tap-action-item="handleCommand"
      :on-upload-change="onUploadChange"
      :on-upload-exceed="onUploadExceed"
      :on-upload-progress="onUploadProgress"
      :on-upload-success="onUploadSuccess"
      :on-upload-error="onUploadError"
      :before-upload="beforeUpload"
    />
  </div>
</template>

<style scoped lang="scss">
@use './index';
</style>
