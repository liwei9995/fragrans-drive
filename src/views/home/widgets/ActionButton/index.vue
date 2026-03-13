<script setup lang="ts" name="action-button">
import type { UploadInstance, UploadProps } from 'element-plus'
import { ref } from 'vue'
import Upload from '../Upload/index.vue'

const uploadRef = ref<UploadInstance>()

type ActionItem = {
  id?: string
  name: string
  isUpload?: boolean
}

interface ActionButtonProps {
  actionItems?: Partial<ActionItem>[]
  uploadFileLimit?: number
  iconSize?: number
  tapActionItem?: (command: string | number | object) => void
  onUploadChange?: UploadProps['onChange']
  onUploadExceed?: UploadProps['onExceed']
  onUploadProgress?: UploadProps['onProgress']
  onUploadSuccess?: UploadProps['onSuccess']
  onUploadError?: UploadProps['onError']
  beforeUpload?: UploadProps['beforeUpload']
}

const props = withDefaults(defineProps<ActionButtonProps>(), {
  uploadFileLimit: () => 10,
  actionItems: () => [],
  iconSize: () => 32,
})

const handleCommand = (command: string | number | object) =>
  props.tapActionItem && props.tapActionItem(command)
</script>

<template>
  <div class="action-button-wrapper">
    <el-dropdown trigger="click" @command="handleCommand">
      <div class="action">
        <el-icon :size="iconSize">
          <CirclePlusFilled />
        </el-icon>
      </div>
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item v-for="item in actionItems" :key="item.id" :command="item.id">
            <template v-if="item.isUpload">
              <Upload
                ref="uploadRef"
                class="upload-zone"
                multiple
                :show-file-list="false"
                :limit="uploadFileLimit"
                :on-upload-change="onUploadChange"
                :on-upload-exceed="onUploadExceed"
                :on-upload-progress="onUploadProgress"
                :on-upload-success="onUploadSuccess"
                :on-upload-error="onUploadError"
                :before-upload="beforeUpload"
              >
                <template #trigger>{{ item.name }}</template>
              </Upload>
            </template>
            <template v-else>{{ item.name }}</template>
          </el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>
  </div>
</template>

<style scoped lang="scss">
@use './index';
</style>
