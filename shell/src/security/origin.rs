//! 页面来源类型定义
//! 
//! 用于区分 WebView 中加载的页面来源，决定其 API 调用权限
//! 
//! 来源类型：
//! - Local: 本地资源（vokex:// 协议），完全信任
//! - Dev: 开发模式（localhost），完全信任
//! - Remote: 远端资源（http/https），不信任，危险 API 默认禁用

use serde::{Deserialize, Serialize};

/// 页面来源枚举
/// 
/// 用于安全权限检查：
/// - Local 和 Dev 被视为可信来源，所有 API 可用
/// - Remote 被视为不可信来源，危险 API 默认禁用
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PageOrigin {
    /// 本地资源（vokex:// 协议），完全信任
    Local,
    /// 开发模式（localhost），完全信任
    Dev,
    /// 远端资源（http/https），不信任
    Remote,
}

impl PageOrigin {
    /// 判断来源是否可信
    /// 
    /// Local 和 Dev 是可信的，Remote 是不可信的
    pub fn is_trusted(&self) -> bool {
        matches!(self, PageOrigin::Local | PageOrigin::Dev)
    }

    /// 转换为字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            PageOrigin::Local => "local",
            PageOrigin::Dev => "dev",
            PageOrigin::Remote => "remote",
        }
    }
}

impl std::fmt::Display for PageOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
