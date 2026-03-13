<script setup lang="ts" name="footer">
import logo from '@/assets/logo.svg'

const currentYear = new Date().getFullYear()
</script>

<template>
  <footer class="footer">
    <div class="footer-container">
      <div class="footer-content">
        <el-image :src="logo" style="width: 14px; height: 14px" />
        <span class="copyright">
          © {{ currentYear }} Fragrans Drive. All rights reserved.
        </span>
      </div>
    </div>
  </footer>
</template>

<style scoped lang="scss">
@use './index';
</style>
