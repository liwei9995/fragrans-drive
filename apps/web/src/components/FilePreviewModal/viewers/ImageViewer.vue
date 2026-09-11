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
}

const props = withDefaults(defineProps<Props>(), {
  src: '',
  name: '',
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
const loadError = ref(false)

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
  () => {
    loading.value = true
    loadError.value = false
    resetTransform()
  },
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
  loadError.value = false
  const img = e.target as HTMLImageElement
  naturalWidth.value = img.naturalWidth
  naturalHeight.value = img.naturalHeight
}

const handleImageError = () => {
  loading.value = false
  loadError.value = true
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
      <div v-if="loading" class="image-loading">
        <el-icon class="is-loading" :size="32"><Refresh /></el-icon>
        <span>正在载入高分辨率原图...</span>
      </div>

      <div v-else-if="loadError" class="image-error">
        <el-icon :size="48"><Refresh /></el-icon>
        <span>图片加载失败，请检查网络或刷新重试</span>
      </div>

      <img
        v-show="!loading && !loadError"
        :src="src"
        :alt="name"
        class="preview-image"
        :style="transformStyle"
        draggable="false"
        @load="handleImageLoaded"
        @error="handleImageError"
      />
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
      max-width: 90%;
      max-height: 85%;
      object-fit: contain;
      transition: transform 0.15s cubic-bezier(0.2, 0, 0, 1);
      box-shadow: 0 20px 40px -15px rgba(0, 0, 0, 0.5);
      border-radius: 6px;
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

    .dimension-tag,
    .scale-tag {
      font-size: 12px;
      color: #cbd5e1;
      font-variant-numeric: tabular-nums;
      padding: 0 4px;
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

      &:hover {
        background: rgba(255, 255, 255, 0.15);
        color: #38bdf8;
        transform: scale(1.08);
      }

      &:active {
        transform: scale(0.95);
      }
    }
  }
}
</style>
