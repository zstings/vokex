use serde_json::{json, Value};

/// 跨线程父窗口句柄包装
/// 在主线程提取原生句柄的 isize 值（Send），后台线程重建为 HasWindowHandle + HasDisplayHandle
struct RawParentHandle {
    raw: isize,
}

impl raw_window_handle::HasWindowHandle for RawParentHandle {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        #[cfg(target_os = "windows")]
        let raw = raw_window_handle::Win32WindowHandle::new(std::num::NonZeroIsize::new(self.raw).unwrap());
        #[cfg(target_os = "linux")]
        let raw = raw_window_handle::XlibWindowHandle::new(self.raw as u64, None);
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw.into()) })
    }
}

impl raw_window_handle::HasDisplayHandle for RawParentHandle {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        #[cfg(target_os = "windows")]
        let raw = raw_window_handle::WindowsDisplayHandle::new();
        // Linux: xdg-portal 仅使用 window ID 构造 "x11:0x..." 字符串，不实际访问 Display 指针
        #[cfg(target_os = "linux")]
        let raw = raw_window_handle::XlibDisplayHandle::new(std::ptr::null_mut(), 0);
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(raw.into()) })
    }
}

fn get_parent(raw_hwnd: Option<isize>) -> Option<RawParentHandle> {
    raw_hwnd.map(|raw| RawParentHandle { raw })
}

pub fn handle(method: &str, params: &Value, _window_id: u32, raw_hwnd: Option<isize>) -> Result<Value, String> {
    match method {
        "dialog.showMessageBox" => handle_show_message_box(params, raw_hwnd),
        "dialog.showErrorBox" => handle_show_error_box(params, raw_hwnd),
        "dialog.showOpenDialog" => handle_show_open_dialog(params, raw_hwnd),
        "dialog.showSaveDialog" => handle_show_save_dialog(params, raw_hwnd),
        _ => Err(format!("Unknown method: {}", method)),
    }
}

/// 解析文件过滤器（健壮版本）
fn parse_filters(filter_arr: Option<&Vec<Value>>) -> Vec<(String, Vec<String>)> {
    filter_arr
        .map(|arr| {
            arr.iter()
                .filter_map(|filter| {
                    let obj = filter.as_object()?;
                    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let extensions: Vec<String> = obj
                        .get("extensions")
                        .and_then(|v| v.as_array())
                        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    if extensions.is_empty() { None } else { Some((name, extensions)) }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 将 PathBuf 转换为 String 的辅助函数
fn path_to_string(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

// ─── showMessageBox ──────────────────────────────────────────────────

fn handle_show_message_box(params: &Value, raw_hwnd: Option<isize>) -> Result<Value, String> {
    let message = params
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'message' parameter")?;
    let icon_type = params.get("icon").and_then(|v| v.as_str());
    let default_title = match icon_type {
        Some("warning") => "警告",
        Some("error") => "错误",
        _ => "提示",
    };
    let title = params.get("title").and_then(|v| v.as_str()).unwrap_or(default_title);

    let level = match icon_type {
        Some("warning") => rfd::MessageLevel::Warning,
        Some("error") => rfd::MessageLevel::Error,
        _ => rfd::MessageLevel::Info,
    };

    // 自定义按钮优先于标准 type
    let (buttons, is_custom) = if let Some(btn_labels) = params.get("buttons").and_then(|v| v.as_array()) {
        let labels: Vec<String> = btn_labels
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        match labels.len() {
            0 => (rfd::MessageButtons::Ok, false),
            1 => (rfd::MessageButtons::OkCustom(labels[0].clone()), true),
            2 => (rfd::MessageButtons::OkCancelCustom(labels[0].clone(), labels[1].clone()), true),
            _ => (rfd::MessageButtons::YesNoCancelCustom(labels[0].clone(), labels[1].clone(), labels[2].clone()), true),
        }
    } else {
        let b = match params.get("type").and_then(|v| v.as_str()) {
            Some("okCancel") => rfd::MessageButtons::OkCancel,
            Some("yesNo") => rfd::MessageButtons::YesNo,
            Some("yesNoCancel") => rfd::MessageButtons::YesNoCancel,
            _ => rfd::MessageButtons::Ok,
        };
        (b, false)
    };

    let mut dialog = rfd::MessageDialog::new()
        .set_title(title)
        .set_description(message)
        .set_level(level)
        .set_buttons(buttons);

    if let Some(parent) = get_parent(raw_hwnd) {
        dialog = dialog.set_parent(&parent);
    }

    let result = dialog.show();

    if is_custom {
        let btn_labels = params.get("buttons").and_then(|v| v.as_array()).unwrap();
        let labels: Vec<&str> = btn_labels.iter().filter_map(|v| v.as_str()).collect();
        let index = match &result {
            rfd::MessageDialogResult::Custom(label) => {
                labels.iter().position(|l| *l == label.as_str()).unwrap_or(0) as u32
            }
            _ => 0,
        };
        Ok(json!(index))
    } else {
        let response = match result {
            rfd::MessageDialogResult::Ok => "ok",
            rfd::MessageDialogResult::Cancel => "cancel",
            rfd::MessageDialogResult::Yes => "yes",
            rfd::MessageDialogResult::No => "no",
            _ => "cancel",
        };
        Ok(json!({ "response": response, "cancelled": response == "cancel" }))
    }
}

// ─── showErrorBox ────────────────────────────────────────────────────

fn handle_show_error_box(params: &Value, raw_hwnd: Option<isize>) -> Result<Value, String> {
    let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("错误");
    let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("");

    let mut dialog = rfd::MessageDialog::new()
        .set_title(title)
        .set_description(message)
        .set_level(rfd::MessageLevel::Error)
        .set_buttons(rfd::MessageButtons::Ok);

    if let Some(parent) = get_parent(raw_hwnd) {
        dialog = dialog.set_parent(&parent);
    }

    dialog.show();

    Ok(json!(true))
}

// ─── showOpenDialog ──────────────────────────────────────────────────

fn handle_show_open_dialog(params: &Value, raw_hwnd: Option<isize>) -> Result<Value, String> {
    let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("打开");
    let multiple = params.get("multiple").and_then(|v| v.as_bool()).unwrap_or(false);
    let directory = params.get("directory").and_then(|v| v.as_bool()).unwrap_or(false);
    let filters = params.get("filters").and_then(|v| v.as_array());

    let mut builder = rfd::FileDialog::new().set_title(title);

    if let Some(parent) = get_parent(raw_hwnd) {
        builder = builder.set_parent(&parent);
    }

    // defaultPath 处理
    if let Some(default_path) = params.get("defaultPath").and_then(|v| v.as_str()) {
        let path = std::path::Path::new(default_path);
        if directory || path.is_dir() {
            builder = builder.set_directory(default_path);
        } else if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                builder = builder.set_directory(parent);
            }
            if params.get("defaultName").is_none() {
                if let Some(name) = path.file_name() {
                    builder = builder.set_file_name(name.to_string_lossy().as_ref());
                }
            }
        }
    }

    if !directory {
        if let Some(default_name) = params.get("defaultName").and_then(|v| v.as_str()) {
            builder = builder.set_file_name(default_name);
        }
        for (name, extensions) in &parse_filters(filters) {
            builder = builder.add_filter(name, extensions);
        }
    }

    if directory {
        if multiple {
            match builder.pick_folders() {
                Some(paths) => {
                    let result: Vec<String> = paths.into_iter().map(|p| path_to_string(&p)).collect();
                    Ok(json!(result))
                }
                None => Ok(json!(null)),
            }
        } else {
            Ok(json!(builder.pick_folder().map(|p| path_to_string(&p))))
        }
    } else if multiple {
        match builder.pick_files() {
            Some(paths) => {
                let result: Vec<String> = paths.into_iter().map(|p| path_to_string(&p)).collect();
                Ok(json!(result))
            }
            None => Ok(json!(null)),
        }
    } else {
        Ok(json!(builder.pick_file().map(|p| path_to_string(&p))))
    }
}

// ─── showSaveDialog ──────────────────────────────────────────────────

fn handle_show_save_dialog(params: &Value, raw_hwnd: Option<isize>) -> Result<Value, String> {
    let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("保存文件");
    let filters = params.get("filters").and_then(|v| v.as_array());

    let mut builder = rfd::FileDialog::new().set_title(title);

    if let Some(parent) = get_parent(raw_hwnd) {
        builder = builder.set_parent(&parent);
    }

    if let Some(default_path) = params.get("defaultPath").and_then(|v| v.as_str()) {
        let path = std::path::Path::new(default_path);
        if path.is_dir() {
            builder = builder.set_directory(default_path);
        } else if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                builder = builder.set_directory(parent);
            }
        }
    }
    if let Some(default_name) = params.get("defaultName").and_then(|v| v.as_str()) {
        builder = builder.set_file_name(default_name);
    }

    for (name, extensions) in &parse_filters(filters) {
        builder = builder.add_filter(name, extensions);
    }

    Ok(json!(builder.save_file().map(|p| path_to_string(&p))))
}
