//! URL 来源解析工具
//! 
//! 根据 URL 判断页面来源类型：
//! - vokex:// → Local
//! - http://localhost/* 或 http://127.0.0.1/* → Dev
//! - https://* 或 http://* → Remote

use crate::security::origin::PageOrigin;

/// 根据 URL 解析页面来源类型
/// 
/// 判断规则：
/// - vokex:// → PageOrigin::Local
/// - http://localhost/* 或 http://127.0.0.1/* → PageOrigin::Dev
/// - https://* 或 http://* → PageOrigin::Remote
/// - 其他 → PageOrigin::Local（默认）
pub fn parse_origin(url: &str) -> PageOrigin {
    let url_lower = url.to_lowercase();

    if url_lower.starts_with("vokex://") {
        return PageOrigin::Local;
    }

    if url_lower.starts_with("http://localhost") || url_lower.starts_with("http://127.0.0.1") {
        return PageOrigin::Dev;
    }

    if url_lower.starts_with("https://") || url_lower.starts_with("http://") {
        return PageOrigin::Remote;
    }

    PageOrigin::Local
}

/// 判断 URL 是否为本地资源
pub fn is_local_url(url: &str) -> bool {
    parse_origin(url) == PageOrigin::Local
}

/// 判断 URL 是否为开发模式（localhost）
pub fn is_dev_url(url: &str) -> bool {
    parse_origin(url) == PageOrigin::Dev
}

/// 判断 URL 是否为远端资源
pub fn is_remote_url(url: &str) -> bool {
    parse_origin(url) == PageOrigin::Remote
}

/// 从 URL 中提取主机名部分
/// 
/// 例如：
/// - "vokex://index.html" → Some("vokex")
/// - "https://example.com/path" → Some("example.com")
/// - "http://localhost:3000/" → Some("localhost:3000")
pub fn get_host(url: &str) -> Option<String> {
    let url_lower = url.to_lowercase();

    if url_lower.starts_with("vokex://") {
        return Some("vokex".to_string());
    }

    if let Some(path_start) = url_lower.find("://") {
        let after_scheme = &url[path_start + 3..];
        if let Some(path_end) = after_scheme.find('/') {
            return Some(after_scheme[..path_end].to_string());
        } else {
            return Some(after_scheme.to_string());
        }
    }

    None
}

/// 判断两个 URL 是否同源
/// 
/// 同源规则：
/// - 完全相同的 URL → 同源
/// - 来源类型不同 → 不同源
/// - Local 永远同源
/// - Dev/Remote 需要主机名相同
pub fn is_same_origin(url1: &str, url2: &str) -> bool {
    if url1 == url2 {
        return true;
    }

    let origin1 = parse_origin(url1);
    let origin2 = parse_origin(url2);

    if origin1 != origin2 {
        return false;
    }

    match origin1 {
        PageOrigin::Local => true,
        PageOrigin::Dev => get_host(url1) == get_host(url2),
        PageOrigin::Remote => {
            get_host(url1) == get_host(url2)
        }
    }
}
