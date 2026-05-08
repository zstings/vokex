# Vokex 安全限制方案

## 一、方案目标

1. 所有危险 API 在远端页面默认禁用
2. 远端页面使用危险 API 需在配置中手动开启
3. 检查并修复 IPC 通信存在的安全漏洞
4. 保持向后兼容性，不影响本地/开发模式使用

## 二、来源分级

| 来源 | 协议/URL | 信任级别 | API 权限 |
|------|----------|----------|----------|
| **本地资源** | `vokex://` | ✅ 完全信任 | 全部可用 |
| **开发模式** | `http://localhost:*` | ✅ 完全信任 | 全部可用 |
| **远端页面** | `https://` / `http://` | ❌ 不信任 | 仅安全 API + 需配置 |

## 三、危险 API 分类

### 3.1 危险 API（远端默认禁用）

| 模块 | API | 风险说明 |
|------|-----|----------|
| **fs** | `readFile`, `readFileBinary` | 读取任意文件 |
| **fs** | `writeFile`, `appendFile` | 写入任意文件 |
| **fs** | `deleteFile`, `removeDir` | 删除任意文件 |
| **fs** | `copyFile`, `moveFile` | 复制/移动任意文件 |
| **fs** | `createDir` | 在任意位置创建目录 |
| **shell** | `execCommand` | 执行任意系统命令 |
| **shell** | `openPath` | 打开任意文件/目录 |
| **shell** | `trashItem` | 删除任意文件到回收站 |
| **process** | `kill` | 终止任意进程 |
| **dialog** | `showSaveDialog` | 保存文件到任意位置 |

### 3.2 受限 API（远端受限）

| 模块 | API | 限制说明 |
|------|-----|----------|
| **fs** | `readDir`, `stat`, `exists` | 受限于允许访问的目录 |
| **http** | `request`, `get`, `post` 等 | 仅允许配置的域名 |
| **storage** | 全部 | 受限于应用数据目录 |
| **clipboard** | `writeText` | 写入剪贴板 |
| **dialog** | `showOpenDialog` | 仅返回用户选择的文件 |

### 3.3 安全 API（远端可用）

| 模块 | API | 说明 |
|------|-----|------|
| **app** | `getName`, `getVersion` 等 | 只读信息 |
| **computer** | `getCpuInfo`, `getOsInfo` 等 | 只读信息 |
| **notification** | `show` | 用户主动触发 |
| **dialog** | `showMessageBox`, `showErrorBox` | 用户交互 |

## 四、当前安全漏洞

### 4.1 IPC 通信漏洞

| 位置 | 漏洞描述 | 严重程度 |
|------|----------|----------|
| `ipc.rs:43` | `handle_message` 没有验证请求来源 | 高 |
| `ipc.rs:67` | `process_request` 没有区分本地/远端页面 | 高 |
| `ipc.rs:172` | `eval` 直接执行 JS，无来源验证 | 高 |
| `window_manager.rs:87` | `evaluate_script` 可以向任意窗口注入代码 | 中 |

### 4.2 缺少来源追踪

| 问题 | 说明 |
|------|------|
| window_id 伪造 | 攻击者可以伪造 window_id 访问其他窗口的 API |
| 无 WebView URL 追踪 | 无法判断请求来自本地还是远端页面 |
| iframe 跨域无限制 | 远端 iframe 内的 JS 可以通过 postMessage 调用本地 API |

## 五、实现步骤

### Phase 1: 来源追踪机制

**1.1 创建来源类型定义**

```
文件: shell/src/security/origin.rs (新建)

pub enum PageOrigin {
    Local,    // vokex:// 协议
    Dev,      // localhost
    Remote,   // https:// / http://
}
```

**1.2 扩展窗口管理器**

```
文件: shell/src/window_manager.rs

struct WindowEntry {
    window: Window,
    webview: WebView,
    origin: PageOrigin,  // 新增
}
```

**1.3 实现 URL 来源判断**

```
文件: shell/src/security/url_parser.rs (新建)

fn parse_origin(url: &str) -> PageOrigin
fn is_local_url(url: &str) -> bool
fn is_dev_url(url: &str) -> bool
fn is_remote_url(url: &str) -> bool
```

### Phase 2: 权限系统

**2.1 定义权限配置结构**

```
文件: shell/src/security/permissions.rs (新建)

pub struct SecurityConfig {
    pub allowed_remote_apis: HashSet<String>,      // 允许远端使用的危险 API
    pub allowed_http_hosts: Vec<String>,            // 允许的 HTTP 域名
    pub allowed_fs_paths: Vec<PathPattern>,        // 允许的文件系统路径
    pub allow_remote_pages: bool,                  // 是否允许加载远端页面
}
```

**2.2 定义危险 API 列表**

```
文件: shell/src/security/dangerous_apis.rs (新建)

pub static DANGEROUS_APIS: Lazy<HashSet<&'static str>>
pub static SAFE_APIS: Lazy<HashSet<&'static str>>
```

**2.3 实现权限检查函数**

```
文件: shell/src/security/mod.rs

fn check_permission(origin: PageOrigin, method: &str, params: &Value) -> Result<(), String>
fn is_api_allowed(origin: PageOrigin, method: &str) -> bool
```

### Phase 3: IPC 安全集成

**3.1 修改 IpcRequest 结构**

```
文件: shell/src/ipc.rs

#[derive(Deserialize, Debug)]
pub struct IpcRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
    pub window_id: u32,
    pub origin: Option<String>,  // 新增: 请求来源 URL
}
```

**3.2 添加权限检查点**

```
文件: shell/src/ipc.rs

fn process_request(window_id: u32, body: &str, origin_url: Option<String>) {
    // 1. 解析请求
    let req = serde_json::from_str::<IpcRequest>(body)?;

    // 2. 获取页面来源
    let origin = get_window_origin(window_id)
        .or_else(|| parse_origin_from_url(origin_url));

    // 3. 权限检查
    if let Some(origin) = origin {
        if !is_api_allowed(origin, &req.method) {
            return Err("Permission denied".into());
        }
    }

    // 4. 继续处理...
}
```

**3.3 WebView URL 追踪**

```
文件: shell/src/main.rs

fn build_webview(..., url: &str) {
    let origin = parse_origin(url);
    register_with_id(window_id, window, webview, origin);
}
```

### Phase 4: 配置文件扩展

**4.1 扩展 vokex-config.json**

```
文件: shell/src/app_config.rs

pub struct SecurityConfigSx {
    pub allowed_remote_apis: Vec<String>,     // 允许远端使用的危险 API
    pub allowed_hosts: Vec<String>,            // 允许的远端域名
    pub allow_remote_pages: bool,              // 是否允许加载远端页面
    pub allow_remote_iframe: bool,             // 是否允许 iframe 加载远端
}
```

**4.2 默认配置**

```json
{
  "security": {
    "allow_remote_pages": true,
    "allow_remote_iframe": false,
    "allowed_remote_apis": [],
    "allowed_hosts": []
  }
}
```

### Phase 5: CSP 和 iframe 防护

**5.1 注入 CSP 头部**

```
文件: shell/src/main.rs

.with_custom_protocol("vokex", move |webview_id, request| {
    let mut response = /* 正常响应 */;
    response.headers_mut().insert(
        "Content-Security-Policy",
        "default-src 'self'; frame-src 'self';"
    );
    response
})
```

**5.2 postMessage 来源验证**

```
文件: shell/src/runtime/index.ts

window.addEventListener('message', (event) => {
    const allowedOrigins = ['vokex://', ...];
    if (!allowedOrigins.some(o => event.origin.startsWith(o))) {
        return; // 拒绝
    }
    // 处理消息
});
```

## 六、配置示例

### 6.1 默认配置（安全优先）

```json
{
  "identifier": "com.example.myapp",
  "name": "我的应用",
  "version": "1.0.0",
  "security": {
    "allow_remote_pages": true,
    "allow_remote_iframe": false,
    "allowed_remote_apis": [],
    "allowed_hosts": []
  }
}
```

### 6.2 开发者配置（宽松）

```json
{
  "identifier": "com.example.myapp",
  "name": "我的应用",
  "version": "1.0.0",
  "security": {
    "allow_remote_pages": true,
    "allow_remote_iframe": true,
    "allowed_remote_apis": ["fs:*", "shell:*"],
    "allowed_hosts": ["https://api.example.com"]
  }
}
```

## 七、文件变更清单

### 7.1 新建文件

| 文件 | 作用 |
|------|------|
| `shell/src/security/mod.rs` | 安全模块入口 |
| `shell/src/security/origin.rs` | 来源类型定义 |
| `shell/src/security/permissions.rs` | 权限配置和检查 |
| `shell/src/security/dangerous_apis.rs` | 危险 API 列表 |
| `shell/src/security/url_parser.rs` | URL 来源解析 |
| `shell/src/security/inject.rs` | JavaScript 安全注入脚本 |

### 7.2 修改文件

| 文件 | 修改内容 |
|------|----------|
| `shell/src/ipc.rs` | 添加来源追踪和权限检查 |
| `shell/src/window_manager.rs` | 添加 origin 字段 |
| `shell/src/main.rs` | 记录 WebView URL |
| `shell/src/app_config.rs` | 添加安全配置字段 |

## 八、向后兼容性

| 变更 | 兼容性说明 |
|------|------------|
| 现有应用 | 无需修改配置文件，新配置有默认值 |
| 本地应用 | 完全不受影响，所有 API 正常工作 |
| 远端页面 | 默认无法调用危险 API，需显式配置 |
| 已有配置 | 自动补充默认值，无需改动 |

## 九、验证测试

| 测试场景 | 输入 | 预期结果 |
|----------|------|----------|
| 本地页面调用 fs.readFile | `fs.readFile({path: "test.txt"})` | ✅ 成功 |
| 远端页面调用 fs.readFile | `fs.readFile({path: "test.txt"})` | ❌ Permission denied |
| 远端页面调用 fs.readFile（已配置） | 配置 `allowed_remote_apis: ["fs.readFile"]` | ✅ 成功 |
| iframe 加载远端页面 | `<iframe src="https://evil.com">` | ❌ CSP 阻止 |
| 伪造 window_id 访问其他窗口 | 伪造 window_id 请求 | ❌ 权限检查失败 |

## 十、实现进度

| 优先级 | 内容 | 状态 |
|--------|------|------|
| P0 | 来源追踪机制（核心） | ✅ 已完成 |
| P0 | IPC 权限检查 | ✅ 已完成 |
| P1 | 配置文件扩展 | ✅ 已完成 |
| P1 | CSP 和 iframe 防护 | ✅ 已完成 |
| P1 | postMessage 来源验证 | ✅ 已完成 |
| P1 | 远端页面加载控制 | ✅ 已完成 |

**完成度：100%**

---

## 十一、核心实现说明

### 11.1 iframe 防护原理

由于 WebView2 (Windows) / WKWebView (macOS) 不支持直接注入 HTTP CSP 头，我们通过以下方式实现 iframe 防护：

1. **CSP Meta 标签**：注入 `Content-Security-Policy` meta 标签
2. **createElement 拦截**：重写 `document.createElement`，拦截 iframe 创建
3. **src 属性拦截**：在 iframe 创建时拦截 `src` 属性设置

### 11.2 postMessage 验证原理

重写 `window.addEventListener`，在处理 `message` 事件时验证来源：

- 空来源（本地页面）：允许
- `vokex://` 来源：允许
- `http://localhost/*` 或 `http://127.0.0.1/*`：允许
- 其他来源：拒绝并记录警告

### 11.3 权限检查流程

```
请求到达 → 获取 window_id → 查询窗口来源 → 权限检查
                                                      ↓
                                              本地/开发 → 允许
                                                      ↓
                                              远端页面 → 检查 API 分类
                                                            ↓
                                                    安全 API → 允许
                                                            ↓
                                                    危险 API → 检查配置
                                                                  ↓
                                                            已配置 → 允许
                                                                  ↓
                                                            未配置 → 拒绝
```

---

## 十二、配置项说明

### security.allowed_remote_apis

允许远端页面调用的危险 API 列表：

```json
{
  "allowed_remote_apis": ["fs.readFile", "shell.*"]
}
```

支持的格式：
- `"fs.readFile"` - 单个 API
- `"fs.*"` - 模块通配符
- `"*"` - 全局通配符（允许所有）

### security.allow_remote_pages

是否允许通过 `browserWindow.create` 加载远端页面：

- `true`（默认）：允许
- `false`：禁止，调用会返回 Permission denied

### security.allow_remote_iframe

是否允许 iframe 加载远端资源：

- `false`（默认）：禁止，CSP 拦截
- `true`：允许（谨慎使用）

---

## 十三、向后兼容性

| 场景 | 行为 |
|------|------|
| 无 security 配置 | 使用默认配置：允许远端页面，危险 API 禁用 |
| 现有应用 | 完全不受影响，所有 API 正常工作 |
| 远端页面调用危险 API | 默认拒绝，需在配置中显式允许 |

