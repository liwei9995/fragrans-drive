import { mount } from '@vue/test-utils'
import axios from 'axios'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import CodeTextViewer from './CodeTextViewer.vue'

vi.mock('axios')

describe('CodeTextViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders JSON file with correct syntax highlighting and NO corrupted HTML text', async () => {
    const jsonSample = `{
  "app": "Fragrans Drive",
  "version": "0.3.0",
  "features": {
    "preview": true
  },
  "activeUsers": 1280
}`
    vi.mocked(axios.get).mockResolvedValueOnce({ data: jsonSample })

    const wrapper = mount(CodeTextViewer, {
      props: {
        src: '/api/v1/storage/app_config.json',
        name: 'app_config.json',
        extName: 'json',
        size: 150,
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

    const codeLines = wrapper.findAll('.code-row')
    expect(codeLines.length).toBe(8)

    // Ensure raw class="tok-string"> NEVER leaks as plain text!
    const fullText = wrapper.find('.code-lines').text()
    expect(fullText).not.toContain('class="tok-string">')
    expect(fullText).not.toContain('class="tok-keyword">')
    expect(fullText).not.toContain('class="tok-key">')

    // Check line 2: "app": "Fragrans Drive",
    const line2Html = codeLines[1].html()
    expect(line2Html).toContain('class="tok-key"')
    expect(line2Html).toContain('"app"')
    expect(line2Html).toContain('class="tok-string"')
    expect(line2Html).toContain('"Fragrans Drive"')

    // Check line 5 (index 4): "preview": true
    const line5Html = codeLines[4].html()
    expect(line5Html).toContain('class="tok-key"')
    expect(line5Html).toContain('class="tok-keyword"')
    expect(line5Html).toContain('true')

    // Check line 7: "activeUsers": 1280
    const line7Html = codeLines[6].html()
    expect(line7Html).toContain('class="tok-number"')
    expect(line7Html).toContain('1280')
  })

  it('formats unindented JSON on format button click', async () => {
    const rawJson = '{"app":"Fragrans","version":"1.0"}'
    vi.mocked(axios.get).mockResolvedValueOnce({ data: rawJson })

    const wrapper = mount(CodeTextViewer, {
      props: {
        src: '/api/v1/storage/config.json',
        name: 'config.json',
        extName: 'json',
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

    expect(wrapper.findAll('.code-row').length).toBe(1)

    // Click format JSON button
    const formatBtn = wrapper.find('button[title*="格式化 JSON"]')
    expect(formatBtn.exists()).toBe(true)
    await formatBtn.trigger('click')
    await wrapper.vm.$nextTick()

    // Formatted JSON should have multiple lines
    expect(wrapper.findAll('.code-row').length).toBe(4)
  })

  it('highlights search query matches with mark element', async () => {
    const codeSample = 'const username = "admin";\nconst password = "secret";'
    vi.mocked(axios.get).mockResolvedValueOnce({ data: codeSample })

    const wrapper = mount(CodeTextViewer, {
      props: {
        src: '/api/v1/storage/main.js',
        name: 'main.js',
        extName: 'js',
        size: 60,
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

    // Set search query
    // @ts-expect-error test search internal
    wrapper.vm.searchQuery = 'admin'
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('.code-row')
    expect(rows[0].html()).toContain('<mark class="code-match">admin</mark>')
  })
})
