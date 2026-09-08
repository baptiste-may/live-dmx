import "./assets/main.css";

import { createPinia } from "pinia";
import { createApp } from "vue";

import Main from "./Main.vue";
import { useSettings } from "./stores/settings.ts";

const app = createApp(Main);
const pinia = createPinia();

app.use(pinia);
useSettings().initTheme();
app.mount("#app");
