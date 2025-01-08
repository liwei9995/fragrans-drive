<script setup lang="ts" name="upload-status">
import { computed, ref } from 'vue'
import { CircleCloseFilled, InfoFilled, SuccessFilled, UploadFilled, WarningFilled } from '@element-plus/icons-vue'

interface UploadStatusProps {
	type?: string
	showClose?: boolean
	title?: string
	percentage?: number
	zIndex?: number
	onClose?: () => void
}

const visible = ref(false)
const TypeComponentsMap = {
	error: CircleCloseFilled,
	info: InfoFilled,
	success: SuccessFilled,
	warning: WarningFilled,
	uploading: UploadFilled
} as Record<string, any>

const props = withDefaults(defineProps<UploadStatusProps>(), {
	type: () => 'uploading',
	title: () => '',
	percentage: () => 0,
	zIndex: () => 0,
	showClose: () => true
})

const icon = computed(() => TypeComponentsMap[props.type])
const uploadPercentage = computed(() => (props.type === 'uploading' ? props.percentage : 0))

const show = () => (visible.value = true)

const close = () => {
	visible.value = false
	props.onClose && props.onClose()
}

defineExpose({
	show,
	close
})
</script>

<template>
	<div v-show="visible" class="upload-status-wrapper">
		<div class="upload-status">
			<el-icon :size="24" :class="type">
				<component :is="icon" />
			</el-icon>
			<h2 class="title" v-text="title" />
			<el-icon v-if="showClose" class="close-btn" @click.stop="close">
				<Close />
			</el-icon>
		</div>
		<el-progress
			v-if="type === 'uploading'"
			class="progress"
			:percentage="uploadPercentage"
			:stroke-width="4"
			:show-text="false"
		/>
	</div>
</template>

<style scoped lang="scss">
@use './index';
</style>
