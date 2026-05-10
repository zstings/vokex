use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use tray_icon::{TrayIcon, TrayIconBuilder, Icon};

struct TrayEntry {
    icon: TrayIcon,
}

thread_local! {
    static TRAY_STORE: RefCell<HashMap<u32, TrayEntry>> = RefCell::new(HashMap::new());
    static NEXT_TRAY_ID: RefCell<u32> = RefCell::new(1);
}

pub fn handle(method: &str, params: &Value) -> Result<Value, String> {
    match method {
        "tray.create" => create(params),
        "tray.setToolTip" => set_tooltip(params),
        "tray.setImage" => set_image(params),
        "tray.setMenu" => set_menu(params),
        "tray.setTitle" => set_title(params),
        "tray.destroy" => destroy(params),
        _ => Err(format!("Unknown tray method: {}", method)),
    }
}

/// 加载托盘图标，复用 utils::load_image_rgba 然后创建 tray_icon::Icon
fn load_tray_icon(icon_path: &str) -> Result<Icon, String> {
    let (rgba, width, height) = crate::utils::load_image_rgba(icon_path)
        .ok_or_else(|| format!("Failed to load icon: {}", icon_path))?;
    Icon::from_rgba(rgba, width, height)
        .map_err(|e| format!("Failed to create tray icon: {}", e))
}

fn create(params: &Value) -> Result<Value, String> {
    let icon_path = params.get("icon").and_then(|v| v.as_str())
        .ok_or("Missing 'icon' parameter")?;
    let tooltip = params.get("tooltip").and_then(|v| v.as_str()).unwrap_or("");
    let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("");

    let icon = load_tray_icon(icon_path)?;

    let mut builder = TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip(tooltip);

    if !title.is_empty() {
        builder = builder.with_title(title);
    }

    if let Some(menu_value) = params.get("menu") {
        let menu = crate::apis::menu::build_menu(menu_value)?;
        builder = builder.with_menu(Box::new(menu));
    }

    let tray_icon = builder.build()
        .map_err(|e| format!("Failed to create tray icon: {}", e))?;

    let id = NEXT_TRAY_ID.with(|n| {
        let mut n = n.borrow_mut();
        let id = *n;
        *n += 1;
        id
    });

    TRAY_STORE.with(|store| {
        store.borrow_mut().insert(id, TrayEntry { icon: tray_icon });
    });

    Ok(json!(id))
}

fn set_tooltip(params: &Value) -> Result<Value, String> {
    let id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let text = params.get("text").and_then(|v| v.as_str())
        .ok_or("Missing 'text' parameter")?;

    TRAY_STORE.with(|store| {
        let store = store.borrow();
        if let Some(entry) = store.get(&id) {
            entry.icon.set_tooltip(Some(text))
                .map_err(|e| format!("{}", e))?;
            Ok(json!(true))
        } else {
            Err(format!("Tray {} not found", id))
        }
    })
}

fn set_image(params: &Value) -> Result<Value, String> {
    let id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let icon_path = params.get("icon").and_then(|v| v.as_str())
        .ok_or("Missing 'icon' parameter")?;

    let icon = load_tray_icon(icon_path)?;

    TRAY_STORE.with(|store| {
        let store = store.borrow();
        if let Some(entry) = store.get(&id) {
            entry.icon.set_icon(Some(icon))
                .map_err(|e| format!("{}", e))?;
            Ok(json!(true))
        } else {
            Err(format!("Tray {} not found", id))
        }
    })
}

fn set_menu(params: &Value) -> Result<Value, String> {
    let id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let menu_value = params.get("menu").ok_or("Missing 'menu' parameter")?;

    let menu = crate::apis::menu::build_menu(menu_value)?;

    TRAY_STORE.with(|store| {
        let store = store.borrow();
        if let Some(entry) = store.get(&id) {
            entry.icon.set_menu(Some(Box::new(menu)));
            Ok(json!(true))
        } else {
            Err(format!("Tray {} not found", id))
        }
    })
}

fn set_title(params: &Value) -> Result<Value, String> {
    let id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("");

    TRAY_STORE.with(|store| {
        let store = store.borrow();
        if let Some(entry) = store.get(&id) {
            entry.icon.set_title(Some(title));
            Ok(json!(true))
        } else {
            Err(format!("Tray {} not found", id))
        }
    })
}

fn destroy(params: &Value) -> Result<Value, String> {
    let id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

    TRAY_STORE.with(|store| {
        if store.borrow_mut().remove(&id).is_some() {
            Ok(json!(true))
        } else {
            Err(format!("Tray {} not found", id))
        }
    })
}
