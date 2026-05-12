use base64::Engine;
use glob::{glob_with, MatchOptions, Pattern};
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// 处理 fs 模块的 API 调用
pub fn handle(method: &str, params: &Value, _window_id: u32) -> Result<Value, String> {
    match method {
        "fs.readFile" => read_file(params),
        "fs.writeFile" => write_file(params),
        "fs.rm" => rm(params),
        "fs.readdir" => readdir(params),
        "fs.mkdir" => mkdir(params),
        "fs.stat" => stat(params),
        "fs.exists" => exists(params),
        "fs.copyFile" => copy_file(params),
        "fs.rename" => rename(params),
        "fs.glob" => glob(params),
        _ => Err(format!("Unknown fs method: {}", method)),
    }
}

// ==============================
// 工具函数
// ==============================

/// 规范化路径：统一将 Windows 反斜杠转为正斜杠
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// 构造 Node.js 风格的错误消息
fn node_error(code: &str, syscall: &str, path: &str) -> String {
    let msg = match code {
        "ENOENT" => "no such file or directory",
        "EEXIST" => "file already exists",
        "EISDIR" => "illegal operation on a directory",
        "ENOTDIR" => "not a directory",
        "EACCES" => "permission denied",
        "EPERM" => "operation not permitted",
        "ENOTEMPTY" => "directory not empty",
        "EBUSY" => "resource busy or locked",
        _ => "unknown error",
    };
    format!("{}: {}, {} '{}'", code, msg, syscall, path)
}

/// 将 std::io::Error 映射为 Node.js 风格错误
fn io_error_to_node_error(e: &std::io::Error, syscall: &str, path: &str) -> String {
    let code = match e.kind() {
        std::io::ErrorKind::NotFound => "ENOENT",
        std::io::ErrorKind::AlreadyExists => "EEXIST",
        std::io::ErrorKind::PermissionDenied => "EACCES",
        std::io::ErrorKind::NotADirectory => "ENOTDIR",
        std::io::ErrorKind::DirectoryNotEmpty => "ENOTEMPTY",
        _ => "EIO",
    };
    node_error(code, syscall, path)
}

/// 将文件时间戳转为 Unix 毫秒时间戳
fn time_to_ms(time: std::io::Result<std::time::SystemTime>) -> u64 {
    time.ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ==============================
// API 实现
// ==============================

fn read_file(params: &Value) -> Result<Value, String> {
    let raw_path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let path = normalize_path(raw_path);
    let encoding = params.get("encoding").and_then(|v| v.as_str());

    match encoding {
        Some("base64") => {
            let bytes = std::fs::read(&path)
                .map_err(|e| io_error_to_node_error(&e, "open", &path))?;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
            Ok(json!(encoded))
        }
        Some("hex") => {
            let bytes = std::fs::read(&path)
                .map_err(|e| io_error_to_node_error(&e, "open", &path))?;
            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
            Ok(json!(hex))
        }
        Some("utf8") => {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| io_error_to_node_error(&e, "open", &path))?;
            Ok(json!(content))
        }
        _ => {
            // 无编码：返回 base64（由 TS 端转为 Uint8Array，比 JSON 数组高效得多）
            let bytes = std::fs::read(&path)
                .map_err(|e| io_error_to_node_error(&e, "open", &path))?;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
            Ok(json!(encoded))
        }
    }
}

fn write_file(params: &Value) -> Result<Value, String> {
    let raw_path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let path = normalize_path(raw_path);
    let data = params.get("data").ok_or("Missing param: data")?;
    let flag = params.get("flag").and_then(|v| v.as_str()).unwrap_or("w");
    let encoding = params.get("encoding").and_then(|v| v.as_str()).unwrap_or("utf8");

    // 解析写入数据
    let bytes: Vec<u8> = match data {
        Value::String(s) => match encoding {
            "base64" => base64::engine::general_purpose::STANDARD
                .decode(s)
                .map_err(|e| format!("Invalid base64 data: {}", e))?,
            _ => s.as_bytes().to_vec(),
        },
        Value::Array(arr) => {
            // number[] 形式的二进制数据
            arr.iter()
                .map(|v| {
                    v.as_u64()
                        .ok_or("Invalid byte value in data array".to_string())
                        .and_then(|n| {
                            if n <= 255 {
                                Ok(n as u8)
                            } else {
                                Err("Byte value out of range (0-255)".to_string())
                            }
                        })
                })
                .collect::<Result<Vec<u8>, String>>()?
        }
        _ => return Err("data must be a string or number array".to_string()),
    };

    // 构建 OpenOptions
    let mut opts = OpenOptions::new();
    match flag {
        "a" => {
            opts.create(true).append(true);
        }
        "wx" => {
            opts.write(true).create_new(true);
        }
        _ => {
            // 'w' (默认)
            opts.write(true).create(true).truncate(true);
        }
    }

    let mut file = opts
        .open(&path)
        .map_err(|e| io_error_to_node_error(&e, "open", &path))?;
    file.write_all(&bytes)
        .map_err(|e| io_error_to_node_error(&e, "write", &path))?;

    Ok(json!(null))
}

fn rm(params: &Value) -> Result<Value, String> {
    let raw_path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let path = normalize_path(raw_path);
    let recursive = params.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
    let force = params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);

    let p = Path::new(&path);

    // force 模式下路径不存在直接返回成功
    if force && !p.exists() {
        return Ok(json!(null));
    }

    if p.is_dir() {
        if recursive {
            std::fs::remove_dir_all(&path)
                .map_err(|e| io_error_to_node_error(&e, "rm", &path))?;
        } else {
            // 非递归删除目录，目录必须为空
            std::fs::remove_dir(&path)
                .map_err(|e| io_error_to_node_error(&e, "rm", &path))?;
        }
    } else {
        std::fs::remove_file(&path)
            .map_err(|e| io_error_to_node_error(&e, "rm", &path))?;
    }

    Ok(json!(null))
}

fn readdir(params: &Value) -> Result<Value, String> {
    let raw_path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let path = normalize_path(raw_path);
    let with_file_types = params.get("withFileTypes").and_then(|v| v.as_bool()).unwrap_or(false);

    let entries = std::fs::read_dir(&path)
        .map_err(|e| io_error_to_node_error(&e, "scandir", &path))?;

    let mut result = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| io_error_to_node_error(&e, "scandir", &path))?;
        let name = entry.file_name().into_string().unwrap_or_default();

        if with_file_types {
            let ft = entry.file_type().map_err(|e| io_error_to_node_error(&e, "scandir", &path))?;
            result.push(json!({
                "name": name,
                "isFile": ft.is_file(),
                "isDir": ft.is_dir(),
                "isSymlink": ft.is_symlink()
            }));
        } else {
            result.push(json!(name));
        }
    }

    Ok(json!(result))
}

fn mkdir(params: &Value) -> Result<Value, String> {
    let raw_path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let path = normalize_path(raw_path);
    let recursive = params.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);

    if recursive {
        std::fs::create_dir_all(&path)
            .map_err(|e| io_error_to_node_error(&e, "mkdir", &path))?;
    } else {
        std::fs::create_dir(&path)
            .map_err(|e| io_error_to_node_error(&e, "mkdir", &path))?;
    }

    Ok(json!(null))
}

fn stat(params: &Value) -> Result<Value, String> {
    let raw_path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let path = normalize_path(raw_path);

    let metadata = std::fs::metadata(&path)
        .map_err(|e| io_error_to_node_error(&e, "stat", &path))?;

    let atime_ms = time_to_ms(metadata.accessed());
    let mtime_ms = time_to_ms(metadata.modified());
    let birthtime_ms = time_to_ms(metadata.created());

    // Unix mode（Windows 上默认返回 0）
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() as u64
    };
    #[cfg(not(unix))]
    let mode: u64 = 0;

    Ok(json!({
        "isFile": metadata.is_file(),
        "isDir": metadata.is_dir(),
        "isSymlink": metadata.file_type().is_symlink(),
        "size": metadata.len(),
        "atimeMs": atime_ms,
        "mtimeMs": mtime_ms,
        "birthtimeMs": birthtime_ms,
        "mode": mode
    }))
}

fn exists(params: &Value) -> Result<Value, String> {
    let raw_path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let path = normalize_path(raw_path);
    Ok(json!(Path::new(&path).exists()))
}

fn copy_file(params: &Value) -> Result<Value, String> {
    let raw_src = params.get("src").and_then(|v| v.as_str()).ok_or("Missing param: src")?;
    let raw_dest = params.get("dest").and_then(|v| v.as_str()).ok_or("Missing param: dest")?;
    let src = normalize_path(raw_src);
    let dest = normalize_path(raw_dest);

    std::fs::copy(&src, &dest)
        .map(|_| json!(null))
        .map_err(|e| io_error_to_node_error(&e, "copyfile", &src))?;

    Ok(json!(null))
}

fn rename(params: &Value) -> Result<Value, String> {
    let raw_old = params.get("oldPath").and_then(|v| v.as_str()).ok_or("Missing param: oldPath")?;
    let raw_new = params.get("newPath").and_then(|v| v.as_str()).ok_or("Missing param: newPath")?;
    let old_path = normalize_path(raw_old);
    let new_path = normalize_path(raw_new);

    std::fs::rename(&old_path, &new_path)
        .map(|_| json!(null))
        .map_err(|e| io_error_to_node_error(&e, "rename", &old_path))?;

    Ok(json!(null))
}

// ==============================
// Glob
// ==============================

#[derive(serde::Deserialize)]
pub struct GlobOptions {
    pub cwd: Option<String>,
    pub ignore: Option<Vec<String>>,
    pub nodir: Option<bool>,
    pub absolute: Option<bool>,
    pub dot: Option<bool>,
}

fn glob(params: &Value) -> Result<Value, String> {
    let opts: GlobOptions =
        serde_json::from_value(params.clone()).map_err(|e| format!("Invalid glob options: {}", e))?;

    let pattern_str = params
        .get("pattern")
        .and_then(|v| v.as_str())
        .unwrap_or("*");

    let pattern = opts
        .cwd
        .as_ref()
        .map(|cwd| {
            format!(
                "{}/{}",
                cwd.trim_end_matches(['/', '\\']),
                pattern_str
            )
        })
        .unwrap_or_else(|| pattern_str.to_string());

    let match_opts = MatchOptions {
        case_sensitive: cfg!(not(windows)),
        require_literal_separator: false,
        require_literal_leading_dot: !opts.dot.unwrap_or(false),
    };

    let ignore_patterns: Vec<Pattern> = opts
        .ignore
        .as_ref()
        .map(|ignores| ignores.iter().filter_map(|p| Pattern::new(p).ok()).collect())
        .unwrap_or_default();

    let nodir = opts.nodir.unwrap_or(false);
    let absolute = opts.absolute.unwrap_or(false);
    let cwd_path = std::env::current_dir().ok();

    let entries =
        glob_with(&pattern, match_opts).map_err(|e| format!("Invalid glob pattern: {}", e))?;

    let mut results: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?;

            let path_str = path.to_string_lossy();
            if ignore_patterns.iter().any(|p| p.matches(&path_str)) {
                return None;
            }

            if nodir && path.is_dir() {
                return None;
            }

            let result = if absolute {
                path.canonicalize()
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string()
            } else {
                cwd_path
                    .as_ref()
                    .and_then(|cwd| path.strip_prefix(cwd).ok())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| path_str.to_string())
            };

            Some(result)
        })
        .collect();

    results.sort();
    results.dedup();

    Ok(json!(results))
}

// ==============================
// 测试
// ==============================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn temp_path(name: &str) -> String {
        std::env::temp_dir()
            .join(format!("vokex_test_{}", name))
            .to_string_lossy()
            .to_string()
    }

    fn cleanup(path: &str) {
        let p = std::path::Path::new(path);
        if p.is_dir() {
            let _ = fs::remove_dir_all(p);
        } else if p.exists() {
            let _ = fs::remove_file(p);
        }
    }

    // ---- readFile / writeFile ----

    #[test]
    fn test_write_and_read_utf8() {
        let path = temp_path("rw_utf8.txt");
        cleanup(&path);

        handle(
            "fs.writeFile",
            &json!({"path": path, "data": "你好 vokex"}),
            0,
        )
        .unwrap();
        let result = handle("fs.readFile", &json!({"path": path, "encoding": "utf8"}), 0).unwrap();
        assert_eq!(result, json!("你好 vokex"));

        cleanup(&path);
    }

    #[test]
    fn test_read_file_no_encoding_returns_base64() {
        let path = temp_path("read_bytes.bin");
        cleanup(&path);

        fs::write(&path, &[0u8, 1, 2, 255]).unwrap();
        let result = handle("fs.readFile", &json!({"path": path}), 0).unwrap();
        // 无编码时返回 base64 字符串（由 TS 端转为 Uint8Array）
        let b64 = result.as_str().unwrap();
        assert!(!b64.is_empty());

        cleanup(&path);
    }

    #[test]
    fn test_read_file_base64() {
        let path = temp_path("read_b64.bin");
        cleanup(&path);

        fs::write(&path, &[72u8, 101, 108, 108, 111]).unwrap(); // "Hello"
        let result =
            handle("fs.readFile", &json!({"path": path, "encoding": "base64"}), 0).unwrap();
        assert_eq!(result, json!("SGVsbG8="));

        cleanup(&path);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = handle("fs.readFile", &json!({"path": "/nonexistent/path/file.txt"}), 0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ENOENT"));
    }

    // ---- writeFile flags ----

    #[test]
    fn test_write_flag_w_overwrite() {
        let path = temp_path("flag_w.txt");
        cleanup(&path);

        handle(
            "fs.writeFile",
            &json!({"path": path, "data": "first"}),
            0,
        )
        .unwrap();
        handle(
            "fs.writeFile",
            &json!({"path": path, "data": "second", "flag": "w"}),
            0,
        )
        .unwrap();
        let result = handle("fs.readFile", &json!({"path": path, "encoding": "utf8"}), 0).unwrap();
        assert_eq!(result, json!("second"));

        cleanup(&path);
    }

    #[test]
    fn test_write_flag_a_append() {
        let path = temp_path("flag_a.txt");
        cleanup(&path);

        handle(
            "fs.writeFile",
            &json!({"path": path, "data": "第一行\n"}),
            0,
        )
        .unwrap();
        handle(
            "fs.writeFile",
            &json!({"path": path, "data": "第二行", "flag": "a"}),
            0,
        )
        .unwrap();
        let result = handle("fs.readFile", &json!({"path": path, "encoding": "utf8"}), 0).unwrap();
        assert_eq!(result, json!("第一行\n第二行"));

        cleanup(&path);
    }

    #[test]
    fn test_write_flag_wx_exists_error() {
        let path = temp_path("flag_wx.txt");
        cleanup(&path);

        handle(
            "fs.writeFile",
            &json!({"path": path, "data": "first"}),
            0,
        )
        .unwrap();
        let result = handle(
            "fs.writeFile",
            &json!({"path": path, "data": "second", "flag": "wx"}),
            0,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("EEXIST"));

        cleanup(&path);
    }

    #[test]
    fn test_write_binary_array() {
        let path = temp_path("write_bin.bin");
        cleanup(&path);

        handle(
            "fs.writeFile",
            &json!({"path": path, "data": [72, 101, 108, 108, 111]}),
            0,
        )
        .unwrap();
        let bytes = fs::read(&path).unwrap();
        assert_eq!(bytes, vec![72, 101, 108, 108, 111]);

        cleanup(&path);
    }

    // ---- rm ----

    #[test]
    fn test_rm_file() {
        let path = temp_path("rm_file.txt");
        cleanup(&path);

        handle("fs.writeFile", &json!({"path": path, "data": "tmp"}), 0).unwrap();
        assert!(handle("fs.exists", &json!({"path": path}), 0).unwrap().as_bool().unwrap());

        handle("fs.rm", &json!({"path": path}), 0).unwrap();
        assert!(!handle("fs.exists", &json!({"path": path}), 0).unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_rm_nonexistent_no_force() {
        let result = handle("fs.rm", &json!({"path": "/nonexistent/path"}), 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ENOENT"));
    }

    #[test]
    fn test_rm_nonexistent_with_force() {
        let result = handle("fs.rm", &json!({"path": "/nonexistent/path", "force": true}), 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rm_dir_recursive() {
        let dir = temp_path("rm_recursive");
        cleanup(&dir);

        fs::create_dir_all(std::path::Path::new(&dir).join("sub")).unwrap();
        handle(
            "fs.writeFile",
            &json!({"path": format!("{}/sub/f.txt", dir), "data": "x"}),
            0,
        )
        .unwrap();

        handle("fs.rm", &json!({"path": dir, "recursive": true}), 0).unwrap();
        assert!(!handle("fs.exists", &json!({"path": dir}), 0).unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_rm_dir_non_recursive_fails_if_not_empty() {
        let dir = temp_path("rm_dir_notempty");
        cleanup(&dir);

        fs::create_dir_all(&dir).unwrap();
        handle(
            "fs.writeFile",
            &json!({"path": format!("{}/f.txt", dir), "data": "x"}),
            0,
        )
        .unwrap();

        let result = handle("fs.rm", &json!({"path": dir}), 0);
        assert!(result.is_err());

        cleanup(&dir);
    }

    // ---- readdir ----

    #[test]
    fn test_readdir_names() {
        let dir = temp_path("readdir_names");
        cleanup(&dir);

        fs::create_dir_all(&dir).unwrap();
        fs::write(std::path::Path::new(&dir).join("a.txt"), "a").unwrap();
        fs::write(std::path::Path::new(&dir).join("b.txt"), "b").unwrap();

        let result = handle("fs.readdir", &json!({"path": dir}), 0).unwrap();
        let entries: Vec<String> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert!(entries.contains(&"a.txt".to_string()));
        assert!(entries.contains(&"b.txt".to_string()));

        cleanup(&dir);
    }

    #[test]
    fn test_readdir_with_file_types() {
        let dir = temp_path("readdir_dirent");
        cleanup(&dir);

        fs::create_dir_all(&dir).unwrap();
        fs::write(std::path::Path::new(&dir).join("file.txt"), "f").unwrap();
        fs::create_dir(std::path::Path::new(&dir).join("subdir")).unwrap();

        let result = handle(
            "fs.readdir",
            &json!({"path": dir, "withFileTypes": true}),
            0,
        )
        .unwrap();
        let entries = result.as_array().unwrap();

        let file_entry = entries
            .iter()
            .find(|e| e["name"] == "file.txt")
            .unwrap();
        assert_eq!(file_entry["isFile"], json!(true));
        assert_eq!(file_entry["isDir"], json!(false));

        let dir_entry = entries
            .iter()
            .find(|e| e["name"] == "subdir")
            .unwrap();
        assert_eq!(dir_entry["isFile"], json!(false));
        assert_eq!(dir_entry["isDir"], json!(true));

        cleanup(&dir);
    }

    // ---- mkdir ----

    #[test]
    fn test_mkdir_basic() {
        let dir = temp_path("mkdir_basic");
        cleanup(&dir);

        handle("fs.mkdir", &json!({"path": dir}), 0).unwrap();
        assert!(Path::new(&dir).is_dir());

        cleanup(&dir);
    }

    #[test]
    fn test_mkdir_recursive() {
        let dir = temp_path("mkdir_recursive/a/b/c");
        cleanup(&temp_path("mkdir_recursive"));

        handle("fs.mkdir", &json!({"path": dir, "recursive": true}), 0).unwrap();
        assert!(Path::new(&dir).is_dir());

        cleanup(&temp_path("mkdir_recursive"));
    }

    #[test]
    fn test_mkdir_existing_no_recursive() {
        let dir = temp_path("mkdir_exist");
        cleanup(&dir);

        fs::create_dir(&dir).unwrap();
        let result = handle("fs.mkdir", &json!({"path": dir}), 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("EEXIST"));

        cleanup(&dir);
    }

    // ---- stat ----

    #[test]
    fn test_stat_file() {
        let path = temp_path("stat.txt");
        cleanup(&path);

        handle("fs.writeFile", &json!({"path": path, "data": "hello world"}), 0).unwrap();
        let result = handle("fs.stat", &json!({"path": path}), 0).unwrap();

        assert_eq!(result["isFile"], json!(true));
        assert_eq!(result["isDir"], json!(false));
        assert_eq!(result["size"], json!(11));
        assert!(result["mtimeMs"].as_u64().unwrap() > 0);
        assert!(result["atimeMs"].as_u64().unwrap() > 0);

        cleanup(&path);
    }

    // ---- exists ----

    #[test]
    fn test_exists() {
        let path = temp_path("exists.txt");
        cleanup(&path);

        assert!(!handle("fs.exists", &json!({"path": path}), 0).unwrap().as_bool().unwrap());
        handle("fs.writeFile", &json!({"path": path, "data": "x"}), 0).unwrap();
        assert!(handle("fs.exists", &json!({"path": path}), 0).unwrap().as_bool().unwrap());

        cleanup(&path);
    }

    // ---- copyFile ----

    #[test]
    fn test_copy_file() {
        let src = temp_path("copy_src.txt");
        let dst = temp_path("copy_dst.txt");
        cleanup(&src);
        cleanup(&dst);

        handle("fs.writeFile", &json!({"path": src, "data": "copy me"}), 0).unwrap();
        handle("fs.copyFile", &json!({"src": src, "dest": dst}), 0).unwrap();

        let result = handle("fs.readFile", &json!({"path": dst, "encoding": "utf8"}), 0).unwrap();
        assert_eq!(result, json!("copy me"));

        cleanup(&src);
        cleanup(&dst);
    }

    // ---- rename ----

    #[test]
    fn test_rename() {
        let src = temp_path("rename_src.txt");
        let dst = temp_path("rename_dst.txt");
        cleanup(&src);
        cleanup(&dst);

        handle("fs.writeFile", &json!({"path": src, "data": "move me"}), 0).unwrap();
        handle("fs.rename", &json!({"oldPath": src, "newPath": dst}), 0).unwrap();

        assert!(!handle("fs.exists", &json!({"path": src}), 0).unwrap().as_bool().unwrap());
        let result = handle("fs.readFile", &json!({"path": dst, "encoding": "utf8"}), 0).unwrap();
        assert_eq!(result, json!("move me"));

        cleanup(&dst);
    }

    // ---- 参数校验 ----

    #[test]
    fn test_missing_params() {
        assert!(handle("fs.readFile", &json!({}), 0).is_err());
        assert!(handle("fs.writeFile", &json!({"path": "/tmp/t"}), 0).is_err());
        assert!(handle("fs.writeFile", &json!({"data": "x"}), 0).is_err());
    }

    #[test]
    fn test_unknown_method() {
        assert!(handle("fs.unknownMethod", &json!({}), 0).is_err());
    }
}
