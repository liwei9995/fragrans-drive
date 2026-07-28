import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import VideoPlayer from './index.vue'

vi.mock('element-plus/theme-chalk/base.css', () => ({}))

describe('VideoPlayer', () => {
  it('renders a native video player', () => {
    const wrapper = mount(VideoPlayer, {
      props: { src: '/video.mp4' },
      global: {
        stubs: ['el-icon', 'Close'],
      },
    })

    const video = wrapper.get('video')
    expect(video.attributes('src')).toBe('/video.mp4')
    expect(video.attributes('controls')).toBeDefined()
    expect(video.attributes('playsinline')).toBeDefined()
  })

  it('closes from the button and Escape, then removes its listener', async () => {
    const close = vi.fn()
    const wrapper = mount(VideoPlayer, {
      props: { close },
      global: {
        stubs: ['el-icon', 'Close'],
      },
    })

    await wrapper.get('.video-player-close-btn').trigger('click')
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(close).toHaveBeenCalledTimes(2)

    wrapper.unmount()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(close).toHaveBeenCalledTimes(2)
  })
})
