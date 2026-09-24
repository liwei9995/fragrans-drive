/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'

  const component: DefineComponent<
    Record<string, unknown>,
    Record<string, unknown>,
    unknown
  >
  export default component
}

interface Window {
  turnstile?: {
    render: (
      container: string | HTMLElement,
      params: {
        sitekey: string
        theme?: 'auto' | 'light' | 'dark'
        callback?: (token: string) => void
        'expired-callback'?: () => void
        'error-callback'?: () => void
      },
    ) => string
    reset: (widgetId?: string) => void
    remove: (widgetId?: string) => void
  }
}
