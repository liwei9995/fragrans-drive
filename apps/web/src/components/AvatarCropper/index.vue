<script setup lang="ts" name="avatar-cropper">
import {
  Camera,
  Check,
  Close,
  Picture,
  Refresh,
  RefreshRight,
  Upload,
  ZoomIn,
  ZoomOut,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

interface Props {
  modelValue: boolean
  initialImage?: string
  outputSize?: number
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  initialImage: '',
  outputSize: 256,
})

const emit = defineEmits(['update:modelValue', 'crop', 'close'])

const fileInputRef = ref<HTMLInputElement | null>(null)
const canvasRef = ref<HTMLCanvasElement | null>(null)
const previewCanvasRef = ref<HTMLCanvasElement | null>(null)
const previewSmallRef = ref<HTMLCanvasElement | null>(null)

// Cropper state
const imageSrc = ref<string>('')
const imgElement = ref<HTMLImageElement | null>(null)
const isImageLoaded = ref(false)
const isDragging = ref(false)
const dragStart = { x: 0, y: 0 }
const offsetStart = { x: 0, y: 0 }

const offsetX = ref(0)
const offsetY = ref(0)
const scale = ref(1)
const minScale = ref(0.2)
const maxScale = ref(4)
const rotate = ref(0) // 0, 90, 180, 270

// Viewport & crop circle parameters
const viewportWidth = 360
const viewportHeight = 320
const cropRadius = 110 // 220px diameter crop circle

// Drag and drop state for upload area
const isDragOver = ref(false)

// Touch handling for pinch-zoom
let initialPinchDistance = 0
let initialPinchScale = 1

const zoomPercent = computed(() => {
  return Math.round((scale.value / minScale.value) * 100)
})

const triggerFileInput = () => {
  fileInputRef.value?.click()
}

const handleFileChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) {
    loadFile(file)
  }
  // reset input value so re-selecting same file triggers change
  target.value = ''
}

const handleDrop = (e: DragEvent) => {
  isDragOver.value = false
  const file = e.dataTransfer?.files?.[0]
  if (file) {
    loadFile(file)
  }
}

const loadFile = (file: File) => {
  if (!file.type.startsWith('image/')) {
    ElMessage.error('请选择有效的图片文件 (PNG, JPG, WEBP, GIF)')
    return
  }
  if (file.size > 10 * 1024 * 1024) {
    ElMessage.error('图片大小不能超过 10MB')
    return
  }

  const reader = new FileReader()
  reader.onload = (ev) => {
    const res = ev.target?.result as string
    if (res) {
      loadImage(res)
    }
  }
  reader.readAsDataURL(file)
}

const loadImage = (src: string) => {
  const img = new Image()
  img.crossOrigin = 'anonymous'
  img.onload = () => {
    imgElement.value = img
    imageSrc.value = src
    isImageLoaded.value = true
    resetTransform()
    nextTick(() => {
      drawCanvas()
    })
  }
  img.onerror = () => {
    ElMessage.error('图片加载失败，请重试')
  }
  img.src = src
}

const resetTransform = () => {
  if (!imgElement.value) return
  const img = imgElement.value
  const cropDiameter = cropRadius * 2
  // Compute scale so the image covers the crop circle completely
  const fitScale = Math.max(cropDiameter / img.width, cropDiameter / img.height)
  minScale.value = Number((fitScale * 0.5).toFixed(3))
  maxScale.value = Number((fitScale * 5).toFixed(3))
  scale.value = Number(fitScale.toFixed(3))
  offsetX.value = 0
  offsetY.value = 0
  rotate.value = 0
  drawCanvas()
}

const handleRotate = () => {
  rotate.value = (rotate.value + 90) % 360
  drawCanvas()
}

const handleZoomIn = () => {
  const step = (maxScale.value - minScale.value) / 10
  scale.value = Math.min(maxScale.value, scale.value + step)
  drawCanvas()
}

const handleZoomOut = () => {
  const step = (maxScale.value - minScale.value) / 10
  scale.value = Math.max(minScale.value, scale.value - step)
  drawCanvas()
}

const handleWheel = (e: WheelEvent) => {
  e.preventDefault()
  const delta = e.deltaY < 0 ? 1 : -1
  const step = (maxScale.value - minScale.value) / 20
  scale.value = Math.min(
    maxScale.value,
    Math.max(minScale.value, scale.value + delta * step),
  )
  drawCanvas()
}

// Mouse dragging
const handleMouseDown = (e: MouseEvent) => {
  if (!isImageLoaded.value) return
  isDragging.value = true
  dragStart.x = e.clientX
  dragStart.y = e.clientY
  offsetStart.x = offsetX.value
  offsetStart.y = offsetY.value
  window.addEventListener('mousemove', handleMouseMove)
  window.addEventListener('mouseup', handleMouseUp)
}

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return
  offsetX.value = offsetStart.x + (e.clientX - dragStart.x)
  offsetY.value = offsetStart.y + (e.clientY - dragStart.y)
  drawCanvas()
}

const handleMouseUp = () => {
  isDragging.value = false
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('mouseup', handleMouseUp)
}

// Touch handling
const getTouchDistance = (t1: Touch, t2: Touch) => {
  return Math.hypot(t2.clientX - t1.clientX, t2.clientY - t1.clientY)
}

const handleTouchStart = (e: TouchEvent) => {
  if (!isImageLoaded.value) return
  if (e.touches.length === 1) {
    const t = e.touches[0]
    isDragging.value = true
    dragStart.x = t.clientX
    dragStart.y = t.clientY
    offsetStart.x = offsetX.value
    offsetStart.y = offsetY.value
  } else if (e.touches.length === 2) {
    isDragging.value = false
    initialPinchDistance = getTouchDistance(e.touches[0], e.touches[1])
    initialPinchScale = scale.value
  }
}

const handleTouchMove = (e: TouchEvent) => {
  if (!isImageLoaded.value) return
  e.preventDefault()
  if (e.touches.length === 1 && isDragging.value) {
    const t = e.touches[0]
    offsetX.value = offsetStart.x + (t.clientX - dragStart.x)
    offsetY.value = offsetStart.y + (t.clientY - dragStart.y)
    drawCanvas()
  } else if (e.touches.length === 2 && initialPinchDistance > 0) {
    const currentDist = getTouchDistance(e.touches[0], e.touches[1])
    const factor = currentDist / initialPinchDistance
    scale.value = Math.min(
      maxScale.value,
      Math.max(minScale.value, initialPinchScale * factor),
    )
    drawCanvas()
  }
}

const handleTouchEnd = () => {
  isDragging.value = false
  initialPinchDistance = 0
}

// Rendering canvas
const drawCanvas = () => {
  const canvas = canvasRef.value
  if (!canvas || !imgElement.value) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  canvas.width = viewportWidth * dpr
  canvas.height = viewportHeight * dpr
  ctx.resetTransform?.()
  ctx.scale(dpr, dpr)

  const cx = viewportWidth / 2
  const cy = viewportHeight / 2
  const img = imgElement.value

  // 1. Draw transformed image
  ctx.save()
  ctx.clearRect(0, 0, viewportWidth, viewportHeight)
  ctx.translate(cx + offsetX.value, cy + offsetY.value)
  ctx.rotate((rotate.value * Math.PI) / 180)
  ctx.scale(scale.value, scale.value)
  ctx.drawImage(img, -img.width / 2, -img.height / 2, img.width, img.height)
  ctx.restore()

  // 2. Draw mask overlay (outside circular aperture)
  ctx.save()
  ctx.fillStyle = 'rgba(15, 23, 42, 0.65)'
  ctx.beginPath()
  ctx.rect(0, 0, viewportWidth, viewportHeight)
  // Create circular hole using counter-clockwise winding
  ctx.arc(cx, cy, cropRadius, 0, Math.PI * 2, true)
  ctx.fill()
  ctx.restore()

  // 3. Draw circular crop border
  ctx.save()
  ctx.beginPath()
  ctx.arc(cx, cy, cropRadius, 0, Math.PI * 2)
  ctx.lineWidth = 2
  ctx.strokeStyle = '#008ffd'
  ctx.stroke()

  // Outer subtle white ring
  ctx.beginPath()
  ctx.arc(cx, cy, cropRadius + 1, 0, Math.PI * 2)
  ctx.lineWidth = 1
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.4)'
  ctx.stroke()

  // 4. Subtle grid lines inside circle (rule of thirds)
  ctx.save()
  ctx.beginPath()
  ctx.arc(cx, cy, cropRadius - 1, 0, Math.PI * 2)
  ctx.clip()

  ctx.strokeStyle = 'rgba(255, 255, 255, 0.22)'
  ctx.lineWidth = 1
  const third = (cropRadius * 2) / 3
  const leftX = cx - cropRadius + third
  const rightX = cx + cropRadius - third
  const topY = cy - cropRadius + third
  const bottomY = cy + cropRadius - third

  ctx.beginPath()
  ctx.moveTo(leftX, cy - cropRadius)
  ctx.lineTo(leftX, cy + cropRadius)
  ctx.moveTo(rightX, cy - cropRadius)
  ctx.lineTo(rightX, cy + cropRadius)
  ctx.moveTo(cx - cropRadius, topY)
  ctx.lineTo(cx + cropRadius, topY)
  ctx.moveTo(cx - cropRadius, bottomY)
  ctx.lineTo(cx + cropRadius, bottomY)
  ctx.stroke()
  ctx.restore()

  ctx.restore()

  // Draw small previews
  drawPreviews()
}

const drawPreviews = () => {
  if (!imgElement.value) return
  const img = imgElement.value

  const renderToPreview = (canvas: HTMLCanvasElement | null, size: number) => {
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const dpr = window.devicePixelRatio || 1
    canvas.width = size * dpr
    canvas.height = size * dpr
    ctx.resetTransform?.()
    ctx.scale(dpr, dpr)

    ctx.clearRect(0, 0, size, size)

    const outRadius = size / 2
    const M = outRadius / cropRadius

    ctx.save()
    ctx.beginPath()
    ctx.arc(outRadius, outRadius, outRadius, 0, Math.PI * 2)
    ctx.clip()

    ctx.translate(outRadius + offsetX.value * M, outRadius + offsetY.value * M)
    ctx.rotate((rotate.value * Math.PI) / 180)
    ctx.scale(scale.value * M, scale.value * M)
    ctx.drawImage(img, -img.width / 2, -img.height / 2, img.width, img.height)
    ctx.restore()
  }

  renderToPreview(previewCanvasRef.value, 72)
  renderToPreview(previewSmallRef.value, 34)
}

const handleConfirmCrop = () => {
  if (!imgElement.value) {
    ElMessage.warning('请先选择图片')
    return
  }

  const outSize = props.outputSize
  const outCanvas = document.createElement('canvas')
  outCanvas.width = outSize
  outCanvas.height = outSize
  const outCtx = outCanvas.getContext('2d')
  if (!outCtx) return

  const outRadius = outSize / 2
  const M = outRadius / cropRadius
  const img = imgElement.value

  // Clip to circle for clean transparent corners
  outCtx.save()
  outCtx.beginPath()
  outCtx.arc(outRadius, outRadius, outRadius, 0, Math.PI * 2)
  outCtx.closePath()
  outCtx.clip()

  outCtx.translate(outRadius + offsetX.value * M, outRadius + offsetY.value * M)
  outCtx.rotate((rotate.value * Math.PI) / 180)
  outCtx.scale(scale.value * M, scale.value * M)
  outCtx.drawImage(img, -img.width / 2, -img.height / 2, img.width, img.height)
  outCtx.restore()

  const croppedDataUrl = outCanvas.toDataURL('image/png')
  emit('crop', croppedDataUrl)
  emit('update:modelValue', false)
}

const handleClose = () => {
  emit('update:modelValue', false)
  emit('close')
}

watch(
  () => props.modelValue,
  (val) => {
    if (val) {
      if (props.initialImage) {
        loadImage(props.initialImage)
      } else {
        isImageLoaded.value = false
        imgElement.value = null
        imageSrc.value = ''
      }
    } else {
      isDragging.value = false
      window.removeEventListener('mousemove', handleMouseMove)
      window.removeEventListener('mouseup', handleMouseUp)
    }
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('mouseup', handleMouseUp)
})
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    title="裁剪并上传头像"
    width="540px"
    class="avatar-cropper-dialog"
    destroy-on-close
    append-to-body
    @close="handleClose"
  >
    <!-- Hidden file input -->
    <input
      ref="fileInputRef"
      type="file"
      accept="image/png,image/jpeg,image/webp,image/gif"
      style="display: none"
      @change="handleFileChange"
    />

    <div class="cropper-body">
      <!-- Upload Dropzone when no image loaded -->
      <div
        v-if="!isImageLoaded"
        class="upload-dropzone"
        :class="{ 'is-dragover': isDragOver }"
        @click="triggerFileInput"
        @dragover.prevent="isDragOver = true"
        @dragleave.prevent="isDragOver = false"
        @drop.prevent="handleDrop"
      >
        <div class="dropzone-icon">
          <el-icon :size="48"><Upload /></el-icon>
        </div>
        <div class="dropzone-title">点击或将图片拖拽到此处上传</div>
        <div class="dropzone-sub">
          支持 JPG、PNG、WEBP、GIF 格式，文件大小不超过 10MB
        </div>
        <el-button type="primary" :icon="Picture" class="choose-file-btn">
          选择本地图片
        </el-button>
      </div>

      <!-- Cropper Workspace when image is loaded -->
      <div v-else class="cropper-workspace">
        <div class="workspace-main">
          <!-- Canvas Viewport -->
          <div
            class="viewport-box"
            :class="{ dragging: isDragging }"
            @mousedown="handleMouseDown"
            @wheel="handleWheel"
            @touchstart="handleTouchStart"
            @touchmove="handleTouchMove"
            @touchend="handleTouchEnd"
          >
            <canvas
              ref="canvasRef"
              :style="{ width: `${viewportWidth}px`, height: `${viewportHeight}px` }"
              class="cropper-canvas"
            ></canvas>
            <div class="viewport-tip">可按住鼠标拖拽平移，滚轮缩放</div>
          </div>

          <!-- Live Round Previews -->
          <div class="preview-sidebar">
            <div class="preview-label">预览效果</div>
            <div class="preview-circle-box large">
              <canvas
                ref="previewCanvasRef"
                class="preview-canvas"
                style="width: 72px; height: 72px"
              ></canvas>
              <span class="size-label">72×72</span>
            </div>
            <div class="preview-circle-box small">
              <canvas
                ref="previewSmallRef"
                class="preview-canvas"
                style="width: 34px; height: 34px"
              ></canvas>
              <span class="size-label">34×34</span>
            </div>
          </div>
        </div>

        <!-- Controls Toolbar -->
        <div class="cropper-toolbar">
          <div class="zoom-control">
            <el-button
              circle
              size="small"
              :icon="ZoomOut"
              title="缩小"
              @click="handleZoomOut"
            />
            <el-slider
              v-model="scale"
              :min="minScale"
              :max="maxScale"
              :step="0.01"
              :show-tooltip="false"
              class="zoom-slider"
              @input="drawCanvas"
            />
            <el-button
              circle
              size="small"
              :icon="ZoomIn"
              title="放大"
              @click="handleZoomIn"
            />
            <span class="zoom-text">{{ zoomPercent }}%</span>
          </div>

          <div class="tool-buttons">
            <el-button
              size="small"
              :icon="RefreshRight"
              title="顺时针旋转90°"
              @click="handleRotate"
            >
              旋转 90°
            </el-button>
            <el-button
              size="small"
              :icon="Refresh"
              title="还原缩放与位置"
              @click="resetTransform"
            >
              重置
            </el-button>
            <el-button
              size="small"
              :icon="Picture"
              title="更换其他图片"
              @click="triggerFileInput"
            >
              换图
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button
          v-if="isImageLoaded"
          type="primary"
          :icon="Check"
          @click="handleConfirmCrop"
        >
          确定使用并保存
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style lang="scss">
.avatar-cropper-dialog {
  border-radius: 16px;
  overflow: hidden;
  max-width: calc(100vw - 20px);
  margin: 16px auto !important;

  @media (max-width: 768px) {
    --el-dialog-width: calc(100vw - 20px) !important;
    width: calc(100vw - 20px) !important;
    max-width: calc(100vw - 20px) !important;
    margin: 12px auto !important;
  }

  .el-dialog__header {
    margin-right: 0;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));

    @media (max-width: 640px) {
      padding: 12px 16px;
    }

    .el-dialog__title {
      font-size: 16px;
      font-weight: 600;
    }
  }

  .el-dialog__body {
    padding: 20px;
    max-height: calc(85vh - 120px);
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;

    @media (max-width: 640px) {
      padding: 12px;
    }
  }

  .el-dialog__footer {
    padding: 14px 20px;
    border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));

    @media (max-width: 640px) {
      padding: 10px 14px;
    }
  }
}
</style>

<style scoped lang="scss">

.cropper-body {
  display: flex;
  flex-direction: column;
}

.upload-dropzone {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 44px 20px;
  border: 2px dashed rgba(0, 143, 253, 0.35);
  border-radius: 14px;
  background-color: rgba(0, 143, 253, 0.02);
  cursor: pointer;
  transition: all 0.2s ease;
  user-select: none;

  &:hover,
  &.is-dragover {
    border-color: var(--c-primary, #008ffd);
    background-color: rgba(0, 143, 253, 0.06);
  }

  .dropzone-icon {
    color: var(--c-primary, #008ffd);
    margin-bottom: 12px;
  }

  .dropzone-title {
    font-size: 15px;
    font-weight: 600;
    color: #222;
    margin-bottom: 6px;
  }

  .dropzone-sub {
    font-size: 12px;
    color: #888;
    margin-bottom: 18px;
    text-align: center;
  }

  .choose-file-btn {
    pointer-events: none;
  }
}

.cropper-workspace {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.workspace-main {
  display: flex;
  gap: 18px;
  align-items: flex-start;

  @media (max-width: 520px) {
    flex-direction: column;
    align-items: center;
  }
}

.viewport-box {
  position: relative;
  width: 360px;
  height: 320px;
  background-color: #0b1120;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  cursor: grab;
  user-select: none;
  touch-action: none;
  flex-shrink: 0;

  @media (max-width: 520px) {
    width: 100%;
    max-width: 100%;
    height: 260px;
  }

  &.dragging {
    cursor: grabbing;
  }

  .cropper-canvas {
    display: block;
    max-width: 100%;
    height: auto;
  }

  .viewport-tip {
    position: absolute;
    bottom: 8px;
    left: 0;
    right: 0;
    text-align: center;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.5);
    pointer-events: none;
  }
}

.preview-sidebar {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 10px 12px;
  background-color: rgba(0, 0, 0, 0.02);
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));
  border-radius: 12px;
  flex: 1;
  min-width: 96px;

  @media (max-width: 520px) {
    flex-direction: row;
    width: 100%;
    justify-content: center;
  }

  .preview-label {
    font-size: 12px;
    font-weight: 600;
    color: #666;
  }

  .preview-circle-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;

    .preview-canvas {
      border-radius: 50%;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
      border: 1px solid var(--c-primary, #008ffd);
      background-color: #fff;
    }

    .size-label {
      font-size: 11px;
      color: #999;
    }
  }
}

.cropper-toolbar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px 14px;
  background-color: rgba(0, 0, 0, 0.02);
  border-radius: 10px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));

  .zoom-control {
    display: flex;
    align-items: center;
    gap: 10px;

    .zoom-slider {
      flex: 1;
      margin: 0 6px;
    }

    .zoom-text {
      font-size: 12px;
      color: #666;
      width: 42px;
      text-align: right;
      font-variant-numeric: tabular-nums;
    }
  }

  .tool-buttons {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: wrap;
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
