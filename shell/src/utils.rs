use std::env;
use std::path::PathBuf;

pub fn get_args(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == name {
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                return Some(args[i + 1].clone());
            }
            return Some(String::new());
        }
        i += 1;
    }
    None
}

pub fn has_flag(name: &str) -> bool {
    let args: Vec<String> = env::args().collect();
    args.iter().any(|arg| arg == name)
}

/// 解码 PNG 数据为 RGBA 像素（自动处理所有颜色类型）
fn decode_png(data: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(data));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    Some((buf, info.width, info.height))
}

/// 加载图片，返回 RGBA 像素数据及宽高（支持 PNG 和 ICO 格式）
/// 开发模式：从 exe 同目录的文件系统读取，正式模式：从嵌入资源读取
pub fn load_image_rgba(path: &str) -> Option<(Vec<u8>, u32, u32)> {
    let data = if crate::app_config::get_config().is_dev {
        let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
        std::fs::read(exe_dir.join(path)).ok()?
    } else {
        let exe_path = std::env::current_exe().ok()?;
        let resources = crate::Resources::load_from_exe(&exe_path).ok()?;
        resources.get(path)?.to_vec()
    };
    if data.len() < 4 { return None; }

    if data[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        // PNG
        decode_png(&data)
    } else if data[0..4] == [0x00, 0x00, 0x01, 0x00] {
        // ICO：取最大尺寸条目，ico crate 内部处理 PNG 和 BMP
        let icon_dir = ico::IconDir::read(std::io::Cursor::new(&data)).ok()?;
        let entry = icon_dir.entries().iter().max_by_key(|e| e.width())?;
        let image = entry.decode().ok()?;
        let rgba = image.rgba_data().to_vec();
        Some((rgba, image.width(), image.height()))
    } else {
        None
    }
}

/// 加载图片为 tao 窗口图标
pub fn load_image(path: &str) -> Option<tao::window::Icon> {
    let (rgba, width, height) = load_image_rgba(path)?;
    tao::window::Icon::from_rgba(rgba, width, height).ok()
}

// 根据 identifier 创建 WebView 数据目录
pub fn get_webview_data_dir(identifier: &str) -> PathBuf {
    let local_appdata = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| {
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        });
    let data_dir = PathBuf::from(local_appdata).join(identifier);
    std::fs::create_dir_all(&data_dir).ok();
    data_dir
}
