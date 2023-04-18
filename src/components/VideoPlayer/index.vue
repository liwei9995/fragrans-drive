<template>
	<div class="video-player-wrapper">
		<videoPlay
			:width="width"
			:height="height"
			:color="color"
			:title="title"
			:muted="muted"
			:webFullScreen="webFullScreen"
			:autoPlay="autoPlay"
			:loop="loop"
			:volume="volume"
			:control="control"
			:src="src"
		/>
	</div>
</template>

<script setup lang="ts" name="video-player">
import 'vue3-video-play/dist/style.css'
import { videoPlay } from 'vue3-video-play'

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
}

withDefaults(defineProps<VideoPlayerProps>(), {
	width: '800px',
	height: '450px',
	src: 'https://api.oyiyio.com/drive/v1/storage/64356e688c080f832f828421?token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiI2MmUxNWVmZmRiNWVlOTJjZDNjNTJjNWQiLCJpYXQiOjE2ODE4MzUwMDAsImV4cCI6MTY4MTgzNTkwMH0.o-P42heNlStaJqmyfQoK8hO8BAKkkf0ppqCQnW8FcH'
})
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
