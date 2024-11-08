import './assets/main.css'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import Tres from '@tresjs/core'
import App from './App.vue'
import router from './router'
import Vue3ProgressBar from "@ctechhindi/vue3-progress-bar";

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(Tres)
app.use(Vue3ProgressBar, {
  // "height": "5px",
  // "customStyle": {
  //   width: "100 %",
  //   height: "100 %",
  //   position: "absolute",
  //   background: "linear-gradient(45deg, #da7748, #d11111)",
  //   left: "0 %",
  // },
});app.mount('#app')
