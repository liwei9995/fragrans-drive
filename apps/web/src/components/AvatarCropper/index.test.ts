import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import AvatarCropper from './index.vue'

describe('AvatarCropper.vue', () => {
  it('renders dialog when modelValue is true and displays dropzone by default', async () => {
    const wrapper = mount(AvatarCropper, {
      props: { modelValue: true },
      global: {
        stubs: {
          'el-dialog': {
            template:
              '<div class="el-dialog"><slot /><slot name="footer" /></div>',
            props: ['modelValue'],
            emits: ['close'],
          },
          'el-button': {
            template:
              '<button class="el-button" @click="$emit(\'click\')"><slot /></button>',
          },
          'el-icon': true,
          'el-slider': true,
        },
      },
    })

    expect(wrapper.find('.upload-dropzone').exists()).toBe(true)
    expect(wrapper.find('.dropzone-title').text()).toContain(
      '点击或将图片拖拽到此处上传',
    )
  })

  it('emits update:modelValue and close when cancel button is clicked', async () => {
    const wrapper = mount(AvatarCropper, {
      props: { modelValue: true },
      global: {
        stubs: {
          'el-dialog': {
            template:
              '<div class="el-dialog"><slot /><slot name="footer" /></div>',
            props: ['modelValue'],
            emits: ['close'],
          },
          'el-button': {
            template:
              '<button class="el-button" @click="$emit(\'click\')"><slot /></button>',
          },
          'el-icon': true,
          'el-slider': true,
        },
      },
    })

    const buttons = wrapper.findAll('.el-button')
    // buttons[0] is choose file, buttons[1] is cancel button in footer
    const cancelBtn = buttons[1]
    expect(cancelBtn.text()).toContain('取消')
    await cancelBtn.trigger('click')
    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([false])
    expect(wrapper.emitted('close')).toBeTruthy()
  })
})
