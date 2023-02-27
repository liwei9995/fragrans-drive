<template>
	<el-dialog class="dialog-wrapper" width="340px" :title="title" v-model="dialogFormVisible" @close="handleClose">
		<el-row justify="center">
			<div class="thumb-wrapper">
				<el-image style="width: 115px; height: 90px" :src="thumbUrl" class="thumb" fit="contain" />
			</div>
		</el-row>
		<el-row justify="center">
			<el-input v-model="inputValue" autofocus maxlength="30" />
		</el-row>
		<el-row justify="end">
			<div class="dialog-footer">
				<el-button type="primary" @click="handleClick" :disabled="!inputValue.trim()">确定</el-button>
			</div>
		</el-row>
	</el-dialog>
</template>

<script setup lang="ts" name="header">
import { ref } from 'vue'

interface DialogProps {
	title: string
	thumbUrl?: string
	name?: string
	onClose?: () => void
	onConfirm?: (name: string) => void
}

const props = withDefaults(defineProps<DialogProps>(), {
	title: () => '',
	thumbUrl: () => 'https://img.alicdn.com/imgextra/i1/O1CN01rGJZac1Zn37NL70IT_!!6000000003238-2-tps-230-180.png',
	name: () => ''
})

const dialogFormVisible = ref(true)
const inputValue = ref(props.name)

const handleClick = () => props.onConfirm && props.onConfirm(inputValue.value)

const handleClose = () => props.onClose && props.onClose()
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
