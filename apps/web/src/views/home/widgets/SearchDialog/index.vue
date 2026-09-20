<script setup lang="ts" name="search-dialog">
import { ArrowRight, Close, Loading, Search } from '@element-plus/icons-vue'
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { StorageNode } from '@/api/interface'
import { getFiles } from '@/api/modules/storage'
import { parseDate } from '@/utils/date'
import { getThumb } from '@/utils/thumb'

const { t } = useI18n()

interface SearchDialogProps {
  visible: boolean
}

const props = defineProps<SearchDialogProps>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'navigate', id: string): void
  (e: 'open-file', item: StorageNode): void
}>()

const searchInputRef = ref<HTMLInputElement | null>(null)
const query = ref('')
const loading = ref(false)
const results = ref<StorageNode[]>([])
const selectedIndex = ref(0)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

const formatBytes = (bytes?: number) => {
  if (typeof bytes !== 'number' || bytes <= 0) return ''
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / k ** i).toFixed(i === 0 ? 0 : 1)} ${sizes[i]}`
}

const formatDate = (dateStr?: string) => {
  const d = parseDate(dateStr)
  if (!d) return ''
  return `${d.getFullYear()}/${String(d.getMonth() + 1).padStart(2, '0')}/${String(d.getDate()).padStart(2, '0')}`
}

const performSearch = async () => {
  const keyword = query.value.trim()
  if (!keyword) {
    results.value = []
    loading.value = false
    return
  }

  loading.value = true
  try {
    const res = await getFiles({
      keyword,
      limit: 30,
      page: 1,
      viewMode: 'all',
    })
    results.value = res.docs || []
    selectedIndex.value = 0
  } catch (error) {
    console.error('Search error:', error)
    results.value = []
  } finally {
    loading.value = false
  }
}

watch(query, () => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    performSearch()
  }, 250)
})

watch(
  () => props.visible,
  (val) => {
    if (val) {
      query.value = ''
      results.value = []
      selectedIndex.value = 0
      nextTick(() => {
        searchInputRef.value?.focus()
      })
    }
  },
)

const handleSelect = (item: StorageNode) => {
  if (item.type === 'folder') {
    emit('navigate', item.id)
  } else {
    emit('navigate', item.parentId)
    emit('open-file', item)
  }
  emit('close')
}

const handleKeydown = (e: KeyboardEvent) => {
  if (!props.visible) return

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (results.value.length > 0) {
      selectedIndex.value = (selectedIndex.value + 1) % results.value.length
      scrollToSelected()
    }
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (results.value.length > 0) {
      selectedIndex.value =
        (selectedIndex.value - 1 + results.value.length) % results.value.length
      scrollToSelected()
    }
  } else if (e.key === 'Enter') {
    e.preventDefault()
    if (results.value[selectedIndex.value]) {
      handleSelect(results.value[selectedIndex.value])
    }
  } else if (e.key === 'Escape') {
    emit('close')
  }
}

const resultsContainerRef = ref<HTMLDivElement | null>(null)
const scrollToSelected = () => {
  nextTick(() => {
    if (!resultsContainerRef.value) return
    const activeEl = resultsContainerRef.value.querySelector(
      '.result-item.active',
    ) as HTMLElement | null
    if (activeEl) {
      activeEl.scrollIntoView({ block: 'nearest' })
    }
  })
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (debounceTimer) clearTimeout(debounceTimer)
})

// Highlight matched keyword in filename safely without v-html (prevents XSS)
interface HighlightSegment {
  text: string
  isMatch: boolean
}

const getHighlightSegments = (
  text: string,
  keyword: string,
): HighlightSegment[] => {
  const trimmed = keyword.trim()
  if (!trimmed) return [{ text, isMatch: false }]
  const escaped = trimmed.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${escaped})`, 'gi')
  const segments: HighlightSegment[] = []
  let lastIndex = 0
  let match = regex.exec(text)
  while (match !== null) {
    if (match.index > lastIndex) {
      segments.push({
        text: text.slice(lastIndex, match.index),
        isMatch: false,
      })
    }
    segments.push({ text: match[0], isMatch: true })
    lastIndex = regex.lastIndex
    match = regex.exec(text)
  }
  if (lastIndex < text.length) {
    segments.push({ text: text.slice(lastIndex), isMatch: false })
  }
  return segments
}
</script>

<template>
  <transition name="search-fade">
    <div v-if="visible" class="search-backdrop" @click.self="emit('close')">
      <div class="search-modal">
        <div class="search-header">
          <el-icon class="search-icon"><Search /></el-icon>
          <input
            ref="searchInputRef"
            v-model="query"
            class="search-input"
            :placeholder="t('search.placeholder')"
            spellcheck="false"
          />
          <el-icon v-if="loading" class="loading-icon is-loading"><Loading /></el-icon>
          <button
            v-else-if="query"
            type="button"
            class="clear-btn"
            :aria-label="t('search.clear')"
            @click="query = ''"
          >
            <el-icon><Close /></el-icon>
          </button>
          <kbd class="shortcut-badge">ESC</kbd>
        </div>

        <div v-if="query.trim()" class="search-body" ref="resultsContainerRef">
          <div v-if="loading && results.length === 0" class="search-state">
            <el-icon class="is-loading" :size="24"><Loading /></el-icon>
            <span>{{ t('search.searching') }}</span>
          </div>

          <div v-else-if="results.length === 0" class="search-state empty">
            <span class="empty-emoji">🔍</span>
            <span class="empty-title">{{ t('search.noResults') }}</span>
            <span class="empty-desc">{{ t('search.noResultsDesc', { query }) }}</span>
          </div>

          <div v-else class="results-list">
            <div class="results-count">
              {{ t('search.resultsCount', { total: results.length }) }}
            </div>
            <div
              v-for="(item, index) in results"
              :key="item.id"
              class="result-item"
              :class="{ active: index === selectedIndex }"
              @mouseenter="selectedIndex = index"
              @click="handleSelect(item)"
            >
              <div class="item-icon-wrapper">
                <img
                  :src="getThumb(item.extName, item.type)"
                  :alt="item.name"
                  class="item-icon"
                />
              </div>
              <div class="item-info">
                <div class="item-name" :title="item.name">
                  <template
                    v-for="(seg, idx) in getHighlightSegments(item.name, query)"
                    :key="idx"
                  >
                    <mark v-if="seg.isMatch" class="highlight">{{ seg.text }}</mark>
                    <span v-else>{{ seg.text }}</span>
                  </template>
                </div>
                <div class="item-meta">
                  <span v-if="item.type === 'folder'" class="meta-tag folder">{{ t('search.folderTag') }}</span>
                  <span v-else class="meta-size">{{ formatBytes(item.size) || '0 B' }}</span>
                  <span class="meta-dot">·</span>
                  <span class="meta-time">{{ formatDate(item.updatedAt) }}</span>
                  <span v-if="item.isPublic" class="meta-tag public">{{ t('search.publicTag') }}</span>
                </div>
              </div>
              <div class="item-action-hint">
                <el-icon><ArrowRight /></el-icon>
              </div>
            </div>
          </div>
        </div>

        <div class="search-footer">
          <div class="footer-left">
            <span v-if="query.trim() && results.length > 0">{{ t('search.resultsCount', { total: results.length }) }}</span>
            <span v-else>{{ t('search.footerHint') }}</span>
          </div>
          <div class="footer-shortcuts">
            <span class="shortcut-item"><kbd>↑</kbd><kbd>↓</kbd> {{ t('search.footerNavHint') }}</span>
            <span class="shortcut-item"><kbd>↵</kbd> {{ t('search.footerEnterHint') }}</span>
            <span class="shortcut-item"><kbd>ESC</kbd> {{ t('common.close') }}</span>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped lang="scss">
.search-backdrop {
  position: fixed;
  inset: 0;
  z-index: 2000;
  background-color: rgba(15, 23, 42, 0.65);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 14vh;

  @media (max-width: 640px) {
    padding-top: 5vh;
    padding-left: 12px;
    padding-right: 12px;
  }
}

.search-modal {
  width: 90%;
  max-width: 640px;
  background-color: #ffffff;
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 16px;
  box-shadow: 0 25px 60px -15px rgba(0, 0, 0, 0.35), 0 0 0 1px rgba(0, 0, 0, 0.04);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  text-align: left;

  @media (max-width: 640px) {
    width: 100%;
    border-radius: 12px;
  }
}

@media (prefers-color-scheme: dark) {
  .search-modal {
    background-color: #1e293b;
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 25px 60px -15px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(255, 255, 255, 0.08);
  }
}

.search-header {
  display: flex;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  gap: 12px;
  text-align: left;

  @media (max-width: 640px) {
    padding: 12px 14px;
    gap: 8px;

    .shortcut-badge {
      display: none;
    }
  }

  .search-icon {
    font-size: 22px;
    color: #008ffd;
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    font-size: 16px;
    font-weight: 500;
    color: #0f172a;
    text-align: left;

    &::placeholder {
      color: #94a3b8;
      font-size: 14.5px;
    }
  }

  .loading-icon {
    font-size: 18px;
    color: #008ffd;
  }

  .clear-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    color: #64748b;
    padding: 4px;
    display: flex;
    align-items: center;
    border-radius: 50%;
    transition: all 0.15s ease;

    &:hover {
      background-color: rgba(0, 0, 0, 0.06);
      color: #0f172a;
    }
  }

  .shortcut-badge {
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 600;
    color: #475569;
    background: #f1f5f9;
    border-radius: 6px;
    border: 1px solid #cbd5e1;
    user-select: none;
  }
}

@media (prefers-color-scheme: dark) {
  .search-header {
    border-bottom-color: rgba(255, 255, 255, 0.08);

    .search-input {
      color: #f8fafc;

      &::placeholder {
        color: #64748b;
      }
    }

    .clear-btn {
      color: #94a3b8;

      &:hover {
        background-color: rgba(255, 255, 255, 0.1);
        color: #f8fafc;
      }
    }

    .shortcut-badge {
      color: #94a3b8;
      background: rgba(255, 255, 255, 0.08);
      border-color: rgba(255, 255, 255, 0.12);
    }
  }
}

.search-body {
  max-height: 420px;
  overflow-y: auto;
  padding: 8px 12px 14px;
  text-align: left;
}

.search-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  gap: 12px;
  color: #64748b;
  font-size: 14px;
  text-align: center;

  &.empty {
    .empty-emoji {
      font-size: 36px;
      margin-bottom: 4px;
    }
    .empty-title {
      font-size: 15px;
      font-weight: 600;
      color: #334155;
    }
    .empty-desc {
      font-size: 13px;
      color: #64748b;
    }
  }
}

@media (prefers-color-scheme: dark) {
  .search-state.empty {
    .empty-title {
      color: #e2e8f0;
    }
    .empty-desc {
      color: #94a3b8;
    }
  }
}

.results-count {
  font-size: 12px;
  color: #64748b;
  padding: 4px 14px 8px;
  font-weight: 500;
  text-align: left;
  user-select: none;
}

@media (prefers-color-scheme: dark) {
  .results-count {
    color: #94a3b8;
  }
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  text-align: left;
}

.result-item {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s ease;
  gap: 14px;
  text-align: left;

  &:hover,
  &.active {
    background-color: rgba(0, 143, 253, 0.08);

    .item-action-hint {
      color: var(--c-primary, #008ffd);
      opacity: 1;
      transform: translateX(2px);
    }
  }

  .item-icon-wrapper {
    width: 36px;
    height: 36px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;

    .item-icon {
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
    }
  }

  .item-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 4px;
    text-align: left;

    .item-name {
      font-size: 14px;
      font-weight: 500;
      color: #0f172a;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      text-align: left;
      line-height: 1.4;

      :deep(.highlight) {
        background-color: #fef08a;
        color: #854d0e;
        font-weight: 600;
        border-radius: 2px;
        padding: 0 2px;
      }
    }

    .item-meta {
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 12px;
      color: #64748b;
      line-height: 1.2;
      text-align: left;

      .meta-tag {
        font-size: 11px;
        padding: 1px 6px;
        border-radius: 4px;
        font-weight: 500;
        line-height: 1.3;

        &.folder {
          background-color: rgba(0, 143, 253, 0.1);
          color: var(--c-primary, #008ffd);
        }

        &.public {
          background-color: #f0fdf4;
          color: #16a34a;
          border: 1px solid #bbf7d0;
        }
      }

      .meta-dot {
        opacity: 0.5;
      }
    }
  }

  .item-action-hint {
    color: #94a3b8;
    font-size: 14px;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    opacity: 0.6;
    transition: all 0.15s ease;
  }
}

@media (prefers-color-scheme: dark) {
  .result-item {
    &:hover,
    &.active {
      background-color: rgba(0, 143, 253, 0.12);

      .item-action-hint {
        color: var(--c-primary-light, #33a5fd);
      }
    }

    .item-info {
      .item-name {
        color: #f8fafc;

        :deep(.highlight) {
          background-color: rgba(254, 240, 138, 0.25);
          color: #fef08a;
        }
      }

      .item-meta {
        color: #94a3b8;

        .meta-tag {
          &.folder {
            background-color: rgba(0, 143, 253, 0.15);
            color: var(--c-primary-light, #33a5fd);
          }

          &.public {
            background-color: rgba(22, 163, 74, 0.15);
            color: #4ade80;
            border-color: rgba(74, 222, 128, 0.3);
          }
        }
      }
    }

    .item-action-hint {
      color: #64748b;
    }
  }
}

.search-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 18px;
  background-color: #f8fafc;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
  font-size: 12px;
  color: #64748b;
  user-select: none;
  text-align: left;

  .footer-left {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .footer-shortcuts {
    display: flex;
    align-items: center;
    gap: 12px;

    .shortcut-item {
      display: flex;
      align-items: center;
      gap: 4px;

      kbd {
        font-family: inherit;
        font-size: 11px;
        font-weight: 600;
        padding: 2px 5px;
        background: #ffffff;
        border: 1px solid #cbd5e1;
        border-radius: 4px;
        color: #475569;
        box-shadow: 0 1px 1px rgba(0, 0, 0, 0.05);
      }
    }
  }

  @media (max-width: 640px) {
    padding: 8px 14px;

    .footer-shortcuts {
      display: none;
    }
  }
}

@media (prefers-color-scheme: dark) {
  .search-footer {
    background-color: #0f172a;
    border-top-color: rgba(255, 255, 255, 0.06);
    color: #94a3b8;

    .footer-shortcuts {
      .shortcut-item {
        kbd {
          background: rgba(255, 255, 255, 0.08);
          border-color: rgba(255, 255, 255, 0.15);
          color: #f1f5f9;
          box-shadow: none;
        }
      }
    }
  }
}

.search-fade-enter-active,
.search-fade-leave-active {
  transition: opacity 0.2s ease;

  .search-modal {
    transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
}

.search-fade-enter-from,
.search-fade-leave-to {
  opacity: 0;

  .search-modal {
    transform: scale(0.96) translateY(-12px);
  }
}
</style>
