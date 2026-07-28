import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import StorageItem from './index.vue'

describe('StorageItem', () => {
  it('renders correctly', () => {
    const wrapper = mount(StorageItem, {
      props: {
        id: '1',
        name: 'test-item',
        disabled: false,
      },
      global: {
        stubs: {
          'el-image': true,
        },
      },
    })
    expect(wrapper.text()).toContain('test-item')
  })

  it('handles click when disabled', async () => {
    const tapMock = vi.fn()
    const wrapper = mount(StorageItem, {
      props: {
        id: '1',
        name: 'test',
        disabled: true,
        tap: tapMock,
      },
      global: { stubs: ['el-image'] },
    })

    await wrapper.trigger('click')
    expect(tapMock).not.toHaveBeenCalled()
  })

  it('handles click when enabled', async () => {
    const tapMock = vi.fn()
    const wrapper = mount(StorageItem, {
      props: {
        id: '1',
        name: 'test',
        disabled: false,
        tap: tapMock,
      },
      global: { stubs: ['el-image'] },
    })

    await wrapper.trigger('click')
    expect(tapMock).toHaveBeenCalledWith('1')
  })
})
