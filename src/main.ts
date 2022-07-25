import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
// reset style sheet
import '@/styles/reset.scss'
// element plus
import ElementPlus from 'element-plus'
// element icons
import * as Icons from '@element-plus/icons-vue'
// element css
import 'element-plus/dist/index.css'
// element dark(内置暗黑模式)
import 'element-plus/theme-chalk/dark/css-vars.css'
// vue Router
import router from '@/routers/index'

const app = createApp(App)

// 注册element Icons组件
Object.keys(Icons).forEach(key => {
	app.component(key, Icons[key as keyof typeof Icons])
})

app.use(router).use(ElementPlus).mount('#app')
