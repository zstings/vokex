# Vokex 安全限制方案

## 一、方案目标

1. ✅ 所有危险 API 在远端页面默认禁用
2. ✅ 远端页面使用危险 API 需在配置中手动开启
3. ✅ 检查并修复 IPC 通信存在的安全漏洞
4. ✅ 保持向后兼容性，不影响本地/开发模式使用
5. ✅ 零新增外部依赖

## 二、来源分级

| 来源 | 协议/URL | 信任级别 | API 权限 |
|------|----------|----------|----------|
| **本地资源** | `vokex://` | ✅ 完全信任 | 全部可用 |
| **开发模式** | `http://localhost:*` | ✅ 完全信任 | 全部可用 |
| **远端页面** | `https://` / `http://` | ❌ 不信任 | 仅安全 API + 需配置 |

## 三、危险 API 列表

### 设计原则

采用**危险列表模式**（而非安全列表）：
- 新增 API 默认可用
- 只标记真正危险的 API
- 列表会越来越短

### 危险 API（14 个）

| 模块 | API | 风险说明 |
|------|-----|----------|
| **fs** | `readFile`, `readFileBinary` | 读取任意文件 |
| **fs** | `writeFile`, `appendFile` | 写入任意文件 |
| **fs** | `deleteFile`, `createDir`, `removeDir` | 删除/创建目录 |
| **fs** | `copyFile`, `moveFile` | 复制/移动文件 |
| **shell** | `execCommand`, `openPath`, `trashItem` | 执行命令/打开路径 |
| **process** | `kill` | 终止进程 |
| **dialog** | `showSaveDialog` | 保存文件 |

### 安全 API

除上述危险 API 外的所有 API 均为安全 API，可直接使用。

## 四、安全配置

### 配置结构

```json
{
  "security": {
    "allowed_remote_apis": ["fs.readFile", "fs.*"],
    "allow_remote_pages": true
  }
}
```

### 配置项说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `allowed_remote_apis` | `Vec<String>` | `[]` | 允许远端调用的危险 API |
| `allow_remote_pages` | `bool` | `true` | 是否允许加载远端页面 |

### allowed_remote_apis 格式

```json
{
  "allowed_remote_apis": [
    "fs.readFile",     // 单个 API
    "fs.*",           // 模块通配符
    "*"               // 全局通配符
  ]
}
```

## 五、实现步骤

| 步骤 | 内容 | 状态 |
|------|------|------|
| 1.1 | 创建来源类型定义 (origin.rs) | ✅ |
| 1.2 | 创建 URL 来源解析 (url_parser.rs) | ✅ |
| 1.3 | 创建安全模块入口 (mod.rs) | ✅ |
| 1.4 | 扩展窗口管理器添加 origin 字段 | ✅ |
| 2.1 | 创建危险 API 列表 (dangerous_apis.rs) | ✅ |
| 2.2 | 创建权限配置结构 (permissions.rs) | ✅ |
| 2.3 | 扩展 app_config 添加安全配置 | ✅ |
| 2.4 | IPC 集成权限检查 | ✅ |
| 3.1 | CSP 注入防止 iframe 加载远端 | ✅ |
| 3.2 | 远端页面加载控制 | ✅ |
| 4.1 | postMessage 来源验证 | ✅ |
| 4.2 | 前端安全脚本注入 | ✅ |

**完成度：100%** ✅

## 六、文件变更清单

### 新建文件

| 文件 | 作用 |
|------|------|
| `shell/src/security/mod.rs` | 安全模块入口 |
| `shell/src/security/origin.rs` | 来源类型定义 |
| `shell/src/security/url_parser.rs` | URL 来源解析 |
| `shell/src/security/dangerous_apis.rs` | 危险 API 列表 |
| `shell/src/security/permissions.rs` | 权限配置和检查 |
| `shell/src/security/inject.rs` | JavaScript 安全注入脚本 |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| `shell/src/main.rs` | 注入安全脚本 |
| `shell/src/ipc.rs` | 添加权限检查 |
| `shell/src/app_config.rs` | 添加 security 配置字段 |
| `shell/src/window_manager.rs` | 添加 origin/url 字段 |

## 七、安全测试结果

### 测试环境

- 远端页面 URL：`https://...`（Remote 来源）

### 测试结果

| 测试类型 | API | 结果 | 状态 |
|----------|-----|------|------|
| 危险 | `fs.readFile` | `Permission denied` | ✅ |
| 危险 | `fs.writeFile` | `Permission denied` | ✅ |
| 危险 | `shell.execCommand` | `Permission denied` | ✅ |
| 危险 | `process.kill` | `Permission denied` | ✅ |
| 危险 | `dialog.showSaveDialog` | `Permission denied` | ✅ |
| 危险 | `fs.moveFile` | `Permission denied` | ✅ |
| 安全 | `app.getName` | 返回应用名称 | ✅ |
| 安全 | `computer.getCpuInfo` | 返回 CPU 信息 | ✅ |

## 八、安全机制说明

### 权限检查流程

```
请求到达 → 获取 window_id → 查询窗口来源
                                      ↓
                              Local/Dev → 允许所有 API ✅
                                      ↓
                              Remote → 检查 API
                                          ↓
                              非危险 API → 允许 ✅
                                          ↓
                              危险 API → 检查配置
                                              ↓
                                      已配置 → 允许 ✅
                                              ↓
                                      未配置 → 拒绝 ❌
```

### iframe 防护原理

由于 WebView 不支持直接注入 HTTP CSP 头，通过以下方式实现：

1. **CSP Meta 标签**：注入 `Content-Security-Policy` meta 标签
2. **createElement 拦截**：重写 `document.createElement`，拦截 iframe 创建
3. **src 属性拦截**：在 iframe 创建时拦截 `src` 属性设置

### postMessage 验证原理

重写 `window.addEventListener`，在处理 `message` 事件时验证来源：

- 空来源（本地页面）：允许
- `vokex://` 来源：允许
- `http://localhost/*` 或 `http://127.0.0.1/*`：允许
- 其他来源：拒绝并记录警告

## 九、向后兼容性

| 场景 | 行为 |
|------|------|
| 无 security 配置 | 使用默认配置 |
| 现有应用 | 完全不受影响，所有 API 正常工作 |
| 远端页面调用危险 API | 默认拒绝，需在配置中显式允许 |
| 新增 API | 默认可用，除非主动标记为危险 |

## 十、设计优势

1. **危险列表模式**：新增 API 默认可用，降低维护成本
2. **零额外依赖**：使用标准库实现，无第三方依赖
3. **渐进式安全**：本地/开发模式零配置，远端页面安全隔离
4. **清晰错误信息**：拒绝时提供配置指导
5. **通配符支持**：支持 `fs.*` 等模块级授权
