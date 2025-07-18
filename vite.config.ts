import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
	plugins: [vue()],
	resolve: {
		alias: {
			'@': resolve(__dirname, './src')
		}
	},
	build: {
		minify: 'esbuild',
		reportCompressedSize: false,
		rollupOptions: {
			output: {
				advancedChunks: {
					groups: [{ name: 'element-plus', test: /\/element-plus?/ }]
				}
			}
		}
	},
	css: {
		postcss: {
			plugins: [
				{
					postcssPlugin: 'internal:charset-removal',
					AtRule: {
						charset: atRule => {
							if (atRule.name === 'charset') {
								atRule.remove()
							}
						}
					}
				}
			]
		}
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
				target: 'http://127.0.0.1:3000',
				changeOrigin: true,
				rewrite: path => path.replace(/^\/api/, '')
			}
		}
	}
})
