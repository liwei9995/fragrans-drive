import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import FileSkeleton from './index.vue'

describe('FileSkeleton', () => {
  it('renders correctly', () => {
    const wrapper = mount(FileSkeleton)
    expect(wrapper.exists()).toBe(true)
    expect(wrapper.find('.skeleton-wrapper').exists()).toBe(true)
  })
})
