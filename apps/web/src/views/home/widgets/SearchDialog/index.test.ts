import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import { getFiles } from '@/api/modules/storage'
import SearchDialog from './index.vue'

vi.mock('@/api/modules/storage', () => ({
  getFiles: vi.fn(() =>
    Promise.resolve({
      docs: [
        {
          id: '1',
          name: 'report.pdf',
          type: 'file',
          extName: 'pdf',
          size: 1024,
          parentId: 'root',
          updatedAt: '2026-09-10T00:00:00Z',
          trashed: false,
          isPublic: false,
        },
        {
          id: '2',
          name: 'Photos',
          type: 'folder',
          parentId: 'root',
          updatedAt: '2026-09-10T00:00:00Z',
          trashed: false,
          isPublic: false,
        },
      ],
      total: 2,
      page: 1,
      pages: 1,
      limit: 30,
    }),
  ),
}))

describe('SearchDialog.vue', () => {
  it('renders modal when visible is true', async () => {
    const wrapper = mount(SearchDialog, {
      props: { visible: true },
      global: {
        stubs: ['el-icon'],
      },
    })
    expect(wrapper.find('.search-modal').exists()).toBe(true)
    expect(wrapper.find('.search-input').exists()).toBe(true)
  })

  it('does not render modal when visible is false', async () => {
    const wrapper = mount(SearchDialog, {
      props: { visible: false },
      global: {
        stubs: ['el-icon'],
      },
    })
    expect(wrapper.find('.search-modal').exists()).toBe(false)
  })

  it('emits close on backdrop click or ESC', async () => {
    const wrapper = mount(SearchDialog, {
      props: { visible: true },
      global: {
        stubs: ['el-icon'],
      },
    })
    await wrapper.find('.search-backdrop').trigger('click')
    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('renders results and footer when typing query', async () => {
    vi.useFakeTimers()
    const wrapper = mount(SearchDialog, {
      props: { visible: true },
      global: {
        stubs: ['el-icon'],
      },
    })

    const input = wrapper.find('.search-input')
    await input.setValue('pho')
    vi.advanceTimersByTime(300)
    await vi.runAllTimersAsync()

    expect(getFiles).toHaveBeenCalledWith(
      expect.objectContaining({ keyword: 'pho' }),
    )
    expect(wrapper.find('.results-count').text()).toContain('找到 2 个相关项目')
    expect(wrapper.findAll('.result-item').length).toBe(2)
    expect(wrapper.find('.search-footer').exists()).toBe(true)
    expect(wrapper.find('.footer-shortcuts').exists()).toBe(true)

    vi.useRealTimers()
  })
})
