<script setup lang="ts">
import {
  Check,
  DocumentCopy,
  Edit,
  Refresh,
  View,
} from '@element-plus/icons-vue'
import axios from 'axios'
import { ElMessage } from 'element-plus'
import { computed, onMounted, ref, watch } from 'vue'
import CodeTextViewer from './CodeTextViewer.vue'

interface Props {
  src: string
  name?: string
  size?: number
}

const props = withDefaults(defineProps<Props>(), {
  src: '',
  name: '',
  size: 0,
})

const markdownText = ref('')
const loading = ref(true)
const loadError = ref('')
const viewMode = ref<'rendered' | 'source'>('rendered')
const copied = ref(false)

const fetchMarkdown = async () => {
  if (!props.src) {
    loading.value = false
    loadError.value = '未提供有效文件地址'
    return
  }

  if (props.size > 5 * 1024 * 1024) {
    loadError.value =
      '该 Markdown 文档超过 5MB，直接在线渲染可能会导致浏览器卡顿，建议点击上方下载按钮在本地编辑器中打开。'
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
    markdownText.value =
      typeof res.data === 'string' ? res.data : String(res.data)
  } catch (err: unknown) {
    console.error('Fetch markdown failed:', err)
    loadError.value = '加载 Markdown 失败，请检查网络或刷新重试'
  } finally {
    loading.value = false
  }
}

// Lightweight, safe, zero-dependency Markdown to HTML parser
const renderedHtml = computed(() => {
  if (!markdownText.value) return ''

  let text = markdownText.value

  // 1. Escape HTML text and attribute delimiters before adding markup.
  text = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')

  // 2. Fenced Code blocks ```lang\ncode\n```
  text = text.replace(
    /```([a-zA-Z0-9_-]*)\n([\s\S]*?)```/g,
    (_, lang, code) => {
      return `<div class="md-code-block"><div class="code-header"><span class="lang">${lang || 'code'}</span></div><pre class="code-body"><code>${code.trim()}</code></pre></div>`
    },
  )

  // 3. Inline code `code`
  text = text.replace(/`([^`\n]+)`/g, '<code class="md-inline-code">$1</code>')

  // 4. Headings (# H1 to ###### H6)
  text = text.replace(/^######\s+(.*$)/gim, '<h6 class="md-h6">$1</h6>')
  text = text.replace(/^#####\s+(.*$)/gim, '<h5 class="md-h5">$1</h5>')
  text = text.replace(/^####\s+(.*$)/gim, '<h4 class="md-h4">$1</h4>')
  text = text.replace(/^###\s+(.*$)/gim, '<h3 class="md-h3">$1</h3>')
  text = text.replace(/^##\s+(.*$)/gim, '<h2 class="md-h2">$1</h2>')
  text = text.replace(/^#\s+(.*$)/gim, '<h1 class="md-h1">$1</h1>')

  // 5. Horizontal rule
  text = text.replace(/^---$/gim, '<hr class="md-hr" />')

  // 6. Blockquote
  text = text.replace(
    /^&gt;\s+(.*$)/gim,
    '<blockquote class="md-quote">$1</blockquote>',
  )

  // 7. Checkboxes
  text = text.replace(
    /^- \[x\]\s+(.*$)/gim,
    '<div class="md-task-item"><span class="check-box checked">☑</span> $1</div>',
  )
  text = text.replace(
    /^- \[ \]\s+(.*$)/gim,
    '<div class="md-task-item"><span class="check-box">☐</span> $1</div>',
  )

  // 8. Lists (- item or * item)
  text = text.replace(/^\s*[-*]\s+(.*$)/gim, '<li class="md-li">$1</li>')

  // 9. Bold & Italic
  text = text.replace(/\*\*\*(.*?)\*\*\*/g, '<strong><em>$1</em></strong>')
  text = text.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
  text = text.replace(/\*(.*?)\*/g, '<em>$1</em>')
  text = text.replace(/~~(.*?)~~/g, '<del>$1</del>')

  // 10. Links [title](url)
  text = text.replace(
    /\[(.*?)\]\((https?:\/\/[^\s)]+)\)/g,
    '<a href="$2" target="_blank" rel="noopener noreferrer" class="md-link">$1</a>',
  )

  // 11. Tables (| col1 | col2 |\n|---|---|\n| val1 | val2 |)
  text = text.replace(
    /(?:^|\n)(\|.+?\|\r?\n\|[-:\s|]+?\|\r?\n(?:\|.+?\|\r?\n?)+)/g,
    (match) => {
      const lines = match.trim().split(/\r?\n/)
      if (lines.length < 3) return match
      const headerCells = lines[0]
        .split('|')
        .slice(1, -1)
        .map((c) => c.trim())
      const bodyRows = lines.slice(2).map((line) =>
        line
          .split('|')
          .slice(1, -1)
          .map((c) => c.trim()),
      )
      const headerHtml = `<thead><tr>${headerCells.map((c) => `<th>${c}</th>`).join('')}</tr></thead>`
      const bodyHtml = `<tbody>${bodyRows.map((row) => `<tr>${row.map((c) => `<td>${c}</td>`).join('')}</tr>`).join('')}</tbody>`
      return `\n<div class="md-table-wrapper"><table class="md-table">${headerHtml}${bodyHtml}</table></div>\n`
    },
  )

  // 12. Paragraphs (split by double newlines)
  const paragraphs = text.split(/\n\n+/)
  return paragraphs
    .map((p) => {
      const trimmed = p.trim()
      if (!trimmed) return ''
      if (
        trimmed.startsWith('<h') ||
        trimmed.startsWith('<div') ||
        trimmed.startsWith('<blockquote') ||
        trimmed.startsWith('<hr') ||
        trimmed.startsWith('<li') ||
        trimmed.startsWith('<table')
      ) {
        return trimmed
      }
      return `<p class="md-p">${trimmed.replace(/\n/g, '<br/>')}</p>`
    })
    .join('\n')
})

const handleCopy = async () => {
  if (!markdownText.value) return
  try {
    await navigator.clipboard.writeText(markdownText.value)
    copied.value = true
    ElMessage.success('已复制 Markdown 源码')
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch {
    ElMessage.error('复制失败')
  }
}

watch(() => props.src, fetchMarkdown)
onMounted(fetchMarkdown)
</script>

<template>
  <div class="markdown-viewer">
    <!-- Top toolbar -->
    <div class="md-toolbar">
      <div class="meta-left">
        <span class="md-tag">Markdown 文档</span>
        <div class="view-toggle">
          <button
            class="toggle-btn"
            :class="{ active: viewMode === 'rendered' }"
            @click="viewMode = 'rendered'"
          >
            <el-icon :size="14"><View /></el-icon>
            <span class="full-text">渲染视图</span>
            <span class="short-text">渲染</span>
          </button>
          <button
            class="toggle-btn"
            :class="{ active: viewMode === 'source' }"
            @click="viewMode = 'source'"
          >
            <el-icon :size="14"><Edit /></el-icon>
            <span class="full-text">源码视图</span>
            <span class="short-text">源码</span>
          </button>
        </div>
      </div>

      <div class="actions-right">
        <button class="tool-btn" title="复制 Markdown" @click="handleCopy">
          <el-icon :size="14">
            <Check v-if="copied" />
            <DocumentCopy v-else />
          </el-icon>
          <span class="full-text">{{ copied ? '已复制' : '复制源码' }}</span>
          <span class="short-text">{{ copied ? '已复制' : '复制' }}</span>
        </button>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="md-content">
      <div v-if="loading" class="md-loading">
        <el-icon class="is-loading" :size="32"><Refresh /></el-icon>
        <span>正在解析 Markdown...</span>
      </div>

      <div v-else-if="loadError" class="md-error">
        <p>{{ loadError }}</p>
      </div>

      <!-- Rendered View -->
      <div v-else-if="viewMode === 'rendered'" class="rendered-scroll">
        <!-- eslint-disable-next-line vue/no-v-html -->
        <article class="markdown-body" v-html="renderedHtml" />
      </div>

      <!-- Raw Source Code View -->
      <div v-else class="source-view">
        <CodeTextViewer
          :src="src"
          :name="name"
          ext-name="md"
          :size="size"
        />
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.markdown-viewer {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #0f172a;
  overflow: hidden;

  .md-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: #1e293b;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);

    @media (max-width: 640px) {
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
    }

    .meta-left {
      display: flex;
      align-items: center;
      gap: 12px;

      @media (max-width: 640px) {
        gap: 6px;
      }

      .md-tag {
        font-size: 12px;
        font-weight: 600;
        color: #06b6d4;
        background: rgba(6, 182, 212, 0.12);
        padding: 2px 8px;
        border-radius: 4px;
        white-space: nowrap;
        flex-shrink: 0;

        @media (max-width: 500px) {
          display: none;
        }
      }

      .view-toggle {
        display: flex;
        background: rgba(15, 23, 42, 0.6);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 6px;
        padding: 2px;
        white-space: nowrap;
        flex-shrink: 0;

        .toggle-btn {
          display: flex;
          align-items: center;
          gap: 4px;
          background: transparent;
          border: none;
          color: #94a3b8;
          font-size: 12px;
          padding: 4px 10px;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.2s;
          white-space: nowrap;
          flex-shrink: 0;

          @media (max-width: 640px) {
            padding: 4px 8px;
            font-size: 11px;
          }

          &:hover {
            color: #f8fafc;
          }

          &.active {
            background: var(--c-primary, #008ffd);
            color: #fff;
          }
        }
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
      padding: 5px 12px;
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
        color: var(--c-primary-light, #33a5fd);
      }
    }
  }

  .md-content {
    flex: 1;
    position: relative;
    overflow: hidden;

    .md-loading,
    .md-error {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      gap: 14px;
      color: #94a3b8;
      font-size: 14px;
    }
    .md-error {
      color: #f87171;
    }

    .rendered-scroll {
      width: 100%;
      height: 100%;
      overflow-y: auto;
      -webkit-overflow-scrolling: touch;
      padding: 32px 24px;
      display: flex;
      justify-content: center;

      @media (max-width: 640px) {
        padding: 16px 12px;
      }

      .markdown-body {
        width: 100%;
        max-width: 860px;
        color: #e2e8f0;
        font-size: 15px;
        line-height: 1.7;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
      }
    }

    .source-view {
      width: 100%;
      height: 100%;
    }
  }
}

// Markdown Rendered Typography Styles
:deep(.markdown-body) {
  .md-h1 {
    font-size: 28px;
    font-weight: 700;
    color: #f8fafc;
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
    padding-bottom: 8px;
    margin: 28px 0 16px;
  }
  .md-h2 {
    font-size: 22px;
    font-weight: 600;
    color: #f1f5f9;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 6px;
    margin: 24px 0 14px;
  }
  .md-h3 {
    font-size: 18px;
    font-weight: 600;
    color: #e2e8f0;
    margin: 20px 0 12px;
  }
  .md-h4, .md-h5, .md-h6 {
    font-size: 15px;
    font-weight: 600;
    color: #cbd5e1;
    margin: 16px 0 10px;
  }
  .md-p {
    margin: 0 0 14px;
  }
  .md-hr {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.12);
    margin: 24px 0;
  }
  .md-quote {
    border-left: 4px solid var(--c-primary, #008ffd);
    background: rgba(0, 143, 253, 0.08);
    padding: 8px 16px;
    margin: 16px 0;
    color: #cbd5e1;
    border-radius: 0 4px 4px 0;
  }
  .md-li {
    margin-left: 20px;
    margin-bottom: 6px;
  }
  .md-task-item {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 6px 0;
    .check-box {
      font-size: 16px;
      color: #94a3b8;
      &.checked {
        color: #10b981;
      }
    }
  }
  .md-link {
    color: var(--c-primary-light, #33a5fd);
    text-decoration: underline;
    text-underline-offset: 2px;
    &:hover {
      color: #66bdfe;
    }
  }
  .md-inline-code {
    background: rgba(255, 255, 255, 0.1);
    color: #f472b6;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 13px;
    font-family: monospace;
  }
  .md-code-block {
    margin: 16px 0;
    border-radius: 8px;
    background: #090d16;
    border: 1px solid rgba(255, 255, 255, 0.08);
    overflow: hidden;

    .code-header {
      padding: 6px 14px;
      background: rgba(255, 255, 255, 0.04);
      border-bottom: 1px solid rgba(255, 255, 255, 0.06);
      font-size: 11px;
      color: #94a3b8;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    .code-body {
      margin: 0;
      padding: 14px 16px;
      overflow-x: auto;
      font-size: 13px;
      line-height: 1.6;
      color: #e2e8f0;
      font-family: monospace;
    }
  }

  .md-table-wrapper {
    margin: 18px 0;
    overflow-x: auto;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);

    .md-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 14px;

      thead {
        background: rgba(255, 255, 255, 0.06);

        th {
          padding: 10px 14px;
          text-align: left;
          font-weight: 600;
          color: #f8fafc;
          border-bottom: 1px solid rgba(255, 255, 255, 0.12);
          border-right: 1px solid rgba(255, 255, 255, 0.06);
        }
      }

      tbody {
        tr {
          &:nth-child(even) {
            background: rgba(255, 255, 255, 0.02);
          }

          &:hover {
            background: rgba(0, 143, 253, 0.08);
          }

          td {
            padding: 8px 14px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
            border-right: 1px solid rgba(255, 255, 255, 0.05);
          }
        }
      }
    }
  }
}

@media (prefers-color-scheme: light) {
  .markdown-viewer {
    background: #ffffff;

    .md-toolbar {
      background: #f8fafc;
      border-bottom: 1px solid rgba(0, 0, 0, 0.08);

      .meta-left {
        .md-tag {
          background: rgba(6, 182, 212, 0.1);
          color: #0891b2;
        }

        .view-toggle {
          background: rgba(0, 0, 0, 0.04);
          border: 1px solid rgba(0, 0, 0, 0.08);

          .toggle-btn {
            color: #64748b;

            &:hover {
              color: #0f172a;
              background: rgba(0, 0, 0, 0.04);
            }

            &.active {
              background: var(--c-primary, #008ffd);
              color: #ffffff;
              box-shadow: 0 1px 3px rgba(0, 143, 253, 0.25);
            }
          }
        }
      }

      .actions-right {
        .tool-btn {
          background: #ffffff;
          border: 1px solid rgba(0, 0, 0, 0.12);
          color: #475569;

          &:hover {
            background: rgba(0, 143, 253, 0.06);
            border-color: rgba(0, 143, 253, 0.3);
            color: var(--c-primary, #008ffd);
          }
        }
      }
    }

    .md-content {
      background: #ffffff;

      .rendered-scroll {
        background: #ffffff;

        .markdown-body {
          color: #334155;
        }
      }
    }
  }

  :deep(.markdown-body) {
    .md-h1,
    .md-h2,
    .md-h3,
    .md-h4,
    .md-h5,
    .md-h6 {
      color: #0f172a;
      border-bottom-color: rgba(0, 0, 0, 0.08);
    }

    .md-hr {
      border-top-color: rgba(0, 0, 0, 0.08);
    }

    .md-quote {
      border-left: 4px solid var(--c-primary, #008ffd);
      background: rgba(0, 143, 253, 0.06);
      color: #475569;
    }

    .md-inline-code {
      background: rgba(0, 0, 0, 0.06);
      color: #be185d;
    }

    .md-code-block {
      background: #f8fafc;
      border: 1px solid rgba(0, 0, 0, 0.08);

      .code-header {
        background: rgba(0, 0, 0, 0.03);
        border-bottom: 1px solid rgba(0, 0, 0, 0.06);
        color: #64748b;
      }

      .code-body {
        color: #1e293b;
      }
    }

    .md-table-wrapper {
      border: 1px solid rgba(0, 0, 0, 0.08);

      .md-table {
        thead {
          background: #f1f5f9;

          th {
            color: #0f172a;
            border-bottom: 1px solid rgba(0, 0, 0, 0.08);
            border-right: 1px solid rgba(0, 0, 0, 0.06);
          }
        }

        tbody {
          tr {
            &:nth-child(even) {
              background: #f8fafc;
            }

            &:hover {
              background: rgba(0, 143, 253, 0.06);
            }

            td {
              color: #334155;
              border-bottom: 1px solid rgba(0, 0, 0, 0.05);
              border-right: 1px solid rgba(0, 0, 0, 0.05);
            }
          }
        }
      }
    }
  }
}
</style>
