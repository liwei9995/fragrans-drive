import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import LoginForm from './LoginForm.vue'

vi.mock('@/store', () => ({
  GlobalStore: vi.fn(() => ({
    setTokens: vi.fn(),
  })),
}))

vi.mock('vue-router', () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
  useRoute: () => ({ query: {} }),
}))

vi.mock('@/api/modules/user', () => ({
  authLogin: vi.fn(() =>
    Promise.resolve({
      access_token: 'access_123',
      refresh_token: 'refresh_123',
    }),
  ),
  getAuthConfig: vi.fn(() =>
    Promise.resolve({
      allowRegistration: true,
      emailVerificationRequired: false,
      captchaRequired: true,
    }),
  ),
  getCaptcha: vi.fn(() =>
    Promise.resolve({
      id: 'cap_123',
      svg: '<svg>captcha</svg>',
    }),
  ),
  registerUser: vi.fn(() => Promise.resolve({ id: 'user_123' })),
  resetPassword: vi.fn(() => Promise.resolve({ message: 'ok' })),
  sendEmailCode: vi.fn(() => Promise.resolve({ message: 'ok' })),
}))

describe('LoginForm.vue', () => {
  it('renders correctly in login mode', () => {
    const wrapper = mount(LoginForm, {
      global: {
        stubs: [
          'el-form',
          'el-form-item',
          'el-input',
          'el-button',
          'el-icon',
          'el-alert',
        ],
      },
    })
    expect(wrapper.exists()).toBe(true)
    expect(wrapper.text()).toContain('登录')
  })

  it('can switch to register mode and forgot password mode', async () => {
    const wrapper = mount(LoginForm, {
      global: {
        stubs: [
          'el-form',
          'el-form-item',
          'el-input',
          'el-button',
          'el-icon',
          'el-alert',
        ],
      },
    })
    const vm = wrapper.vm as any

    // Switch to register
    vm.switchMode('register')
    await wrapper.vm.$nextTick()
    expect(vm.mode).toBe('register')
    expect(wrapper.text()).toContain('注册')

    // Switch to forgot
    vm.switchMode('forgot')
    await wrapper.vm.$nextTick()
    expect(vm.mode).toBe('forgot')
    expect(wrapper.text()).toContain('找回密码')

    // Switch back to login
    vm.switchMode('login')
    await wrapper.vm.$nextTick()
    expect(vm.mode).toBe('login')
    expect(wrapper.text()).toContain('登录')
  })
})
