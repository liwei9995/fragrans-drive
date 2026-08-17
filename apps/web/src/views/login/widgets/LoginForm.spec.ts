import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import LoginForm from './LoginForm.vue'

vi.mock('@/store', () => ({
  GlobalStore: vi.fn(() => ({
    setToken: vi.fn(),
  })),
}))

vi.mock('vue-router', () => ({
  useRouter: () => ({ push: vi.fn() }),
  useRoute: () => ({ query: {} }),
}))

vi.mock('@/api/modules/user', () => ({
  loginApi: vi.fn(() => Promise.resolve({ data: { access_token: '123' } })),
}))

describe('LoginForm.vue', () => {
  it('renders correctly', () => {
    const wrapper = mount(LoginForm, {
      global: {
        stubs: ['el-form', 'el-form-item', 'el-input', 'el-button', 'el-icon'],
      },
    })
    expect(wrapper.exists()).toBe(true)
  })
})
