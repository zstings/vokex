import { defineConfig } from "vite";
import { vokexPlugin } from "vokex.app/vite-plugin";
import { version } from "./package.json";

export default defineConfig(({mode}) => {
  return {
    plugins: [
      vokexPlugin({
        name: "Vokex Demo",
        identifier: "com.vokex.vokex",
        version: version,
        icon: [
          "icon/icon.ico",
          "icon/32x32.png",
        ],
        window: {
          title: "Vokex App Demo",
          width: 1200,
          height: 800,
          center: true,
          transparent: true,
        },
        verbose: true,
        devtools: mode == 'development',
        new_window: {
          value: 1
        },
        security: {
          allowed_remote_apis: ["fs.readFile", "computer.*"],
          allow_remote_pages: true
        }
      }),
    ],
    build: {
      rollupOptions: {
        input: {
          index: 'index.html',
          test: 'test.html',  // 加入构建
        }
      }
    }
  }
});
