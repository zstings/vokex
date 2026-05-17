use std::sync::OnceLock;
use serde::Deserialize;
use crate::Resources;
use crate::security::SecurityConfig;

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct AppConfigSx {
    pub identifier: String,
    pub name: String,
    pub version: String,
    pub icon: Vec<String>,
    pub window: AppConfigWindowSx,
    pub dev_url: Option<String>,
    pub devtools: bool,
    pub new_window: AppConfigNewWindow,
    pub is_dev: bool,
    pub security: SecurityConfig,
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigWindowSx {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub resizable: Option<bool>,
    pub fullscreen: Option<bool>,
    pub maximized: Option<bool>,
    pub transparent: Option<bool>,
    pub decorations: Option<bool>,
    pub always_on_top: Option<bool>,
    pub center: Option<bool>,
    pub background_color: Option<String>,
    pub skip_taskbar: Option<bool>,
    pub opacity: Option<f64>,
    pub closable: Option<bool>,
    pub minimizable: Option<bool>,
    pub maximizable: Option<bool>,
    pub content_protection: Option<bool>,
    pub icon: Option<String>,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct AppConfigNewWindow {
    pub value: u32,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

static GLOBAL_CONFIG: OnceLock<AppConfigSx> = OnceLock::new();

pub fn init_app_config() {
    // 检查命令行参数中是否包含 "--env_dev"
    let is_dev = std::env::args().any(|arg| arg == "--env_dev");
    let mut config = load_config(is_dev);
    config.is_dev = is_dev;
    GLOBAL_CONFIG.set(config).expect("Failed to initialize app config");
}

fn load_config(is_dev: bool) -> AppConfigSx {
    if is_dev {
        return load_config_dev();
    } else {
        return load_config_prod();
    }
}

fn load_config_dev() -> AppConfigSx {
    // 读取壳目录下的 vokex-config.json
    let config_path = std::env::current_exe()
        .expect("Failed to get exe path")
        .parent()
        .expect("Failed to get exe directory")
        .join("vokex-config.json");

    let config_json = std::fs::read_to_string(&config_path)
        .expect("Failed to read vokex-config.json");

    serde_json::from_str(&config_json)
        .expect("Failed to parse vokex-config.json")
}

fn load_config_prod() -> AppConfigSx {
    let exe_path = std::env::current_exe()
        .expect("Failed to get current exe path");
    
    let resources = Resources::load_from_exe(&exe_path)
        .expect("Failed to load resources from exe");
    
    let config_bytes = resources.get("vokex-config.json")
        .expect("vokex-config.json not found in resources");

    let config_json = String::from_utf8(config_bytes.to_vec())
        .expect("vokex-config.json is not valid UTF-8");

    serde_json::from_str(&config_json)
        .expect("Failed to parse vokex-config.json")
}

pub fn get_config() -> &'static AppConfigSx {
    GLOBAL_CONFIG.get().expect("App config not initialized")
}

#[cfg(test)]
pub fn init_test_config() {
    let config = AppConfigSx {
        identifier: "com.vokex.test".to_string(),
        name: "Vokex Test".to_string(),
        version: "0.1.0".to_string(),
        icon: vec![],
        window: AppConfigWindowSx {
            title: "Test".to_string(),
            width: 800,
            height: 600,
            ..Default::default()
        },
        ..Default::default()
    };
    // OnceLock::set 只会成功一次，后续调用自动忽略
    let _ = GLOBAL_CONFIG.set(config);
}
