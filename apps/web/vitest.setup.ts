import { config } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { createPinia, setActivePinia } from 'pinia'
import i18n from './src/languages'

const pinia = createPinia()
setActivePinia(pinia)

// Install ElementPlus, Pinia, and i18n globally for all tests
config.global.plugins = [ElementPlus, pinia, i18n]

// Mock window/document properties if needed
if (typeof window !== 'undefined') {
  window.URL.createObjectURL = () => ''
}
