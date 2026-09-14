import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import UnsupportedViewer from './UnsupportedViewer.vue'

vi.mock('@/api/modules/storage', () => ({
  getDownloadUrl: vi.fn(async () =>
    Promise.resolve(
      'http://localhost:3821/v1/storage/file1?download=1&token=fresh-tok',
    ),
  ),
}))

vi.mock('element-plus', async (importOriginal) => {
  const actual = await importOriginal<typeof import('element-plus')>()
  return {
    ...actual,
    ElMessage: {
      success: vi.fn(),
      error: vi.fn(),
    },
  }
})

describe('UnsupportedViewer copy link', () => {
  beforeEach(() => {
    import.meta.env.VITE_API_URL = '/api'
    vi.stubGlobal('location', { origin: 'https://drive.oyiyio.com' })
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn(async () => undefined),
      },
    })
  })

  it('copies an absolute getDownloadUrl link for private files', async () => {
    const wrapper = mount(UnsupportedViewer, {
      props: {
        name: 'Compass China.zip',
        fileId: 'file1',
        downloadUrl: '/api/v1/storage/file1?token=stale',
      },
      global: {
        stubs: {
          ElIcon: true,
          ElButton: {
            template:
              '<button v-bind="$attrs" @click="$emit(\'click\')"><slot /></button>',
          },
        },
      },
    })

    await wrapper.find('.link-btn').trigger('click')
    await nextTick()
    await Promise.resolve()

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
      'https://drive.oyiyio.com/api/v1/storage/file1?download=1&token=fresh-tok',
    )
  })
})
