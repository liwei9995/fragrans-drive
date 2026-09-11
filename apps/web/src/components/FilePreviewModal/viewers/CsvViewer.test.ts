import { mount } from '@vue/test-utils'
import axios from 'axios'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import CsvViewer from './CsvViewer.vue'

vi.mock('axios')

describe('CsvViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders loading state then parses CSV data into table rows', async () => {
    const csvContent = 'id,name,age\n1,Alice,24\n2,Bob,30\n'
    vi.mocked(axios.get).mockResolvedValueOnce({ data: csvContent })

    const wrapper = mount(CsvViewer, {
      props: {
        src: '/api/v1/storage/data.csv',
        name: 'data.csv',
        extName: 'csv',
        size: 100,
      },
      global: {
        stubs: {
          ElIcon: true,
          ElInput: true,
        },
      },
    })

    expect(wrapper.find('.csv-loading').exists()).toBe(true)

    // Wait for axios promise to resolve and DOM to update
    await new Promise((resolve) => setTimeout(resolve, 20))
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.csv-loading').exists()).toBe(false)
    expect(wrapper.text()).toContain('CSV 数据表')
    expect(wrapper.text()).toContain('共 2 行 · 3 列')

    const rows = wrapper.findAll('.data-row')
    expect(rows.length).toBe(2)
    expect(rows[0].text()).toContain('Alice')
    expect(rows[1].text()).toContain('Bob')
  })

  it('handles TSV files with tab delimiter', async () => {
    const tsvContent = 'item\tprice\nBook\t$15\nPen\t$2\n'
    vi.mocked(axios.get).mockResolvedValueOnce({ data: tsvContent })

    const wrapper = mount(CsvViewer, {
      props: {
        src: '/api/v1/storage/data.tsv',
        name: 'data.tsv',
        extName: 'tsv',
        size: 50,
      },
      global: {
        stubs: {
          ElIcon: true,
          ElInput: true,
        },
      },
    })

    await new Promise((resolve) => setTimeout(resolve, 20))
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('TSV 数据表')
    const headers = wrapper.findAll('.col-header')
    expect(headers.length).toBe(2)
    expect(headers[0].text()).toBe('item')
    expect(headers[1].text()).toBe('price')
  })

  it('handles network error gracefully', async () => {
    vi.mocked(axios.get).mockRejectedValueOnce(new Error('Network error'))

    const wrapper = mount(CsvViewer, {
      props: {
        src: '/api/v1/storage/error.csv',
        name: 'error.csv',
      },
      global: {
        stubs: {
          ElIcon: true,
          ElInput: true,
        },
      },
    })

    await new Promise((resolve) => setTimeout(resolve, 20))
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.csv-error').exists()).toBe(true)
    expect(wrapper.text()).toContain('加载表格数据失败')
  })
})
