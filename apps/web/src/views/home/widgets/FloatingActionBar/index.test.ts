import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import FloatingActionBar from './index.vue'

describe('FloatingActionBar', () => {
  it('does not render when selectedCount is 0', () => {
    const wrapper = mount(FloatingActionBar, {
      props: {
        selectedCount: 0,
      },
    })
    expect(wrapper.find('.floating-action-bar-wrapper').exists()).toBe(false)
  })

  it('renders when selectedCount > 0', () => {
    const wrapper = mount(FloatingActionBar, {
      props: {
        selectedCount: 2,
      },
    })
    expect(wrapper.find('.floating-action-bar-wrapper').exists()).toBe(true)
    expect(wrapper.find('.count').text()).toBe('2')
  })

  it('emits events on click', async () => {
    const wrapper = mount(FloatingActionBar, {
      props: {
        selectedCount: 1,
      },
    })
    await wrapper.find('.action-item.delete').trigger('click')
    expect(wrapper.emitted('delete')).toBeTruthy()

    await wrapper.find('.close-btn').trigger('click')
    expect(wrapper.emitted('clear')).toBeTruthy()
  })
})
