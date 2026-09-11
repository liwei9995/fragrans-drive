<script setup lang="ts">
import {
  ArrowLeft,
  ArrowRight,
  Check,
  Document,
  DocumentCopy,
  Grid,
  Refresh,
  Search,
} from '@element-plus/icons-vue'
import axios from 'axios'
import { ElMessage } from 'element-plus'
import { computed, onMounted, ref, watch } from 'vue'
import CodeTextViewer from './CodeTextViewer.vue'

interface Props {
  src: string
  name?: string
  extName?: string
  size?: number
}

const props = withDefaults(defineProps<Props>(), {
  src: '',
  name: '',
  extName: 'csv',
  size: 0,
})

const rawContent = ref('')
const loading = ref(true)
const loadError = ref('')
const viewMode = ref<'table' | 'raw'>('table')
const searchQuery = ref('')
const copied = ref(false)

const pageSize = ref(100)
const currentPage = ref(1)

const isTsv = computed(() => {
  return (
    props.extName?.toLowerCase() === 'tsv' ||
    props.name?.toLowerCase().endsWith('.tsv')
  )
})

// Parse CSV or TSV safely handling quotes and delimiters
const parseCsv = (text: string, delimiter = ','): string[][] => {
  const rows: string[][] = []
  let currentRow: string[] = []
  let currentCell = ''
  let inQuotes = false

  for (let i = 0; i < text.length; i++) {
    const char = text[i]
    const nextChar = text[i + 1]

    if (char === '"') {
      if (inQuotes && nextChar === '"') {
        currentCell += '"'
        i++ // skip next quote
      } else {
        inQuotes = !inQuotes
      }
    } else if (char === delimiter && !inQuotes) {
      currentRow.push(currentCell.trim())
      currentCell = ''
    } else if ((char === '\r' || char === '\n') && !inQuotes) {
      if (char === '\r' && nextChar === '\n') {
        i++
      }
      currentRow.push(currentCell.trim())
      currentCell = ''
      if (currentRow.some((c) => c !== '')) {
        rows.push(currentRow)
      }
      currentRow = []
    } else {
      currentCell += char
    }
  }

  if (currentCell || currentRow.length > 0) {
    currentRow.push(currentCell.trim())
    if (currentRow.some((c) => c !== '')) {
      rows.push(currentRow)
    }
  }

  return rows
}

const parsedData = computed(() => {
  if (!rawContent.value) return { headers: [], rows: [] }
  const delimiter = isTsv.value ? '\t' : ','
  const allRows = parseCsv(rawContent.value, delimiter)
  if (allRows.length === 0) return { headers: [], rows: [] }

  const headers = allRows[0]
  const rows = allRows.slice(1)
  return { headers, rows }
})

const filteredRows = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return parsedData.value.rows

  return parsedData.value.rows.filter((row) =>
    row.some((cell) => cell.toLowerCase().includes(query)),
  )
})

const totalPages = computed(() => {
  return Math.max(1, Math.ceil(filteredRows.value.length / pageSize.value))
})

const paginatedRows = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filteredRows.value.slice(start, start + pageSize.value)
})

watch(searchQuery, () => {
  currentPage.value = 1
})

const fetchContent = async () => {
  if (!props.src) {
    loading.value = false
    loadError.value = '未提供有效文件地址'
    return
  }

  if (props.size > 10 * 1024 * 1024) {
    loadError.value =
      '该表格文件超过 10MB，在线直接解析可能导致卡顿，建议点击上方下载按钮在 Excel 或 WPS 中查看。'
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
    rawContent.value =
      typeof res.data === 'string' ? res.data : String(res.data)
  } catch (err: unknown) {
    console.error('Fetch csv content failed:', err)
    loadError.value = '加载表格数据失败，请检查网络或重新下载'
  } finally {
    loading.value = false
  }
}

const handleCopy = async () => {
  if (!rawContent.value) return
  try {
    await navigator.clipboard.writeText(rawContent.value)
    copied.value = true
    ElMessage.success('已复制表格源码数据')
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch {
    ElMessage.error('复制失败')
  }
}

watch(() => props.src, fetchContent)
onMounted(fetchContent)
</script>

<template>
  <div class="csv-viewer">
    <!-- Top subbar -->
    <div class="csv-toolbar">
      <div class="meta-left">
        <span class="type-tag">{{ isTsv ? 'TSV 数据表' : 'CSV 数据表' }}</span>
        <span v-if="parsedData.headers.length" class="stat-tag">
          共 {{ parsedData.rows.length }} 行 · {{ parsedData.headers.length }} 列
        </span>

        <!-- View Mode Toggle -->
        <div class="view-toggle">
          <button
            class="toggle-btn"
            :class="{ active: viewMode === 'table' }"
            @click="viewMode = 'table'"
          >
            <el-icon :size="14"><Grid /></el-icon>
            <span class="btn-text full-text">表格视图</span>
            <span class="btn-text short-text">表格</span>
          </button>
          <button
            class="toggle-btn"
            :class="{ active: viewMode === 'raw' }"
            @click="viewMode = 'raw'"
          >
            <el-icon :size="14"><Document /></el-icon>
            <span class="btn-text full-text">纯文本视图</span>
            <span class="btn-text short-text">文本</span>
          </button>
        </div>
      </div>

      <div class="actions-right">
        <!-- Search filter (table mode) -->
        <div v-if="viewMode === 'table'" class="search-box">
          <el-input
            v-model="searchQuery"
            size="small"
            placeholder="搜索表格内容..."
            clearable
            :prefix-icon="Search"
          />
        </div>

        <!-- Copy button -->
        <button class="tool-btn" title="复制文本内容" @click="handleCopy">
          <el-icon :size="14">
            <Check v-if="copied" />
            <DocumentCopy v-else />
          </el-icon>
          <span class="copy-text full-text">{{ copied ? '已复制' : '复制数据' }}</span>
          <span class="copy-text short-text">{{ copied ? '已复制' : '复制' }}</span>
        </button>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="csv-main">
      <div v-if="loading" class="csv-loading">
        <el-icon class="is-loading" :size="32"><Refresh /></el-icon>
        <span>正在解析表格数据...</span>
      </div>

      <div v-else-if="loadError" class="csv-error">
        <p>{{ loadError }}</p>
      </div>

      <!-- Raw text view -->
      <div v-else-if="viewMode === 'raw'" class="raw-view">
        <CodeTextViewer
          :src="src"
          :name="name"
          :ext-name="extName"
          :size="size"
        />
      </div>

      <!-- Interactive Table view -->
      <div v-else class="table-container">
        <div class="table-scroll">
          <table class="data-table">
            <thead>
              <tr>
                <th class="col-index">#</th>
                <th
                  v-for="(header, idx) in parsedData.headers"
                  :key="idx"
                  class="col-header"
                >
                  {{ header || `列 ${idx + 1}` }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(row, rIdx) in paginatedRows"
                :key="rIdx"
                class="data-row"
              >
                <td class="cell-index">
                  {{ (currentPage - 1) * pageSize + rIdx + 1 }}
                </td>
                <td
                  v-for="(cell, cIdx) in row"
                  :key="cIdx"
                  class="data-cell"
                  :title="cell"
                >
                  {{ cell }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Pagination Footer -->
        <div v-if="totalPages > 1" class="table-footer">
          <span class="pagination-info">
            第 {{ currentPage }} / {{ totalPages }} 页 (共 {{ filteredRows.length }} 条)
          </span>
          <div class="pagination-btns">
            <button
              class="page-btn"
              :disabled="currentPage <= 1"
              @click="currentPage--"
            >
              <el-icon><ArrowLeft /></el-icon>
              <span>上一页</span>
            </button>
            <button
              class="page-btn"
              :disabled="currentPage >= totalPages"
              @click="currentPage++"
            >
              <span>下一页</span>
              <el-icon><ArrowRight /></el-icon>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.csv-viewer {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #0f172a;
  overflow: hidden;

  .csv-toolbar {
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
    }

    .meta-left {
      display: flex;
      align-items: center;
      gap: 12px;
      min-width: 0;

      @media (max-width: 768px) {
        gap: 8px;
        flex: 1;
        justify-content: space-between;
      }

      .type-tag {
        font-size: 12px;
        font-weight: 600;
        color: #10b981;
        background: rgba(16, 185, 129, 0.12);
        padding: 2px 8px;
        border-radius: 4px;
        white-space: nowrap;
        flex-shrink: 0;

        @media (max-width: 640px) {
          display: none;
        }
      }

      .stat-tag {
        font-size: 12px;
        color: #94a3b8;
        white-space: nowrap;
        flex-shrink: 0;

        @media (max-width: 480px) {
          font-size: 11px;
        }
      }

      .view-toggle {
        display: flex;
        background: rgba(15, 23, 42, 0.6);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 6px;
        padding: 2px;
        flex-shrink: 0;
        white-space: nowrap;

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
            padding: 3px 8px;
            font-size: 11px;
          }

          &:hover {
            color: #f8fafc;
          }

          &.active {
            background: #059669;
            color: #fff;
          }
        }
      }
    }

    .actions-right {
      display: flex;
      align-items: center;
      gap: 10px;
      flex-shrink: 0;

      @media (max-width: 768px) {
        width: 100%;
        gap: 8px;
      }

      .search-box {
        width: 180px;

        @media (max-width: 768px) {
          flex: 1;
          width: auto;
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
  }

  .csv-main {
    flex: 1;
    position: relative;
    overflow: hidden;

    .csv-loading,
    .csv-error {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      gap: 14px;
      color: #94a3b8;
      font-size: 14px;
    }
    .csv-error {
      color: #f87171;
    }

    .raw-view {
      width: 100%;
      height: 100%;
    }

    .table-container {
      width: 100%;
      height: 100%;
      display: flex;
      flex-direction: column;

      .table-scroll {
        flex: 1;
        overflow: auto;
        position: relative;
        -webkit-overflow-scrolling: touch;

        .data-table {
          width: 100%;
          border-collapse: collapse;
          font-size: 13px;
          color: #e2e8f0;

          @media (max-width: 768px) {
            font-size: 12px;
          }

          thead {
            position: sticky;
            top: 0;
            z-index: 2;
            background: #1e293b;
            box-shadow: 0 1px 0 rgba(255, 255, 255, 0.1);

            th {
              padding: 10px 14px;
              font-weight: 600;
              color: #f8fafc;
              text-align: left;
              white-space: nowrap;
              border-right: 1px solid rgba(255, 255, 255, 0.06);
              border-bottom: 1px solid rgba(255, 255, 255, 0.1);

              @media (max-width: 768px) {
                padding: 8px 10px;
              }

              &.col-index {
                width: 50px;
                text-align: center;
                background: #172554;
                color: #93c5fd;
              }
            }
          }

          tbody {
            .data-row {
              transition: background-color 0.15s;

              &:nth-child(even) {
                background: rgba(255, 255, 255, 0.02);
              }

              &:hover {
                background: rgba(56, 189, 248, 0.08);
              }

              .cell-index {
                text-align: center;
                color: #64748b;
                font-family: monospace;
                font-size: 12px;
                background: rgba(15, 23, 42, 0.5);
                border-right: 1px solid rgba(255, 255, 255, 0.06);
                border-bottom: 1px solid rgba(255, 255, 255, 0.04);
                user-select: none;
              }

              .data-cell {
                padding: 8px 14px;
                border-right: 1px solid rgba(255, 255, 255, 0.04);
                border-bottom: 1px solid rgba(255, 255, 255, 0.04);
                white-space: nowrap;
                max-width: 360px;
                overflow: hidden;
                text-overflow: ellipsis;

                @media (max-width: 768px) {
                  padding: 6px 10px;
                  max-width: 220px;
                }
              }
            }
          }
        }
      }

      .table-footer {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 8px 16px;
        background: #1e293b;
        border-top: 1px solid rgba(255, 255, 255, 0.08);

        @media (max-width: 640px) {
          padding: 6px 10px;

          .pagination-info {
            font-size: 11px;
          }

          .pagination-btns .page-btn {
            padding: 3px 8px;
            font-size: 11px;
          }
        }

        .pagination-info {
          font-size: 12px;
          color: #94a3b8;
          white-space: nowrap;
        }

        .pagination-btns {
          display: flex;
          gap: 8px;
          white-space: nowrap;

          .page-btn {
            display: flex;
            align-items: center;
            gap: 4px;
            padding: 4px 10px;
            font-size: 12px;
            background: rgba(255, 255, 255, 0.06);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 4px;
            color: #cbd5e1;
            cursor: pointer;
            transition: all 0.2s;
            white-space: nowrap;

            &:hover:not(:disabled) {
              background: rgba(255, 255, 255, 0.15);
              color: var(--c-primary-light, #33a5fd);
            }

            &:disabled {
              opacity: 0.3;
              cursor: not-allowed;
            }
          }
        }
      }
    }
  }
}
</style>
