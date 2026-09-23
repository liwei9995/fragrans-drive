import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import ImageViewer from './ImageViewer.vue'

describe('ImageViewer', () => {
  const commonStubs = {
    ElIcon: true,
    ElTooltip: { template: '<div><slot /></div>' },
  }

  it('renders LQIP thumbnail immediately and non-blocking badge while HD is loading', () => {
    const wrapper = mount(ImageViewer, {
      props: {
        src: '/api/v1/storage/photo_hd.jpg',
        name: 'photo_hd.jpg',
        thumb: '/api/v1/storage/photo_thumb.jpg',
      },
      global: {
        stubs: commonStubs,
      },
    })

    // Low-res thumbnail is rendered immediately
    const thumbImg = wrapper.find('.preview-thumb')
    expect(thumbImg.exists()).toBe(true)
    expect(thumbImg.attributes('src')).toBe('/api/v1/storage/photo_thumb.jpg')

    // Floating pill badge is rendered
    const badge = wrapper.find('.image-loading-badge')
    expect(badge.exists()).toBe(true)
    expect(badge.text()).toContain('高清载入中...')

    // Fullscreen blocking spinner is NOT rendered
    expect(wrapper.find('.image-loading').exists()).toBe(false)

    // Main HD image is mounted with .is-loading
    const mainImg = wrapper.find('.main-image')
    expect(mainImg.exists()).toBe(true)
    expect(mainImg.classes()).toContain('is-loading')
  })

  it('smoothly transitions to HD view when main image loads', async () => {
    const wrapper = mount(ImageViewer, {
      props: {
        src: '/api/v1/storage/photo_hd.jpg',
        name: 'photo_hd.jpg',
        thumb: '/api/v1/storage/photo_thumb.jpg',
      },
      global: {
        stubs: commonStubs,
      },
    })

    const mainImg = wrapper.find('.main-image')
    // Simulate image loaded event
    Object.defineProperty(mainImg.element, 'naturalWidth', {
      value: 3840,
      configurable: true,
    })
    Object.defineProperty(mainImg.element, 'naturalHeight', {
      value: 2160,
      configurable: true,
    })
    await mainImg.trigger('load')

    // High res loaded state
    expect(mainImg.classes()).not.toContain('is-loading')
    const thumbImg = wrapper.find('.preview-thumb')
    expect(thumbImg.classes()).toContain('is-fading-out')

    // Loading badge disappears
    expect(wrapper.find('.image-loading-badge').exists()).toBe(false)

    // Natural dimensions displayed in toolbar
    expect(wrapper.find('.dimension-tag').text()).toBe('3840 × 2160')
  })

  it('shows fallback full spinner when no real thumbnail is provided', () => {
    const wrapper = mount(ImageViewer, {
      props: {
        src: '/api/v1/storage/photo_hd.jpg',
        name: 'photo_hd.jpg',
        thumb: '',
      },
      global: {
        stubs: commonStubs,
      },
    })

    // No thumbnail rendered
    expect(wrapper.find('.preview-thumb').exists()).toBe(false)
    // Fullscreen spinner is shown
    expect(wrapper.find('.image-loading').exists()).toBe(true)
    expect(wrapper.find('.image-loading').text()).toContain(
      '正在载入高分辨率原图...',
    )
    // No floating pill badge
    expect(wrapper.find('.image-loading-badge').exists()).toBe(false)
  })

  it('handles image error and supports retry', async () => {
    const wrapper = mount(ImageViewer, {
      props: {
        src: '/api/v1/storage/corrupted.jpg',
        name: 'corrupted.jpg',
        thumb: '/api/v1/storage/corrupted_thumb.jpg',
      },
      global: {
        stubs: commonStubs,
      },
    })

    const mainImg = wrapper.find('.main-image')
    await mainImg.trigger('error')

    // Error UI is shown
    expect(wrapper.find('.image-error').exists()).toBe(true)
    expect(wrapper.find('.image-error').text()).toContain('图片加载失败')

    // Retry button triggers reload
    const retryBtn = wrapper.find('.retry-btn')
    expect(retryBtn.exists()).toBe(true)
    await retryBtn.trigger('click')

    expect(wrapper.find('.image-error').exists()).toBe(false)
  })

  it('supports zoom in, zoom out, and reset controls', async () => {
    const wrapper = mount(ImageViewer, {
      props: {
        src: '/api/v1/storage/photo_hd.jpg',
        name: 'photo_hd.jpg',
      },
      global: {
        stubs: commonStubs,
      },
    })

    expect(wrapper.find('.scale-tag').text()).toBe('100%')

    // Click Zoom In
    const buttons = wrapper.findAll('.tool-btn')
    // Order: ZoomOut, Reset, ZoomIn, Rotate
    const zoomInBtn = buttons[2]
    await zoomInBtn.trigger('click')
    expect(wrapper.find('.scale-tag').text()).toBe('125%')

    // Click Reset
    const resetBtn = buttons[1]
    await resetBtn.trigger('click')
    expect(wrapper.find('.scale-tag').text()).toBe('100%')
  })

  it('emits loaded event when image loads successfully', async () => {
    const wrapper = mount(ImageViewer, {
      props: {
        src: '/api/v1/storage/photo.jpg',
        name: 'photo.jpg',
      },
      global: {
        stubs: commonStubs,
      },
    })

    const mainImg = wrapper.find('.main-image')
    await mainImg.trigger('load')
    expect(wrapper.emitted('loaded')).toBeTruthy()
  })

  it('renders View Original button and loads original on click', async () => {
    const wrapper = mount(ImageViewer, {
      props: {
        src: '/api/v1/storage/photo.jpg?preview=1',
        originalSrc: '/api/v1/storage/photo.jpg',
        name: 'photo.jpg',
      },
      global: {
        stubs: commonStubs,
      },
    })

    const viewOriginalBtn = wrapper.find('.text-tool-btn')
    expect(viewOriginalBtn.exists()).toBe(true)
    expect(viewOriginalBtn.text()).toContain('查看原图')

    await viewOriginalBtn.trigger('click')

    const mainImg = wrapper.find('.main-image')
    expect(mainImg.attributes('src')).toBe('/api/v1/storage/photo.jpg')

    // Simulate original image load
    await mainImg.trigger('load')
    expect(viewOriginalBtn.text()).toContain('已是原图')
    expect(viewOriginalBtn.classes()).toContain('active')
  })

  it('preserves and caches aspect ratio across switching to eliminate stage flash', async () => {
    const wrapper = mount(ImageViewer, {
      props: {
        src: '/api/v1/storage/photo_1.jpg',
        name: 'photo_1.jpg',
        thumb: '/api/v1/storage/photo_1_thumb.jpg',
      },
      global: {
        stubs: commonStubs,
      },
    })

    const thumbImg = wrapper.find('.preview-thumb')
    Object.defineProperty(thumbImg.element, 'naturalWidth', {
      value: 800,
      configurable: true,
    })
    Object.defineProperty(thumbImg.element, 'naturalHeight', {
      value: 600,
      configurable: true,
    })
    await thumbImg.trigger('load')

    const stage = wrapper.find('.image-stage')
    const initialStyle = stage.attributes('style')
    expect(initialStyle).toContain('aspect-ratio: 1.3333')

    // Switch to photo_2 with known cached thumb
    await wrapper.setProps({
      src: '/api/v1/storage/photo_2.jpg',
      name: 'photo_2.jpg',
      thumb: '/api/v1/storage/photo_2_thumb.jpg',
    })

    // Notice that style never collapses to 0 or resets to auto/max-width: 90%
    const switchedStyle = wrapper.find('.image-stage').attributes('style')
    expect(switchedStyle).not.toContain('width: auto')
    expect(switchedStyle).not.toContain('max-width: 90%')
  })

  describe('Mobile Touch Swipe Navigation', () => {
    it('swipes left to navigate to next image when hasNext is true', async () => {
      const wrapper = mount(ImageViewer, {
        props: {
          src: '/api/v1/storage/photo_1.jpg',
          name: 'photo_1.jpg',
          hasNext: true,
          hasPrev: false,
        },
        global: {
          stubs: commonStubs,
        },
      })

      const viewport = wrapper.find('.image-viewport')

      // Touch start at clientX = 200
      await viewport.trigger('touchstart', {
        touches: [{ clientX: 200, clientY: 100 }],
      })

      // Drag left to clientX = 120 (deltaX = -80px)
      await viewport.trigger('touchmove', {
        touches: [{ clientX: 120, clientY: 100 }],
      })

      // Touch end
      await viewport.trigger('touchend')

      expect(wrapper.emitted('next')).toBeTruthy()
      expect(wrapper.emitted('prev')).toBeFalsy()
    })

    it('swipes right to navigate to previous image when hasPrev is true', async () => {
      const wrapper = mount(ImageViewer, {
        props: {
          src: '/api/v1/storage/photo_2.jpg',
          name: 'photo_2.jpg',
          hasNext: true,
          hasPrev: true,
        },
        global: {
          stubs: commonStubs,
        },
      })

      const viewport = wrapper.find('.image-viewport')

      // Touch start at clientX = 100
      await viewport.trigger('touchstart', {
        touches: [{ clientX: 100, clientY: 100 }],
      })

      // Drag right to clientX = 180 (deltaX = +80px)
      await viewport.trigger('touchmove', {
        touches: [{ clientX: 180, clientY: 100 }],
      })

      // Touch end
      await viewport.trigger('touchend')

      expect(wrapper.emitted('prev')).toBeTruthy()
      expect(wrapper.emitted('next')).toBeFalsy()
    })

    it('does not emit next when hasNext is false (at the end of gallery)', async () => {
      const wrapper = mount(ImageViewer, {
        props: {
          src: '/api/v1/storage/photo_last.jpg',
          name: 'photo_last.jpg',
          hasNext: false,
          hasPrev: true,
        },
        global: {
          stubs: commonStubs,
        },
      })

      const viewport = wrapper.find('.image-viewport')

      await viewport.trigger('touchstart', {
        touches: [{ clientX: 200, clientY: 100 }],
      })
      await viewport.trigger('touchmove', {
        touches: [{ clientX: 100, clientY: 100 }],
      })
      await viewport.trigger('touchend')

      expect(wrapper.emitted('next')).toBeFalsy()
    })

    it('does not emit prev when hasPrev is false (at the start of gallery)', async () => {
      const wrapper = mount(ImageViewer, {
        props: {
          src: '/api/v1/storage/photo_first.jpg',
          name: 'photo_first.jpg',
          hasNext: true,
          hasPrev: false,
        },
        global: {
          stubs: commonStubs,
        },
      })

      const viewport = wrapper.find('.image-viewport')

      await viewport.trigger('touchstart', {
        touches: [{ clientX: 100, clientY: 100 }],
      })
      await viewport.trigger('touchmove', {
        touches: [{ clientX: 190, clientY: 100 }],
      })
      await viewport.trigger('touchend')

      expect(wrapper.emitted('prev')).toBeFalsy()
    })

    it('does not emit next or prev if swipe distance is below threshold', async () => {
      const wrapper = mount(ImageViewer, {
        props: {
          src: '/api/v1/storage/photo_1.jpg',
          name: 'photo_1.jpg',
          hasNext: true,
          hasPrev: true,
        },
        global: {
          stubs: commonStubs,
        },
      })

      const viewport = wrapper.find('.image-viewport')

      await viewport.trigger('touchstart', {
        touches: [{ clientX: 200, clientY: 100 }],
      })
      // Small displacement of only 15px
      await viewport.trigger('touchmove', {
        touches: [{ clientX: 185, clientY: 100 }],
      })
      await viewport.trigger('touchend')

      expect(wrapper.emitted('next')).toBeFalsy()
      expect(wrapper.emitted('prev')).toBeFalsy()
    })

    it('does not navigate on swipe when zoomed in (scale > 1), pans instead', async () => {
      const wrapper = mount(ImageViewer, {
        props: {
          src: '/api/v1/storage/photo_1.jpg',
          name: 'photo_1.jpg',
          hasNext: true,
          hasPrev: true,
        },
        global: {
          stubs: commonStubs,
        },
      })

      // Zoom in
      const buttons = wrapper.findAll('.tool-btn')
      const zoomInBtn = buttons[2]
      await zoomInBtn.trigger('click')
      expect(wrapper.find('.scale-tag').text()).toBe('125%')

      const viewport = wrapper.find('.image-viewport')

      await viewport.trigger('touchstart', {
        touches: [{ clientX: 200, clientY: 100 }],
      })
      await viewport.trigger('touchmove', {
        touches: [{ clientX: 100, clientY: 100 }],
      })
      await viewport.trigger('touchend')

      // Should NOT emit next/prev when zoomed in
      expect(wrapper.emitted('next')).toBeFalsy()
      expect(wrapper.emitted('prev')).toBeFalsy()
    })
  })
})
