//! 权限配置和检查逻辑
//! 
//! 提供安全配置的序列化和权限检查函数：
//! - SecurityConfig: 安全配置结构，支持 JSON 反序列化
//! - check_api_permission: 根据来源和配置检查 API 调用权限

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::security::origin::PageOrigin;
use crate::security::dangerous_apis::{is_dangerous_api, is_restricted_api, is_safe_api};

/// 安全配置结构
/// 
/// 在 vokex-config.json 中配置：
/// ```json
/// {
///   "security": {
///     "allowed_remote_apis": ["fs.readFile", "fs.*"],
///     "allowed_hosts": ["example.com", "*.api.com"],
///     "allow_remote_pages": true,
///     "allow_remote_iframe": false
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 允许远端页面调用的危险 API 列表
    /// 支持完整名称（如 "fs.readFile"）或通配符（如 "fs.*"）
    pub allowed_remote_apis: Vec<String>,
    /// 允许加载的远端域名列表
    /// 支持精确匹配（如 "example.com"）或子域名（如 "*.api.com"）
    pub allowed_hosts: Vec<String>,
    /// 是否允许加载远端页面
    pub allow_remote_pages: bool,
    /// 是否允许 iframe 加载远端资源
    pub allow_remote_iframe: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            // 默认空列表，远端页面无法调用危险 API
            allowed_remote_apis: vec![],
            // 默认空列表，允许所有域名（与 allow_remote_pages 配合使用）
            allowed_hosts: vec![],
            // 默认允许加载远端页面
            allow_remote_pages: true,
            // 默认禁止 iframe 加载远端资源
            allow_remote_iframe: false,
        }
    }
}

impl SecurityConfig {
    /// 将 allowed_remote_apis 转换为 HashSet，支持通配符匹配
    /// 
    /// 支持的格式：
    /// - "*" - 允许所有危险 API
    /// - "fs.*" - 允许 fs 模块所有危险 API
    /// - "fs.readFile" - 允许特定 API
    pub fn allowed_remote_api_set(&self) -> HashSet<&'static str> {
        self.allowed_remote_apis
            .iter()
            .filter_map(|s| {
                match s.as_str() {
                    "*" => Some("*"),
                    s if s.ends_with(".*") => Some(s),
                    s => Some(s),
                }
            })
            .collect()
    }

    /// 检查主机是否在允许列表中
    /// 
    /// 匹配规则：
    /// - 精确匹配："example.com" 匹配 "example.com"
    /// - 子域名："example.com" 匹配 "api.example.com"
    /// - 通配符："*" 匹配所有域名
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if self.allowed_hosts.is_empty() {
            // 空列表表示允许所有
            return true;
        }
        self.allowed_hosts.iter().any(|h| {
            host == h || host.ends_with(&format!(".{}", h)) || h == "*"
        })
    }
}

/// 检查 API 调用权限
/// 
/// 权限检查流程：
/// 1. 如果来源可信（Local/Dev），直接允许
/// 2. 如果是安全 API，直接允许
/// 3. 如果是受限 API，直接允许
/// 4. 如果是危险 API，检查是否在 allowed_remote_apis 中
/// 
/// 返回：
/// - Ok(()) - 允许调用
/// - Err(message) - 拒绝调用，包含错误信息
pub fn check_api_permission(
    origin: PageOrigin,
    method: &str,
    config: &SecurityConfig,
) -> Result<(), String> {
    // 可信来源：Local 和 Dev 所有 API 都可用
    if origin.is_trusted() {
        return Ok(());
    }

    // 安全 API：任何来源都可以调用
    if is_safe_api(method) {
        return Ok(());
    }

    // 受限 API：暂时直接允许，后续可添加更多限制
    if is_restricted_api(method) {
        return Ok(());
    }

    // 危险 API：远端来源需要显式配置
    if is_dangerous_api(method) {
        let allowed = config.allowed_remote_api_set();
        // 检查完整 API 名称
        if allowed.contains(method) {
            return Ok(());
        }
        // 检查模块通配符（如 "fs.*"）
        let module = method.split('.').next().unwrap_or("");
        if allowed.contains(&format!("{}.*", module)) {
            return Ok(());
        }
        // 检查全局通配符
        if allowed.contains("*") {
            return Ok(());
        }
        // 拒绝访问，提供清晰的错误信息
        return Err(format!(
            "Permission denied: '{}' is not allowed for remote pages. Add it to 'security.allowed_remote_apis' in vokex-config.json",
            method
        ));
    }

    // 未知 API：默认允许（向后兼容）
    Ok(())
}
