import { createApp } from "vue";
import App from "./App.vue";
import Antd from 'ant-design-vue';
import 'ant-design-vue/dist/reset.css';
import Router from "./router";
import { loadSettings } from "./stores/settingStore";

// 预加载设置（先用默认值渲染避免闪烁，异步从磁盘读取后更新）
loadSettings();

const app = createApp(App);
app.config.productionTip = false;
app.use(Antd).use(Router).mount('#app');
