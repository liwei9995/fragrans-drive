<script setup lang="ts" name="empty">
import { UploadFilled } from '@element-plus/icons-vue'
import type { UploadProps } from 'element-plus'
import EmptyItem from './Item/index.vue'

interface EmptyProps {
  limit?: number
  onUploadChange?: UploadProps['onChange']
  onUploadExceed?: UploadProps['onExceed']
  onUploadProgress?: UploadProps['onProgress']
  onUploadSuccess?: UploadProps['onSuccess']
  onUploadError?: UploadProps['onError']
  beforeUpload?: UploadProps['beforeUpload']
  tapItem?: (command: string | number | object) => void
}

const props = withDefaults(defineProps<EmptyProps>(), {
  limit: () => 10,
})

const items = [
  {
    id: 'folder',
    description: '新建文件夹',
    icon: 'https://img.alicdn.com/imgextra/i2/O1CN018yXBXY1caApf7qUew_!!6000000003616-2-tps-224-224.png',
  },
  {
    id: 'upload',
    description: '上传文件',
    icon: 'https://img.alicdn.com/imgextra/i4/O1CN01Ojh9qS1rrJtSy0dN4_!!6000000005684-2-tps-224-224.png',
    isUpload: true,
  },
]

const handleTapItem = (id: string) => props.tapItem && props.tapItem(id)
</script>

<template>
  <div class="empty-placeholder">
    <div class="drag-hint">
      <el-icon><UploadFilled /></el-icon>
      <span>拖拽文件到此处上传</span>
    </div>
    <div class="items">
      <EmptyItem
        v-for="item in items"
        :id="item.id"
        :key="item.id"
        :description="item.description"
        :icon="item.icon"
        :limit="limit"
        :is-upload="item.isUpload"
        :tap-item="handleTapItem"
        :on-upload-change="onUploadChange"
        :on-upload-exceed="onUploadExceed"
        :on-upload-progress="onUploadProgress"
        :on-upload-success="onUploadSuccess"
        :on-upload-error="onUploadError"
        :before-upload="beforeUpload"
      />
    </div>
  </div>
</template>

<style scoped lang="scss">
@use './index';
</style>
