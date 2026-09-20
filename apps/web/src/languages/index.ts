import { createI18n } from 'vue-i18n'
import en from './modules/en'
import zh from './modules/zh'

export const LANGUAGE_KEY = 'fragrans_language'

export function getStoredLanguage(): 'zh' | 'en' {
  if (typeof window === 'undefined') return 'zh'
  try {
    const directLang = window.localStorage.getItem(LANGUAGE_KEY)
    if (directLang === 'en' || directLang === 'zh') {
      return directLang
    }
    const rawGlobalState = window.localStorage.getItem('GlobalState')
    if (rawGlobalState) {
      const parsed = JSON.parse(rawGlobalState)
      if (parsed.language === 'en' || parsed.language === 'zh') {
        return parsed.language
      }
    }
  } catch {
    // fallback
  }
  return 'zh'
}

const i18n = createI18n({
  legacy: false,
  locale: getStoredLanguage(),
  fallbackLocale: 'en',
  messages: {
    zh,
    en,
  },
})

export default i18n
