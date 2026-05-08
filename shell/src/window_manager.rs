use std::cell::RefCell;
use std::collections::HashMap;
use tao::window::{Window, WindowId};
use wry::WebView;
use serde::Serialize;
use crate::security::origin::PageOrigin;

/// 窗口信息
#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub id: u32,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_maximized: bool,
    pub is_minimized: bool,
    pub is_fullscreen: bool,
    pub is_focused: bool,
    pub is_visible: bool,
    pub origin: String,
}

impl WindowEntry {
    pub fn get_info(&self, id: u32) -> WindowInfo {
        let size = self.window.inner_size();
        let pos = self.window.outer_position().unwrap_or(tao::dpi::PhysicalPosition::new(0, 0));
        WindowInfo {
            id,
            title: self.window.title(),
            width: size.width,
            height: size.height,
            x: pos.x,
            y: pos.y,
            is_maximized: self.window.is_maximized(),
            is_minimized: self.window.is_minimized(),
            is_fullscreen: self.window.fullscreen().is_some(),
            is_focused: self.window.is_focused(),
            is_visible: self.window.is_visible(),
            origin: self.origin.to_string(),
        }
    }
}

/// 窗口条目，同时持有 tao Window 和 wry WebView
pub struct WindowEntry {
    pub window: Window,
    pub webview: WebView,
    /// 页面来源：Local/Dev/Remote，用于安全权限检查
    pub origin: PageOrigin,
    /// 页面 URL，用于追踪和同源验证
    pub url: String,
}

/// 窗口管理器 - 存储所有窗口的 WebView
/// 只在主线程使用，所以用 RefCell 不需要 Mutex
pub struct WindowManager {
    next_id: u32,
    windows: HashMap<u32, WindowEntry>,
    /// tao WindowId → vokex window_id 的映射
    tao_to_vokex: HashMap<WindowId, u32>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            windows: HashMap::new(),
            tao_to_vokex: HashMap::new(),
        }
    }

    /// 注册窗口，返回分配的窗口 ID
    pub fn register(&mut self, window: Window, webview: WebView, origin: PageOrigin, url: String) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let tao_id = window.id();
        self.tao_to_vokex.insert(tao_id, id);
        self.windows.insert(id, WindowEntry { window, webview, origin, url });
        id
    }

    /// 注销窗口
    pub fn unregister(&mut self, id: u32) {
        if let Some(entry) = self.windows.remove(&id) {
            self.tao_to_vokex.remove(&entry.window.id());
        }
    }

    /// 向指定窗口执行 JS 脚本
    pub fn eval(&self, id: u32, script: &str) {
        if let Some(entry) = self.windows.get(&id) {
            let _ = entry.webview.evaluate_script(script);
        }
    }

    /// 向所有窗口执行 JS 脚本
    pub fn eval_all(&self, script: &str) {
        for entry in self.windows.values() {
            let _ = entry.webview.evaluate_script(script);
        }
    }

    /// 获取所有窗口 ID
    pub fn get_all_ids(&self) -> Vec<u32> {
        self.windows.keys().copied().collect()
    }

    /// 获取指定窗口的引用
    pub fn get(&self, id: u32) -> Option<&WindowEntry> {
        self.windows.get(&id)
    }

    /// 获取指定窗口的可变引用
    pub fn get_mut(&mut self, id: u32) -> Option<&mut WindowEntry> {
        self.windows.get_mut(&id)
    }

    /// 通过 tao WindowId 查找 vokex window_id
    pub fn get_id_by_tao_id(&self, tao_id: WindowId) -> Option<u32> {
        self.tao_to_vokex.get(&tao_id).copied()
    }

    /// 获取主窗口 ID（第一个注册的窗口，ID 为 1）
    pub fn get_main_window_id(&self) -> Option<u32> {
        if self.windows.contains_key(&1) {
            Some(1)
        } else {
            None
        }
    }

    /// 获取窗口的页面来源，用于安全权限检查
    pub fn get_origin(&self, id: u32) -> Option<PageOrigin> {
        self.windows.get(&id).map(|e| e.origin)
    }

    /// 获取窗口的页面 URL
    pub fn get_url(&self, id: u32) -> Option<&str> {
        self.windows.get(&id).map(|e| e.url.as_str())
    }
}

// 全局单例，thread_local 因为只在主线程用
thread_local! {
    pub static MANAGER: RefCell<WindowManager> = RefCell::new(WindowManager::new());
}

/// 注册窗口，返回窗口 ID
pub fn register(window: Window, webview: WebView, origin: PageOrigin, url: String) -> u32 {
    MANAGER.with(|m| m.borrow_mut().register(window, webview, origin, url))
}

/// 注销窗口
pub fn unregister(id: u32) {
    MANAGER.with(|m| m.borrow_mut().unregister(id));
}

/// 向指定窗口执行 JS
pub fn eval(id: u32, script: &str) {
    MANAGER.with(|m| m.borrow().eval(id, script));
}

/// 向所有窗口执行 JS
pub fn eval_all(script: &str) {
    MANAGER.with(|m| m.borrow().eval_all(script));
}

/// 预分配下一个窗口 ID（不注册 WebView）
pub fn next_id() -> u32 {
    MANAGER.with(|m| {
        let id = m.borrow().next_id;
        m.borrow_mut().next_id += 1;
        id
    })
}

/// 用指定 ID 注册窗口（主窗口用，因为 ID 在 WebView 创建前就分配了）
pub fn register_with_id(id: u32, window: Window, webview: WebView, origin: PageOrigin, url: String) {
    MANAGER.with(|m| {
        let tao_id = window.id();
        m.borrow_mut().tao_to_vokex.insert(tao_id, id);
        m.borrow_mut().windows.insert(id, WindowEntry { window, webview, origin, url });
    });
}

pub fn get_all_ids() -> Vec<u32> {
    MANAGER.with(|m| m.borrow().get_all_ids())
}

pub fn get_id_by_tao_id(tao_id: WindowId) -> Option<u32> {
    MANAGER.with(|m| m.borrow().get_id_by_tao_id(tao_id))
}

pub fn get_main_window_id() -> Option<u32> {
    MANAGER.with(|m| m.borrow().get_main_window_id())
}

/// 获取窗口的页面来源，用于安全权限检查
pub fn get_window_origin(id: u32) -> Option<PageOrigin> {
    MANAGER.with(|m| m.borrow().get_origin(id))
}

/// 获取窗口的页面 URL
pub fn get_window_url(id: u32) -> Option<String> {
    MANAGER.with(|m| m.borrow().get_url(id).map(|s| s.to_string()))
}
