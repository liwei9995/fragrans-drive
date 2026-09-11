<script setup lang="ts">
import { Download, Refresh, TopRight } from '@element-plus/icons-vue'
import { ref } from 'vue'

interface Props {
  src: string
  name?: string
}

const props = withDefaults(defineProps<Props>(), {
  src: '',
  name: '',
})

const emit = defineEmits<(e: 'download') => void>()

const isLoading = ref(true)
const loadError = ref(false)

const handleIframeLoad = () => {
  isLoading.value = false
}

const openInNewTab = () => {
  if (props.src) {
    window.open(props.src, '_blank')
  }
}
</script>

<template>
  <div class="pdf-viewer">
    <!-- Top mini subbar for PDF actions -->
    <div class="pdf-action-bar">
      <span class="pdf-tip">💡 提示：使用现代浏览器内置 PDF 引擎浏览，支持页面缩放、目录书签与打印</span>
      <div class="actions">
        <el-button size="small" :icon="TopRight" @click="openInNewTab">
          独立新标签页打开
        </el-button>
        <el-button size="small" type="primary" :icon="Download" @click="emit('download')">
          下载文件
        </el-button>
      </div>
    </div>

    <div class="pdf-viewport">
      <div v-if="isLoading" class="pdf-loading">
        <el-icon class="is-loading" :size="36"><Refresh /></el-icon>
        <span>正在载入 PDF 文档...</span>
      </div>

      <iframe
        v-show="!loadError"
        :src="src"
        class="pdf-frame"
        title="PDF Preview"
        @load="handleIframeLoad"
      />
    </div>
  </div>
</template>

<style scoped lang="scss">
.pdf-viewer {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #1e293b;
  overflow: hidden;

  .pdf-action-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: rgba(15, 23, 42, 0.9);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);

    .pdf-tip {
      font-size: 12px;
      color: #94a3b8;
    }

    .actions {
      display: flex;
      gap: 8px;
    }
  }

  .pdf-viewport {
    position: relative;
    flex: 1;
    width: 100%;
    height: 100%;

    .pdf-loading {
      position: absolute;
      inset: 0;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 12px;
      color: #94a3b8;
      background: #0f172a;
      z-index: 2;
    }

    .pdf-frame {
      width: 100%;
      height: 100%;
      border: none;
      background: #525659;
    }
  }
}
</style>
