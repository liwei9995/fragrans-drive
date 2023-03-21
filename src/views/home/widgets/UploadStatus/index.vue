<template>
	<div v-show="visible" class="upload-status-wrapper">
		<div class="upload-status">
			<el-icon :size="24" class="upload-icon">
				<UploadFilled />
			</el-icon>
			<h2 class="title" v-text="title" />
			<el-icon v-if="showClose" class="close-btn" @click.stop="close">
				<Close />
			</el-icon>
		</div>
		<el-progress class="progress" :percentage="percentage" :stroke-width="4" :show-text="false" />
	</div>
</template>

<script setup lang="ts" name="upload-status">
import { ref, onMounted } from 'vue'

interface UploadStatusProps {
	showClose?: boolean
	title?: string
	percentage?: number
	zIndex?: number
}

const visible = ref(false)

withDefaults(defineProps<UploadStatusProps>(), {
	title: () => '正在上传 ∙ 剩余5项',
	percentage: () => 50,
	zIndex: () => 0,
	showClose: () => true
})

const close = () => (visible.value = false)

onMounted(() => {
	visible.value = true
})

defineExpose({
	visible,
	close
})
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
