<script setup lang="ts" name="global-dropzone">
import { UploadFilled } from '@element-plus/icons-vue'
import type { UploadProps } from 'element-plus'
import Upload from '../Upload/index.vue'

interface GlobalDropzoneProps {
  show: boolean
  onUploadChange?: UploadProps['onChange']
  onUploadExceed?: UploadProps['onExceed']
  onUploadProgress?: UploadProps['onProgress']
  beforeUpload?: UploadProps['beforeUpload']
}

defineProps<GlobalDropzoneProps>()
</script>

<template>
  <transition name="fade">
    <div v-if="show" class="global-dropzone">
      <Upload
        drag
        multiple
        :show-file-list="false"
        :on-upload-change="onUploadChange"
        :on-upload-exceed="onUploadExceed"
        :on-upload-progress="onUploadProgress"
        :before-upload="beforeUpload"
      >
        <template #trigger>
          <div class="drop-content">
            <el-icon class="drop-icon"><UploadFilled /></el-icon>
            <div class="drop-text">
              <h3>松开鼠标即可上传</h3>
              <p>快到碗里来</p>
            </div>
          </div>
        </template>
      </Upload>
    </div>
  </transition>
</template>

<style scoped lang="scss">
.global-dropzone {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-glass);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 4px dashed var(--c-primary);
  margin: 10px;
  border-radius: 24px;
  pointer-events: none;

  :deep(.upload-zone) {
    width: 100%;
    height: 100%;
    pointer-events: auto;

    .el-upload {
      width: 100%;
      height: 100%;
    }

    .el-upload-dragger {
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      background-color: transparent;
      border: none;
    }
  }

  .drop-content {
    color: var(--c-primary);
    text-align: center;

    .drop-icon {
      font-size: 80px;
      margin-bottom: 20px;
      animation: float 2s ease-in-out infinite;
    }

    .drop-text {
      h3 {
        font-size: 28px;
        margin-bottom: 8px;
        font-family: 'Outfit', sans-serif;
      }
      p {
        font-size: 16px;
        opacity: 0.7;
      }
    }
  }
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-15px); }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
