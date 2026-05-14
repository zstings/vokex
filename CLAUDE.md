# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

Vokex 是一个超轻量级桌面应用构建库。将前端代码通过 Vite 构建后，一键嵌入到预编译的 Rust 壳中，生成单个原生可执行文件（最小 ~1.8MB）。架构类似 Tauri 的极简版，壳基于 tao + wry（系统 WebView）。

## 常用命令

```bash
# TypeScript 编译
npm run build          # tsc 编译 src/ → dist/
npm run dev            # tsc --watch

# Rust 壳编译（仅在修改 shell/ 时需要）
cd shell
cargo build --release  # 输出到 shell/target/release/

# 验证构建产物
npx vokex validate release/应用名.exe
```

`npm run build` 只需要 tsc 编译 — Vite 插件和资源嵌入是在**使用方项目**中由 `vite build` 触发的，不在本仓库内运行。

## 架构

```
src/                    # TypeScript 源码（发布为 npm 包）
├── index.ts            # 入口，re-export 所有运行时 API
├── runtime/
│   ├── index.ts        # window.__VOKEX__ 初始化（壳注入的桥接对象）
│   ├── api.ts          # 所有面向用户的 API 封装（app, fs, http, browserWindow 等）
│   └── apis/           # 每个 API 模块的具体实现（调用 vokexCall → 壳）
├── vite-plugin/
│   └── index.ts        # Vite 插件：开发时自动启动壳，构建时自动嵌入资源
├── build/
│   ├── embed.ts        # 资源嵌入器：将 dist/ 文件 zlib 压缩追加到壳二进制尾部
│   └── icon-embed.ts   # PE 图标注入：使用 resedit 将 .ico 图标注入 Windows PE 资源段
└── cli.ts              # CLI 工具（仅 validate 命令）

shell/                  # Rust 原生壳
├── src/main.rs         # 事件循环、窗口创建、WebView 构建、资源加载
├── src/ipc.rs          # IPC 协议：postMessage ↔ 事件循环，同步/异步分发
├── src/app_config.rs   # 配置加载（dev: 读 vokex-config.json；prod: 从嵌入资源读）
├── src/window_manager.rs  # 多窗口管理
├── src/security/       # 安全模块（URL 解析、API 权限检查、危险 API 判定）
├── src/utils.rs        # 工具函数（命令行参数解析、PNG/ICO 解码等）
├── src/apis/           # API 实现（app.rs, fs.rs, http.rs, browser_window.rs, shell.rs, process.rs, storage.rs, clipboard.rs, dialog.rs, notification.rs, computer.rs, menu.rs, tray.rs, shortcut.rs, path.rs）
└── Cargo.toml          # 依赖：wry 0.55, tao 0.34, minreq 2, rfd 0.17, muda 0.17, tray-icon 0.21, global-hotkey 0.8, sysinfo 0.38, arboard 3

prebuilt/               # 预编译壳二进制（随 npm 包发布）
├── win32-x64.exe       # Windows 生产壳（release 编译）
├── darwin-x64          # macOS 生产壳
├── linux-x64           # Linux 生产壳
├── vokex-config.json   # 默认配置文件
└── icon/               # 默认图标（icon.ico, 32x32.png）
```

## 核心流程

**IPC 通信链路：**
```
前端 JS: window.__VOKEX__.call("fs.readFile", [path])
  → window.ipc.postMessage(JSON)
    → Rust wry IPC handler
      → 主线程事件循环 (Event::UserEvent::HandleRequest)
        → ipc::process_request → dispatch 到对应 API 模块
          → 同步直接 eval 回 JS / 异步投递到线程池，完成后再 eval
            → window.__VOKEX_IPC__(response)
              → Promise resolve
```

**构建流程：**
```
vite build 完成
  → vite-plugin closeBundle() 钩子
    → embed.ts: scanDir(dist/) → 压缩 → 追加到壳二进制尾部
    → icon-embed.ts: 将 .ico 注入 PE 资源段（Windows，在 VOKEX 尾部嵌入之前）
      → 输出 release/应用名.exe
```

二进制尾部格式：`[MAGIC(5B "VOKEX")] [索引长度(4B LE)] [索引JSON] [zlib压缩数据] [偏移量(8B LE)]`

**双模式运行：**
- 开发模式：`vite` 启动 dev server → 插件将配置写入 public/vokex-config.json → 复制 public/ 到壳目录 → 启动壳 `--env_dev` → 壳加载 `http://localhost:port`
- 生产模式：`vite build` 后壳从自身尾部嵌入资源加载 `vokex://index.html`（custom protocol）

## API 同步/异步策略

IPC 层区分同步和异步 API（`ipc.rs` 中的 `is_async_api` 函数）：
- **同步**（主线程直接执行）：窗口操作、剪贴板、通知、菜单、托盘、快捷键、存储、路径、应用信息、系统硬件信息
- **异步**（线程池执行）：文件 I/O（fs.*）、HTTP 请求（http.*）、Shell 命令（shell.exec/spawn/kill）、进程信息（process.*）、对话框（dialog.*）

避免阻塞主线程导致窗口卡顿。对话框虽然是 UI 操作，但因为需要等待用户交互且涉及文件系统扫描，也放入线程池执行。

## HTTP 模块设计

HTTP 模块是前后端协作最复杂的 API 之一，涉及 body 智能处理和类 fetch 响应封装。

**TS 层（http.ts）：**
- `resolveBody()` — 发送前自动处理 body：纯对象 → JSON.stringify + 自动 Content-Type；FormData → URL-encoded；string/undefined → 原样
- `cleanHeaders()` — 移除空 key/value 的请求头
- `VokexHeaders` — 不区分大小写的响应头包装（内部 key 统一小写）
- `VokexResponse` — 模拟浏览器 fetch Response，提供 `status`/`headers`/`ok`/`text()`/`json()`/`clone()`
- 所有方法（get/post/put/delete/request）共用 `doRequest()` 统一入口

**Rust 层（http.rs）：**
- 使用 `minreq` 发送请求，headers 为 `HashMap<String, String>`（key 自动小写）
- 返回 JSON 包含 `statusCode`/`statusText`/`headers`/`body`/`ok` 五个字段
- `status_text()` 函数根据状态码映射标准 HTTP 状态文本
- 仅网络连接失败返回 `Err`，服务器有响应（含 500）都返回 `Ok`

## 安全体系

1. **来源追踪**：`PageOrigin` 枚举（Local/Dev/Remote），在窗口注册时根据 URL 判定
2. **危险 API 列表**：`DANGEROUS_API_LIST` 标记 fs 读写、shell 命令、process.kill 等高风险 API
3. **权限检查**：`check_api_permission()` — Local/Dev 全部允许，Remote 非危险 API 允许，Remote 危险 API 需显式配置 `allowed_remote_apis`
4. **JS 安全注入**：CSP meta 标签、iframe 创建拦截、postMessage 来源验证、嵌套 frame 阻止
