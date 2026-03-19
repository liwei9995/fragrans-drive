import { resolve } from 'node:path'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'
import Components from 'unplugin-vue-components/vite'
import { defineConfig } from 'vite'
import checker from 'vite-plugin-checker'
import compression from 'vite-plugin-compression'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    AutoImport({
      imports: ['vue', 'vue-router', 'pinia'],
      resolvers: [ElementPlusResolver()],
      dts: 'src/auto-imports.d.ts',
    }),
    Components({
      resolvers: [ElementPlusResolver()],
      dts: 'src/components.d.ts',
    }),
    checker({
      vueTsc: true,
    }),
    compression({
      ext: '.gz',
      deleteOriginFile: false,
    }),
    compression({
      ext: '.gz',
      deleteOriginFile: false,
    }),
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  build: {
    reportCompressedSize: false,
    rollupOptions: {
      output: {
        chunkFileNames: 'static/js/[name]-[hash].js',
        entryFileNames: 'static/js/[name]-[hash].js',
        assetFileNames: 'static/[ext]/[name]-[hash].[ext]',
      },
    },
  },
  css: {
    postcss: {
      plugins: [
        {
          postcssPlugin: 'internal:charset-removal',
          AtRule: {
            charset: (atRule) => {
              if (atRule.name === 'charset') {
                atRule.remove()
              }
            },
          },
        },
      ],
    },
  },
  server: {
    host: '0.0.0.0', // 服务器主机名，如果允许外部访问，可设置为"0.0.0.0"
    open: true,
    cors: true,
    // https: false,
    // 代理跨域（mock 不需要配置，这里只是个事列）
    proxy: {
      '/api': {
        // target: "https://www.fastmock.site/mock/f81e8333c1a9276214bcdbc170d9e0a0", // fastmock
        target: 'http://127.0.0.1:3821',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})
