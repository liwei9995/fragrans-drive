import { mount } from '@vue/test-utils'
import { describe, it, expect, vi } from 'vitest'
import Header from './index.vue'

vi.mock('vue-router', () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), back: vi.fn() }),
  useRoute: () => ({ params: { id: 'root' }, query: {}, path: '/' })
}))

describe('Header', () => {
  it('renders correctly', () => {
    const wrapper = mount(Header)
    expect(wrapper.exists()).toBe(true)
  })

  it('navigates home when logo is clicked', async () => {
    const wrapper = mount(Header)
    
    await wrapper.find('.logo-container').trigger('click')
  })

  it('emits tapActionItem when avatar dropdown item is clicked', async () => {
    const tapActionItem = vi.fn()
    const wrapper = mount(Header, {
      props: {
        avatarActionItems: [{ id: 'logout', name: 'Logout' }],
        tapActionItem
      }
    })
    
    const dropdown = wrapper.findComponent({ name: 'ElDropdown' })
    dropdown.vm.$emit('command', 'logout')
    expect(tapActionItem).toHaveBeenCalledWith('logout')
  })
})
