import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import FilePreviewModal from './index.vue'
import type { FilePreviewItem } from './types'

describe('FilePreviewModal', () => {
  const mockFile: FilePreviewItem = {
    id: 'f1',
    name: 'test-image.png',
    extName: 'png',
    mimeType: 'image/png',
    size: 2048,
    url: '/api/v1/storage/f1',
    updatedAt: '2026-09-10T12:00:00Z',
  }

  const mockFileList: FilePreviewItem[] = [
    mockFile,
    {
      id: 'f2',
      name: 'notes.md',
      extName: 'md',
      mimeType: 'text/markdown',
      size: 512,
      url: '/api/v1/storage/f2',
    },
  ]

  const commonStubs = {
    teleport: true,
    ElIcon: true,
    ElTooltip: { template: '<div><slot /></div>' },
    ElButton: true,
    ImageViewer: true,
    VideoViewer: true,
    AudioViewer: true,
    PdfViewer: true,
    MarkdownViewer: true,
    CsvViewer: true,
    CodeTextViewer: true,
    UnsupportedViewer: true,
  }

  it('does not render when visible is false', () => {
    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: false,
        file: mockFile,
      },
      global: {
        stubs: commonStubs,
      },
    })
    expect(wrapper.find('.file-preview-overlay').exists()).toBe(false)
  })

  it('renders header with file information when visible is true', () => {
    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: true,
        file: mockFile,
        fileList: mockFileList,
      },
      global: {
        stubs: commonStubs,
      },
    })

    expect(wrapper.find('.file-preview-overlay').exists()).toBe(true)
    expect(wrapper.find('.file-title').text()).toBe('test-image.png')
    expect(wrapper.find('.size-tag').text()).toBe('2.00 KB')
    expect(wrapper.find('.type-badge').text()).toBe('PNG')
    expect(wrapper.find('.nav-indicator').text()).toBe('1 / 2')
  })

  it('emits close event when close button is clicked', async () => {
    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: true,
        file: mockFile,
      },
      global: {
        stubs: commonStubs,
      },
    })

    await wrapper.find('.close-btn').trigger('click')
    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits download event when download button is clicked', async () => {
    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: true,
        file: mockFile,
      },
      global: {
        stubs: commonStubs,
      },
    })

    await wrapper.find('.download-btn').trigger('click')
    expect(wrapper.emitted('download')).toBeTruthy()
    expect(wrapper.emitted('download')?.[0]).toEqual([mockFile])
  })

  it('navigates to next file when next button is clicked', async () => {
    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: true,
        file: mockFile,
        fileList: mockFileList,
      },
      global: {
        stubs: commonStubs,
      },
    })

    const nextBtn = wrapper.findAll('.nav-btn')[1]
    await nextBtn.trigger('click')
    expect(wrapper.emitted('change-file')).toBeTruthy()
    expect(wrapper.emitted('change-file')?.[0]).toEqual([mockFileList[1]])
  })

  it('renders correctly for CSV file', () => {
    const csvFile: FilePreviewItem = {
      id: 'f3',
      name: 'data.csv',
      extName: 'csv',
      mimeType: 'text/csv',
      size: 1024,
      url: '/api/v1/storage/f3',
    }

    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: true,
        file: csvFile,
      },
      global: {
        stubs: commonStubs,
      },
    })

    expect(wrapper.find('.type-badge').text()).toBe('CSV')
    expect(wrapper.findComponent({ name: 'CsvViewer' }).exists()).toBe(true)
  })

  it('emits close when browser back (popstate) fires while preview is open', async () => {
    const pushStateSpy = vi.spyOn(window.history, 'pushState')

    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: false,
        file: mockFile,
      },
      global: {
        stubs: commonStubs,
      },
    })

    await wrapper.setProps({ visible: true })
    await nextTick()

    expect(pushStateSpy).toHaveBeenCalled()
    expect(
      (pushStateSpy.mock.calls.at(-1)?.[0] as { filePreview?: boolean })
        ?.filePreview,
    ).toBe(true)

    window.dispatchEvent(new PopStateEvent('popstate'))
    await nextTick()

    expect(wrapper.emitted('close')).toBeTruthy()

    pushStateSpy.mockRestore()
    wrapper.unmount()
  })

  it('pops history when preview is closed via UI after being opened', async () => {
    const backSpy = vi.spyOn(window.history, 'back')

    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: false,
        file: mockFile,
      },
      global: {
        stubs: commonStubs,
      },
    })

    await wrapper.setProps({ visible: true })
    await nextTick()
    await wrapper.setProps({ visible: false })
    await nextTick()

    expect(backSpy).toHaveBeenCalled()

    backSpy.mockRestore()
    wrapper.unmount()
  })

  it('passes activeImageThumb to ImageViewer when previewing image file', () => {
    const fileWithThumb: FilePreviewItem = {
      ...mockFile,
      thumbnail: '/api/v1/storage/thumb_123',
    }

    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: true,
        file: fileWithThumb,
        fileList: [fileWithThumb],
      },
      global: {
        stubs: {
          ...commonStubs,
          ImageViewer: {
            props: ['src', 'name', 'thumb', 'originalSrc'],
            template:
              '<div class="stub-image-viewer" :data-thumb="thumb" :data-src="src" :data-orig="originalSrc"></div>',
          },
        },
      },
    })

    const viewer = wrapper.find('.stub-image-viewer')
    expect(viewer.exists()).toBe(true)
    expect(viewer.attributes('data-thumb')).toBe('/api/v1/storage/thumb_123')
    expect(viewer.attributes('data-src')).toBe(`${mockFile.url}?preview=1`)
    expect(viewer.attributes('data-orig')).toBe(mockFile.url)
  })

  it('binds hasPrev, hasNext and changes file on prev/next emitted from ImageViewer', async () => {
    const wrapper = mount(FilePreviewModal, {
      props: {
        visible: true,
        file: mockFile,
        fileList: mockFileList,
      },
      global: {
        stubs: {
          ...commonStubs,
          ImageViewer: {
            props: [
              'src',
              'name',
              'thumb',
              'originalSrc',
              'hasPrev',
              'hasNext',
            ],
            emits: ['prev', 'next'],
            template: `
              <div class="stub-image-viewer" :data-has-prev="hasPrev" :data-has-next="hasNext">
                <button class="mock-swipe-prev" @click="$emit('prev')">Prev</button>
                <button class="mock-swipe-next" @click="$emit('next')">Next</button>
              </div>
            `,
          },
        },
      },
    })

    const viewer = wrapper.find('.stub-image-viewer')
    expect(viewer.exists()).toBe(true)
    // Initially on first file (mockFile): hasPrev is false, hasNext is true
    expect(viewer.attributes('data-has-prev')).toBe('false')
    expect(viewer.attributes('data-has-next')).toBe('true')

    // Trigger next (swiped left)
    await wrapper.find('.mock-swipe-next').trigger('click')
    expect(wrapper.emitted('change-file')).toBeTruthy()
    expect(wrapper.emitted('change-file')?.[0]).toEqual([mockFileList[1]])
  })
})
