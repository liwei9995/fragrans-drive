<script setup lang="ts" name="form-dialog">
import type { InputInstance } from 'element-plus'
import { nextTick, onMounted, ref, watch } from 'vue'

interface DialogProps {
  title: string
  thumbUrl?: string
  name?: string
  onClose?: () => void
  onConfirm?: (name: string) => void
}

const props = withDefaults(defineProps<DialogProps>(), {
  title: () => '',
  thumbUrl: () =>
    'https://img.alicdn.com/imgextra/i1/O1CN01rGJZac1Zn37NL70IT_!!6000000003238-2-tps-230-180.png',
  name: () => '',
})

const dialogFormVisible = ref(true)
const inputValue = ref(props.name)
const inputRef = ref<InputInstance>()
let hasUserTyped: boolean = false

const handleInput = () => {
  hasUserTyped = true
}

const selectAll = () => {
  const inputEl =
    (inputRef.value?.ref as HTMLInputElement | undefined) ||
    (inputRef.value?.$el?.querySelector?.('input') as HTMLInputElement | null)
  if (inputEl) {
    inputEl.focus()
    inputEl.select()
  } else {
    inputRef.value?.focus?.()
    inputRef.value?.select?.()
  }
}

const handleOpened = () => {
  if (!hasUserTyped) {
    selectAll()
  }
}

onMounted(() => {
  nextTick(() => {
    if (!hasUserTyped) {
      selectAll()
    }
  })
  // 兜底：处理部分浏览器或动画延迟导致的选择未生效
  setTimeout(() => {
    if (!hasUserTyped) {
      selectAll()
    }
  }, 100)
})

watch(
  () => props.name,
  (val) => {
    inputValue.value = val
    hasUserTyped = false
    nextTick(() => selectAll())
  },
)

const handleClick = () => {
  const name = inputValue.value.trim()
  if (name) props.onConfirm?.(name)
}

const handleClose = () => props.onClose?.()

defineExpose({ dialogFormVisible, handleClick, handleClose, selectAll })
</script>

<template>
  <el-dialog
    v-model="dialogFormVisible"
    class="dialog-wrapper"
    width="340px"
    :title="title"
    @opened="handleOpened"
    @close="handleClose"
  >
    <form @submit.prevent="handleClick">
      <el-row justify="center">
        <div class="thumb-wrapper">
          <el-image style="width: 115px; height: 90px" :src="thumbUrl" class="thumb" fit="contain" />
        </div>
      </el-row>
      <el-row justify="center">
        <el-input
          ref="inputRef"
          v-model="inputValue"
          autofocus
          maxlength="30"
          @input="handleInput"
        />
      </el-row>
      <el-row justify="end">
        <div class="dialog-footer">
          <el-button native-type="submit" type="primary" :disabled="!inputValue.trim()"> 确定 </el-button>
        </div>
      </el-row>
    </form>
  </el-dialog>
</template>

<style scoped lang="scss">
@use './index';
</style>
