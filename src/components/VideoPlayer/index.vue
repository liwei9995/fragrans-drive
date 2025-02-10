<script setup lang="ts" name="video-player">
import { onMounted } from 'vue'
import 'vue3-video-play/dist/style.css'
import { videoPlay } from 'vue3-video-play'
import { Close } from '@element-plus/icons-vue'

interface VideoPlayerProps {
	show?: boolean
	width?: string
	height?: string
	color?: string
	title?: string
	muted?: boolean
	webFullScreen?: boolean
	autoPlay?: boolean
	loop?: boolean
	volume?: number
	control?: boolean
	src?: string
	close?: () => void
}

const props = withDefaults(defineProps<VideoPlayerProps>(), {
	width: '100%',
	height: '100%',
	control: true,
	src: ''
})

onMounted(() => {
	// 监听esc事件
	document.onkeydown = (e: any) => {
		e = window.event || e

		if (e.code === 'Escape' || e.code === 'Esc') {
			props.close && props.close()
		}
	}
})

const handleClose = () => props.close && props.close()
</script>

<template>
	<div class="video-player-wrapper">
		<div class="video-player-mask" />
		<div class="video-player-close-btn" @click="handleClose">
			<el-icon circle size="24">
				<Close />
			</el-icon>
		</div>
		<div class="video-wrapper">
			<div class="inner-wrapper">
				<videoPlay
					:width="width"
					:height="height"
					:color="color"
					:title="title"
					:muted="muted"
					:web-full-screen="webFullScreen"
					:auto-play="autoPlay"
					:loop="loop"
					:volume="volume"
					:control="control"
					:src="src"
				/>
			</div>
		</div>
	</div>
</template>

<style scoped lang="scss">
@use './index';
</style>
