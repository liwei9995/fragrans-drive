import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useRouter } from 'vue-router'
import StorageCard from './index.vue'

vi.mock('vue-router', () => ({
  useRouter: vi.fn(),
  useRoute: vi.fn(() => ({ params: { id: 'root' }, query: {}, path: '/' }))
}))

vi.mock('@/store', () => ({
  GlobalStore: vi.fn(() => ({ isMobile: false }))
}))

describe('StorageCard', () => {
  const pushMock = vi.fn()
  
  beforeEach(() => {
    vi.mocked(useRouter).mockReturnValue({ push: pushMock, replace: vi.fn(), back: vi.fn() } as any)
  })

  it('renders correctly for empty state', () => {
    const wrapper = mount(StorageCard, {
      props: { id: '1', isEmpty: true },
      global: {
        stubs: {
          More: true,
          SuccessFilled: true
        }
      }
    })
    expect(wrapper.find('.card-wrapper').classes()).toContain('empty')
    expect(wrapper.find('.drop-wrapper').exists()).toBe(false)
  })

  it('renders correctly for normal state', () => {
    const wrapper = mount(StorageCard, {
      props: {
        id: '1',
        title: 'test.mp4',
        desc: '10MB',
        mimeType: 'video/mp4'
      },
      global: {
        stubs: {
          More: true,
          SuccessFilled: true
        }
      }
    })
    expect(wrapper.text()).toContain('test.mp4')
    expect(wrapper.text()).toContain('10MB')
  })

  it('handles click card for folder', async () => {
    const wrapper = mount(StorageCard, {
      props: {
        id: 'folder123',
        type: 'folder'
      },
      global: {
        stubs: {
          More: true,
          SuccessFilled: true
        }
      }
    })

    await wrapper.find('.card-container').trigger('click')
    expect(pushMock).toHaveBeenCalledWith('/home/folder123')
  })

  it('emits toggle-select when checkbox clicked', async () => {
    const wrapper = mount(StorageCard, {
      props: { id: '1' },
      global: {
        stubs: {
          More: true,
          SuccessFilled: true
        }
      }
    })
    
    await wrapper.find('.selection-checkbox').trigger('click')
    expect(wrapper.emitted('toggle-select')).toBeTruthy()
    expect(wrapper.emitted('toggle-select')?.[0]).toEqual(['1'])
  })

  it('handles preview video', async () => {
    const previewVideoMock = vi.fn()
    const wrapper = mount(StorageCard, {
      props: {
        id: '1',
        mimeType: 'video/mp4',
        videoUrl: 'http://test.mp4',
        previewVideo: previewVideoMock
      },
      global: {
        stubs: {
          More: true,
          SuccessFilled: true
        }
      }
    })

    await wrapper.find('.cover').trigger('click')
    expect(previewVideoMock).toHaveBeenCalledWith('http://test.mp4')
  })

  it('handles commands', async () => {
    const tapActionItemMock = vi.fn()
    const wrapper = mount(StorageCard, {
      props: {
        id: '1',
        title: 'test',
        mimeType: 'text/plain',
        tapActionItem: tapActionItemMock
      },
      global: {
        stubs: {
          More: true,
          SuccessFilled: true
        }
      }
    })

    // simulate command event on el-dropdown directly if we can't find it easily
    const dropdown = wrapper.findComponent({ name: 'ElDropdown' })
    expect(dropdown.exists()).toBe(true)
    dropdown.vm.$emit('command', 'rename')
    expect(tapActionItemMock).toHaveBeenCalledWith('rename', '1', 'test', 'text/plain', expect.any(String), expect.any(String))
  })
})
