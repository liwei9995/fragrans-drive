import { createApp } from 'vue'
import App from './App.vue'
// element plus
import ElementPlus from 'element-plus'
// reset style sheet
import '@/styles/reset.scss'
// app style sheet
import './style.css'
// element icons
import * as Icons from '@element-plus/icons-vue'
// element css
import 'element-plus/dist/index.css'
// element dark(内置暗黑模式)
import 'element-plus/theme-chalk/dark/css-vars.css'
// element 基于断点的隐藏类
import 'element-plus/theme-chalk/display.css'
// vue Router
import router from '@/routers/index'
// pinia store
import pinia from '@/store/index'

const app = createApp(App)

// 注册element Icons组件
Object.keys(Icons).forEach(key => {
	app.component(key, Icons[key as keyof typeof Icons])
})

app.use(router).use(pinia).use(ElementPlus).mount('#app')
