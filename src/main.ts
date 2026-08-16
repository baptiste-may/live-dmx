import "./assets/main.css";

import { createPinia } from "pinia";
import { createApp } from "vue";

import Main from "./Main.vue";

const app = createApp(Main);
const pinia = createPinia();

app.use(pinia);
app.mount("#app");
