// element plus
import { createApp } from 'vue'
import App from './App.vue'
// reset style sheet
import '@/styles/reset.scss'
// app style sheet
import './style.css'
// element css
import 'element-plus/theme-chalk/el-message-box.css'
import 'element-plus/theme-chalk/el-message.css'
import 'element-plus/theme-chalk/el-notification.css'
import 'element-plus/theme-chalk/el-overlay.css'
// element dark(内置暗黑模式)
import 'element-plus/theme-chalk/dark/css-vars.css'
// element 基于断点的隐藏类
import 'element-plus/theme-chalk/display.css'
// vue Router
import router from '@/routers/index'
// pinia store
import pinia from '@/store/index'

const app = createApp(App)

app.use(router).use(pinia).mount('#app')
