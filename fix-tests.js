const fs = require('fs')
const path = require('path')

const files = [
  'ActionButton/index.test.ts',
  'Empty/index.test.ts',
  'Empty/Item/index.test.ts',
  'GlobalDropzone/index.test.ts'
]

files.forEach(file => {
  const p = path.join('src/views/home/widgets', file)
  let content = fs.readFileSync(p, 'utf-8')
  if (!content.includes('vi.mock(\'vue-router\'')) {
    content = content.replace(/import { describe, it, expect.*} from 'vitest'/, 
`import { describe, it, expect, vi } from 'vitest'

vi.mock('vue-router', () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), back: vi.fn() }),
  useRoute: () => ({ params: { id: 'root' }, query: {}, path: '/' })
}))`)
    fs.writeFileSync(p, content)
  }
})
