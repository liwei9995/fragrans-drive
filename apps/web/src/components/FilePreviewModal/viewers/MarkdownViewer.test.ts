import { flushPromises, mount } from '@vue/test-utils'
import axios from 'axios'
import { expect, it, vi } from 'vitest'
import MarkdownViewer from './MarkdownViewer.vue'

it('renders a quoted Markdown URL without creating an event handler', async () => {
  vi.spyOn(axios, 'get').mockResolvedValueOnce({
    data: '[open](https://example.test"onmouseover="x)',
  })
  const wrapper = mount(MarkdownViewer, { props: { src: '/file.md' } })
  await flushPromises()
  const link = wrapper.find('.markdown-body a')
  expect(link.exists()).toBe(true)
  expect(link.attributes('onmouseover')).toBeUndefined()
  expect(decodeURIComponent(link.attributes('href') || '')).toContain(
    '"onmouseover="',
  )
  vi.restoreAllMocks()
})
