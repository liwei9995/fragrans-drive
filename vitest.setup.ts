import { config } from '@vue/test-utils'
import ElementPlus from 'element-plus'

// Install ElementPlus globally for all tests
config.global.plugins = [ElementPlus]

// Mock window/document properties if needed
if (typeof window !== 'undefined') {
  window.URL.createObjectURL = () => ''
}
