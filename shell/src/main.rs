//! Vokex Shell - 最小可用版本
//! 只做一件事：用 tao 创建窗口，用 wry 加载空白页面
//!
//! `//!` 是 Rust 的模块级注释（文档注释），描述整个文件的作用

// `#![]` 是 Rust 的属性标记，用来给编译器下指令
// `cfg_attr` 意思是"如果满足某个条件，就加上某个属性"
// `not(debug_assertions)` = 如果不是 debug 模式（即 release 模式）
// `windows_subsystem = "windows"` = 告诉 Windows：这是窗口程序，不要弹黑框控制台
// 效果：debug 编译会有控制台方便看日志，release 编译只有窗口没有控制台
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_config;
mod utils;
mod ipc;
mod window_manager;
mod apis;
mod security;
use utils::{load_image, get_webview_data_dir};
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;
use serde_json::json;
#[cfg(target_os = "windows")]
use raw_window_handle::HasWindowHandle;
use std::io::{self, Read, Seek, SeekFrom};
use std::fs::File;
use std::path::Path;
use serde_json::Value;
use flate2::read::ZlibDecoder;
// `use` = 引入其他模块的东西，类似 JS 的 import
// 从 tao 库的 event 模块引入 Event（所有事件的枚举）和 WindowEvent（窗口事件的枚举）
use tao::event::{Event, WindowEvent};
// 从 tao 的 event_loop 模块引入：
// - ControlFlow：控制事件循环是否继续运行（Wait=继续，Exit=退出）
// - EventLoop：事件循环本身，每个 GUI 程序有且只有一个
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::sync::atomic::{AtomicU64, Ordering};
pub static START_TIME: AtomicU64 = AtomicU64::new(0);

/// 线程池
struct ThreadPool {
    sender: mpsc::Sender<Box<dyn FnOnce() + Send>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
        let receiver = Arc::new(Mutex::new(receiver));
        for _ in 0..size {
            let receiver = receiver.clone();
            thread::spawn(move || {
                while let Ok(task) = receiver.lock().unwrap().recv() {
                    task();
                }
            });
        }
        Self { sender }
    }

    fn run<F: FnOnce() + Send + 'static>(&self, task: F) {
        let _ = self.sender.send(Box::new(task));
    }
}


// ==============================
// 资源加载
// ==============================

const MAGIC: &[u8] = b"VOKEX";
const MAGIC_SIZE: usize = 5;
const INDEX_LENGTH_SIZE: usize = 4;
const OFFSET_SIZE: usize = 8;

#[derive(Debug)]
pub struct Resources {
    index: serde_json::Map<String, Value>,
    data: Vec<u8>,
}

impl Resources {
    pub fn load_from_exe(exe_path: &Path) -> io::Result<Self> {
        let mut file = File::open(exe_path)?;
        let file_size = file.metadata()?.len();

        if file_size < OFFSET_SIZE as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "File too small"));
        }

        file.seek(SeekFrom::End(-(OFFSET_SIZE as i64)))?;
        let mut offset_buf = [0u8; OFFSET_SIZE];
        file.read_exact(&mut offset_buf)?;
        let offset = u64::from_le_bytes(offset_buf);

        if offset >= file_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid offset"));
        }

        file.seek(SeekFrom::Start(offset))?;
        let mut magic_buf = [0u8; MAGIC_SIZE];
        file.read_exact(&mut magic_buf)?;

        if magic_buf != MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }

        let mut index_length_buf = [0u8; INDEX_LENGTH_SIZE];
        file.read_exact(&mut index_length_buf)?;
        let index_length = u32::from_le_bytes(index_length_buf) as usize;

        let mut index_json = vec![0u8; index_length];
        file.read_exact(&mut index_json)?;
        let index: serde_json::Map<String, Value> = serde_json::from_slice(&index_json)?;

        let compressed_data_length = file_size - offset - MAGIC_SIZE as u64 - INDEX_LENGTH_SIZE as u64 - index_length as u64 - OFFSET_SIZE as u64;
        let mut compressed_data = vec![0u8; compressed_data_length as usize];
        file.read_exact(&mut compressed_data)?;

        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut data = Vec::new();
        decoder.read_to_end(&mut data)?;

        Ok(Self { index, data })
    }

    pub fn get(&self, path: &str) -> Option<&[u8]> {
        if let Some(Value::Array(offsets)) = self.index.get(path) {
            if offsets.len() == 2 {
                if let (Some(Value::Number(start)), Some(Value::Number(end))) = (
                    offsets.get(0),
                    offsets.get(1),
                ) {
                    let start = start.as_u64()? as usize;
                    let end = end.as_u64()? as usize;
                    if end <= self.data.len() {
                        return Some(&self.data[start..end]);
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
enum IpcTask {
    HandleRequest { window_id: u32, body: String },
    HandleAsyncResponse { window_id: u32, id: u64, result: Option<serde_json::Value>, error: Option<String> },
    Quit,
    CreateWindow {
        requester_id: u32,
        callback_id: u64,
        params: serde_json::Value,
    },
    ContextMenu {
        window_id: u32,
        callback_id: u64,
        menu: serde_json::Value,
        x: f64,
        y: f64,
    },
    SetApplicationMenu {
        window_id: u32,
        callback_id: u64,
        menu: serde_json::Value,
    },
    RemoveApplicationMenu {
        window_id: u32,
        callback_id: u64,
    },
    /// Shell 流式事件推送（从后台线程发送到主线程执行）
    ShellEmit {
        window_id: u32,
        event: String,
        data: serde_json::Value,
    },
    MenuEvent(muda::MenuEvent),
    TrayEvent(tray_icon::TrayIconEvent),
}

/// 统一构建 WebView（所有窗口共用）
fn build_webview(
    window: &tao::window::Window,
    window_id: u32,
    url: &str,
    web_context: &mut wry::WebContext,
    resources: &Option<std::sync::Arc<Resources>>,
) -> Result<wry::WebView, String> {
    let config = app_config::get_config();
    let mut builder = wry::WebViewBuilder::new_with_web_context(web_context)
        .with_url(url)
        .with_devtools(config.devtools)
        .with_ipc_handler(move |message| {
            ipc::handle_message(window_id, message);
        })
        .with_initialization_script(ipc::get_init_script(window_id))
        // 注入安全脚本：限制 iframe、验证 postMessage 来源
        .with_initialization_script(crate::security::get_security_script());

    if !config.is_dev {
        if let Some(resources) = resources {
            let resources = resources.clone();
            builder = builder.with_custom_protocol(
                "vokex".to_string(),
                move |_webview_id, request| {
                    let uri = request.uri();
                    let path = uri.path().trim_start_matches('/');
                    let path = if path.is_empty() { "index.html" } else { path };
                    if let Some(content) = resources.get(path) {
                        let mime = mime_guess::from_path(path).first_or_text_plain().to_string();
                        wry::http::Response::builder()
                            .header("Content-Type", mime)
                            .body(content.to_vec().into())
                            .unwrap()
                    } else {
                        wry::http::Response::builder()
                            .status(404)
                            .body("Not Found".as_bytes().to_vec().into())
                            .unwrap()
                    }
                },
            );
        }
    }

    #[cfg(target_os = "windows")]
    let builder = builder.with_https_scheme(true);

    // window.open 拦截
    let builder = builder.with_new_window_req_handler(move |url, _features| {
        match config.new_window.value {
            0 => wry::NewWindowResponse::Deny,
            1 => {
                println!("[new_window] intercepted: {}", url);
                let width = config.new_window.width.unwrap_or(800) as f64;
                let height = config.new_window.height.unwrap_or(600) as f64;
                let script = format!(
                    "if(window.__VOKEX__&&window.__VOKEX__.call){{window.__VOKEX__.call('browserWindow.create',{{url:'{}',width:{},height:{}}});}}",
                    url.replace('\\', "\\\\").replace('\'', "\\'"),
                    width, height
                );
                crate::window_manager::eval(window_id, &script);
                wry::NewWindowResponse::Deny
            }
            2 => {
                let script = format!(
                    "if(window.__VOKEX__&&window.__VOKEX__.call){{window.__VOKEX__.call('shell.openExternal',{{url:'{}'}});}}",
                    url.replace('\\', "\\\\").replace('\'', "\\'")
                );
                crate::window_manager::eval(window_id, &script);
                wry::NewWindowResponse::Deny
            }
            _ => wry::NewWindowResponse::Deny,
        }
    });

    builder.build(window).map_err(|e| format!("Failed to build webview: {}", e))
}

/// 显示右键上下文菜单（跨平台）
fn show_context_menu(window_id: u32, menu_json: &serde_json::Value, x: f64, y: f64) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return show_context_menu_windows(window_id, menu_json, x, y);

    #[cfg(target_os = "macos")]
    return show_context_menu_macos(window_id, menu_json, x, y);

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (window_id, menu_json, x, y);
        return Err("Context menu not supported on this platform".to_string());
    }
}

#[cfg(target_os = "windows")]
fn show_context_menu_windows(window_id: u32, menu_json: &serde_json::Value, x: f64, y: f64) -> Result<(), String> {
    use muda::ContextMenu;
    let menu = crate::apis::menu::build_menu(menu_json)?;
    crate::window_manager::MANAGER.with(|m| {
        let manager = m.borrow();
        let entry = manager.get(window_id).ok_or("Window not found")?;
        let handle = entry.window.window_handle()
            .map_err(|e| format!("Failed to get window handle: {}", e))?;
        let raw = handle.as_raw();
        let hwnd = match raw {
            raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get() as isize,
            _ => return Err("Not a Windows window".to_string()),
        };
        unsafe {
            let position = muda::dpi::PhysicalPosition::new(x, y);
            let ok = menu.show_context_menu_for_hwnd(hwnd, Some(position.into()));
            if !ok {
                return Err("Failed to show context menu".to_string());
            }
        }
        Ok(())
    })
}

#[cfg(target_os = "macos")]
fn show_context_menu_macos(window_id: u32, menu_json: &serde_json::Value, x: f64, y: f64) -> Result<(), String> {
    use muda::ContextMenu;
    let menu = crate::apis::menu::build_menu(menu_json)?;
    crate::window_manager::MANAGER.with(|m| {
        let manager = m.borrow();
        let entry = manager.get(window_id).ok_or("Window not found")?;
        let handle = entry.window.window_handle()
            .map_err(|e| format!("Failed to get window handle: {}", e))?;
        let raw = handle.as_raw();
        let ns_view = match raw {
            raw_window_handle::RawWindowHandle::AppKit(h) => h.ns_view.as_ptr(),
            _ => return Err("Not a macOS window".to_string()),
        };
        unsafe {
            let position = muda::dpi::PhysicalPosition::new(x, y);
            let ok = menu.show_context_menu_for_nsview(ns_view as *mut std::ffi::c_void, Some(position.into()));
            if !ok {
                return Err("Failed to show context menu".to_string());
            }
        }
        Ok(())
    })
}

// 程序入口函数
fn main() {
    // 记录进程运行的起始时间
    START_TIME.store(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        Ordering::Relaxed
    );

    // 初始化配置
    app_config::init_app_config();
    let app_config = app_config::get_config().clone();

    println!("{:#?}", app_config);

    // 提前加载 Resources（图标和 custom protocol 都需要）
    let resources = if !app_config.is_dev {
        let exe_path = std::env::current_exe().expect("Failed to get exe path");
        let res = Resources::load_from_exe(&exe_path)
            .expect("Failed to load resources from exe");
        Some(std::sync::Arc::new(res))
    } else {
        None
    };

    // ============================================================
    // 第 1 步：创建事件循环
    // ============================================================
    // 事件循环 = 一个无限循环，不断从操作系统接收事件（点击、按键、拖动等）
    // 它是整个程序的"心脏"，所有 GUI 程序都必须有一个
    let event_loop = EventLoopBuilder::<IpcTask>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    ipc::set_proxy(proxy.clone());

    let thread_pool = Arc::new(ThreadPool::new(4));
    ipc::set_thread_pool(thread_pool.clone());

    // 右键菜单点击事件转发到事件循环（muda 处理右键菜单的事件）
    let menu_proxy = proxy.clone();
    muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
        let _ = menu_proxy.send_event(IpcTask::MenuEvent(event));
    }));

    // 托盘图标点击事件转发到事件循环
    let tray_proxy = proxy.clone();
    tray_icon::TrayIconEvent::set_event_handler(Some(move |event: tray_icon::TrayIconEvent| {
        let _ = tray_proxy.send_event(IpcTask::TrayEvent(event));
    }));

    // 创建窗口（标题、大小、图标 → WindowBuilder）WindowBuilder（窗口壳子）
    let icon = app_config.icon.iter()
        .find_map(|p| load_image(p));
    let window = WindowBuilder::new()
        .with_title(app_config.window.title)
        .with_inner_size(tao::dpi::LogicalSize::new(app_config.window.width, app_config.window.height))
        .with_window_icon(icon)
        .build(&event_loop)
        .unwrap();
    // 创建 WebView（所有窗口共用 web_context）
    let data_dir = get_webview_data_dir(&app_config.identifier);
    let web_context = std::sync::Arc::new(Mutex::new(wry::WebContext::new(Some(data_dir))));
    // 默认 URL：debug 用开发地址，release 用 vokex://index.html
    let default_url = if app_config.is_dev { 
        app_config.dev_url.clone().unwrap_or_else(|| "http://localhost:3000".to_string()) 
    } else { 
        "vokex://index.html".to_string() 
    };
    // 先分配 window_id，这样闭包和 init_script 都能用
    let window_id = window_manager::next_id();
    let origin = security::parse_origin(&default_url);
    // 构建主窗口 WebView
    {
        let mut ctx = web_context.lock().unwrap();
        let webview = build_webview(&window, window_id, &default_url, &mut ctx, &resources).unwrap();
        window_manager::register_with_id(window_id, window, webview, origin, default_url.clone());
    }

    // ============================================================
    // 第 4 步：运行事件循环
    // ============================================================
    // event_loop.run() = 启动事件循环，程序会在这里阻塞，直到退出
    //   move |event, _, control_flow| = 闭包，每次有事件时被调用
    //     move = 闭包获取外部变量的所有权（这里需要获取 window 和 webview 的所有权）
    //     event = 发生的事件（点击、按键、窗口缩放等）
    //     _ = 事件循环目标（EventLoopWindowTarget），这里不需要用，用 _ 忽略
    //     control_flow = 控制事件循环行为的引用（Wait=继续，Exit=退出）
    event_loop.run(move |event, target, control_flow| {
        // 轮询全局快捷键事件（非阻塞，必须在 ControlFlow 设置之前）
        crate::apis::shortcut::poll_events();

        // 设置默认行为：没有事件时休眠等待（不占 CPU）
        // 当有已注册的快捷键时，使用 WaitUntil 定期唤醒以轮询热键事件
        if crate::apis::shortcut::has_hotkeys() {
            *control_flow = ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(50)
            );
        } else {
            *control_flow = ControlFlow::Wait;
        }

        // match = Rust 的模式匹配，类似 switch-case，但更强大
        match event {
            // 匹配"窗口事件"中的"关闭请求"
            // Event::WindowEvent { event, .. } = 解构窗口事件
            //   event = 具体的窗口事件类型
            //   .. = 忽略其他字段（比如 window_id）
            // WindowEvent::CloseRequested = 用户点了窗口右上角的 X 按钮
            Event::WindowEvent { window_id: tao_id, event, .. } => {
                let vokex_id = window_manager::get_id_by_tao_id(tao_id).unwrap_or(0);
                match event {
                    WindowEvent::CloseRequested => {
                        // 只有主窗口关闭才退出应用
                        if vokex_id == 1 {
                            // 先通知前端，给前端留出处理时间（如保存子进程输出）
                            ipc::emit_all("app.before-quit", serde_json::json!({}));
                            // 再清理所有子进程
                            crate::apis::shell::cleanup_all();
                            *control_flow = ControlFlow::Exit;
                        } else if vokex_id != 0 {
                            // 非主窗口：drop Window 来关闭它
                            window_manager::unregister(vokex_id);
                        }
                    }
                    WindowEvent::Destroyed => {
                        ipc::emit(vokex_id, "window.closed", serde_json::json!({
                            "windowId": vokex_id
                        }));
                        // 非主窗口关闭时注销
                        if vokex_id != 0 && vokex_id != 1 {
                            window_manager::unregister(vokex_id);
                        }
                    }
                    WindowEvent::Resized(size) => {
                        ipc::emit(vokex_id, "window.resized", serde_json::json!({
                            "width": size.width,
                            "height": size.height
                        }));
                    }
                    WindowEvent::Moved(position) => {
                        ipc::emit(vokex_id, "window.moved", serde_json::json!({
                            "x": position.x,
                            "y": position.y
                        }));
                    }
                    WindowEvent::Focused(focused) => {
                        let event_name = if focused { "window.focus" } else { "window.blur" };
                        ipc::emit(vokex_id, event_name, serde_json::json!({
                            "focused": focused
                        }));
                    }
                    _ => {}
                }
            },
            
            Event::UserEvent(IpcTask::HandleRequest { window_id, body }) => {
                ipc::process_request(window_id, &body);
            }

            Event::UserEvent(IpcTask::HandleAsyncResponse { window_id, id, result, error }) => {
                ipc::resolve_async_response(window_id, id, result, error);
            }

            Event::UserEvent(IpcTask::Quit) => {
                ipc::emit_all("app.before-quit", serde_json::json!({}));
                *control_flow = ControlFlow::Exit;
            }

            Event::UserEvent(IpcTask::CreateWindow { requester_id, callback_id, params }) => {
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("Vokex").to_string();
                let width = params.get("width").and_then(|v| v.as_f64()).unwrap_or(800.0);
                let height = params.get("height").and_then(|v| v.as_f64()).unwrap_or(600.0);
                let url = {
                    let raw_url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    if raw_url.is_empty() {
                        default_url.clone()
                    } else if raw_url.starts_with("http://") || raw_url.starts_with("https://") || raw_url.starts_with("vokex://") {
                        raw_url.to_string()
                    } else {
                        let base = default_url.trim_end_matches('/');
                        format!("{}/{}", base, raw_url.trim_start_matches('/'))
                    }
                };
                let icon = params.get("icon")
                    .and_then(|v| v.as_str())
                    .and_then(|path| crate::utils::load_image(path));
                let new_window = WindowBuilder::new()
                    .with_title(&title)
                    .with_inner_size(tao::dpi::LogicalSize::new(width, height))
                    .with_window_icon(icon)
                    .build(target)
                    .unwrap();

                let new_window_id = window_manager::next_id();
                let new_origin = security::parse_origin(&url);
                let new_webview = {
                    let mut ctx = web_context.lock().unwrap();
                    let wv = build_webview(&new_window, new_window_id, &url, &mut ctx, &resources).unwrap();
                    wv
                };

                window_manager::register_with_id(new_window_id, new_window, new_webview, new_origin, url);

                let script = ipc::build_response_script(
                    callback_id,
                    Some(serde_json::json!({ "id": new_window_id })),
                    None,
                );
                window_manager::eval(requester_id, &script);
            }
            
            // 右键菜单（muda）
            Event::UserEvent(IpcTask::ContextMenu { window_id, callback_id, menu, x, y }) => {
                let result = show_context_menu(window_id, &menu, x, y);
                let script = ipc::build_response_script(
                    callback_id,
                    Some(json!(true)),
                    result.err(),
                );
                window_manager::eval(window_id, &script);
            }

            // 设置应用菜单栏（必须在事件循环中执行，避免 IPC 消息处理中调用 SetWindowPos 导致消息泵重入）
            Event::UserEvent(IpcTask::SetApplicationMenu { window_id, callback_id, menu }) => {
                let result = crate::apis::menu::set_application_menu(&menu);
                let script = ipc::build_response_script(
                    callback_id,
                    Some(json!(true)),
                    result.err(),
                );
                window_manager::eval(window_id, &script);
            }

            // 移除应用菜单栏
            Event::UserEvent(IpcTask::RemoveApplicationMenu { window_id, callback_id }) => {
                crate::apis::menu::remove_application_menu();
                let script = ipc::build_response_script(
                    callback_id,
                    Some(json!(true)),
                    None,
                );
                window_manager::eval(window_id, &script);
            }
            
            // Shell 流式事件推送（在主线程执行 eval）
            Event::UserEvent(IpcTask::ShellEmit { window_id, event, data }) => {
                ipc::emit(window_id, &event, data);
            }

            // 右键菜单点击事件（muda 处理）
            Event::UserEvent(IpcTask::MenuEvent(event)) => {
                let id = event.id.0.to_string();
                ipc::emit_all("menu.click", json!({ "id": id }));
            }

            // 托盘图标点击事件
            Event::UserEvent(IpcTask::TrayEvent(event)) => {
                use tray_icon::{TrayIconEvent, MouseButton};
                let (event_name, button_name) = match &event {
                    TrayIconEvent::Click { button, .. } => {
                        let btn = match button {
                            MouseButton::Left => "left",
                            MouseButton::Right => "right",
                            MouseButton::Middle => "middle",
                        };
                        ("tray.click", btn)
                    }
                    TrayIconEvent::DoubleClick { button, .. } => {
                        let btn = match button {
                            MouseButton::Left => "left",
                            MouseButton::Right => "right",
                            MouseButton::Middle => "middle",
                        };
                        ("tray.double-click", btn)
                    }
                    _ => ("", ""),
                };
                if !event_name.is_empty() {
                    ipc::emit_all(event_name, json!({
                        "button": button_name
                    }));
                }
            }

            // _ = 其他所有事件，不处理（忽略）
            // 包括：鼠标移动、键盘输入、窗口缩放、系统托盘等
            _ => {}
        }
    });
}
