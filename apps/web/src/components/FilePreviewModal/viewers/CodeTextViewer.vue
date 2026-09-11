<script setup lang="ts">
import { Check, DocumentCopy, Refresh, Search } from '@element-plus/icons-vue'
import axios from 'axios'
import { ElMessage } from 'element-plus'
import { computed, onMounted, ref, watch } from 'vue'
import { getLanguageFromExt } from '../previewHelper'

interface Props {
  src: string
  name?: string
  extName?: string
  size?: number
}

const props = withDefaults(defineProps<Props>(), {
  src: '',
  name: '',
  extName: '',
  size: 0,
})

const content = ref('')
const loading = ref(true)
const loadError = ref('')
const isWordWrap = ref(false)
const searchQuery = ref('')
const copied = ref(false)

const languageLabel = computed(() =>
  getLanguageFromExt(props.extName, props.name),
)

const lines = computed(() => {
  return content.value.split('\n')
})

const charCount = computed(() => content.value.length)

const isJson = computed(() => {
  return (
    props.extName?.toLowerCase() === 'json' ||
    props.name?.toLowerCase().endsWith('.json') ||
    languageLabel.value === 'JSON'
  )
})

const handleFormatJson = () => {
  try {
    const parsed = JSON.parse(content.value)
    content.value = JSON.stringify(parsed, null, 2)
    ElMessage.success('JSON 已格式化排版')
  } catch {
    ElMessage.warning('当前文本不是有效的 JSON 格式')
  }
}

const fetchContent = async () => {
  if (!props.src) {
    loading.value = false
    loadError.value = '未提供有效文件地址'
    return
  }

  // If file > 5MB, warn
  if (props.size > 5 * 1024 * 1024) {
    loadError.value =
      '该文本文件超过 5MB，直接在线渲染可能会导致浏览器卡顿，建议点击上方下载按钮在本地编辑器中打开。'
    loading.value = false
    return
  }

  loading.value = true
  loadError.value = ''
  try {
    const res = await axios.get<string>(props.src, {
      responseType: 'text',
      transformResponse: [(data) => data],
    })
    content.value =
      typeof res.data === 'string'
        ? res.data
        : JSON.stringify(res.data, null, 2)
  } catch (err: unknown) {
    console.error('Fetch text content failed:', err)
    loadError.value = '加载文本内容失败，请检查网络或重新下载'
  } finally {
    loading.value = false
  }
}

const handleCopy = async () => {
  if (!content.value) return
  try {
    await navigator.clipboard.writeText(content.value)
    copied.value = true
    ElMessage.success('已复制到剪贴板')
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch {
    ElMessage.error('复制失败')
  }
}

const escapeHtml = (str: string): string => {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

// Single-pass tokenizer: matches comments, object keys, strings, numbers, keywords
// Running in a single pass prevents injected HTML tags (e.g. class="tok-string") from being matched as keywords
const TOKEN_REGEX =
  /(\/\/[^\n]*|#[^\n]*|--[^\n]*)|("(?:[^"\\]|\\.)*"(?=\s*:))|("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)|(\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b)|(\b(?:true|false|null|import|export|from|default|const|let|var|function|return|if|else|switch|case|break|continue|for|while|do|class|interface|type|extends|implements|new|this|async|await|try|catch|finally|throw|pub|fn|struct|enum|impl|mut|match|use|mod|trait|where|def|self|None|True|False|package|select|insert|update|delete)\b)/g

// Tokenization for keywords, keys, strings, comments
const highlightLine = (text: string): string => {
  if (!text) return ' '

  // If searching, highlight query safely without breaking HTML entities
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.trim()
    const regex = new RegExp(
      `(${q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`,
      'gi',
    )
    let res = ''
    let last = 0
    while (true) {
      const m = regex.exec(text)
      if (!m) break
      if (m.index > last) {
        res += escapeHtml(text.slice(last, m.index))
      }
      res += `<mark class="code-match">${escapeHtml(m[0])}</mark>`
      last = m.index + m[0].length
    }
    if (last < text.length) {
      res += escapeHtml(text.slice(last))
    }
    return res
  }

  // Single-pass syntax colorizer
  TOKEN_REGEX.lastIndex = 0
  let html = ''
  let lastIndex = 0

  while (true) {
    const match = TOKEN_REGEX.exec(text)
    if (!match) break
    if (match.index > lastIndex) {
      html += escapeHtml(text.slice(lastIndex, match.index))
    }
    const [raw, comment, key, str, num, kw] = match
    if (comment) {
      html += `<span class="tok-comment">${escapeHtml(raw)}</span>`
    } else if (key) {
      html += `<span class="tok-key">${escapeHtml(raw)}</span>`
    } else if (str) {
      html += `<span class="tok-string">${escapeHtml(raw)}</span>`
    } else if (num) {
      html += `<span class="tok-number">${escapeHtml(raw)}</span>`
    } else if (kw) {
      html += `<span class="tok-keyword">${escapeHtml(raw)}</span>`
    } else {
      html += escapeHtml(raw)
    }
    lastIndex = match.index + raw.length
  }

  if (lastIndex < text.length) {
    html += escapeHtml(text.slice(lastIndex))
  }

  return html
}

watch(() => props.src, fetchContent)
onMounted(fetchContent)
</script>

<template>
  <div class="code-viewer">
    <!-- Top toolbar -->
    <div class="code-toolbar">
      <div class="meta-left">
        <span class="lang-tag">{{ languageLabel }}</span>
        <span class="stat-tag">{{ lines.length }} 行</span>
        <span class="stat-tag desktop-only">{{ charCount }} 字符</span>
        <span class="stat-tag desktop-only">UTF-8</span>
      </div>

      <div class="actions-right">
        <!-- Search Input -->
        <div class="search-box">
          <el-input
            v-model="searchQuery"
            size="small"
            placeholder="在文本中搜索..."
            clearable
            :prefix-icon="Search"
          />
        </div>

        <!-- Format JSON Button -->
        <button
          v-if="isJson"
          class="tool-btn"
          title="格式化 JSON (2空格缩进)"
          @click="handleFormatJson"
        >
          <span class="full-text">格式化 JSON</span>
          <span class="short-text">格式化</span>
        </button>

        <!-- Wrap Toggle -->
        <button
          class="tool-btn"
          :class="{ active: isWordWrap }"
          :title="isWordWrap ? '取消自动折行' : '开启自动折行'"
          @click="isWordWrap = !isWordWrap"
        >
          <span class="full-text">{{ isWordWrap ? '折行已开启' : '自动折行' }}</span>
          <span class="short-text">{{ isWordWrap ? '已折行' : '折行' }}</span>
        </button>

        <!-- Copy Button -->
        <button class="tool-btn copy-btn" title="复制全文" @click="handleCopy">
          <el-icon :size="14">
            <Check v-if="copied" />
            <DocumentCopy v-else />
          </el-icon>
          <span class="full-text">{{ copied ? '已复制' : '复制全部' }}</span>
          <span class="short-text">{{ copied ? '已复制' : '复制' }}</span>
        </button>
      </div>
    </div>

    <!-- Main editor body -->
    <div class="code-body">
      <div v-if="loading" class="code-loading">
        <el-icon class="is-loading" :size="32"><Refresh /></el-icon>
        <span>正在读取文件源码...</span>
      </div>

      <div v-else-if="loadError" class="code-error">
        <p>{{ loadError }}</p>
      </div>

      <div
        v-else
        class="code-container"
        :class="{ 'word-wrap': isWordWrap }"
      >
        <div class="gutter">
          <span v-for="(_, index) in lines" :key="index" class="line-num">
            {{ index + 1 }}
          </span>
        </div>
        <div class="code-lines">
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div
            v-for="(line, index) in lines"
            :key="index"
            class="code-row"
            v-html="highlightLine(line)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.code-viewer {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #0f172a;
  color: #e2e8f0;
  overflow: hidden;
  font-family: 'JetBrains Mono', 'Fira Code', Menlo, Monaco, Consolas, monospace;

  .code-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: #1e293b;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);

    @media (max-width: 768px) {
      flex-wrap: wrap;
      gap: 8px;
      padding: 8px 12px;
    }

    .short-text {
      display: none;
    }

    @media (max-width: 640px) {
      .full-text {
        display: none;
      }
      .short-text {
        display: inline;
      }
      .desktop-only {
        display: none;
      }
    }

    .meta-left {
      display: flex;
      align-items: center;
      gap: 10px;
      min-width: 0;

      @media (max-width: 768px) {
        gap: 6px;
      }

      .lang-tag {
        font-size: 12px;
        font-weight: 600;
        color: #38bdf8;
        background: rgba(56, 189, 248, 0.12);
        padding: 2px 8px;
        border-radius: 4px;
        white-space: nowrap;
        flex-shrink: 0;
      }

      .stat-tag {
        font-size: 12px;
        color: #94a3b8;
        white-space: nowrap;
        flex-shrink: 0;
      }
    }

    .actions-right {
      display: flex;
      align-items: center;
      gap: 10px;
      flex-shrink: 0;

      @media (max-width: 768px) {
        width: 100%;
        gap: 6px;
        flex-wrap: wrap;
      }

      .search-box {
        width: 180px;

        @media (max-width: 768px) {
          width: 100%;
          order: 2;
        }
      }

      .tool-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        background: rgba(255, 255, 255, 0.06);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: #cbd5e1;
        font-size: 12px;
        padding: 5px 10px;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s ease;
        white-space: nowrap;
        flex-shrink: 0;

        @media (max-width: 640px) {
          padding: 4px 8px;
          font-size: 11px;
        }

        &:hover {
          background: rgba(255, 255, 255, 0.15);
          color: #f8fafc;
        }

        &.active {
          background: #0284c7;
          border-color: #0284c7;
          color: #fff;
        }

        &.copy-btn:hover {
          border-color: #38bdf8;
          color: #38bdf8;
        }
      }
    }
  }

  .code-body {
    flex: 1;
    position: relative;
    overflow: auto;

    .code-loading,
    .code-error {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      gap: 14px;
      color: #94a3b8;
      font-size: 14px;
      padding: 24px;
      text-align: center;
    }

    .code-error {
      color: #f87171;
    }

    .code-container {
      display: flex;
      min-width: 100%;
      font-size: 13px;
      line-height: 20px;

      .gutter {
        user-select: none;
        padding: 12px 14px;
        text-align: right;
        background: #090d16;
        color: #475569;
        display: flex;
        flex-direction: column;
        border-right: 1px solid rgba(255, 255, 255, 0.06);
        min-width: 48px;

        .line-num {
          height: 20px;
          line-height: 20px;
        }
      }

      .code-lines {
        flex: 1;
        padding: 12px 16px;
        white-space: pre;

        .code-row {
          height: 20px;
          line-height: 20px;
        }
      }

      &.word-wrap {
        .code-lines {
          white-space: pre-wrap;
          word-break: break-all;

          .code-row {
            height: auto;
            min-height: 20px;
          }
        }
      }
    }
  }
}

// Global syntax token styling inside CodeTextViewer
:deep(.tok-comment) {
  color: #64748b;
  font-style: italic;
}
:deep(.tok-key) {
  color: #38bdf8;
  font-weight: 500;
}
:deep(.tok-string) {
  color: #34d399;
}
:deep(.tok-keyword) {
  color: #f472b6;
  font-weight: 500;
}
:deep(.tok-number) {
  color: #fb923c;
}
:deep(.code-match) {
  background: #fbbf24;
  color: #000;
  padding: 1px 3px;
  border-radius: 2px;
}
</style>
