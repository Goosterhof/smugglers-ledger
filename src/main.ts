import { createApp } from "vue";
// The reset must land BEFORE the uno layer: without it the webview paints
// every <button> in the UA's dark buttonface gray — the slab the Clotted
// Chrome pass (#00013 v0.7.0) was summoned to kill.
import "@unocss/reset/tailwind.css";
import "virtual:uno.css";
import "./assets/fonts.css";
import App from "./App.vue";

createApp(App).mount("#app");
