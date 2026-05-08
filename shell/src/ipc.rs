use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use tao::event_loop::EventLoopProxy;
use std::sync::Arc;

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

thread_local! {
    pub static PROXY: RefCell<Option<EventLoopProxy<crate::IpcTask>>> = RefCell::new(None);
}

thread_local! {
    pub static THREAD_POOL: RefCell<Option<Arc<crate::ThreadPool>>> = RefCell::new(None);
}

pub fn set_thread_pool(pool: Arc<crate::ThreadPool>) {
    THREAD_POOL.with(|p| {
        *p.borrow_mut() = Some(pool);
    });
}

pub fn set_proxy(proxy: EventLoopProxy<crate::IpcTask>) {
    PROXY.with(|p| {
        *p.borrow_mut() = Some(proxy);
    });
}

pub fn handle_message(window_id: u32, request: wry::http::Request<String>) {
    let body = request.into_body();
    PROXY.with(|p| {
        if let Some(proxy) = p.borrow().as_ref() {
            let _ = proxy.send_event(crate::IpcTask::HandleRequest { window_id, body });
        }
    });
}

/// 判断是否为耗时 API（需要在线程池中执行）
fn is_async_api(method: &str) -> bool {
    matches!(
        method,
        "fs.readFile" | "fs.readFileBinary" | "fs.writeFile" |
        "fs.appendFile" | "fs.deleteFile" | "fs.readDir" |
        "fs.createDir" | "fs.removeDir" | "fs.stat" |
        "fs.exists" | "fs.copyFile" | "fs.moveFile" |
        "http.request" | "http.get" | "http.post" |
        "http.put" | "http.delete" |
        "shell.execCommand" |
        "process.getUptime" | "process.getCpuUsage" | "process.getMemoryInfo"
    )
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

        PROXY.with(|p| {
            if let Some(proxy) = p.borrow().as_ref() {
                let _ = proxy.send_event(crate::IpcTask::CreateWindow {
                    requester_id: window_id,
                    callback_id: req.id,
                    params: req.params,
                });
            }
        });
        return;
    }
    // menu.setApplicationMenu 路由到事件循环（避免 IPC 消息处理中调用 SetWindowPos 导致消息泵重入）
    if req.method == "menu.setApplicationMenu" {
        let menu = req.params.get("menu").cloned().unwrap_or(serde_json::json!([]));
        PROXY.with(|p| {
            if let Some(proxy) = p.borrow().as_ref() {
                let _ = proxy.send_event(crate::IpcTask::SetApplicationMenu {
                    window_id,
                    callback_id: req.id,
                    menu,
                });
            }
        });
        return;
    }
    if req.method == "menu.removeApplicationMenu" {
        PROXY.with(|p| {
            if let Some(proxy) = p.borrow().as_ref() {
                let _ = proxy.send_event(crate::IpcTask::RemoveApplicationMenu {
                    window_id,
                    callback_id: req.id,
                });
            }
        });
        return;
    }
    if req.method == "menu.setContextMenu" {
        let x = req.params.get("x").and_then(|v: &serde_json::Value| v.as_f64()).unwrap_or(0.0);
        let y = req.params.get("y").and_then(|v: &serde_json::Value| v.as_f64()).unwrap_or(0.0);
        let menu = req.params.get("menu").cloned().unwrap_or(serde_json::json!([]));
        PROXY.with(|p| {
            if let Some(proxy) = p.borrow().as_ref() {
                let _ = proxy.send_event(crate::IpcTask::ContextMenu {
                    window_id,
                    callback_id: req.id,
                    menu,
                    x,
                    y,
                });
            }
        });
        return;
    }

    if is_async_api(&req.method) {
        // 异步 API：投递到线程池执行，结果通过 proxy 回主线程
        let proxy = PROXY.with(|p| p.borrow().as_ref().map(|p| p.clone()));
        let method = req.method.clone();
        let params = req.params.clone();

        THREAD_POOL.with(|tp| {
            if let Some(pool) = tp.borrow().as_ref() {
                let pool = pool.clone();
                pool.run(move || {
                    let response = match dispatch(&method, &params) {
                        Ok(result) => IpcResponse { id: req.id, result: Some(result), error: None },
                        Err(err) => IpcResponse { id: req.id, result: None, error: Some(err) },
                    };

                    if let Some(proxy) = proxy {
                        let _ = proxy.send_event(crate::IpcTask::HandleAsyncResponse {
                            window_id,
                            id: response.id,
                            result: response.result,
                            error: response.error,
                        });
                    }
                });
            }
        });
    } else {
        // 同步 API：直接在主线程执行
        let response = match dispatch(&req.method, &req.params) {
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

fn dispatch(method: &str, params: &serde_json::Value) -> Result<serde_json::Value, String> {
    // 按模块前缀分发
    if let Some(module) = method.split('.').next() {
        match module {
            "app" => crate::apis::app::handle(method, params),
            "fs" => crate::apis::fs::handle(method, params),
            "browserWindow" => crate::apis::browser_window::handle(method, params),
            "storage" => crate::apis::storage::handle(method, params),
            "shell" => crate::apis::shell::handle(method, params),
            "process" => crate::apis::process::handle(method, params),
            "http" => crate::apis::http::handle(method, params),
            "clipboard" => crate::apis::clipboard::handle(method, params),
            "dialog" => crate::apis::dialog::handle(method, params),
            "notification" => crate::apis::notification::handle(method, params),
            "computer" => crate::apis::computer::handle(method, params),
            "tray" => crate::apis::tray::handle(method, params),
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
    PROXY.with(|p| {
        if let Some(proxy) = p.borrow().as_ref() {
            let _ = proxy.send_event(crate::IpcTask::Quit);
        }
    });
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
