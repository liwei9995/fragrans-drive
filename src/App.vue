<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { isMobile } from 'react-device-detect'
import { GlobalStore } from '@/store'

const globalStore = GlobalStore()

globalStore.setIsMobile(isMobile)
</script>

<template>
	<router-view></router-view>
</template>

<style scoped></style>
