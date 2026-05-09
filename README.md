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
- **丰富的 API**：14 个模块，112 个公开方法
- **TypeScript 优先**：完整的类型支持
- **安全沙箱**：远端页面默认禁用危险 API，支持白名单配置
- **远程窗口**：支持加载远端 URL 创建子窗口
- **文件搜索**：支持 glob 模式匹配（`*.txt`, `**/*.js`）

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
import { app, fs, browserWindow, menu, dialog } from "vokex.app";

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
npx vokex.app validate release/我的应用.exe
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

所有 API 都是异步的，返回 Promise。

### app - 应用管理

```typescript
import { app } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `quit()` | 退出应用 |
| `exit(code?)` | 退出并返回指定退出码 |
| `restart()` | 重启应用 |
| `getAppPath()` | 获取应用可执行文件路径 |
| `getPath(name)` | 获取系统路径：`home` `appData` `desktop` `documents` `downloads` `temp` `cwd` |
| `getVersion()` | 获取版本号 |
| `getName()` | 获取应用名称 |
| `getIdentifier()` | 获取应用标识符 |
| `getLocale()` | 获取系统语言 |
| `getPid()` | 获取进程 PID |
| `getArgv()` | 获取启动参数 |
| `getEnv(key)` | 获取环境变量 |
| `getPlatform()` | 获取平台：`win32` `darwin` `linux` |
| `getArch()` | 获取架构：`x64` `arm64` |
| `requestSingleInstanceLock()` | 请求单实例锁，返回 `true` 表示是首个实例 |
| `on(event, callback)` | 监听应用事件 |

**事件：** `ready` `window-all-closed` `before-quit` `second-instance` `activate`

### browserWindow - 窗口管理

```typescript
import { browserWindow } from "vokex.app";
```

**创建窗口：**
```typescript
const win = await browserWindow.create({
  title: "子窗口",
  width: 800,
  height: 600,
  url: "page.html",     // 相对路径或完整 URL
  center: true,
  resizable: true,
  icon: "icon.png",
});
```

**静态方法：**

| 方法 | 说明 |
|---|---|
| `create(options?)` | 创建新窗口，返回 `BrowserWindow` 实例 |
| `getAll()` | 获取所有窗口信息 |
| `getFocused()` | 获取当前聚焦窗口信息 |
| `getById(id)` | 按 ID 获取窗口实例 |
| `getWindow(id)` | 同 `getById` |
| `getCurrentWindow()` | 获取当前窗口实例 |
| `getFocusedWindow()` | 获取当前聚焦窗口实例 |

**窗口实例方法：**

| 方法 | 说明 |
|---|---|
| `getId()` | 获取窗口 ID |
| `close()` | 关闭窗口 |
| `show()` / `hide()` | 显示 / 隐藏 |
| `minimize()` / `maximize()` / `restore()` | 最小化 / 最大化 / 恢复 |
| `unmaximize()` | 取消最大化 |
| `focus()` / `blur()` | 聚焦 / 失焦 |
| `isMaximized()` `isMinimized()` `isFullScreen()` `isFocused()` `isVisible()` | 状态查询 |
| `isResizable()` / `setResizable(flag)` | 可调整大小 |
| `isMinimizable()` / `setMinimizable(flag)` | 可最小化 |
| `isMaximizable()` / `setMaximizable(flag)` | 可最大化 |
| `isClosable()` / `setClosable(flag)` | 可关闭 |
| `isDecorated()` / `setDecorated(flag)` | 窗口装饰（标题栏） |
| `setFullScreen(flag)` | 全屏切换 |
| `setTitle(title)` / `getTitle()` | 设置 / 获取标题 |
| `setSize(w, h)` / `getSize()` | 设置 / 获取大小 |
| `setMinimumSize(w, h)` / `setMaximumSize(w, h)` | 限制窗口尺寸范围 |
| `setPosition(x, y)` / `getPosition()` | 设置 / 获取位置 |
| `center()` | 窗口居中 |
| `setOpacity(opacity)` | 设置透明度 (0.0~1.0) |
| `setBackgroundColor(color)` | 设置背景色 |
| `setIcon(icon)` | 设置窗口图标 |
| `setAlwaysOnTop(flag)` | 置顶 |
| `setAlwaysOnBottom(flag)` | 置底 |
| `loadFile(path)` / `loadURL(url)` | 加载本地文件 / URL |
| `reload()` | 重新加载 |
| `setProgressBar(progress)` | 任务栏进度条 (0.0~1.0) [Windows] |
| `setSkipTaskbar(flag)` | 隐藏任务栏图标 |
| `flashTaskbar(flag)` | 闪烁任务栏 |
| `requestUserAttention(level?)` | 请求用户关注 |
| `setContentProtection(enabled)` | 内容保护（防截图） |
| `setVisibleOnAllWorkspaces(visible)` | 所有工作区可见 |
| `scaleFactor()` | 获取缩放因子 |
| `getInnerPosition()` | 获取客户区位置 |
| `getOuterSize()` | 获取窗口外部大小 |
| `setCursorIcon(icon)` | 设置光标图标 |
| `setCursorPosition(x, y)` | 移动光标 |
| `setCursorGrab(grab)` | 锁定光标 |
| `setCursorVisible(visible)` | 隐藏/显示光标 |
| `sendMessage(message, targetWindow)` | 窗口间通信 |
| `on(event, callback)` / `off(event, callback)` | 监听 / 移除事件 |

**事件：** `close` `resize` `move` `minimize` `maximize` `restore` `focus` `blur` `enter-full-screen` `leave-full-screen` `window.message`

### fs - 文件系统

```typescript
import { fs } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `readFile(path)` | 读取文件为文本 |
| `readFileBinary(path)` | 读取文件为 Base64 |
| `writeFile(path, data)` | 写入文件 |
| `appendFile(path, data)` | 追加内容 |
| `deleteFile(path)` | 删除文件 |
| `readDir(path)` | 列出目录内容，返回 `DirEntry[]` |
| `createDir(path)` | 创建目录 |
| `removeDir(path)` | 删除目录 |
| `stat(path)` | 获取文件信息 |
| `exists(path)` | 检查文件是否存在 |
| `copyFile(src, dest)` | 复制文件 |
| `moveFile(src, dest)` | 移动 / 重命名文件 |
| `glob(options)` | glob 模式搜索文件 |

**GlobOptions：**

| 选项 | 类型 | 说明 |
|---|---|---|
| `pattern` | `string` | glob 模式，如 `*.txt`, `**/*.js` |
| `cwd` | `string?` | 搜索目录，默认当前目录 |
| `ignore` | `string[]?` | 排除模式列表 |
| `nodir` | `boolean?` | 只返回文件，不返回目录 |
| `absolute` | `boolean?` | 返回绝对路径 |
| `dot` | `boolean?` | 是否包含隐藏文件（以 `.` 开头） |

### http - 网络请求

```typescript
import { http } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `request(url, options?)` | 通用请求 |
| `get(url, options?)` | GET 请求 |
| `post(url, data?, options?)` | POST 请求 |
| `put(url, data?, options?)` | PUT 请求 |
| `delete(url, options?)` | DELETE 请求 |

**RequestOptions：** `method` `headers` `body` `timeout`

**HttpResponse：** `statusCode` `statusText` `headers` `body` `ok`

### dialog - 对话框

```typescript
import { dialog } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `showMessageBox(options)` | 消息对话框，返回 `{ response, cancelled }` |
| `showErrorBox(options)` | 错误对话框 |
| `showOpenDialog(options?)` | 打开文件对话框，返回文件路径或 `null` |
| `showSaveDialog(options?)` | 保存文件对话框，返回文件路径或 `null` |

**showMessageBox 支持的 type：** `none` `okCancel` `yesNo` `yesNoCancel`
**showMessageBox 支持的 icon：** `info` `warning` `error`

**对话框选项：** `title` `defaultPath` `defaultName` `multiple` `directory` `filters`

### menu - 原生菜单

```typescript
import { menu } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `setApplicationMenu(menu)` | 设置应用顶部菜单栏 |
| `removeApplicationMenu()` | 移除应用菜单栏 |
| `setContextMenu(menu, x?, y?)` | 弹出右键上下文菜单 |
| `onMenuClick(callback)` | 监听菜单项点击事件 |

**菜单项类型：**

| type | 说明 |
|---|---|
| `normal`（默认） | 普通菜单项，需要 `id` 和 `label` |
| `separator` | 分隔线 |
| `submenu` | 子菜单，需要 `label` 和 `submenu` |
| `checkbox` | 复选框菜单项，支持 `checked` 属性 |
| `native` | 系统原生菜单项，需要 `nativeLabel` |

**NativeLabel（原生菜单项）：** `separator` `copy` `cut` `paste` `selectAll` `undo` `redo` `minimize` `maximize` `fullscreen` `hide` `hideOthers` `showAll` `closeWindow` `quit` `about` `services` `bringAllToFront`

**示例：**
```typescript
// 应用菜单栏
await menu.setApplicationMenu([
  { type: 'submenu', label: '文件', submenu: [
    { id: 'new', label: '新建' },
    { type: 'separator' },
    { type: 'native', nativeLabel: 'quit' },
  ]},
  { type: 'submenu', label: '编辑', submenu: [
    { type: 'native', nativeLabel: 'copy' },
    { type: 'native', nativeLabel: 'paste' },
  ]},
]);

// 右键菜单
await menu.setContextMenu([
  { id: 'copy', label: '复制' },
  { id: 'paste', label: '粘贴' },
], e.x, e.y);

// 监听点击
menu.onMenuClick(({ id }) => {
  console.log('点击了:', id);
});
```

### tray - 系统托盘

```typescript
import { tray } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `tray.create(options)` | 创建托盘图标，返回 `Tray` 实例 |

**TrayOptions：** `icon`（必填） `tooltip` `title` `menu`

**Tray 实例方法：**

| 方法 | 说明 |
|---|---|
| `getId()` | 获取托盘 ID |
| `setToolTip(text)` | 设置提示文本 |
| `setTitle(title)` | 设置标题 (macOS) |
| `setMenu(template)` | 更新右键菜单 |
| `setImage(icon)` | 更新图标 |
| `destroy()` | 销毁托盘 |
| `on(event, callback)` | 监听事件 |

**事件：** `click` `right-click` `double-click`

**示例：**
```typescript
const myTray = await tray.create({
  icon: "icon.png",
  tooltip: "我的应用",
  menu: [
    { id: "show", label: "显示窗口" },
    { type: "separator" },
    { id: "quit", label: "退出" },
  ],
});

myTray.on("click", () => {
  // 托盘左键点击
});
```

### clipboard - 剪贴板

```typescript
import { clipboard } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `readText()` | 读取剪贴板文本 |
| `writeText(text)` | 写入剪贴板 |
| `clear()` | 清空剪贴板 |

### notification - 系统通知

```typescript
import { notification } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `show(options)` | 发送系统通知 |

**NotificationOptions：** `title`（必填） `body`

### shell - 系统命令

```typescript
import { shell } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `openExternal(url)` | 用默认浏览器打开 URL |
| `openPath(path)` | 用系统默认程序打开文件/目录 |
| `execCommand(command, options?)` | 执行 shell 命令，返回 `{ code, stdout, stderr, success }` |
| `trashItem(path)` | 移到回收站 |

**ExecOptions：** `cwd` `env`

### process - 进程信息

```typescript
import { process } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `getUptime()` | 获取进程运行时长（秒） |
| `getCpuUsage()` | 获取 CPU 使用率，返回 `{ user, system }` |
| `getMemoryInfo()` | 获取内存信息，返回 `{ total, available, used }` |
| `hostname()` | 获取主机名 |
| `env()` | 获取所有环境变量 |
| `kill(pid)` | 终止指定进程 |

### computer - 系统硬件信息

```typescript
import { computer } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `getCpuInfo()` | CPU 信息：制造商、型号、核心数、频率等 |
| `getOsInfo()` | 操作系统信息：名称、版本、内核版本等 |
| `getDisplays()` | 显示器信息列表 |
| `getMousePosition()` | 鼠标位置 |
| `getKeyboardLayout()` | 键盘布局 |

### storage - 本地持久化存储

```typescript
import { storage } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `setData(key, value)` | 存储数据（JSON 序列化） |
| `getData(key)` | 读取数据 |
| `getKeys()` | 获取所有键名 |
| `has(key)` | 检查键是否存在 |
| `removeData(key)` | 删除指定键 |
| `clear()` | 清空所有数据 |

### events - 事件总线

```typescript
import { events } from "vokex.app";
```

| 方法 | 说明 |
|---|---|
| `on(event, listener)` | 监听事件，返回取消监听函数 |
| `off(event, listener)` | 移除监听 |
| `emit(event, data?)` | 触发事件 |

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
