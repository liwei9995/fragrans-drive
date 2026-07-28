<script setup lang="ts" name="upload-status">
import {
  CircleCloseFilled,
  Close,
  InfoFilled,
  SuccessFilled,
  UploadFilled,
  WarningFilled,
} from '@element-plus/icons-vue'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

interface UploadStatusProps {
  type?: string
  showClose?: boolean
  title?: string
  percentage?: number
  zIndex?: number
  onClose?: () => void
}

const visible = ref(false)
const TypeComponentsMap = {
  error: CircleCloseFilled,
  info: InfoFilled,
  success: SuccessFilled,
  warning: WarningFilled,
  uploading: UploadFilled,
} as Record<string, unknown>

const props = withDefaults(defineProps<UploadStatusProps>(), {
  type: () => 'uploading',
  title: () => '',
  percentage: () => 0,
  zIndex: () => 0,
  showClose: () => true,
})

const icon = computed(() => TypeComponentsMap[props.type])
const uploadPercentage = computed(() =>
  props.type === 'uploading' ? props.percentage : 0,
)

let timer: ReturnType<typeof setTimeout> | null = null

const show = () => {
  visible.value = true
}

const close = () => {
  visible.value = false
  if (timer) {
    clearTimeout(timer)
    timer = null
  }
  props.onClose?.()
}

watch(
  () => props.type,
  (newType) => {
    if (newType === 'success' || newType === 'error') {
      if (timer) clearTimeout(timer)
      timer = setTimeout(() => {
        close()
      }, 3000)
    } else {
      if (timer) {
        clearTimeout(timer)
        timer = null
      }
    }
  },
)

onBeforeUnmount(() => {
  if (timer) clearTimeout(timer)
})

defineExpose({
  show,
  close,
  icon,
  uploadPercentage,
})
</script>

<template>
  <div v-show="visible" class="upload-status-wrapper">
    <div class="upload-status">
      <el-icon :size="24" :class="type">
        <component :is="icon" />
      </el-icon>
      <h2 class="title" v-text="title" />
      <el-icon v-if="showClose" class="close-btn" @click.stop="close">
        <Close />
      </el-icon>
    </div>
    <el-progress
      v-if="type === 'uploading'"
      class="progress"
      :percentage="uploadPercentage"
      :stroke-width="4"
      :show-text="false"
    />
  </div>
</template>

<style scoped lang="scss">
@use './index';
</style>
