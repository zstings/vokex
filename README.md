# Vokex

[![npm version](https://img.shields.io/npm/v/vokex.app)](https://www.npmjs.com/package/vokex.app)
[![license](https://img.shields.io/npm/l/vokex.app)](LICENSE)

超轻量级桌面应用构建库。将前端代码通过 Vite 构建后一键嵌入到预编译的 Rust 壳中，生成单个原生可执行文件（最小 ~1.8MB）。架构类似 Tauri 的极简版，壳基于 tao + wry（系统 WebView）。

目前仅支持 Windows。macOS、Linux 将在后续版本中添加。

## 特性

- **超轻量**：构建产物最小 ~1.8MB，仅依赖系统 WebView
- **零 Rust 门槛**：`npm install vokex.app` 即可使用，不需要 Rust 工具链
- **Vite 原生集成**：Vite 插件自动接管开发和构建流程
- **单文件输出**：前端资源 zlib 压缩后嵌入到可执行文件尾部
- **双模式运行**：开发时加载 localhost，生产时从自身尾部读取资源
- **丰富的 API**：16 个模块，120+ 个公开方法
- **TypeScript 优先**：完整的类型支持
- **安全沙箱**：远端页面默认禁用危险 API，支持白名单配置
- **远程窗口**：支持加载远端 URL 创建子窗口
- **文件搜索**：支持 glob 模式匹配（`*.txt`, `**/*.js`）
- **PE 图标注入**：支持将 .ico 图标注入 Windows 可执行文件

## 快速开始

### 1. 创建项目

```bash
npm create vite@latest my-app -- --template vanilla
cd my-app
npm install vokex.app
```

### 2. 配置 Vite

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { vokexPlugin } from "vokex.app/vite-plugin";

export default defineConfig({
  plugins: [
    vokexPlugin({
      name: "我的应用",
      identifier: "com.example.myapp",
      version: "1.0.0",
      icon: "icon.png",
      window: {
        title: "我的应用",
        width: 1200,
        height: 800,
      },
      devtools: process.env.NODE_ENV === 'development',
    }),
  ],
});
```

### 3. 使用 API

```typescript
import { app, fs, browserWindow, menu, dialog, shortcut } from "vokex.app";

// 应用就绪后执行
app.on("ready", async () => {
  const name = await app.getName();
  console.log(`${name} 已启动`);
});

// 文件操作
const content = await fs.readFile("config.json");
await fs.writeFile("output.txt", "Hello World");

// 窗口管理
const win = await browserWindow.create({ title: "子窗口", width: 600, height: 400 });

// 原生菜单
await menu.setApplicationMenu([
  { type: 'submenu', label: '文件', submenu: [
    { id: 'new', label: '新建' },
    { type: 'separator' },
    { type: 'native', nativeLabel: 'quit' },
  ]},
]);

// 对话框
const result = await dialog.showOpenDialog({
  filters: [{ name: "文本文件", extensions: ["txt", "md"] }],
});
```

### 4. 构建与开发

```bash
# 开发模式（启动 dev server + 原生壳）
npm run dev

# 构建打包（输出到 release/ 目录）
npm run build

# 验证构建产物
npx vokex validate release/我的应用.exe
```

## 架构

```
┌───────────────────────────────────────┐
│  前端代码 (HTML/JS/CSS)               │
│  import { app, fs } from "vokex.app"  │
├───────────────────────────────────────┤
│  运行时 Bridge (注入 JS)              │
│  window.__VOKEX__.call("fs.readFile") │
├───────────────────────────────────────┤
│  IPC: postMessage ↔ evaluate_script   │
├───────────────────────────────────────┤
│  Rust 壳 (wry + tao)                  │
│  窗口管理 │ API 路由 │ 资源加载        │
├───────────────────────────────────────┤
│  系统 WebView                         │
│  Windows: WebView2  macOS: WKWebView  │
└───────────────────────────────────────┘
```

**IPC 通信链路：**
```
前端: window.__VOKEX__.call("fs.readFile", [path])
  → window.ipc.postMessage(JSON)
    → Rust wry IPC handler
      → 主线程事件循环
        → dispatch 到对应 API 模块
          → 同步直接 eval / 异步线程池完成后 eval
            → window.__VOKEX_IPC__(response)
              → Promise resolve
```

## API 参考

> **完整的 TypeScript 类型定义和详细参数说明请参阅 [API 文档](api-doc.md)**

## 资源嵌入格式

二进制文件尾部追加：

```
[MAGIC(5B "VOKEX")] [索引长度(4B LE)] [索引JSON] [zlib压缩数据] [偏移量(8B LE)]
```

索引格式：`{ "index.html": [offset, length], "assets/main.js": [offset, length] }`

## 编译壳

如果你需要自己编译壳（而不是使用预编译版本）：

```bash
cd shell
cargo build --release
# 输出: shell/target/release/vokex-shell.exe (Windows)
```

## 许可证

MIT
