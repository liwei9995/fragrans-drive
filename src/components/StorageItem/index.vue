<script setup lang="ts" name="storage-item">
interface StorageItemProps {
  id: string
  disabled?: boolean
  thumbUrl?: string
  thumbPlaceholder?: string
  name: string
  tap?: (id: string) => void
}

const props = withDefaults(defineProps<StorageItemProps>(), {
  disabled: true,
  thumbUrl: '',
})

const isHover = ref(false)

const handleClick = () => {
  if (props.disabled) return

  props.tap?.(props.id)
}

defineExpose({
  isHover,
  handleClick,
})
</script>

<template>
  <div class="storage-item-wrapper" :class="{ disabled: disabled }" @click="handleClick">
    <el-image class="icon" :src="thumbUrl" fit="contain">
      <template #error>
        <div class="icon-placeholder" :style="{ backgroundImage: 'url(' + thumbPlaceholder + ')' }" />
      </template>
    </el-image>
    <span>{{ name }}</span>
  </div>
</template>

<style scoped lang="scss">
@use './index';
</style>
