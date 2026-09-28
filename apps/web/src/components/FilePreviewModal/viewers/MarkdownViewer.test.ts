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

it('renders markdown elements including headings, blockquotes, code and tables', async () => {
  const mdContent = `
# Title 1
## Section 2
> A blockquote with **bold** text

\`\`\`javascript
const a = 1;
\`\`\`

| Col 1 | Col 2 |
|---|---|
| A | B |
`
  vi.spyOn(axios, 'get').mockResolvedValueOnce({
    data: mdContent,
  })
  const wrapper = mount(MarkdownViewer, { props: { src: '/sample.md' } })
  await flushPromises()
  expect(wrapper.find('.markdown-body h1').text()).toBe('Title 1')
  expect(wrapper.find('.markdown-body h2').text()).toBe('Section 2')
  expect(wrapper.find('.markdown-body blockquote').exists()).toBe(true)
  expect(wrapper.find('.markdown-body blockquote strong').text()).toBe('bold')
  expect(wrapper.find('.markdown-body pre code').exists()).toBe(true)
  expect(wrapper.find('.markdown-body table').exists()).toBe(true)
  vi.restoreAllMocks()
})
