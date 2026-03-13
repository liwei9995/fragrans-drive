<script setup lang="ts" name="file-skeleton">
// No logic needed, purely visual
</script>

<template>
<div class="skeleton-wrapper">
  <el-skeleton animated>
    <template #template>
      <div class="skeleton-card">
        <el-skeleton-item variant="image" class="skeleton-cover" />
        <div class="skeleton-info">
          <el-skeleton-item variant="text" class="skeleton-title" />
          <el-skeleton-item variant="text" class="skeleton-desc" />
        </div>
      </div>
    </template>
  </el-skeleton>
</div>
</template>

<style scoped lang="scss">
.skeleton-wrapper {
  width: 160px;
  height: 207px;
  display: flex;
  justify-content: center;
  align-items: center;

  .skeleton-card {
    width: 123px;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 8px 3px 10px;

    .skeleton-cover {
      width: 115px;
      height: 90px;
      margin-bottom: 12px;
      border-radius: 12px;
    }

    .skeleton-info {
      width: 100%;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 8px;

      .skeleton-title {
        width: 80%;
        height: 16px;
      }

      .skeleton-desc {
        width: 50%;
        height: 12px;
      }
    }
  }
}

@media (width <= 960px) {
  .skeleton-wrapper {
    width: 140px;
    height: 180px;
  }
}

@media (width <= 576px) {
  .skeleton-wrapper {
    width: 100px;
    height: 180px;
    .skeleton-card {
      width: 100px;
      .skeleton-cover {
        width: 84px;
        height: 84px;
      }
    }
  }
}
</style>
