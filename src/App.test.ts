import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import App from './App.vue'

describe('App.vue', () => {
  it('renders the current route', () => {
    const wrapper = mount(App, {
      global: {
        stubs: ['router-view'],
      },
    })

    expect(wrapper.find('router-view-stub').exists()).toBe(true)
  })
})
