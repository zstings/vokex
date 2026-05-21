use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock, Mutex};
use tao::event_loop::EventLoopProxy;

#[derive(Deserialize, Debug)]
pub struct IpcRequest {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub window_id: u32,
}

#[derive(Serialize)]
struct IpcResponse {
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

/// 全局 PROXY，供后台线程发送事件到主线程
/// 使用 std::sync::LazyLock + Mutex 保证跨线程安全
static GLOBAL_PROXY: LazyLock<Mutex<Option<EventLoopProxy<crate::IpcTask>>>> =
    LazyLock::new(|| Mutex::new(None));

/// 线程池类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PoolType {
    /// 网络请求线程池
    Network,
    /// 文件IO/存储线程池
    IO,
}

/// 网络请求专用线程池
static NETWORK_POOL: LazyLock<Mutex<Option<Arc<crate::ThreadPool>>>> =
    LazyLock::new(|| Mutex::new(None));

/// 文件IO/存储专用线程池
static IO_POOL: LazyLock<Mutex<Option<Arc<crate::ThreadPool>>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn set_thread_pools(network_pool: Arc<crate::ThreadPool>, io_pool: Arc<crate::ThreadPool>) {
    *NETWORK_POOL.lock().unwrap() = Some(network_pool);
    *IO_POOL.lock().unwrap() = Some(io_pool);
}

pub fn set_proxy(proxy: EventLoopProxy<crate::IpcTask>) {
    *GLOBAL_PROXY.lock().unwrap() = Some(proxy);
}

/// 通过全局 proxy 发送 shell 流式事件到主线程执行
/// 可在任意后台线程调用，事件会被投递到主线程事件循环中执行 eval
pub fn emit_via_proxy(window_id: u32, event: String, data: serde_json::Value) {
    if let Some(proxy) = GLOBAL_PROXY.lock().unwrap().as_ref() {
        let _ = proxy.send_event(crate::IpcTask::ShellEmit { window_id, event, data });
    }
}

pub fn handle_message(window_id: u32, request: wry::http::Request<String>) {
    let body = request.into_body();
    if let Some(proxy) = GLOBAL_PROXY.lock().unwrap().as_ref() {
        let _ = proxy.send_event(crate::IpcTask::HandleRequest { window_id, body });
    }
}

/// 在主线程提取窗口原生句柄的 isize 值，用于跨线程传递给 dialog 模块的 set_parent()
/// isize 是 Send，可安全从主线程传递到线程池 worker
fn extract_parent_hwnd(method: &str, window_id: u32) -> Option<isize> {
    if !method.starts_with("dialog.") {
        return None;
    }
    crate::window_manager::MANAGER.with(|m| {
        let manager = m.borrow();
        manager.get(window_id).and_then(|entry| {
            use raw_window_handle::HasWindowHandle;
            entry.window.window_handle().ok().and_then(|h| {
                match h.as_raw() {
                    #[cfg(target_os = "windows")]
                    raw_window_handle::RawWindowHandle::Win32(w) => Some(w.hwnd.get() as isize),
                    #[cfg(target_os = "linux")]
                    raw_window_handle::RawWindowHandle::Xlib(w) => Some(w.window as isize),
                    _ => None,
                }
            })
        })
    })
}

/// 判断 API 属于哪个线程池
fn get_pool_type(method: &str) -> Option<PoolType> {
    if method == "http.request" {
        return Some(PoolType::Network);
    }
    if matches!(
        method,
        "fs.readFile" | "fs.writeFile" | "fs.rm" |
        "fs.readdir" | "fs.mkdir" | "fs.stat" |
        "fs.exists" | "fs.copyFile" | "fs.rename" |
        "fs.glob" | "fs.globStream" |
        "shell.exec" | "shell.spawn" | "shell.kill" |
        "process.getUptime" | "process.getCpuUsage" | "process.getMemoryInfo" |
        "safeStorage.setData" | "safeStorage.getData" | "safeStorage.getKeys" |
        "safeStorage.has" | "safeStorage.removeData" | "safeStorage.clear" |
        "storage.setData" | "storage.getData" | "storage.getKeys" |
        "storage.has" | "storage.removeData" | "storage.clear"
    ) {
        return Some(PoolType::IO);
    }
    None
}

/// 辅助函数：将任务发送到指定线程池
fn send_to_pool(
    pool_type: PoolType,
    request_id: u64,
    method: String,
    params: serde_json::Value,
    window_id: u32,
) {
    let raw_hwnd = extract_parent_hwnd(&method, window_id);
    let proxy = GLOBAL_PROXY.lock().unwrap().clone();
    let wid = window_id;

    let pool = match pool_type {
        PoolType::Network => NETWORK_POOL.lock().unwrap().clone(),
        PoolType::IO => IO_POOL.lock().unwrap().clone(),
    };

    if let Some(pool) = pool {
        pool.run(move || {
            let response = match dispatch(&method, &params, wid, raw_hwnd) {
                Ok(result) => IpcResponse { id: request_id, result: Some(result), error: None },
                Err(err) => IpcResponse { id: request_id, result: None, error: Some(err) },
            };

            if let Some(proxy) = proxy {
                let _ = proxy.send_event(crate::IpcTask::HandleAsyncResponse {
                    window_id: wid,
                    id: response.id,
                    result: response.result,
                    error: response.error,
                });
            }
        });
    }
}

pub fn process_request(window_id: u32, body: &str) {
    let req: IpcRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[IPC] Invalid message: {}", e);
            return;
        }
    };

    eprintln!("[IPC] window_id={}, method={}", window_id, req.method);

    // 安全检查：根据页面来源检查 API 调用权限
    // 远端页面默认禁止调用危险 API，除非在配置中显式允许
    if let Some(origin) = crate::window_manager::get_window_origin(window_id) {
        let config = crate::app_config::get_config();
        if let Err(e) = crate::security::check_api_permission(origin, &req.method, &config.security) {
            eprintln!("[IPC] Permission denied: {}", e);
            // 返回权限拒绝错误给前端
            let response = IpcResponse {
                id: req.id,
                result: None,
                error: Some(e),
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            let script = format!("window.__VOKEX_IPC__ && window.__VOKEX_IPC__({})", json);
            crate::window_manager::eval(window_id, &script);
            return;
        }
    }

    // browserWindow.create 需要在事件循环中创建窗口
    if req.method == "browserWindow.create" {
        // 安全检查：远端页面加载控制
        let config = crate::app_config::get_config();
        let url = req.params.get("url").and_then(|v| v.as_str()).unwrap_or("");
        if crate::security::is_remote_url(url) && !config.security.allow_remote_pages {
            let e = "Permission denied: remote pages are not allowed. Set 'security.allow_remote_pages' to true in vokex-config.json to enable.";
            eprintln!("[IPC] {}", e);
            let response = IpcResponse {
                id: req.id,
                result: None,
                error: Some(e.to_string()),
            };
            let json = serde_json::to_string(&response).unwrap_or_default();
            let script = format!("window.__VOKEX_IPC__ && window.__VOKEX_IPC__({})", json);
            crate::window_manager::eval(window_id, &script);
            return;
        }

        if let Some(proxy) = GLOBAL_PROXY.lock().unwrap().as_ref() {
            let _ = proxy.send_event(crate::IpcTask::CreateWindow {
                requester_id: window_id,
                callback_id: req.id,
                params: req.params,
            });
        }
        return;
    }
    // menu.setApplicationMenu 路由到事件循环（避免 IPC 消息处理中调用 SetWindowPos 导致消息泵重入）
    if req.method == "menu.setApplicationMenu" {
        let menu = req.params.get("menu").cloned().unwrap_or(serde_json::json!([]));
        if let Some(proxy) = GLOBAL_PROXY.lock().unwrap().as_ref() {
            let _ = proxy.send_event(crate::IpcTask::SetApplicationMenu {
                window_id,
                callback_id: req.id,
                menu,
            });
        }
        return;
    }
    if req.method == "menu.removeApplicationMenu" {
        if let Some(proxy) = GLOBAL_PROXY.lock().unwrap().as_ref() {
            let _ = proxy.send_event(crate::IpcTask::RemoveApplicationMenu {
                window_id,
                callback_id: req.id,
            });
        }
        return;
    }
    if req.method == "menu.setContextMenu" {
        let x = req.params.get("x").and_then(|v: &serde_json::Value| v.as_f64()).unwrap_or(0.0);
        let y = req.params.get("y").and_then(|v: &serde_json::Value| v.as_f64()).unwrap_or(0.0);
        let menu = req.params.get("menu").cloned().unwrap_or(serde_json::json!([]));
        if let Some(proxy) = GLOBAL_PROXY.lock().unwrap().as_ref() {
            let _ = proxy.send_event(crate::IpcTask::ContextMenu {
                window_id,
                callback_id: req.id,
                menu,
                x,
                y,
            });
        }
        return;
    }

    // 检查是否需要发送到线程池
    if let Some(pool_type) = get_pool_type(&req.method) {
        send_to_pool(pool_type, req.id, req.method, req.params, window_id);
    } else {
        // 同步 API：直接在主线程执行
        let raw_hwnd = extract_parent_hwnd(&req.method, window_id);
        let response = match dispatch(&req.method, &req.params, window_id, raw_hwnd) {
            Ok(result) => IpcResponse { id: req.id, result: Some(result), error: None },
            Err(err) => IpcResponse { id: req.id, result: None, error: Some(err) },
        };

        let json = serde_json::to_string(&response).unwrap_or_default();
        let script = format!(
            "window.__VOKEX_IPC__ && window.__VOKEX_IPC__({})",
            json
        );
        crate::window_manager::eval(window_id, &script);
    }
}

/// 处理异步 API 的返回结果（在主线程执行）
pub fn resolve_async_response(window_id: u32, id: u64, result: Option<serde_json::Value>, error: Option<String>) {
    let response = IpcResponse { id, result, error };
    let json = serde_json::to_string(&response).unwrap_or_default();
    let script = format!(
        "window.__VOKEX_IPC__ && window.__VOKEX_IPC__({})",
        json
    );
    crate::window_manager::eval(window_id, &script);
}

fn dispatch(method: &str, params: &serde_json::Value, window_id: u32, raw_hwnd: Option<isize>) -> Result<serde_json::Value, String> {
    // 按模块前缀分发
    if let Some(module) = method.split('.').next() {
        match module {
            "app" => crate::apis::app::handle(method, params),
            "fs" => crate::apis::fs::handle(method, params, window_id),
            "browserWindow" => crate::apis::browser_window::handle(method, params),
            "storage" => crate::apis::storage::handle(method, params),
            "safeStorage" => crate::apis::safe_storage::handle(method, params),
            "shell" => crate::apis::shell::handle(method, params, window_id),
            "process" => crate::apis::process::handle(method, params),
            "http" => crate::apis::http::handle(method, params, window_id),
            "clipboard" => crate::apis::clipboard::handle(method, params),
            "dialog" => crate::apis::dialog::handle(method, params, window_id, raw_hwnd),
            "notification" => crate::apis::notification::handle(method, params),
            "computer" => crate::apis::computer::handle(method, params),
            "tray" => crate::apis::tray::handle(method, params),
            "shortcut" => crate::apis::shortcut::handle(method, params),
            "path" => crate::apis::path::handle(method, params),
            _ => Err(format!("Unknown method: {}", method)),
        }
    } else {
        Err(format!("Invalid method: {}", method))
    }
}

/// 向指定窗口推送事件
pub fn emit(window_id: u32, event: &str, data: serde_json::Value) {
    let event_escaped = event.replace('\\', "\\\\").replace('\'', "\\'");
    let data_json = serde_json::to_string(&data).unwrap_or("null".to_string());
    let script = format!(
        "window.__VOKEX__ && window.__VOKEX__.__emit__('{}', {})",
        event_escaped, data_json
    );
    crate::window_manager::eval(window_id, &script);
}

/// 向所有窗口推送事件
pub fn emit_all(event: &str, data: serde_json::Value) {
    let event_escaped = event.replace('\\', "\\\\").replace('\'', "\\'");
    let data_json = serde_json::to_string(&data).unwrap_or("null".to_string());
    let script = format!(
        "window.__VOKEX__ && window.__VOKEX__.__emit__('{}', {})",
        event_escaped, data_json
    );
    crate::window_manager::eval_all(&script);
}

/// 发送退出事件到主线程
pub fn send_quit_event() {
    if let Some(proxy) = GLOBAL_PROXY.lock().unwrap().as_ref() {
        let _ = proxy.send_event(crate::IpcTask::Quit);
    }
}

/// 构建 IPC 响应脚本（供 main.rs 中的 CreateWindow 等使用）
pub fn build_response_script(id: u64, result: Option<serde_json::Value>, error: Option<String>) -> String {
    let response = IpcResponse { id, result, error };
    let json = serde_json::to_string(&response).unwrap_or_default();
    format!("window.__VOKEX_IPC__ && window.__VOKEX_IPC__({})", json)
}

pub fn get_init_script(window_id: u32) -> String {
    r#"
    (function() {
        var _pendingCalls = new Map();
        var _callId = 0;
        var _windowId = __WINDOW_ID__;

        window.__VOKEX__ = {
            __windowId__: _windowId,
            call: function(method, params) {
                var id = ++_callId;
                return new Promise(function(resolve, reject) {
                    _pendingCalls.set(id, { resolve: resolve, reject: reject });
                    window.ipc.postMessage(JSON.stringify({
                        id: id,
                        method: method,
                        params: params || {},
                        windowId: _windowId
                    }));
                });
            },
            on: function(event, listener) {
                if (!this.__eventListeners) this.__eventListeners = {};
                if (!this.__eventListeners[event]) this.__eventListeners[event] = [];
                this.__eventListeners[event].push(listener);
                return listener;
            },
            off: function(event, listener) {
                if (!this.__eventListeners || !this.__eventListeners[event]) return;
                this.__eventListeners[event] = this.__eventListeners[event].filter(function(l) { return l !== listener; });
            },
            __emit__: function(event, data) {
                if (!this.__eventListeners || !this.__eventListeners[event]) return;
                var listeners = this.__eventListeners[event];
                for (var i = 0; i < listeners.length; i++) {
                    try { listeners[i](data); } catch(e) { console.error('Event listener error:', e); }
                }
            }
        };

        window.__VOKEX_IPC__ = function(response) {
            var callback = _pendingCalls.get(response.id);
            if (callback) {
                _pendingCalls.delete(response.id);
                if (response.error) {
                    callback.reject(new Error(response.error));
                } else {
                    callback.resolve(response.result);
                }
            }
        };
        // 延迟触发 app.ready，确保前端 JS 已注册监听器
        setTimeout(function() {
            window.__VOKEX__.__emit__('app.ready', {});
        }, 100);
    })();
    "#
    .replace("__WINDOW_ID__", &window_id.to_string())
}
