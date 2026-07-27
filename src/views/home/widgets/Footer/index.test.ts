import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import Footer from './index.vue'

describe('Footer', () => {
  it('renders correctly', () => {
    const wrapper = mount(Footer)
    expect(wrapper.exists()).toBe(true)
    const currentYear = new Date().getFullYear()
    expect(wrapper.text()).toContain(currentYear.toString())
    expect(wrapper.text()).toContain('Fragrans Drive')
  })
})
