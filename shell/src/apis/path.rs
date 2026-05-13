use serde_json::{json, Value};
use std::path::{Component, Path, PathBuf};

pub fn handle(method: &str, params: &Value) -> Result<Value, String> {
    match method {
        "path.join" => {
            let paths = params
                .get("paths")
                .and_then(|v| v.as_array())
                .ok_or("Missing 'paths' parameter")?;
            let result = join(paths);
            Ok(json!(result))
        }
        "path.resolve" => {
            let paths = params
                .get("paths")
                .and_then(|v| v.as_array())
                .ok_or("Missing 'paths' parameter")?;
            let result = resolve(paths)?;
            Ok(json!(result))
        }
        "path.normalize" => {
            let p = params
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path' parameter")?;
            let result = normalize(p);
            Ok(json!(result))
        }
        "path.getSep" => {
            #[cfg(windows)]
            { Ok(json!("\\")) }
            #[cfg(not(windows))]
            { Ok(json!("/")) }
        }
        _ => Err(format!("Unknown method: {}", method)),
    }
}

/// 规范化路径：处理 `.` 和 `..`，统一使用平台分隔符
fn normalize(p: &str) -> String {
    let path = Path::new(p);
    let mut components = path.components().peekable();

    // 提取前缀（Windows 盘符 C: 或 UNC \\server\share）
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek() {
        let buf = PathBuf::from(c.as_os_str());
        components.next();
        buf
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::RootDir => {
                ret.push(Component::RootDir);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                // 只在安全时 pop：不弹出根目录，不弹出 .. 本身
                let can_pop = match ret.components().last() {
                    Some(Component::Normal(_)) => true,
                    Some(Component::Prefix(_)) => false,
                    Some(Component::RootDir) => false,
                    Some(Component::ParentDir) => false,
                    Some(Component::CurDir) => false,
                    None => false,
                };
                if can_pop {
                    ret.pop();
                } else if !ret.has_root() {
                    // 相对路径且无法 pop，保留 ..
                    ret.push(Component::ParentDir);
                }
                // 绝对路径的多余 .. 直接丢弃
            }
            Component::Normal(c) => {
                ret.push(c);
            }
            // 前缀已在循环外提取，不会进入此分支
            Component::Prefix(_) => {}
        }
    }

    let s = path_to_string(&ret);
    if s.is_empty() { ".".to_string() } else { s }
}

fn join(segments: &[Value]) -> String {
    if segments.is_empty() {
        return ".".to_string();
    }

    let mut result = PathBuf::new();
    for seg in segments {
        if let Some(s) = seg.as_str() {
            result.push(s);
        }
    }
    normalize(&path_to_string(&result))
}

fn resolve(segments: &[Value]) -> Result<String, String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    let mut base: Option<PathBuf> = None;

    for seg in segments {
        if let Some(s) = seg.as_str() {
            let p = Path::new(s);
            if p.is_absolute() {
                // 完整绝对路径（含盘符或 UNC），直接作为新基准
                base = Some(p.to_path_buf());
            } else {
                // 检查是否为 Unix 风格绝对路径（Windows 上 /foo 不算 is_absolute）
                #[cfg(windows)]
                if s.starts_with('/') || s.starts_with('\\') {
                    // 转为当前盘符根目录 + 剩余部分
                    let drive = cwd.components().find_map(|c| match c {
                        Component::Prefix(p) => Some(p.as_os_str().to_string_lossy().to_string()),
                        _ => None,
                    });
                    if let Some(d) = drive {
                        let mut new_base = PathBuf::from(format!("{}\\", d));
                        // 去掉开头的 / 或 \
                        let trimmed = s.trim_start_matches(|c| c == '/' || c == '\\');
                        new_base.push(trimmed);
                        base = Some(new_base);
                        continue;
                    }
                }

                let current_base = base.as_ref().unwrap_or(&cwd);
                let mut new_path = current_base.clone();
                new_path.push(p);
                base = Some(new_path);
            }
        }
    }

    let final_path = base.unwrap_or(cwd);
    Ok(normalize(&path_to_string(&final_path)))
}

fn path_to_string(p: &Path) -> String {
    p.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_join_simple() {
        let result = join(&[json!("users"), json!("admin"), json!("config.json")]);
        assert!(!result.contains(".."));
        assert!(!result.contains("/./"));
        assert!(result.contains("users"));
        assert!(result.contains("config.json"));
    }

    #[test]
    fn test_join_with_dotdot() {
        let result = join(&[json!("a"), json!("b"), json!(".."), json!("c")]);
        // a/b/../c → a/c
        assert!(!result.contains(".."));
        assert!(result.contains("a"));
        assert!(result.contains("c"));
        assert!(!result.contains("b"));
    }

    #[test]
    fn test_join_dotdot_at_root() {
        let result = join(&[json!("a"), json!("..")]);
        // a/.. → .
        assert_eq!(result, ".");
    }

    #[test]
    fn test_join_leading_dotdot() {
        let result = join(&[json!(".."), json!("a")]);
        assert!(result.contains(".."));
        assert!(result.contains("a"));
    }

    #[test]
    fn test_join_empty() {
        let result = join(&[]);
        assert_eq!(result, ".");
    }

    #[test]
    fn test_join_single() {
        let result = join(&[json!("foo")]);
        assert_eq!(result, "foo");
    }

    #[test]
    fn test_normalize_dots() {
        let result = normalize("/home/user/../admin/./config.json");
        assert!(!result.contains(".."));
        assert!(!result.contains("/./"));
        assert!(result.contains("admin"));
        assert!(result.contains("config.json"));
    }

    #[test]
    fn test_normalize_consecutive_slashes() {
        let result = normalize("foo//bar///baz");
        assert!(!result.contains("//"));
        assert!(result.contains("foo"));
        assert!(result.contains("bar"));
        assert!(result.contains("baz"));
    }

    #[test]
    fn test_normalize_dotdot_at_root() {
        let result = normalize("/..");
        // 根目录的 .. 应被忽略，仍然是 /
        assert!(!result.contains(".."));
    }

    #[test]
    fn test_normalize_empty() {
        assert_eq!(normalize(""), ".");
    }

    #[test]
    fn test_normalize_curdir() {
        let result = normalize("./foo/./bar");
        assert!(!result.contains("/./"));
        assert!(result.contains("foo"));
        assert!(result.contains("bar"));
    }

    #[test]
    fn test_resolve_absolute() {
        let result = resolve(&[json!("temp")]).unwrap();
        assert!(Path::new(&result).is_absolute());
        assert!(result.ends_with("temp"));
    }

    #[test]
    fn test_resolve_with_dotdot() {
        let result = resolve(&[json!("a"), json!("b"), json!(".."), json!("c")]).unwrap();
        assert!(result.ends_with("a") || result.contains("a"));
        assert!(result.contains("c"));
        assert!(!result.contains(".."));
    }

    #[test]
    fn test_resolve_overrides_previous() {
        // 第二个绝对路径应覆盖第一个
        let result = resolve(&[json!("/first"), json!("/second")]).unwrap();
        assert!(result.contains("second"));
        assert!(!result.contains("first"));
    }

    #[test]
    fn test_get_sep() {
        let result = handle("path.getSep", &json!({})).unwrap();
        let sep = result.as_str().unwrap();
        #[cfg(windows)]
        assert_eq!(sep, "\\");
        #[cfg(not(windows))]
        assert_eq!(sep, "/");
    }

    #[test]
    fn test_unknown_method() {
        let result = handle("path.unknown", &json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_join_preserves_leading_dotdot() {
        // ../a/b 应保留开头的 ..
        let result = join(&[json!(".."), json!("a"), json!("b")]);
        assert!(result.starts_with(".."));
    }

    #[test]
    fn test_join_multiple_dotdot() {
        // a/../../b → ../b
        let result = join(&[json!("a"), json!(".."), json!(".."), json!("b")]);
        assert!(result.contains(".."));
        assert!(result.contains("b"));
        assert!(!result.contains("a"));
    }
}
