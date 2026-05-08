//! 权限配置和检查逻辑
//! 
//! 提供安全配置的序列化和权限检查函数

use serde::{Deserialize, Serialize};
use crate::security::origin::PageOrigin;
use crate::security::dangerous_apis::is_dangerous_api;

/// 安全配置结构
/// 
/// 在 vokex-config.json 中配置：
/// ```json
/// {
///   "security": {
///     "allowed_remote_apis": ["fs.readFile", "fs.*"],
///     "allow_remote_pages": true
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 允许远端页面调用的危险 API 列表
    /// 支持完整名称（如 "fs.readFile"）或通配符（如 "fs.*"）
    pub allowed_remote_apis: Vec<String>,
    /// 是否允许加载远端页面
    pub allow_remote_pages: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_remote_apis: vec![],
            allow_remote_pages: true,
        }
    }
}

impl SecurityConfig {
    /// 检查 API 是否在允许列表中（支持通配符）
    pub fn is_api_allowed(&self, method: &str) -> bool {
        // 检查完整名称
        if self.allowed_remote_apis.iter().any(|s| s == method) {
            return true;
        }
        // 检查模块通配符（如 "fs.*"）
        let module = method.split('.').next().unwrap_or("");
        if self.allowed_remote_apis.iter().any(|s| s == &format!("{}.*", module)) {
            return true;
        }
        // 检查全局通配符
        if self.allowed_remote_apis.iter().any(|s| s == "*") {
            return true;
        }
        false
    }
}

/// 检查 API 调用权限
/// 
/// 权限检查流程：
/// 1. 如果来源可信（Local/Dev），直接允许
/// 2. 如果不是危险 API，直接允许
/// 3. 危险 API 检查是否在 allowed_remote_apis 配置中
pub fn check_api_permission(
    origin: PageOrigin,
    method: &str,
    config: &SecurityConfig,
) -> Result<(), String> {
    // 可信来源：Local 和 Dev 所有 API 都可用
    if origin.is_trusted() {
        return Ok(());
    }

    // 非危险 API：任何来源都可以调用
    if !is_dangerous_api(method) {
        return Ok(());
    }

    // 危险 API：远端来源需要显式配置
    if config.is_api_allowed(method) {
        return Ok(());
    }

    Err(format!(
        "Permission denied: '{}' is not allowed for remote pages. Add it to 'security.allowed_remote_apis' in vokex-config.json",
        method
    ))
}
