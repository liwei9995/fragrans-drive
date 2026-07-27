import { mount } from '@vue/test-utils'
import { describe, it, expect, vi } from 'vitest'
import App from './App.vue'
import { createTestingPinia } from '@pinia/testing'
import { GlobalStore } from '@/store'

vi.mock('@/utils/is', () => ({
  isMobile: vi.fn(() => true)
}))

describe('App.vue', () => {
  it('initializes globalStore with isMobile', () => {
    const wrapper = mount(App, {
      global: {
        plugins: [createTestingPinia({
          createSpy: vi.fn,
        })],
        stubs: ['router-view']
      }
    })
    const globalStore = GlobalStore()
    expect(globalStore.setIsMobile).toHaveBeenCalledWith(true)
    expect(wrapper.find('router-view-stub').exists()).toBe(true)
  })
})
