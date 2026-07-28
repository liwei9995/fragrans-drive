import { defineConfig, mergeConfig } from 'vitest/config'
import viteConfig from './vite.config'

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      server: {
        deps: {
          inline: ['element-plus'],
        },
      },
      globals: true,
      environment: 'jsdom',
      setupFiles: ['./vitest.setup.ts'],
      include: ['src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'],
      coverage: {
        provider: 'v8',
        reporter: ['text', 'json', 'html'],
        include: ['src/**/*.{ts,vue}'],
        exclude: [
          'src/**/*.d.ts',
          'src/auto-imports.d.ts',
          'src/components.d.ts',
          'src/main.ts',
          'src/routers/index.ts', // Exclude router since testing route changes in unit tests is often low value and requires e2e
          'src/routers/router.ts', // Static route declarations are covered by Playwright navigation tests
          'src/views/**/*.vue', // Views are tested via Playwright E2E
          'src/components/ErrorMessage/*.vue', // Simple static error pages
        ],
        thresholds: {
          statements: 90,
          branches: 88,
          functions: 85,
          lines: 90,
        },
      },
    },
  }),
)
