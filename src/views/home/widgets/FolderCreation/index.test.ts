import { mount } from '@vue/test-utils'
import { describe, it, expect, vi } from 'vitest'
import FolderCreation from './index.vue'

vi.mock('@/hooks/useCreateFolder', () => ({
  useCreateFolder: vi.fn((name, parentId, cb) => {
    cb({ id: '1', name })
  })
}))

describe('FolderCreation', () => {
  it('renders correctly', () => {
    const wrapper = mount(FolderCreation)
    expect(wrapper.exists()).toBe(true)
  })

  it('calls useCreateFolder and emits success', async () => {
    const success = vi.fn()
    const wrapper = mount(FolderCreation, {
      props: {
        parentId: 'root',
        success
      }
    })
    
    // Find SuccessFilled icon wrapper and trigger click
    await wrapper.find('.primary').trigger('click')
    expect(success).toHaveBeenCalledWith({ id: '1', name: '新建文件夹' })
  })

  it('calls close on close icon click', async () => {
    const close = vi.fn()
    const wrapper = mount(FolderCreation, {
      props: {
        close
      }
    })
    
    await wrapper.find('.info').trigger('click')
    expect(close).toHaveBeenCalled()
  })
})
