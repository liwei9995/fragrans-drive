<script setup lang="ts" name="video-player">
import { Close } from '@element-plus/icons-vue'
import { onBeforeUnmount, onMounted } from 'vue'

interface VideoPlayerProps {
  src?: string
  close?: () => void
}

const props = withDefaults(defineProps<VideoPlayerProps>(), {
  src: '',
})

const handleClose = () => props.close?.()
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') handleClose()
}

onMounted(() => document.addEventListener('keydown', handleKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', handleKeydown))

defineExpose({ handleClose })
</script>

<template>
  <div class="video-player-wrapper">
    <div class="video-player-mask" />
    <button
      type="button"
      class="video-player-close-btn"
      aria-label="关闭视频"
      @click="handleClose"
    >
      <el-icon circle size="24">
        <Close />
      </el-icon>
    </button>
    <div class="video-wrapper">
      <div class="inner-wrapper">
        <video class="video" :src="src" controls playsinline />
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use './index';
</style>
