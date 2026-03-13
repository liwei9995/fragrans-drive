<script setup lang="ts" name="403">
import { useRouter } from 'vue-router'
import { HOME_URL } from '@/config/config'

const router = useRouter()
</script>

<template>
  <div class="not-container">
    <img src="@/assets/403.png" class="not-img" alt="403" />
    <div class="not-detail">
      <h2>403</h2>
      <h4>抱歉，您无权访问该页面~🙅‍♂️🙅‍♀️</h4>
      <el-button type="primary" @click="router.push(HOME_URL)"> 返回首页 </el-button>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use './index';
</style>
