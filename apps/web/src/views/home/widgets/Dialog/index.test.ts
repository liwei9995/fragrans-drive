import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import Dialog from './index.vue'

describe('Dialog', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('renders correctly with title and default name', async () => {
    mount(Dialog, {
      props: {
        title: '新建文件夹',
        name: '新建文件夹',
      },
      attachTo: document.body,
    })
    await new Promise((resolve) => setTimeout(resolve, 150))
    const input = document.body.querySelector('input')
    expect(input).not.toBeNull()
    expect(input?.value).toBe('新建文件夹')
  })

  it('selects input text on mount', async () => {
    const selectSpy = vi.spyOn(HTMLInputElement.prototype, 'select')
    const focusSpy = vi.spyOn(HTMLInputElement.prototype, 'focus')

    mount(Dialog, {
      props: {
        title: '新建文件夹',
        name: '新建文件夹',
      },
      attachTo: document.body,
    })

    // Wait for nextTick and setTimeout
    await new Promise((resolve) => setTimeout(resolve, 150))

    expect(focusSpy).toHaveBeenCalled()
    expect(selectSpy).toHaveBeenCalled()

    selectSpy.mockRestore()
    focusSpy.mockRestore()
  })

  it('calls onConfirm when handleClick is called with valid name', async () => {
    const onConfirm = vi.fn()
    const wrapper = mount(Dialog, {
      props: {
        title: '新建文件夹',
        name: '我的资料',
        onConfirm,
      },
      attachTo: document.body,
    })

    await new Promise((resolve) => setTimeout(resolve, 150))
    wrapper.vm.handleClick()
    expect(onConfirm).toHaveBeenCalledWith('我的资料')
  })

  it('calls onClose when close is triggered', async () => {
    const onClose = vi.fn()
    const wrapper = mount(Dialog, {
      props: {
        title: '新建文件夹',
        name: '新建文件夹',
        onClose,
      },
      attachTo: document.body,
    })

    await wrapper.findComponent({ name: 'ElDialog' }).vm.$emit('close')
    expect(onClose).toHaveBeenCalled()
  })
})
