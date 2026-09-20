<script setup lang="ts">
import { ElConfigProvider } from 'element-plus'
import en from 'element-plus/es/locale/lang/en'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import { computed } from 'vue'
import { GlobalStore } from '@/store'

const globalStore = GlobalStore()
const elLocale = computed(() => (globalStore.language === 'en' ? en : zhCn))
</script>

<template>
  <el-config-provider :locale="elLocale">
    <router-view />
  </el-config-provider>
</template>
