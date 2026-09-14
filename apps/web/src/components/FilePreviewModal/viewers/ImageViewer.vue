<script setup lang="ts">
import {
  FullScreen,
  Refresh,
  RefreshRight,
  ZoomIn,
  ZoomOut,
} from '@element-plus/icons-vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

interface Props {
  src: string
  name?: string
  thumb?: string
}

const props = withDefaults(defineProps<Props>(), {
  src: '',
  name: '',
  thumb: '',
})

const scale = ref(1)
const rotate = ref(0)
const flipH = ref(false)
const flipV = ref(false)
const isDragging = ref(false)
const position = ref({ x: 0, y: 0 })
const dragStart = ref({ x: 0, y: 0 })
const naturalWidth = ref(0)
const naturalHeight = ref(0)
const loading = ref(true)
const highResLoaded = ref(false)
const loadError = ref(false)
const imageSrc = ref(props.src)

const hasRealThumb = computed(() => {
  if (!props.thumb) return false
  const t = props.thumb.toLowerCase()
  if (
    t.includes('assets/icons/') ||
    t.includes('file_unknown') ||
    t.includes('file_image') ||
    t.includes('img.alicdn.com')
  ) {
    return false
  }
  return true
})

const transformStyle = computed(() => {
  const sx = flipH.value ? -1 : 1
  const sy = flipV.value ? -1 : 1
  return {
    transform: `translate(${position.value.x}px, ${position.value.y}px) scale(${scale.value}) rotate(${rotate.value}deg) scale(${sx}, ${sy})`,
    cursor:
      scale.value > 1 ? (isDragging.value ? 'grabbing' : 'grab') : 'default',
  }
})

const resetTransform = () => {
  scale.value = 1
  rotate.value = 0
  flipH.value = false
  flipV.value = false
  position.value = { x: 0, y: 0 }
}

watch(
  () => props.src,
  (newSrc) => {
    imageSrc.value = newSrc
    loading.value = true
    highResLoaded.value = false
    loadError.value = false
    naturalWidth.value = 0
    naturalHeight.value = 0
    resetTransform()
  },
  { immediate: true },
)

const handleZoomIn = () => {
  scale.value = Math.min(5, Number((scale.value + 0.25).toFixed(2)))
}

const handleZoomOut = () => {
  scale.value = Math.max(0.2, Number((scale.value - 0.25).toFixed(2)))
}

const handleRotate = () => {
  rotate.value = (rotate.value + 90) % 360
}

const handleReset = () => {
  resetTransform()
}

const handleWheel = (e: WheelEvent) => {
  e.preventDefault()
  if (e.deltaY < 0) {
    handleZoomIn()
  } else {
    handleZoomOut()
  }
}

const handleMouseDown = (e: MouseEvent) => {
  if (scale.value <= 1) return
  isDragging.value = true
  dragStart.value = {
    x: e.clientX - position.value.x,
    y: e.clientY - position.value.y,
  }
}

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return
  position.value = {
    x: e.clientX - dragStart.value.x,
    y: e.clientY - dragStart.value.y,
  }
}

const handleMouseUp = () => {
  isDragging.value = false
}

const handleImageLoaded = (e: Event) => {
  loading.value = false
  highResLoaded.value = true
  loadError.value = false
  const img = e.target as HTMLImageElement
  naturalWidth.value = img.naturalWidth
  naturalHeight.value = img.naturalHeight
}

const handleImageError = () => {
  loading.value = false
  loadError.value = true
}

const handleRetry = () => {
  loadError.value = false
  loading.value = true
  highResLoaded.value = false
  const sep = props.src.includes('?') ? '&' : '?'
  imageSrc.value = `${props.src}${sep}_retry=${Date.now()}`
}

onMounted(() => {
  window.addEventListener('mouseup', handleMouseUp)
  window.addEventListener('mousemove', handleMouseMove)
})

onBeforeUnmount(() => {
  window.removeEventListener('mouseup', handleMouseUp)
  window.removeEventListener('mousemove', handleMouseMove)
})
</script>

<template>
  <div class="image-viewer" @wheel="handleWheel">
    <div class="image-viewport" @mousedown="handleMouseDown">
      <!-- Full blocking spinner fallback ONLY when no LQIP thumbnail is available -->
      <div v-if="loading && !hasRealThumb && !loadError" class="image-loading">
        <el-icon class="is-loading" :size="32"><Refresh /></el-icon>
        <span>正在载入高分辨率原图...</span>
      </div>

      <!-- Error State with Retry -->
      <div v-else-if="loadError" class="image-error">
        <el-icon :size="48"><Refresh /></el-icon>
        <span>图片加载失败，请检查网络或刷新重试</span>
        <button class="retry-btn" @click="handleRetry">重新加载</button>
      </div>

      <!-- Thumbnail / LQIP layer: shown immediately while high-res image is loading -->
      <img
        v-if="hasRealThumb && !loadError"
        :src="thumb"
        :alt="name"
        class="preview-image preview-thumb"
        :class="{ 'is-fading-out': highResLoaded }"
        :style="transformStyle"
        draggable="false"
      />

      <!-- High-Res Original: rendered immediately, fades in smoothly once loaded -->
      <img
        v-show="!loadError"
        :src="imageSrc"
        :alt="name"
        class="preview-image main-image"
        :class="{ 'is-loading': !highResLoaded }"
        :style="transformStyle"
        draggable="false"
        @load="handleImageLoaded"
        @error="handleImageError"
      />

      <!-- Non-blocking floating status badge when previewing with thumbnail placeholder -->
      <div v-if="loading && hasRealThumb && !loadError" class="image-loading-badge">
        <el-icon class="is-loading" :size="14"><Refresh /></el-icon>
        <span>高清载入中...</span>
      </div>
    </div>

    <!-- Floating bottom toolbar -->
    <div class="image-toolbar">
      <span v-if="naturalWidth && naturalHeight" class="dimension-tag">
        {{ naturalWidth }} × {{ naturalHeight }}
      </span>
      <span class="scale-tag">{{ Math.round(scale * 100) }}%</span>

      <div class="divider"></div>

      <el-tooltip content="缩小 (滚轮向下)" placement="top">
        <button class="tool-btn" @click="handleZoomOut">
          <el-icon :size="16"><ZoomOut /></el-icon>
        </button>
      </el-tooltip>

      <el-tooltip content="重置大小" placement="top">
        <button class="tool-btn" @click="handleReset">
          <el-icon :size="16"><FullScreen /></el-icon>
        </button>
      </el-tooltip>

      <el-tooltip content="放大 (滚轮向上)" placement="top">
        <button class="tool-btn" @click="handleZoomIn">
          <el-icon :size="16"><ZoomIn /></el-icon>
        </button>
      </el-tooltip>

      <el-tooltip content="向右顺时针旋转 90°" placement="top">
        <button class="tool-btn" @click="handleRotate">
          <el-icon :size="16"><RefreshRight /></el-icon>
        </button>
      </el-tooltip>
    </div>
  </div>
</template>

<style scoped lang="scss">
.image-viewer {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  user-select: none;
  background: radial-gradient(circle at 50% 50%, rgba(26, 32, 44, 0.6) 0%, rgba(15, 18, 26, 0.95) 100%);

  .image-viewport {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;

    .preview-image {
      width: 90%;
      height: 85%;
      max-width: 90%;
      max-height: 85%;
      object-fit: contain;
      transition: transform 0.15s cubic-bezier(0.2, 0, 0, 1);
      box-shadow: 0 20px 40px -15px rgba(0, 0, 0, 0.5);
      border-radius: 6px;

      &.preview-thumb {
        position: absolute;
        filter: blur(8px);
        transform-origin: center center;
        opacity: 1;
        transition: transform 0.15s cubic-bezier(0.2, 0, 0, 1), opacity 0.35s ease;

        &.is-fading-out {
          opacity: 0;
          pointer-events: none;
        }
      }

      &.main-image {
        position: relative;
        opacity: 1;
        transition: transform 0.15s cubic-bezier(0.2, 0, 0, 1), opacity 0.3s ease;

        &.is-loading {
          opacity: 0;
          pointer-events: none;
        }
      }
    }

    .image-loading,
    .image-error {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 14px;
      color: #94a3b8;
      font-size: 14px;
    }

    .image-error {
      color: #f87171;

      .retry-btn {
        margin-top: 4px;
        padding: 6px 16px;
        background: rgba(239, 68, 68, 0.15);
        color: #fca5a5;
        border: 1px solid rgba(239, 68, 68, 0.3);
        border-radius: 6px;
        font-size: 13px;
        cursor: pointer;
        transition: all 0.2s ease;

        &:hover {
          background: rgba(239, 68, 68, 0.25);
          color: #ffffff;
        }
      }
    }

    .image-loading-badge {
      position: absolute;
      top: 20px;
      left: 50%;
      transform: translateX(-50%);
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 6px 14px;
      background: rgba(15, 23, 42, 0.8);
      border: 1px solid rgba(255, 255, 255, 0.12);
      backdrop-filter: blur(12px);
      border-radius: 9999px;
      box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
      color: #cbd5e1;
      font-size: 13px;
      z-index: 10;
      pointer-events: none;
      animation: fadeInBadge 0.25s cubic-bezier(0.16, 1, 0.3, 1);

      .is-loading {
        color: var(--c-primary, #008ffd);
      }
    }
  }

  @keyframes fadeInBadge {
    from {
      opacity: 0;
      transform: translate(-50%, -8px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }

  .image-toolbar {
    position: absolute;
    bottom: 24px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 16px;
    background: rgba(15, 23, 42, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(16px);
    border-radius: 9999px;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5);
    z-index: 10;
    white-space: nowrap;
    max-width: 95vw;

    @media (max-width: 640px) {
      bottom: 16px;
      padding: 4px 10px;
      gap: 4px;

      .dimension-tag {
        display: none;
      }

      .tool-btn {
        width: 28px;
        height: 28px;
      }
    }

    .dimension-tag,
    .scale-tag {
      font-size: 12px;
      color: #cbd5e1;
      font-variant-numeric: tabular-nums;
      padding: 0 4px;
      white-space: nowrap;
    }

    .divider {
      width: 1px;
      height: 14px;
      background: rgba(255, 255, 255, 0.2);
      margin: 0 4px;
    }

    .tool-btn {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 32px;
      height: 32px;
      border-radius: 50%;
      background: transparent;
      border: none;
      color: #f1f5f9;
      cursor: pointer;
      transition: all 0.2s ease;
      flex-shrink: 0;

      &:hover {
        background: rgba(255, 255, 255, 0.15);
        color: var(--c-primary-light, #33a5fd);
        transform: scale(1.08);
      }

      &:active {
        transform: scale(0.95);
      }
    }
  }
}

@media (prefers-color-scheme: light) {
  .image-viewer {
    background: radial-gradient(circle at 50% 50%, rgba(248, 250, 252, 0.8) 0%, rgba(226, 232, 240, 0.9) 100%);

    .image-viewport {
      .preview-image {
        box-shadow: 0 20px 40px -15px rgba(0, 0, 0, 0.18), 0 0 0 1px rgba(0, 0, 0, 0.05);
      }

      .image-loading {
        color: #64748b;
      }

      .image-loading-badge {
        background: rgba(255, 255, 255, 0.88);
        border: 1px solid rgba(0, 0, 0, 0.08);
        color: #475569;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);

        .is-loading {
          color: var(--c-primary, #008ffd);
        }
      }
    }

    .image-toolbar {
      background: rgba(255, 255, 255, 0.88);
      border: 1px solid rgba(0, 0, 0, 0.08);
      box-shadow: 0 10px 30px -5px rgba(0, 0, 0, 0.12);

      .dimension-tag,
      .scale-tag {
        color: #475569;
      }

      .divider {
        background: rgba(0, 0, 0, 0.1);
      }

      .tool-btn {
        color: #334155;

        &:hover {
          background: rgba(0, 0, 0, 0.06);
          color: var(--c-primary, #008ffd);
        }
      }
    }
  }
}
</style>
