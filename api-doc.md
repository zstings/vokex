# Vokex API 完整文档

本文档包含 Vokex 所有 API 模块的完整 TypeScript 类型定义、参数说明和使用示例。

## 目录

- [app - 应用管理](#app---应用管理)
- [browserWindow - 窗口管理](#browserwindow---窗口管理)
- [fs - 文件系统](#fs---文件系统)
- [http - 网络请求](#http---网络请求)
- [dialog - 对话框](#dialog---对话框)
- [menu - 原生菜单](#menu---原生菜单)
- [tray - 系统托盘](#tray---系统托盘)
- [shortcut - 全局快捷键](#shortcut---全局快捷键)
- [shell - 系统命令](#shell---系统命令)
- [clipboard - 剪贴板](#clipboard---剪贴板)
- [notification - 系统通知](#notification---系统通知)
- [process - 进程信息](#process---进程信息)
- [computer - 系统硬件信息](#computer---系统硬件信息)
- [storage - 本地持久化存储](#storage---本地持久化存储)
- [safeStorage - 安全持久化存储](#safestorage---安全持久化存储)
- [events - 事件总线](#events---事件总线)
- [path - 路径操作](#path---路径操作)

---

## app - 应用管理

```typescript
import { app } from "vokex.app";
```

### 类型定义

```typescript
type AppEvent = 'ready' | 'window-all-closed' | 'before-quit' | 'second-instance' | 'activate';

interface AppAPI {
  quit(): Promise<void>;
  exit(code?: number): Promise<void>;
  restart(): Promise<void>;
  getAppPath(): Promise<string>;
  getPath(name: "home" | "appData" | "desktop" | "documents" | "downloads" | "temp" | "cwd"): Promise<string>;
  getVersion(): Promise<string>;
  getName(): Promise<string>;
  getIdentifier(): Promise<string>;
  getLocale(): Promise<string>;
  getPid(): Promise<number>;
  getArgv(): Promise<string[]>;
  getEnv(key: string): Promise<string>;
  getPlatform(): Promise<string>;
  getArch(): Promise<string>;
  requestSingleInstanceLock(): Promise<boolean>;
  on(event: AppEvent, callback: (data?: any) => void): void;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `quit()` | 无 | `Promise<void>` | 退出应用，触发 `before-quit` 事件 |
| `exit(code?)` | `code?: number` - 退出码，默认 0 | `Promise<void>` | 立即退出应用，不触发生命周期事件 |
| `restart()` | 无 | `Promise<void>` | 重启应用 |
| `getAppPath()` | 无 | `Promise<string>` | 获取应用可执行文件所在目录路径 |
| `getPath(name)` | `name` - 目录名称 | `Promise<string>` | 获取系统特殊目录路径 |
| `getVersion()` | 无 | `Promise<string>` | 获取应用版本号 |
| `getName()` | 无 | `Promise<string>` | 获取应用名称 |
| `getIdentifier()` | 无 | `Promise<string>` | 获取应用标识符 |
| `getLocale()` | 无 | `Promise<string>` | 获取系统语言标识，如 `zh-CN`、`en-US` |
| `getPid()` | 无 | `Promise<number>` | 获取当前进程 PID |
| `getArgv()` | 无 | `Promise<string[]>` | 获取命令行参数数组 |
| `getEnv(key)` | `key: string` - 环境变量名 | `Promise<string>` | 获取环境变量值 |
| `getPlatform()` | 无 | `Promise<string>` | 获取操作系统类型：`win32`、`darwin`、`linux` |
| `getArch()` | 无 | `Promise<string>` | 获取系统架构：`x64`、`arm64` |
| `requestSingleInstanceLock()` | 无 | `Promise<boolean>` | 请求单实例锁，返回 `true` 表示是首个实例 |
| `on(event, callback)` | `event` - 事件名，`callback` - 回调函数 | `void` | 监听应用事件 |

### getPath 支持的目录名

| 名称 | 说明 |
|------|------|
| `home` | 用户主目录 |
| `appData` | 应用数据目录 |
| `desktop` | 桌面目录 |
| `documents` | 文档目录 |
| `downloads` | 下载目录 |
| `temp` | 临时目录 |
| `cwd` | 当前工作目录 |

### 应用事件

| 事件 | 说明 |
|------|------|
| `ready` | 应用就绪 |
| `window-all-closed` | 所有窗口已关闭 |
| `before-quit` | 应用即将退出 |
| `second-instance` | 第二个实例启动（单实例模式下） |
| `activate` | 应用被激活（macOS） |

### 使用示例

```typescript
import { app } from "vokex.app";

app.on("ready", async () => {
  const name = await app.getName();
  const version = await app.getVersion();
  console.log(`${name} v${version} 已启动`);
  
  const homePath = await app.getPath("home");
  console.log("用户主目录:", homePath);
});

const isFirst = await app.requestSingleInstanceLock();
if (!isFirst) {
  console.log("已有实例运行，退出");
  await app.quit();
}
```

---

## browserWindow - 窗口管理

```typescript
import { browserWindow } from "vokex.app";
```

### 类型定义

```typescript
interface WindowOptions {
  title?: string;
  width?: number;
  height?: number;
  x?: number;
  y?: number;
  resizable?: boolean;
  minimizable?: boolean;
  maximizable?: boolean;
  closable?: boolean;
  alwaysOnTop?: boolean;
  fullscreen?: boolean;
  skipTaskbar?: boolean;
  opacity?: number;
  backgroundColor?: string;
  minWidth?: number;
  minHeight?: number;
  maxWidth?: number;
  maxHeight?: number;
  icon?: string;
  show?: boolean;
  center?: boolean;
  url?: string;
}

interface WindowInfo {
  id: number;
  title: string;
  width: number;
  height: number;
  x: number;
  y: number;
  is_maximized: boolean;
  is_minimized: boolean;
  is_fullscreen: boolean;
  is_focused: boolean;
  is_visible: boolean;
}

type WindowEventType =
  | 'close'
  | 'resize'
  | 'move'
  | 'minimize'
  | 'maximize'
  | 'restore'
  | 'focus'
  | 'blur'
  | 'enter-full-screen'
  | 'leave-full-screen';

class BrowserWindow {
  getId(): number;
  close(): Promise<void>;
  show(): Promise<void>;
  hide(): Promise<void>;
  minimize(): Promise<void>;
  maximize(): Promise<void>;
  unmaximize(): Promise<void>;
  restore(): Promise<void>;
  focus(): Promise<void>;
  blur(): Promise<void>;
  isMaximized(): Promise<boolean>;
  isMinimized(): Promise<boolean>;
  isFullScreen(): Promise<boolean>;
  isFocused(): Promise<boolean>;
  isVisible(): Promise<boolean>;
  isResizable(): Promise<boolean>;
  setFullScreen(flag: boolean): Promise<void>;
  setTitle(title: string): Promise<void>;
  getTitle(): Promise<string>;
  setSize(width: number, height: number): Promise<void>;
  getSize(): Promise<[number, number]>;
  setMinimumSize(width: number, height: number): Promise<void>;
  setMaximumSize(width: number, height: number): Promise<void>;
  setResizable(flag: boolean): Promise<void>;
  setAlwaysOnTop(flag: boolean): Promise<void>;
  setAlwaysOnBottom(flag: boolean): Promise<void>;
  setPosition(x: number, y: number): Promise<void>;
  getPosition(): Promise<[number, number]>;
  center(): Promise<void>;
  setOpacity(opacity: number): Promise<void>;
  setBackgroundColor(color: string): Promise<void>;
  setIcon(icon: string): Promise<void>;
  loadFile(path: string): Promise<void>;
  loadURL(url: string): Promise<void>;
  reload(): Promise<void>;
  setProgressBar(progress: number): Promise<void>;
  setSkipTaskbar(flag: boolean): Promise<void>;
  flashTaskbar(flag: boolean): Promise<void>;
  scaleFactor(): Promise<number>;
  getInnerPosition(): Promise<{ x: number; y: number }>;
  getOuterSize(): Promise<{ width: number; height: number }>;
  isMinimizable(): Promise<boolean>;
  setMinimizable(flag: boolean): Promise<void>;
  isMaximizable(): Promise<boolean>;
  setMaximizable(flag: boolean): Promise<void>;
  isClosable(): Promise<boolean>;
  setClosable(flag: boolean): Promise<void>;
  isDecorated(): Promise<boolean>;
  setDecorated(flag: boolean): Promise<void>;
  requestUserAttention(level?: 'normal' | 'informational' | 'critical'): Promise<void>;
  setContentProtection(enabled: boolean): Promise<void>;
  setVisibleOnAllWorkspaces(visible: boolean): Promise<void>;
  setCursorIcon(icon: string): Promise<void>;
  setCursorPosition(x: number, y: number): Promise<void>;
  setCursorGrab(grab: boolean): Promise<void>;
  setCursorVisible(visible: boolean): Promise<void>;
  sendMessage(message: any, targetWindow: BrowserWindow): Promise<void>;
  on(event: WindowEventType | 'window.message', callback: (data?: any) => void): () => void;
  off(event: WindowEventType | 'window.message', callback: (data?: any) => void): void;
}
```

### 静态方法

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `create(options?)` | `options?: WindowOptions` | `Promise<BrowserWindow>` | 创建新窗口 |
| `getAll()` | 无 | `Promise<WindowInfo[]>` | 获取所有窗口信息 |
| `getFocused()` | 无 | `Promise<WindowInfo \| null>` | 获取当前聚焦窗口信息 |
| `getById(id)` | `id: number` | `Promise<BrowserWindow \| null>` | 按 ID 获取窗口实例 |
| `getWindow(id)` | `id: number` | `Promise<BrowserWindow \| null>` | 同 `getById` |
| `getCurrentWindow()` | 无 | `BrowserWindow \| null` | 获取当前窗口实例（同步） |
| `getFocusedWindow()` | 无 | `Promise<BrowserWindow \| null>` | 获取当前聚焦窗口实例 |

### WindowOptions 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `title` | `string` | - | 窗口标题 |
| `width` | `number` | - | 窗口宽度 |
| `height` | `number` | - | 窗口高度 |
| `x` | `number` | - | 窗口 X 坐标 |
| `y` | `number` | - | 窗口 Y 坐标 |
| `resizable` | `boolean` | `true` | 是否可调整大小 |
| `minimizable` | `boolean` | `true` | 是否可最小化 |
| `maximizable` | `boolean` | `true` | 是否可最大化 |
| `closable` | `boolean` | `true` | 是否可关闭 |
| `alwaysOnTop` | `boolean` | `false` | 是否置顶 |
| `fullscreen` | `boolean` | `false` | 是否全屏 |
| `skipTaskbar` | `boolean` | `false` | 是否隐藏任务栏图标 |
| `opacity` | `number` | `1.0` | 窗口透明度 (0.0~1.0) |
| `backgroundColor` | `string` | - | 背景色，如 `#FFFFFF` |
| `minWidth` | `number` | - | 最小宽度 |
| `minHeight` | `number` | - | 最小高度 |
| `maxWidth` | `number` | - | 最大宽度 |
| `maxHeight` | `number` | - | 最大高度 |
| `icon` | `string` | - | 窗口图标路径 |
| `show` | `boolean` | `true` | 是否显示 |
| `center` | `boolean` | `false` | 是否居中 |
| `url` | `string` | - | 加载的 URL 或本地文件路径 |

### 窗口事件

| 事件 | 说明 |
|------|------|
| `close` | 窗口关闭 |
| `resize` | 窗口大小改变 |
| `move` | 窗口移动 |
| `minimize` | 窗口最小化 |
| `maximize` | 窗口最大化 |
| `restore` | 窗口恢复 |
| `focus` | 窗口获得焦点 |
| `blur` | 窗口失去焦点 |
| `enter-full-screen` | 进入全屏 |
| `leave-full-screen` | 退出全屏 |
| `window.message` | 收到其他窗口消息 |

### 使用示例

```typescript
import { browserWindow } from "vokex.app";

const win = await browserWindow.create({
  title: "子窗口",
  width: 800,
  height: 600,
  center: true,
  resizable: true,
  url: "page.html",
});

win.on("close", () => {
  console.log("窗口已关闭");
});

await win.setProgressBar(0.5);
await win.flashTaskbar(true);

const currentWin = browserWindow.getCurrentWindow();
if (currentWin) {
  await currentWin.sendMessage({ type: "greeting", data: "Hello" }, win);
}
```

---

## fs - 文件系统

```typescript
import { fs } from "vokex.app";
```

### 类型定义

```typescript
interface Stats {
  isFile: boolean;
  isDir: boolean;
  isSymlink: boolean;
  size: number;
  atimeMs: number;
  mtimeMs: number;
  birthtimeMs: number;
  mode: number;
}

interface Dirent {
  name: string;
  isFile: boolean;
  isDir: boolean;
  isSymlink: boolean;
}

interface ReadFileOptions {
  encoding?: "utf8" | "base64" | "hex" | null;
}

interface WriteFileOptions {
  flag?: "w" | "a" | "wx";
  mode?: number;
  encoding?: "utf8" | "base64";
}

interface RmOptions {
  recursive?: boolean;
  force?: boolean;
}

interface ReaddirOptions {
  withFileTypes?: boolean;
}

interface MkdirOptions {
  recursive?: boolean;
}

interface GlobOptions {
  pattern: string;
  cwd?: string;
  ignore?: string[];
  nodir?: boolean;
  absolute?: boolean;
  dot?: boolean;
}

interface GlobStreamCallbacks {
  onMatch: (path: string, index: number) => void;
  onDone: (total: number) => void;
  onError?: (error: Error) => void;
}

interface FsAPI {
  readFile(path: string): Promise<Uint8Array>;
  readFile(path: string, options: { encoding: "utf8" }): Promise<string>;
  readFile(path: string, options: { encoding: "base64" }): Promise<string>;
  readFile(path: string, options: { encoding: "hex" }): Promise<string>;
  readFile(path: string, options?: ReadFileOptions): Promise<Uint8Array | string>;
  
  writeFile(path: string, data: string | Uint8Array, options?: WriteFileOptions): Promise<void>;
  rm(path: string, options?: RmOptions): Promise<void>;
  readdir(path: string, options?: ReaddirOptions): Promise<string[] | Dirent[]>;
  mkdir(path: string, options?: MkdirOptions): Promise<void>;
  stat(path: string): Promise<Stats>;
  exists(path: string): Promise<boolean>;
  copyFile(src: string, dest: string): Promise<void>;
  rename(oldPath: string, newPath: string): Promise<void>;
  glob(options: GlobOptions): Promise<string[]>;
  globStream(options: GlobOptions, callbacks: GlobStreamCallbacks): Promise<string>;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `readFile(path, options?)` | `path: string`, `options?: ReadFileOptions` | `Promise<Uint8Array \| string>` | 读取文件内容 |
| `writeFile(path, data, options?)` | `path: string`, `data: string \| Uint8Array`, `options?: WriteFileOptions` | `Promise<void>` | 写入文件 |
| `rm(path, options?)` | `path: string`, `options?: RmOptions` | `Promise<void>` | 删除文件或目录 |
| `readdir(path, options?)` | `path: string`, `options?: ReaddirOptions` | `Promise<string[] \| Dirent[]>` | 列出目录内容 |
| `mkdir(path, options?)` | `path: string`, `options?: MkdirOptions` | `Promise<void>` | 创建目录 |
| `stat(path)` | `path: string` | `Promise<Stats>` | 获取文件信息 |
| `exists(path)` | `path: string` | `Promise<boolean>` | 检查路径是否存在 |
| `copyFile(src, dest)` | `src: string`, `dest: string` | `Promise<void>` | 复制文件 |
| `rename(oldPath, newPath)` | `oldPath: string`, `newPath: string` | `Promise<void>` | 重命名/移动文件 |
| `glob(options)` | `options: GlobOptions` | `Promise<string[]>` | glob 模式搜索文件 |
| `globStream(options, callbacks)` | `options: GlobOptions`, `callbacks: GlobStreamCallbacks` | `Promise<string>` | 流式 glob 搜索 |

### ReadFileOptions

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `encoding` | `"utf8" \| "base64" \| "hex" \| null` | `null` | 编码方式，为空返回 `Uint8Array` |

### WriteFileOptions

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `flag` | `"w" \| "a" \| "wx"` | `"w"` | 写入模式：`w` 覆盖，`a` 追加，`wx` 排他创建 |
| `mode` | `number` | - | 文件权限（仅 Unix） |
| `encoding` | `"utf8" \| "base64"` | `"utf8"` | 数据编码 |

### RmOptions

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `recursive` | `boolean` | `false` | 是否递归删除目录 |
| `force` | `boolean` | `false` | 路径不存在时不报错 |

### GlobOptions

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `pattern` | `string` | - | glob 模式，如 `*.txt`、`**/*.js` |
| `cwd` | `string` | 当前目录 | 搜索目录 |
| `ignore` | `string[]` | - | 排除模式列表 |
| `nodir` | `boolean` | `false` | 只返回文件，不返回目录 |
| `absolute` | `boolean` | `false` | 返回绝对路径 |
| `dot` | `boolean` | `false` | 是否包含隐藏文件 |

### 使用示例

```typescript
import { fs } from "vokex.app";

const content = await fs.readFile("config.json", { encoding: "utf8" });
const config = JSON.parse(content);

await fs.writeFile("output.txt", "Hello World", { flag: "a" });

const files = await fs.readdir("./src");
const entries = await fs.readdir("./src", { withFileTypes: true });

const jsFiles = await fs.glob({
  pattern: "**/*.js",
  cwd: "./src",
  ignore: ["**/node_modules/**"],
});

const stat = await fs.stat("file.txt");
console.log(`文件大小: ${stat.size} 字节`);
console.log(`最后修改: ${new Date(stat.mtimeMs)}`);

await fs.rm("./dist", { recursive: true, force: true });
await fs.mkdir("./new-dir", { recursive: true });
```

---

## http - 网络请求

```typescript
import { http } from "vokex.app";
```

### 类型定义

```typescript
interface HttpInit extends Omit<RequestInit, 'body'> {
  body?: any;
  stream?: boolean;
  timeout?: number;
}

interface HttpAPI {
  fetch(url: string | URL, init?: HttpInit): Promise<Response>;
  get(url: string, init?: Omit<HttpInit, 'method' | 'body'>): Promise<Response>;
  post(url: string, body?: any, init?: Omit<HttpInit, 'method' | 'body'>): Promise<Response>;
  put(url: string, body?: any, init?: Omit<HttpInit, 'method' | 'body'>): Promise<Response>;
  delete(url: string, init?: Omit<HttpInit, 'method' | 'body'>): Promise<Response>;
}
```

### HttpInit 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `method` | `string` | `"GET"` | 请求方法 |
| `headers` | `HeadersInit` | - | 请求头 |
| `body` | `any` | - | 请求体，支持 `string`、纯对象（自动 JSON 序列化）、`FormData` |
| `stream` | `boolean` | `false` | 是否启用流式（SSE）模式 |
| `timeout` | `number` | - | 超时时间（秒） |

### Body 自动处理规则

| body 类型 | 处理方式 | 自动添加的 Content-Type |
|-----------|----------|------------------------|
| `string` | 原样传递 | 不自动添加 |
| 纯对象 `{}` | `JSON.stringify()` | `application/json` |
| `FormData` | 转 URL-encoded | `application/x-www-form-urlencoded` |

### 使用示例

```typescript
import { http } from "vokex.app";

const res = await http.get("https://api.example.com/data");
const data = await res.json();

const res2 = await http.post("https://api.example.com/users", {
  name: "Alice",
  email: "alice@example.com",
});

const res3 = await http.get("https://api.example.com/data", {
  headers: { "Authorization": "Bearer token" },
  timeout: 30,
});

const sseRes = await http.fetch("https://api.example.com/chat", {
  method: "POST",
  body: { prompt: "Hello" },
  stream: true,
});
const reader = sseRes.body!.getReader();
```

---

## dialog - 对话框

```typescript
import { dialog } from "vokex.app";
```

### 类型定义

```typescript
interface FileFilter {
  name: string;
  extensions: string[];
}

interface MessageBoxOptions {
  title?: string;
  message: string;
  type?: 'none' | 'okCancel' | 'yesNo' | 'yesNoCancel';
  icon?: 'info' | 'warning' | 'error';
}

interface CustomMessageBoxOptions {
  title?: string;
  message: string;
  buttons: string[];
  icon?: 'info' | 'warning' | 'error';
}

interface MessageBoxResult {
  response: 'ok' | 'cancel' | 'yes' | 'no';
  cancelled: boolean;
}

interface OpenDialogOptions {
  title?: string;
  defaultPath?: string;
  defaultName?: string;
  multiple?: boolean;
  directory?: boolean;
  filters?: FileFilter[];
}

interface SaveDialogOptions {
  title?: string;
  defaultPath?: string;
  defaultName?: string;
  filters?: FileFilter[];
}

interface ErrorBoxOptions {
  title?: string;
  message: string;
}

interface DialogAPI {
  showMessageBox(options: CustomMessageBoxOptions): Promise<number>;
  showMessageBox(options: MessageBoxOptions): Promise<MessageBoxResult>;
  showErrorBox(options: ErrorBoxOptions): Promise<void>;
  showOpenDialog(options: OpenDialogOptions & { multiple: true }): Promise<string[] | null>;
  showOpenDialog(options?: OpenDialogOptions): Promise<string | null>;
  showSaveDialog(options?: SaveDialogOptions): Promise<string | null>;
  confirm(options: Omit<MessageBoxOptions, 'type'>): Promise<boolean>;
  info(options: Omit<MessageBoxOptions, 'type' | 'icon'>): Promise<void>;
  error(options: ErrorBoxOptions): Promise<void>;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `showMessageBox(options)` | `MessageBoxOptions \| CustomMessageBoxOptions` | `MessageBoxResult \| number` | 显示消息对话框 |
| `showErrorBox(options)` | `ErrorBoxOptions` | `Promise<void>` | 显示错误对话框 |
| `showOpenDialog(options?)` | `OpenDialogOptions?` | `Promise<string \| string[] \| null>` | 显示打开文件对话框 |
| `showSaveDialog(options?)` | `SaveDialogOptions?` | `Promise<string \| null>` | 显示保存文件对话框 |
| `confirm(options)` | `Omit<MessageBoxOptions, 'type'>` | `Promise<boolean>` | 确认对话框 |
| `info(options)` | `Omit<MessageBoxOptions, 'type' \| 'icon'>` | `Promise<void>` | 信息提示对话框 |
| `error(options)` | `ErrorBoxOptions` | `Promise<void>` | 错误提示对话框 |

### MessageBoxOptions

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `title` | `string` | - | 对话框标题 |
| `message` | `string` | - | 消息内容（必填） |
| `type` | `"none" \| "okCancel" \| "yesNo" \| "yesNoCancel"` | `"none"` | 按钮类型 |
| `icon` | `"info" \| "warning" \| "error"` | - | 图标类型 |

### OpenDialogOptions

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `title` | `string` | - | 对话框标题 |
| `defaultPath` | `string` | - | 默认路径 |
| `defaultName` | `string` | - | 默认文件名 |
| `multiple` | `boolean` | `false` | 是否多选 |
| `directory` | `boolean` | `false` | 是否选择目录 |
| `filters` | `FileFilter[]` | - | 文件过滤器 |

### FileFilter

| 参数 | 类型 | 说明 |
|------|------|------|
| `name` | `string` | 过滤器名称 |
| `extensions` | `string[]` | 扩展名列表，`["*"]` 表示所有文件 |

### 使用示例

```typescript
import { dialog } from "vokex.app";

const confirmed = await dialog.confirm({
  title: "确认删除",
  message: "确定要删除此文件吗？",
});

const filePath = await dialog.showOpenDialog({
  title: "选择文件",
  filters: [{ name: "文本文件", extensions: ["txt", "md"] }],
});

const files = await dialog.showOpenDialog({
  multiple: true,
  directory: false,
  filters: [{ name: "图片", extensions: ["png", "jpg", "gif"] }],
});

const savePath = await dialog.showSaveDialog({
  defaultName: "untitled.txt",
  filters: [{ name: "文本文件", extensions: ["txt"] }],
});

await dialog.info({ title: "提示", message: "操作成功" });
await dialog.error({ title: "错误", message: "操作失败" });
```

---

## menu - 原生菜单

```typescript
import { menu } from "vokex.app";
```

### 类型定义

```typescript
type NativeLabel =
  | 'separator' | 'copy' | 'cut' | 'paste' | 'selectAll'
  | 'undo' | 'redo' | 'minimize' | 'maximize' | 'fullscreen'
  | 'hide' | 'hideOthers' | 'showAll' | 'closeWindow'
  | 'quit' | 'about' | 'services' | 'bringAllToFront';

interface MenuItem {
  id?: string;
  label?: string;
  type?: 'normal' | 'separator' | 'submenu' | 'checkbox' | 'native';
  enabled?: boolean;
  checked?: boolean;
  submenu?: MenuItem[];
  nativeLabel?: NativeLabel;
}

interface MenuAPI {
  setApplicationMenu(menu: MenuItem[]): Promise<void>;
  removeApplicationMenu(): Promise<void>;
  setContextMenu(menu: MenuItem[], x?: number, y?: number): Promise<void>;
  onMenuClick(callback: (data: { id: string }) => void): void;
}
```

### MenuItem 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `id` | `string` | - | 菜单项 ID（点击事件中返回） |
| `label` | `string` | - | 显示文本 |
| `type` | `"normal" \| "separator" \| "submenu" \| "checkbox" \| "native"` | `"normal"` | 菜单项类型 |
| `enabled` | `boolean` | `true` | 是否启用 |
| `checked` | `boolean` | `false` | 是否选中（仅 checkbox） |
| `submenu` | `MenuItem[]` | - | 子菜单（仅 submenu） |
| `nativeLabel` | `NativeLabel` | - | 原生菜单项标签（仅 native） |

### NativeLabel 支持的值

| 值 | 说明 |
|----|------|
| `separator` | 分隔线 |
| `copy` | 复制 |
| `cut` | 剪切 |
| `paste` | 粘贴 |
| `selectAll` | 全选 |
| `undo` | 撤销 |
| `redo` | 重做 |
| `minimize` | 最小化 |
| `maximize` | 最大化 |
| `fullscreen` | 全屏 |
| `hide` | 隐藏 |
| `hideOthers` | 隐藏其他 |
| `showAll` | 显示全部 |
| `closeWindow` | 关闭窗口 |
| `quit` | 退出 |
| `about` | 关于 |
| `services` | 服务 |
| `bringAllToFront` | 全部置前 |

### 使用示例

```typescript
import { menu } from "vokex.app";

await menu.setApplicationMenu([
  {
    type: 'submenu',
    label: '文件',
    submenu: [
      { id: 'new', label: '新建' },
      { id: 'open', label: '打开...' },
      { type: 'separator' },
      { id: 'save', label: '保存' },
      { type: 'separator' },
      { type: 'native', nativeLabel: 'quit' },
    ],
  },
  {
    type: 'submenu',
    label: '编辑',
    submenu: [
      { type: 'native', nativeLabel: 'copy' },
      { type: 'native', nativeLabel: 'cut' },
      { type: 'native', nativeLabel: 'paste' },
      { type: 'separator' },
      { type: 'native', nativeLabel: 'selectAll' },
    ],
  },
]);

menu.onMenuClick(({ id }) => {
  switch (id) {
    case 'new': createNewFile(); break;
    case 'open': openFile(); break;
    case 'save': saveFile(); break;
  }
});

document.addEventListener('contextmenu', async (e) => {
  e.preventDefault();
  await menu.setContextMenu([
    { id: 'copy', label: '复制' },
    { id: 'paste', label: '粘贴' },
  ], e.clientX, e.clientY);
});
```

---

## tray - 系统托盘

```typescript
import { tray } from "vokex.app";
```

### 类型定义

```typescript
interface TrayOptions {
  icon: string;
  tooltip?: string;
  title?: string;
  menu?: MenuItem[];
}

type TrayEventType = "click" | "right-click" | "double-click";

class Tray {
  getId(): number;
  setToolTip(text: string): Promise<void>;
  setTitle(title: string): Promise<void>;
  setMenu(template: MenuItem[]): Promise<void>;
  setImage(icon: string): Promise<void>;
  destroy(): Promise<void>;
  on(event: TrayEventType, callback: (data: { trayId: number; button: string }) => void): () => void;
}

interface TrayAPI {
  create(options: TrayOptions): Promise<Tray>;
}
```

### TrayOptions 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `icon` | `string` | - | 图标文件路径（必填） |
| `tooltip` | `string` | - | 鼠标悬停提示文本 |
| `title` | `string` | - | 托盘标题（macOS/Linux） |
| `menu` | `MenuItem[]` | - | 右键菜单 |

### Tray 实例方法

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `getId()` | 无 | `number` | 获取托盘 ID |
| `setToolTip(text)` | `text: string` | `Promise<void>` | 设置提示文本 |
| `setTitle(title)` | `title: string` | `Promise<void>` | 设置标题 |
| `setMenu(template)` | `template: MenuItem[]` | `Promise<void>` | 设置右键菜单 |
| `setImage(icon)` | `icon: string` | `Promise<void>` | 更新图标 |
| `destroy()` | 无 | `Promise<void>` | 销毁托盘 |
| `on(event, callback)` | `event`, `callback` | `() => void` | 监听事件，返回取消函数 |

### 托盘事件

| 事件 | 说明 |
|------|------|
| `click` | 左键点击 |
| `right-click` | 右键点击 |
| `double-click` | 双击 |

### 使用示例

```typescript
import { tray, browserWindow } from "vokex.app";

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
  const win = browserWindow.getCurrentWindow();
  if (win) win.show();
});

myTray.on("right-click", () => {
  console.log("右键点击托盘");
});
```

---

## shortcut - 全局快捷键

```typescript
import { shortcut } from "vokex.app";
```

### 类型定义

```typescript
interface ShortcutTriggeredEvent {
  id: number;
  accelerator: string;
}

interface HotKeyInfo {
  id: number;
  accelerator: string;
}

type UnregisterFn = () => Promise<void>;
type ShortcutHandler = (event: ShortcutTriggeredEvent) => void;

interface ShortcutAPI {
  register(accelerator: string, handler: ShortcutHandler): Promise<UnregisterFn>;
  registerAll(bindings: Record<string, ShortcutHandler>): Promise<UnregisterFn>;
  unregisterAll(): Promise<void>;
  isRegistered(accelerator: string): Promise<boolean>;
  list(): Promise<HotKeyInfo[]>;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `register(accelerator, handler)` | `accelerator: string`, `handler: ShortcutHandler` | `Promise<UnregisterFn>` | 注册快捷键，返回注销函数 |
| `registerAll(bindings)` | `bindings: Record<string, ShortcutHandler>` | `Promise<UnregisterFn>` | 批量注册，返回统一注销函数 |
| `unregisterAll()` | 无 | `Promise<void>` | 注销所有已注册的快捷键 |
| `isRegistered(accelerator)` | `accelerator: string` | `Promise<boolean>` | 查询是否已注册 |
| `list()` | 无 | `Promise<HotKeyInfo[]>` | 列出所有已注册的快捷键 |

### Accelerator 格式

修饰键 + 按键，支持：
- 修饰键：`Ctrl`、`Shift`、`Alt`、`Super`（Win/Cmd）
- 按键：`A-Z`、`F1-F24`、`Space`、`Enter`、`Escape`、`0-9` 等
- 跨平台：使用 `CmdOrCtrl` 前缀（macOS 用 Cmd，其他用 Ctrl）
- 空格会被自动忽略：`"Ctrl + S"` 等价于 `"Ctrl+S"`

### 使用示例

```typescript
import { shortcut } from "vokex.app";

const unbind = await shortcut.register("Ctrl+Shift+S", (ev) => {
  console.log("保存快捷键触发:", ev.accelerator);
  saveFile();
});

const unbindAll = await shortcut.registerAll({
  "Ctrl+N": () => createNew(),
  "Ctrl+W": () => closeTab(),
  "CmdOrCtrl+P": () => openPalette(),
});

const isRegistered = await shortcut.isRegistered("Ctrl+S");
console.log("Ctrl+S 已注册:", isRegistered);

const hotkeys = await shortcut.list();
console.log("已注册快捷键:", hotkeys);

await unbind();
await unbindAll();
await shortcut.unregisterAll();
```

---

## shell - 系统命令

```typescript
import { shell } from "vokex.app";
```

### 类型定义

```typescript
interface ExecOptions {
  cwd?: string;
  env?: Record<string, string>;
}

interface ExecResult {
  code: number | null;
  stdout: string;
  stderr: string;
  success: boolean;
}

interface ShellProcess {
  pid: number;
  onStdout(cb: (data: string) => void): () => void;
  onStderr(cb: (data: string) => void): () => void;
  onExit(cb: (code: number | null) => void): () => void;
  kill(): Promise<void>;
}

interface ShellAPI {
  openExternal(url: string): Promise<void>;
  openPath(path: string): Promise<void>;
  exec(program: string, args?: string[], options?: ExecOptions): Promise<ExecResult>;
  spawn(program: string, args?: string[], options?: ExecOptions): Promise<ShellProcess>;
  trashItem(path: string): Promise<void>;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `openExternal(url)` | `url: string` | `Promise<void>` | 用默认浏览器打开 URL |
| `openPath(path)` | `path: string` | `Promise<void>` | 用系统默认程序打开文件/目录 |
| `exec(program, args?, options?)` | `program: string`, `args?: string[]`, `options?: ExecOptions` | `Promise<ExecResult>` | 一次性执行程序并返回完整结果 |
| `spawn(program, args?, options?)` | `program: string`, `args?: string[]`, `options?: ExecOptions` | `Promise<ShellProcess>` | 启动流式进程 |
| `trashItem(path)` | `path: string` | `Promise<void>` | 移到回收站 |

### ExecOptions

| 参数 | 类型 | 说明 |
|------|------|------|
| `cwd` | `string` | 工作目录 |
| `env` | `Record<string, string>` | 环境变量 |

### ExecResult

| 字段 | 类型 | 说明 |
|------|------|------|
| `code` | `number \| null` | 退出码 |
| `stdout` | `string` | 标准输出 |
| `stderr` | `string` | 标准错误 |
| `success` | `boolean` | 是否成功（退出码为 0） |

### ShellProcess

| 属性/方法 | 类型 | 说明 |
|-----------|------|------|
| `pid` | `number` | 进程 ID |
| `onStdout(cb)` | `(cb) => () => void` | 监听 stdout，返回取消函数 |
| `onStderr(cb)` | `(cb) => () => void` | 监听 stderr，返回取消函数 |
| `onExit(cb)` | `(cb) => () => void` | 监听退出，返回取消函数 |
| `kill()` | `() => Promise<void>` | 杀死进程 |

### 使用示例

```typescript
import { shell } from "vokex.app";

await shell.openExternal("https://example.com");
await shell.openPath("C:\\Users\\Documents");

const result = await shell.exec("git", ["status"], { cwd: "./my-repo" });
console.log(result.stdout);
console.log("成功:", result.success);

const child = await shell.spawn("npm", ["run", "dev"], { cwd: "./project" });
child.onStdout((data) => console.log(`[stdout]: ${data}`));
child.onStderr((data) => console.error(`[stderr]: ${data}`));
child.onExit((code) => console.log(`进程退出: ${code}`));

setTimeout(() => child.kill(), 5000);

await shell.trashItem("C:\\path\\to\\file.txt");
```

---

## clipboard - 剪贴板

```typescript
import { clipboard } from "vokex.app";
```

### 类型定义

```typescript
interface ClipboardAPI {
  readText(): Promise<string>;
  writeText(text: string): Promise<void>;
  clear(): Promise<void>;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `readText()` | 无 | `Promise<string>` | 读取剪贴板文本 |
| `writeText(text)` | `text: string` | `Promise<void>` | 写入文本到剪贴板 |
| `clear()` | 无 | `Promise<void>` | 清空剪贴板 |

### 使用示例

```typescript
import { clipboard } from "vokex.app";

const text = await clipboard.readText();
console.log("剪贴板内容:", text);

await clipboard.writeText("Hello World");

await clipboard.clear();
```

---

## notification - 系统通知

```typescript
import { notification } from "vokex.app";
```

### 类型定义

```typescript
interface NotificationOptions {
  title: string;
  body?: string;
}

interface NotificationAPI {
  show(options: NotificationOptions): Promise<void>;
}
```

### NotificationOptions

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `title` | `string` | - | 通知标题（必填） |
| `body` | `string` | - | 通知内容 |

### 使用示例

```typescript
import { notification } from "vokex.app";

await notification.show({
  title: "下载完成",
  body: "文件已保存到 Downloads 文件夹",
});
```

---

## process - 进程信息

```typescript
import { process } from "vokex.app";
```

### 类型定义

```typescript
interface CpuUsage {
  user: number;
  system: number;
}

interface MemoryInfo {
  total: number;
  available: number;
  used: number;
}

interface ProcessAPI {
  getUptime(): Promise<number>;
  getCpuUsage(): Promise<CpuUsage>;
  getMemoryInfo(): Promise<MemoryInfo>;
  hostname(): Promise<string>;
  env(): Promise<Record<string, string>>;
  kill(pid: number): Promise<void>;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `getUptime()` | 无 | `Promise<number>` | 获取系统运行时长（秒） |
| `getCpuUsage()` | 无 | `Promise<CpuUsage>` | 获取 CPU 使用率 |
| `getMemoryInfo()` | 无 | `Promise<MemoryInfo>` | 获取内存信息 |
| `hostname()` | 无 | `Promise<string>` | 获取主机名 |
| `env()` | 无 | `Promise<Record<string, string>>` | 获取所有环境变量 |
| `kill(pid)` | `pid: number` | `Promise<void>` | 终止指定进程 |

### CpuUsage

| 字段 | 类型 | 说明 |
|------|------|------|
| `user` | `number` | 用户态 CPU 时间（秒） |
| `system` | `number` | 内核态 CPU 时间（秒） |

### MemoryInfo

| 字段 | 类型 | 说明 |
|------|------|------|
| `total` | `number` | 总内存（字节） |
| `available` | `number` | 可用内存（字节） |
| `used` | `number` | 已用内存（字节） |

### 使用示例

```typescript
import { process } from "vokex.app";

const uptime = await process.getUptime();
console.log(`系统运行 ${uptime} 秒`);

const cpu = await process.getCpuUsage();
console.log(`CPU 使用: 用户 ${cpu.user}s, 系统 ${cpu.system}s`);

const mem = await process.getMemoryInfo();
console.log(`内存: ${mem.used / 1024 / 1024}MB / ${mem.total / 1024 / 1024}MB`);

const hostname = await process.hostname();
console.log(`主机名: ${hostname}`);

const allEnv = await process.env();
console.log(`PATH: ${allEnv.PATH}`);
```

---

## computer - 系统硬件信息

```typescript
import { computer } from "vokex.app";
```

### 类型定义

```typescript
interface CpuInfo {
  manufacturer: string;
  model: string;
  cores: number;
  logicalProcessors: number;
  architecture: string;
  frequency: number;
}

interface OsInfo {
  name: string;
  version: string;
  longVersion: string;
  kernelVersion: string;
  platform: string;
  arch: string;
}

interface MousePosition {
  x: number;
  y: number;
}

interface KeyboardLayout {
  layout: string;
}

interface DisplayInfo {
  name: string | null;
  width: number;
  height: number;
  x: number;
  y: number;
  scaleFactor: number;
  isPrimary: boolean;
}

interface DisplaysInfo {
  displays: DisplayInfo[];
  primary: string | null;
}

interface ComputerAPI {
  getCpuInfo(): Promise<CpuInfo>;
  getOsInfo(): Promise<OsInfo>;
  getMousePosition(): Promise<MousePosition>;
  getKeyboardLayout(): Promise<KeyboardLayout>;
  getDisplays(): Promise<DisplaysInfo>;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `getCpuInfo()` | 无 | `Promise<CpuInfo>` | 获取 CPU 硬件信息 |
| `getOsInfo()` | 无 | `Promise<OsInfo>` | 获取操作系统信息 |
| `getMousePosition()` | 无 | `Promise<MousePosition>` | 获取鼠标位置 |
| `getKeyboardLayout()` | 无 | `Promise<KeyboardLayout>` | 获取键盘布局 |
| `getDisplays()` | 无 | `Promise<DisplaysInfo>` | 获取所有显示器信息 |

### CpuInfo

| 字段 | 类型 | 说明 |
|------|------|------|
| `manufacturer` | `string` | CPU 制造商 |
| `model` | `string` | CPU 型号 |
| `cores` | `number` | 物理核心数 |
| `logicalProcessors` | `number` | 逻辑处理器数 |
| `architecture` | `string` | 架构 |
| `frequency` | `number` | 频率 |

### OsInfo

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `string` | 操作系统名称 |
| `version` | `string` | 版本 |
| `longVersion` | `string` | 完整版本 |
| `kernelVersion` | `string` | 内核版本 |
| `platform` | `string` | 平台 |
| `arch` | `string` | 架构 |

### DisplayInfo

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `string \| null` | 显示器名称 |
| `width` | `number` | 宽度 |
| `height` | `number` | 高度 |
| `x` | `number` | X 坐标 |
| `y` | `number` | Y 坐标 |
| `scaleFactor` | `number` | 缩放因子 |
| `isPrimary` | `boolean` | 是否主显示器 |

### 使用示例

```typescript
import { computer } from "vokex.app";

const cpu = await computer.getCpuInfo();
console.log(`CPU: ${cpu.manufacturer} ${cpu.model}`);
console.log(`核心: ${cpu.cores} 物理, ${cpu.logicalProcessors} 逻辑`);

const os = await computer.getOsInfo();
console.log(`系统: ${os.name} ${os.version}`);
console.log(`内核: ${os.kernelVersion}`);

const mouse = await computer.getMousePosition();
console.log(`鼠标位置: (${mouse.x}, ${mouse.y})`);

const keyboard = await computer.getKeyboardLayout();
console.log(`键盘布局: ${keyboard.layout}`);

const displays = await computer.getDisplays();
displays.displays.forEach((d, i) => {
  console.log(`显示器 ${i}: ${d.width}x${d.height}, 缩放: ${d.scaleFactor}, 主显示器: ${d.isPrimary}`);
});
```

---

## storage - 本地持久化存储

```typescript
import { storage } from "vokex.app";
```

### 类型定义

```typescript
interface StorageAPI {
  setData(key: string, value: any): Promise<void>;
  getData(key: string): Promise<any>;
  getKeys(): Promise<string[]>;
  has(key: string): Promise<boolean>;
  removeData(key: string): Promise<void>;
  clear(): Promise<void>;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `setData(key, value)` | `key: string`, `value: any` | `Promise<void>` | 存储数据（JSON 序列化） |
| `getData(key)` | `key: string` | `Promise<any>` | 读取数据 |
| `getKeys()` | 无 | `Promise<string[]>` | 获取所有键名 |
| `has(key)` | `key: string` | `Promise<boolean>` | 检查键是否存在 |
| `removeData(key)` | `key: string` | `Promise<void>` | 删除指定键 |
| `clear()` | 无 | `Promise<void>` | 清空所有数据 |

### 使用示例

```typescript
import { storage } from "vokex.app";

await storage.setData("user", { name: "Alice", age: 30 });
await storage.setData("theme", "dark");
await storage.setData("fontSize", 14);

const user = await storage.getData("user");
console.log(user.name);

const keys = await storage.getKeys();
console.log("所有键:", keys);

const hasTheme = await storage.has("theme");
console.log("是否有 theme:", hasTheme);

await storage.removeData("fontSize");
await storage.clear();
```

---

## safeStorage - 安全持久化存储

安全的持久化键值对存储 API，数据通过 AES-256-GCM 加密后存储在本地文件中，密钥由系统安全区（Windows 凭据管理器/macOS 钥匙串）保护。适用于存储 API 密钥、用户 Token 等敏感数据。

```typescript
import { safeStorage } from "vokex.app";
```

### 类型定义

```typescript
interface SafeStorageAPI {
  setItem(key: string, value: any): Promise<void>;
  getItem(key: string): Promise<any>;
  removeItem(key: string): Promise<void>;
  clear(): Promise<void>;
  keys(): Promise<string[]>;
  has(key: string): Promise<boolean>;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `setItem(key, value)` | `key: string`, `value: any` | `Promise<void>` | 加密存储数据 |
| `getItem(key)` | `key: string` | `Promise<any>` | 解密读取数据 |
| `removeItem(key)` | `key: string` | `Promise<void>` | 删除指定键 |
| `clear()` | 无 | `Promise<void>` | 清空所有安全存储 |
| `keys()` | 无 | `Promise<string[]>` | 获取所有键名 |
| `has(key)` | `key: string` | `Promise<boolean>` | 检查键是否存在 |

### 安全机制

| 特性 | 说明 |
|------|------|
| 加密算法 | AES-256-GCM（认证加密，防篡改） |
| 密钥管理 | 首次启动时生成 32 字节随机 Master Key，存储在系统安全区 |
| 存储位置 | `%LOCALAPPDATA%/{identifier}/safeStorage.json.enc`（Windows） |
| 密钥存储 | Windows 凭据管理器 / macOS 钥匙串 / Linux Secret Service |

### 与 storage 的区别

| 特性 | storage | safeStorage |
|------|---------|-------------|
| 数据格式 | 明文 JSON | AES-256-GCM 加密 |
| 适用场景 | 主题、配置等非敏感数据 | API 密钥、Token 等敏感数据 |
| 性能 | 更快（无加解密开销） | 稍慢（加解密需要计算） |
| 存储文件 | `storage.json` | `safeStorage.json.enc` |

### 使用示例

```typescript
import { safeStorage } from "vokex.app";

// 存储 API 密钥
await safeStorage.setItem("openai_key", {
  provider: "openai",
  apiKey: "sk-xxxxxxxxxxxx",
  model: "gpt-4",
});

// 读取 API 密钥
const config = await safeStorage.getItem("openai_key");
console.log(config.apiKey);

// 获取所有键名
const keys = await safeStorage.keys();
console.log("安全存储中的键:", keys);

// 检查键是否存在
const exists = await safeStorage.has("openai_key");

// 删除指定键
await safeStorage.removeItem("openai_key");

// 清空所有安全存储
await safeStorage.clear();
```

---

## events - 事件总线

```typescript
import { events } from "vokex.app";
```

### 类型定义

```typescript
interface EventsAPI {
  on(event: string, listener: (data: any) => void): () => void;
  off(event: string, listener: (data: any) => void): void;
  emit(event: string, data?: any): void;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `on(event, listener)` | `event: string`, `listener: (data) => void` | `() => void` | 监听事件，返回取消监听函数 |
| `off(event, listener)` | `event: string`, `listener: (data) => void` | `void` | 移除监听 |
| `emit(event, data?)` | `event: string`, `data?: any` | `void` | 触发事件 |

### 使用示例

```typescript
import { events } from "vokex.app";

const unsub = events.on("custom-event", (data) => {
  console.log("收到事件:", data);
});

events.emit("custom-event", { message: "Hello" });

unsub();

events.off("custom-event", handler);
```

---

## path - 路径操作

```typescript
import { path } from "vokex.app";
```

### 类型定义

```typescript
interface PathAPI {
  join(...paths: string[]): Promise<string>;
  resolve(...paths: string[]): Promise<string>;
  normalize(path: string): Promise<string>;
  basename(path: string, suffix?: string): string;
  dirname(path: string): string;
  extname(path: string): string;
  readonly sep: string;
}
```

### 方法说明

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `join(...paths)` | `...paths: string[]` | `Promise<string>` | 拼接路径片段（异步） |
| `resolve(...paths)` | `...paths: string[]` | `Promise<string>` | 解析为绝对路径（异步） |
| `normalize(path)` | `path: string` | `Promise<string>` | 规范化路径（异步） |
| `basename(path, suffix?)` | `path: string`, `suffix?: string` | `string` | 获取路径最后一部分（同步） |
| `dirname(path)` | `path: string` | `string` | 获取目录名（同步） |
| `extname(path)` | `path: string` | `string` | 获取扩展名（同步） |
| `sep` | 无 | `string` | 路径分隔符（同步属性） |

### 使用示例

```typescript
import { path } from "vokex.app";

const joined = await path.join("src", "components", "Button.tsx");
const resolved = await path.resolve("./config.json");
const normalized = await path.normalize("/a/b/../c/./d");

const name = path.basename("/path/to/file.txt");
const nameWithoutExt = path.basename("/path/to/file.txt", ".txt");
const dir = path.dirname("/path/to/file.txt");
const ext = path.extname("/path/to/file.txt");

console.log(`分隔符: ${path.sep}`);
```

---

## 完整导入示例

```typescript
import {
  app,
  browserWindow,
  fs,
  http,
  dialog,
  menu,
  tray,
  shortcut,
  shell,
  clipboard,
  notification,
  process,
  computer,
  storage,
  events,
  path,
} from "vokex.app";
```
