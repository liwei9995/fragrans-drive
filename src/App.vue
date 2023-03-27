<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { GlobalStore } from '@/store'
import { isMobile } from '@/utils/is'

const globalStore = GlobalStore()

globalStore.setIsMobile(isMobile())
</script>

<template>
	<router-view></router-view>
</template>

<style scoped></style>
