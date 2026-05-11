//! 全局快捷键 API 模块
//!
//! 基于 `global-hotkey` crate 实现跨平台全局快捷键注册与管理。
//! 使用 `thread_local!` 维护状态（与 window_manager 设计一致），仅在主线程访问。

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;

/// 快捷键管理器
///
/// 存储已注册的热键映射：
/// - `id_to_hotkey`: HotKey ID → HotKey（用于 unregister）
/// - `id_to_accelerator`: HotKey ID → 加速器字符串（用于事件通知和 list 查询）
struct ShortcutManager {
    manager: GlobalHotKeyManager,
    id_to_hotkey: HashMap<u32, HotKey>,
    id_to_accelerator: HashMap<u32, String>,
}

impl ShortcutManager {
    fn new() -> Self {
        Self {
            manager: GlobalHotKeyManager::new().expect("Failed to create GlobalHotKeyManager"),
            id_to_hotkey: HashMap::new(),
            id_to_accelerator: HashMap::new(),
        }
    }
}

// 全局单例，thread_local 因为只在主线程用
thread_local! {
    static SHORTCUT: RefCell<ShortcutManager> = RefCell::new(ShortcutManager::new());
}

/// IPC 分发入口
///
/// 所有 `shortcut.*` 方法统一在此处理。注册/注销操作调用 `GlobalHotKeyManager`，
/// 查询操作读取本地映射表。
pub fn handle(method: &str, params: &Value) -> Result<Value, String> {
    match method {
        "shortcut.register" => register(params),
        "shortcut.unregister" => unregister(params),
        "shortcut.isRegistered" => is_registered(params),
        "shortcut.list" => list(),
        _ => Err(format!("Unknown shortcut method: {}", method)),
    }
}

/// 注册全局快捷键
///
/// 参数：
/// - `accelerator`: 加速器字符串，如 "Ctrl+Shift+A"、"CmdOrCtrl+S"
///
/// 返回：HotKey ID（u32），前端可用于后续 unregister 或事件匹配
///
/// 解析失败返回错误，重复注册同一加速器会先注销旧的再注册新的。
fn register(params: &Value) -> Result<Value, String> {
    let accelerator = params
        .get("accelerator")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'accelerator' parameter")?;

    SHORTCUT.with(|s| {
        let mut s = s.borrow_mut();

        // 如果同一加速器已注册，先注销旧的
        if let Some((&old_id, _)) = s
            .id_to_accelerator
            .iter()
            .find(|(_, acc)| acc.as_str() == accelerator)
        {
            if let Some(old_hotkey) = s.id_to_hotkey.remove(&old_id) {
                let _ = s.manager.unregister(old_hotkey);
            }
            s.id_to_accelerator.remove(&old_id);
        }

        // 解析加速器字符串为 HotKey
        // 格式：Modifiers+Key，如 "Ctrl+Shift+A"、"Alt+F4"、"CmdOrCtrl+S"
        let hotkey = HotKey::from_str(accelerator)
            .map_err(|e| format!("Invalid accelerator '{}': {}", accelerator, e))?;

        let id = hotkey.id();

        // 注册到系统
        s.manager
            .register(hotkey)
            .map_err(|e| format!("Failed to register hotkey '{}': {}", accelerator, e))?;

        // 记录映射
        s.id_to_hotkey.insert(id, hotkey);
        s.id_to_accelerator.insert(id, accelerator.to_string());

        Ok(json!(id))
    })
}

/// 注销全局快捷键
///
/// 参数：
/// - `id`: HotKey ID（由 register 返回）
fn unregister(params: &Value) -> Result<Value, String> {
    let id = params
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or("Missing 'id' parameter")? as u32;

    SHORTCUT.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(hotkey) = s.id_to_hotkey.remove(&id) {
            s.manager
                .unregister(hotkey)
                .map_err(|e| format!("Failed to unregister hotkey {}: {}", id, e))?;
            s.id_to_accelerator.remove(&id);
            Ok(json!(true))
        } else {
            Err(format!("Hotkey {} not found", id))
        }
    })
}

/// 查询加速器是否已注册
///
/// 参数：
/// - `accelerator`: 加速器字符串
///
/// 返回：布尔值
fn is_registered(params: &Value) -> Result<Value, String> {
    let accelerator = params
        .get("accelerator")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'accelerator' parameter")?;

    SHORTCUT.with(|s| {
        let s = s.borrow();
        let found = s
            .id_to_accelerator
            .values()
            .any(|acc| acc.as_str() == accelerator);
        Ok(json!(found))
    })
}

/// 列出所有已注册的快捷键
///
/// 返回：`[{ id, accelerator }, ...]`
fn list() -> Result<Value, String> {
    SHORTCUT.with(|s| {
        let s = s.borrow();
        let list: Vec<Value> = s
            .id_to_accelerator
            .iter()
            .map(|(&id, acc)| json!({ "id": id, "accelerator": acc }))
            .collect();
        Ok(json!(list))
    })
}

/// 检查是否有已注册的快捷键
///
/// 返回 true 时，事件循环应使用 `ControlFlow::WaitUntil` 定期唤醒以轮询事件。
/// 返回 false 时，可以安全使用 `ControlFlow::Wait` 休眠。
pub fn has_hotkeys() -> bool {
    SHORTCUT.with(|s| !s.borrow().id_to_hotkey.is_empty())
}

/// 轮询全局快捷键事件（在主事件循环中调用）
///
/// 非阻塞读取 `GlobalHotKeyEvent` 通道，当热键按下时，
/// 构造 `shortcut.triggered` 事件并广播到所有窗口。
///
/// 返回：是否有事件被处理（供调用方决定是否继续轮询）
pub fn poll_events() -> bool {
    let receiver = GlobalHotKeyEvent::receiver();
    let mut had_events = false;

    while let Ok(event) = receiver.try_recv() {
        had_events = true;

        // 只处理按下事件（忽略 Released）
        if event.state != HotKeyState::Pressed {
            continue;
        }

        // 查找对应的加速器字符串
        let accelerator = SHORTCUT.with(|s| {
            s.borrow()
                .id_to_accelerator
                .get(&event.id)
                .cloned()
        });

        if let Some(accelerator) = accelerator {
            // 广播到所有窗口：window.__VOKEX__.__emit__('shortcut.triggered', { id, accelerator })
            crate::ipc::emit_all(
                "shortcut.triggered",
                json!({ "id": event.id, "accelerator": accelerator }),
            );
        }
    }

    had_events
}
