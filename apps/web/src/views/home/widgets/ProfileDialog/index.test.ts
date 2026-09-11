import { flushPromises, mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import { getStorageUsage } from '@/api/modules/storage'
import { getProfile, updatePassword, updateProfile } from '@/api/modules/user'
import ProfileDialog from './index.vue'

vi.mock('@/store', () => ({
  GlobalStore: vi.fn(() => ({
    userInfo: null,
    setUserInfo: vi.fn(),
    setAvatar: vi.fn(),
  })),
}))

vi.mock('@/api/modules/user', () => ({
  getProfile: vi.fn(() =>
    Promise.resolve({
      id: '1',
      email: 'user@example.com',
      firstName: 'Alex',
      lastName: 'Li',
      avatar: 'https://example.com/avatar.png',
      gender: 1,
      age: 25,
      roles: ['user'],
    }),
  ),
  updateProfile: vi.fn(() =>
    Promise.resolve({
      id: '1',
      email: 'user@example.com',
      firstName: 'Alex',
      lastName: 'Updated',
    }),
  ),
  updatePassword: vi.fn(() => Promise.resolve()),
}))

vi.mock('@/api/modules/storage', () => ({
  getStorageUsage: vi.fn(() =>
    Promise.resolve({
      usedBytes: 1048576,
      fileCount: 10,
      quotaBytes: 53687091200,
    }),
  ),
}))

describe('ProfileDialog.vue', () => {
  it('renders dialog and loads profile/usage when visible', async () => {
    const wrapper = mount(ProfileDialog, {
      props: { visible: true },
      global: {
        stubs: {
          'el-dialog': {
            template: '<div class="el-dialog"><slot /></div>',
            props: ['modelValue'],
          },
          'el-avatar': true,
          'el-tabs': true,
          'el-tab-pane': true,
          'el-progress': true,
          'el-form': true,
          'el-form-item': true,
          'el-input': true,
          'el-input-number': true,
          'el-select': true,
          'el-option': true,
          'el-button': true,
          'el-icon': true,
        },
      },
    })
    expect(wrapper.find('.profile-container').exists()).toBe(true)
    expect(getProfile).toHaveBeenCalled()
    expect(getStorageUsage).toHaveBeenCalled()
    await flushPromises()
    expect(wrapper.find('.user-name').text()).toBe('Alex Li')
  })

  it('emits close when dialog is closed', async () => {
    const wrapper = mount(ProfileDialog, {
      props: { visible: true },
      global: {
        stubs: {
          'el-dialog': {
            template: '<div class="el-dialog"><slot /></div>',
            props: ['modelValue'],
          },
          'el-avatar': true,
          'el-tabs': true,
          'el-tab-pane': true,
          'el-progress': true,
          'el-form': true,
          'el-form-item': true,
          'el-input': true,
          'el-input-number': true,
          'el-select': true,
          'el-option': true,
          'el-button': true,
          'el-icon': true,
          AvatarCropper: true,
        },
      },
    })
    const dialog = wrapper.findComponent({ name: 'ElDialog' })
    if (dialog.exists()) {
      dialog.vm.$emit('close')
      expect(wrapper.emitted('close')).toBeTruthy()
    }
  })

  it('updates avatar and saves profile when avatar is cropped', async () => {
    const wrapper = mount(ProfileDialog, {
      props: { visible: true },
      global: {
        stubs: {
          'el-dialog': {
            template: '<div class="el-dialog"><slot /></div>',
            props: ['modelValue'],
          },
          'el-avatar': true,
          'el-tabs': true,
          'el-tab-pane': true,
          'el-progress': true,
          'el-form': true,
          'el-form-item': true,
          'el-input': true,
          'el-input-number': true,
          'el-select': true,
          'el-option': true,
          'el-button': true,
          'el-icon': true,
          AvatarCropper: {
            name: 'avatar-cropper',
            template: '<div class="avatar-cropper-stub"></div>',
            props: ['modelValue'],
            emits: ['crop'],
          },
        },
      },
    })
    await flushPromises()

    const cropper = wrapper.findComponent({ name: 'avatar-cropper' })
    expect(cropper.exists()).toBe(true)

    await cropper.vm.$emit('crop', 'data:image/png;base64,mockcroppeddata')
    await flushPromises()

    expect(updateProfile).toHaveBeenCalledWith(
      expect.objectContaining({
        avatar: 'data:image/png;base64,mockcroppeddata',
      }),
    )
  })
})
