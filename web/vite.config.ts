import { defineConfig } from "vite";
import { resolve } from "node:path";
import { readFileSync } from "node:fs";

// one version for every platform: the desktop app's
const version = JSON.parse(readFileSync(resolve(__dirname, "../desktop/src-tauri/tauri.conf.json"), "utf8")).version;

export default defineConfig({
  base: "./",
  define: { __APP_VERSION__: JSON.stringify(version) },
  build: {
    target: "es2022",
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        app: resolve(__dirname, "app/index.html"),
      },
    },
  },
  server: { port: 5173, strictPort: true },
});
