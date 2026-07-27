import { mount } from '@vue/test-utils'
import { describe, it, expect, vi } from 'vitest'
import Home from './index.vue'

vi.mock('@/store', () => ({
  GlobalStore: vi.fn(() => ({
    isMobile: false,
    token: '123'
  }))
}))

vi.mock('vue-router', () => ({
  useRouter: () => ({ push: vi.fn(), currentRoute: { value: { params: { id: 'root' } } } }),
  useRoute: () => ({ params: { id: 'root' }, query: {} })
}))

vi.mock('@/hooks/useFetchFiles', () => ({
  useFetchFiles: vi.fn(() => ({
    fetchFiles: vi.fn(),
    isFetching: false,
    listData: { docs: [], page: 1, pages: 1, limit: 50, total: 0 }
  }))
}))

vi.mock('@/api/modules/storage', () => ({
  createFolder: vi.fn(),
  moveFile: vi.fn(),
  renameFile: vi.fn(),
  deleteFile: vi.fn(),
  getDownloadUrl: vi.fn(),
  getPath: vi.fn(() => Promise.resolve([]))
}))

describe('Home.vue', () => {
  it('renders correctly', () => {
    const wrapper = mount(Home, {
      global: {
        stubs: [
          'Header', 'Breadcrumb', 'ActionButton', 'FileSkeleton', 'StorageCard',
          'StorageItem', 'Upload', 'UploadStatus', 'FolderCreation', 'Dialog',
          'Move', 'GlobalDropzone', 'Footer', 'FloatingActionBar',
          'el-container', 'el-header', 'el-main', 'el-footer', 'el-empty'
        ]
      }
    })
    expect(wrapper.exists()).toBe(true)
  })
})
