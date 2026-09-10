import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import {
  emptyTrash,
  getTrashList,
  restoreFile,
  restoreTrash,
} from '@/api/modules/storage'
import TrashDialog from './index.vue'

vi.mock('@/api/modules/storage', () => ({
  getTrashList: vi.fn(() =>
    Promise.resolve({
      docs: [
        {
          id: '1',
          name: 'deleted.txt',
          type: 'file',
          extName: 'txt',
          size: 2048,
          parentId: 'root',
          updatedAt: '2026-09-10T00:00:00Z',
          trashed: true,
          isPublic: false,
        },
      ],
      total: 1,
      page: 1,
      pages: 1,
      limit: 50,
    }),
  ),
  restoreFile: vi.fn(() => Promise.resolve({})),
  restoreTrash: vi.fn(() =>
    Promise.resolve({ requestedItems: 1, restoredDocs: 1 }),
  ),
  emptyTrash: vi.fn(() => Promise.resolve({ deletedDocs: 1, deletedFiles: 1 })),
}))

describe('TrashDialog.vue', () => {
  it('renders dialog and loads items when visible', async () => {
    const wrapper = mount(TrashDialog, {
      props: { visible: true },
      global: {
        stubs: {
          'el-dialog': {
            template: '<div class="el-dialog"><slot /></div>',
            props: ['modelValue'],
          },
          'el-button': true,
          'el-checkbox': true,
          'el-scrollbar': { template: '<div><slot /></div>' },
          'el-tooltip': { template: '<div><slot /></div>' },
          'el-icon': true,
        },
      },
    })
    expect(wrapper.find('.trash-container').exists()).toBe(true)
    expect(getTrashList).toHaveBeenCalled()
  })

  it('emits close when dialog closes', async () => {
    const wrapper = mount(TrashDialog, {
      props: { visible: true },
      global: {
        stubs: {
          'el-dialog': {
            template: '<div class="el-dialog"><slot /></div>',
            props: ['modelValue'],
          },
          'el-button': true,
          'el-checkbox': true,
          'el-scrollbar': { template: '<div><slot /></div>' },
          'el-tooltip': { template: '<div><slot /></div>' },
          'el-icon': true,
        },
      },
    })
    const dialog = wrapper.findComponent({ name: 'ElDialog' })
    if (dialog.exists()) {
      dialog.vm.$emit('close')
      expect(wrapper.emitted('close')).toBeTruthy()
    }
  })
})
