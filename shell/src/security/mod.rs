/// 安全模块 - 实现来源追踪和权限检查
/// 
/// 架构：
/// - origin: 页面来源类型定义（Local/Dev/Remote）
/// - url_parser: URL 来源解析工具
/// - dangerous_apis: 危险 API 列表定义
/// - permissions: 权限配置和检查逻辑
/// - inject: JavaScript 安全注入脚本

pub mod origin;
pub mod url_parser;
pub mod dangerous_apis;
pub mod permissions;
pub mod inject;

/// 导出常用类型和函数
pub use origin::PageOrigin;
pub use url_parser::{parse_origin, is_local_url, is_dev_url, is_remote_url, get_host, is_same_origin};
pub use dangerous_apis::{is_dangerous_api, is_restricted_api, is_safe_api, is_api_allowed_for_remote};
pub use permissions::{SecurityConfig, check_api_permission};
pub use inject::get_security_script;

/// 允许的开发模式主机地址
pub const ALLOWED_DEV_HOSTS: &[&str] = &[
    "localhost",
    "127.0.0.1",
];
