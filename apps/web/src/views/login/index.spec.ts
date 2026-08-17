import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import Login from './index.vue'

vi.mock('./widgets/LoginForm.vue', () => ({
  default: {
    template: '<div>Mocked Form</div>',
  },
}))

describe('Login.vue', () => {
  it('renders correctly', () => {
    const wrapper = mount(Login)
    expect(wrapper.exists()).toBe(true)
  })
})
