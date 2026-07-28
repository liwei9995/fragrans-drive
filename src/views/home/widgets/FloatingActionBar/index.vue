<script setup lang="ts" name="floating-action-bar">
import { Close, Delete } from '@element-plus/icons-vue'

interface FloatingActionBarProps {
  selectedCount: number
}

defineProps<FloatingActionBarProps>()

const emit = defineEmits(['delete', 'clear'])
</script>

<template>
  <transition name="slide-up">
    <div v-if="selectedCount > 0" class="floating-action-bar-wrapper">
      <div class="floating-action-bar">
        <div class="selection-info">
          <span class="count">{{ selectedCount }}</span>
          <span class="text">项已选择</span>
        </div>
        <div class="divider"></div>
        <div class="actions">
          <button
            type="button"
            class="action-item delete"
            @click="emit('delete')"
          >
            <el-icon><Delete /></el-icon>
            <span>删除</span>
          </button>
        </div>
        <div class="divider"></div>
        <button
          type="button"
          class="close-btn"
          aria-label="清除选择"
          @click="emit('clear')"
        >
          <el-icon><Close /></el-icon>
        </button>
      </div>
    </div>
  </transition>
</template>

<style scoped lang="scss">
.floating-action-bar-wrapper {
  position: fixed;
  bottom: 30px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 1001;
  display: flex;
  justify-content: center;
  width: auto;
}

.floating-action-bar {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  background-color: var(--bg-glass);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid var(--border-color);
  border-radius: 20px;
  box-shadow: 0 10px 40px -10px rgba(0, 0, 0, 0.4);

  .selection-info {
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-family: 'Outfit', sans-serif;

    .count {
      font-size: 18px;
      font-weight: 700;
      color: var(--c-primary);
      margin-right: 4px;
    }
    .text {
      font-size: 14px;
      color: var(--text-color);
      opacity: 0.8;
    }
  }

  .divider {
    width: 1px;
    height: 24px;
    background-color: var(--border-color);
    margin: 0 8px;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 4px;

    .action-item {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 8px 16px;
      border-radius: 12px;
      color: var(--text-color);
      cursor: pointer;
      background: transparent;
      border: 0;
      transition: all 0.2s ease;
      font-size: 14px;
      font-weight: 500;

      &:hover {
        background-color: var(--border-color);
        color: var(--c-primary);
      }

      &.delete:hover {
        color: #f87171;
        background-color: rgba(248, 113, 113, 0.1);
      }
    }
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    cursor: pointer;
    color: var(--text-color);
    background: transparent;
    border: 0;
    opacity: 0.6;
    transition: all 0.2s ease;

    &:hover {
      opacity: 1;
      background-color: var(--border-color);
    }
  }
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translate(-50%, 40px) scale(0.9);
}
</style>
