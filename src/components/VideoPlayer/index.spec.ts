import { mount } from '@vue/test-utils'
import { describe, it, expect, vi } from 'vitest'
import VideoPlayer from './index.vue'

vi.mock('vue3-video-play/dist/style.css', () => ({}))
vi.mock('element-plus/theme-chalk/base.css', () => ({}))

describe('VideoPlayer', () => {
  it('renders correctly with default props', () => {
    const wrapper = mount(VideoPlayer, {
      global: {
        stubs: {
          videoPlay: true,
          'el-icon': true,
          Close: true
        }
      }
    })
    expect(wrapper.find('.video-player-wrapper').exists()).toBe(true)
    const videoPlay = wrapper.findComponent({ name: 'videoPlay' })
    expect(videoPlay.exists()).toBe(true)
    // with stubs, properties might be passed as props rather than attributes
    expect(videoPlay.vm.$props.width || videoPlay.attributes('width')).toBe('100%')
    expect(videoPlay.vm.$props.height || videoPlay.attributes('height')).toBe('100%')
    expect(videoPlay.vm.$props.control || videoPlay.attributes('control')).toBe('true')
  })

  it('calls close prop when close button is clicked', async () => {
    const closeMock = vi.fn()
    const wrapper = mount(VideoPlayer, {
      props: { close: closeMock },
      global: {
        stubs: ['videoPlay', 'el-icon', 'Close']
      }
    })
    
    await wrapper.find('.video-player-close-btn').trigger('click')
    expect(closeMock).toHaveBeenCalled()
  })

  it('calls close prop when Esc is pressed', () => {
    const closeMock = vi.fn()
    const wrapper = mount(VideoPlayer, {
      props: { close: closeMock },
      global: {
        stubs: ['videoPlay', 'el-icon', 'Close']
      }
    })

    const event = new KeyboardEvent('keydown', { code: 'Escape' })
    document.dispatchEvent(event)
    expect(closeMock).toHaveBeenCalled()
    
    wrapper.unmount()
  })

  it('calls close prop when Esc is pressed (alternative code)', () => {
    const closeMock = vi.fn()
    const wrapper = mount(VideoPlayer, {
      props: { close: closeMock },
      global: {
        stubs: ['videoPlay', 'el-icon', 'Close']
      }
    })

    const event = new KeyboardEvent('keydown', { code: 'Esc' })
    document.dispatchEvent(event)
    expect(closeMock).toHaveBeenCalled()
    
    wrapper.unmount()
  })
})
