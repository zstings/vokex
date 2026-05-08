use serde_json::{json, Value};
use std::io::Write;
use glob::{glob_with, MatchOptions, Pattern};

/// 处理 fs 模块的 API 调用
pub fn handle(method: &str, params: &Value) -> Result<Value, String> {
    match method {
        "fs.readFile" => read_file(params),
        "fs.readFileBinary" => read_file_binary(params),
        "fs.writeFile" => write_file(params),
        "fs.appendFile" => append_file(params),
        "fs.deleteFile" => delete_file(params),
        "fs.readDir" => read_dir(params),
        "fs.createDir" => create_dir(params),
        "fs.removeDir" => remove_dir(params),
        "fs.stat" => stat(params),
        "fs.exists" => exists(params),
        "fs.copyFile" => copy_file(params),
        "fs.moveFile" => move_file(params),
        "fs.glob" => glob(params),
        _ => Err(format!("Unknown fs method: {}", method)),
    }
}

fn read_file(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    std::fs::read_to_string(path)
        .map(|content| json!(content))
        .map_err(|e| e.to_string())
}

fn read_file_binary(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    // 返回 base64 编码，比字节数组更紧凑
    Ok(json!(base64_encode(&bytes)))
}

fn write_file(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let data = params.get("data").and_then(|v| v.as_str()).ok_or("Missing param: data")?;
    std::fs::write(path, data)
        .map(|_| json!(null))
        .map_err(|e| e.to_string())
}

fn append_file(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let data = params.get("data").and_then(|v| v.as_str()).ok_or("Missing param: data")?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(data.as_bytes()))
        .map(|_| json!(null))
        .map_err(|e| e.to_string())
}

fn delete_file(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    std::fs::remove_file(path)
        .map(|_| json!(null))
        .map_err(|e| e.to_string())
}

fn read_dir(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            let name = entry.file_name().into_string().unwrap_or_default();
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            result.push(json!({
                "name": name,
                "isDir": is_dir
            }));
        }
    }
    Ok(json!(result))
}

fn create_dir(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    std::fs::create_dir_all(path)
        .map(|_| json!(null))
        .map_err(|e| e.to_string())
}

fn remove_dir(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    std::fs::remove_dir_all(path)
        .map(|_| json!(null))
        .map_err(|e| e.to_string())
}

fn stat(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let modified = metadata.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as u64)
        .unwrap_or(0);
    Ok(json!({
        "isFile": metadata.is_file(),
        "isDir": metadata.is_dir(),
        "size": metadata.len(),
        "modified": modified
    }))
}

fn exists(params: &Value) -> Result<Value, String> {
    let path = params.get("path").and_then(|v| v.as_str()).ok_or("Missing param: path")?;
    Ok(json!(std::path::Path::new(path).exists()))
}

fn copy_file(params: &Value) -> Result<Value, String> {
    let source = params.get("source").and_then(|v| v.as_str()).ok_or("Missing param: source")?;
    let destination = params.get("destination").and_then(|v| v.as_str()).ok_or("Missing param: destination")?;
    std::fs::copy(source, destination)
        .map(|_| json!(null))
        .map_err(|e| e.to_string())
}

fn move_file(params: &Value) -> Result<Value, String> {
    let source = params.get("source").and_then(|v| v.as_str()).ok_or("Missing param: source")?;
    let destination = params.get("destination").and_then(|v| v.as_str()).ok_or("Missing param: destination")?;
    std::fs::rename(source, destination)
        .map(|_| json!(null))
        .map_err(|e| e.to_string())
}

/// Glob 配置选项
#[derive(serde::Deserialize)]
pub struct GlobOptions {
    pub cwd: Option<String>,
    pub ignore: Option<Vec<String>>,
    pub nodir: Option<bool>,
    pub absolute: Option<bool>,
    pub dot: Option<bool>,
}

fn glob(params: &Value) -> Result<Value, String> {
    let opts: GlobOptions = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid glob options: {}", e))?;
    
    let pattern_str = params.get("pattern").and_then(|v| v.as_str()).unwrap_or("*");
    
    let pattern = opts.cwd.as_ref()
        .map(|cwd| format!("{}/{}", cwd.trim_end_matches(['/', '\\']), pattern_str))
        .unwrap_or_else(|| pattern_str.to_string());
    
    let match_opts = MatchOptions {
        case_sensitive: cfg!(not(windows)),
        require_literal_separator: false,
        require_literal_leading_dot: !opts.dot.unwrap_or(false),
    };
    
    let ignore_patterns: Vec<Pattern> = opts.ignore.as_ref()
        .map(|ignores| ignores.iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect())
        .unwrap_or_default();
    
    let nodir = opts.nodir.unwrap_or(false);
    let absolute = opts.absolute.unwrap_or(false);
    let cwd_path = std::env::current_dir().ok();
    
    let entries = glob_with(&pattern, match_opts)
        .map_err(|e| format!("Invalid glob pattern: {}", e))?;
    
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
                cwd_path.as_ref()
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

fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

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

    #[test]
    fn test_write_and_read_file() {
        let path = temp_path("rw.txt");
        cleanup(&path);

        handle("fs.writeFile", &json!({"path": path, "data": "你好 vokex"})).unwrap();
        let result = handle("fs.readFile", &json!({"path": path})).unwrap();
        assert_eq!(result, json!("你好 vokex"));

        cleanup(&path);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = handle("fs.readFile", &json!({"path": "/nonexistent/path/file.txt"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_and_read_binary() {
        let path = temp_path("binary.bin");
        cleanup(&path);

        let data: Vec<u8> = vec![0, 1, 2, 255, 254, 253];
        fs::write(&path, &data).unwrap();

        let result = handle("fs.readFileBinary", &json!({"path": path})).unwrap();
        let b64 = result.as_str().unwrap();
        // 验证返回的是 base64 字符串
        assert!(!b64.is_empty());

        cleanup(&path);
    }

    #[test]
    fn test_append_file() {
        let path = temp_path("append.txt");
        cleanup(&path);

        handle("fs.writeFile", &json!({"path": path, "data": "第一行\n"})).unwrap();
        handle("fs.appendFile", &json!({"path": path, "data": "第二行"})).unwrap();
        let result = handle("fs.readFile", &json!({"path": path})).unwrap();
        assert_eq!(result, json!("第一行\n第二行"));

        cleanup(&path);
    }

    #[test]
    fn test_delete_file() {
        let path = temp_path("delete.txt");
        cleanup(&path);

        handle("fs.writeFile", &json!({"path": path, "data": "tmp"})).unwrap();
        assert!(handle("fs.exists", &json!({"path": path})).unwrap().as_bool().unwrap());

        handle("fs.deleteFile", &json!({"path": path})).unwrap();
        assert!(!handle("fs.exists", &json!({"path": path})).unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_create_and_read_dir() {
        let dir = temp_path("testdir");
        cleanup(&dir);

        handle("fs.createDir", &json!({"path": dir})).unwrap();
        // 在目录里创建几个文件
        fs::write(std::path::Path::new(&dir).join("a.txt"), "a").unwrap();
        fs::write(std::path::Path::new(&dir).join("b.txt"), "b").unwrap();

        let result = handle("fs.readDir", &json!({"path": dir})).unwrap();
        let entries: Vec<_> = result.as_array().unwrap().iter()
            .map(|e| e["name"].as_str().unwrap().to_string())
            .collect();
        assert!(entries.contains(&"a.txt".to_string()));
        assert!(entries.contains(&"b.txt".to_string()));

        cleanup(&dir);
    }

    #[test]
    fn test_remove_dir() {
        let dir = temp_path("rmdir");
        cleanup(&dir);

        fs::create_dir_all(std::path::Path::new(&dir).join("sub")).unwrap();
        handle("fs.writeFile", &json!({"path": format!("{}/sub/f.txt", dir), "data": "x"})).unwrap();

        handle("fs.removeDir", &json!({"path": dir})).unwrap();
        assert!(!handle("fs.exists", &json!({"path": dir})).unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_stat_file() {
        let path = temp_path("stat.txt");
        cleanup(&path);

        handle("fs.writeFile", &json!({"path": path, "data": "hello world"})).unwrap();
        let result = handle("fs.stat", &json!({"path": path})).unwrap();

        assert_eq!(result["isFile"], json!(true));
        assert_eq!(result["isDir"], json!(false));
        assert_eq!(result["size"], json!(11));
        assert!(result["modified"].as_u64().unwrap() > 0);

        cleanup(&path);
    }

    #[test]
    fn test_exists() {
        let path = temp_path("exists.txt");
        cleanup(&path);

        assert!(!handle("fs.exists", &json!({"path": path})).unwrap().as_bool().unwrap());
        handle("fs.writeFile", &json!({"path": path, "data": "x"})).unwrap();
        assert!(handle("fs.exists", &json!({"path": path})).unwrap().as_bool().unwrap());

        cleanup(&path);
    }

    #[test]
    fn test_copy_file() {
        let src = temp_path("copy_src.txt");
        let dst = temp_path("copy_dst.txt");
        cleanup(&src);
        cleanup(&dst);

        handle("fs.writeFile", &json!({"path": src, "data": "copy me"})).unwrap();
        handle("fs.copyFile", &json!({"source": src, "destination": dst})).unwrap();

        let result = handle("fs.readFile", &json!({"path": dst})).unwrap();
        assert_eq!(result, json!("copy me"));

        cleanup(&src);
        cleanup(&dst);
    }

    #[test]
    fn test_move_file() {
        let src = temp_path("move_src.txt");
        let dst = temp_path("move_dst.txt");
        cleanup(&src);
        cleanup(&dst);

        handle("fs.writeFile", &json!({"path": src, "data": "move me"})).unwrap();
        handle("fs.moveFile", &json!({"source": src, "destination": dst})).unwrap();

        assert!(!handle("fs.exists", &json!({"path": src})).unwrap().as_bool().unwrap());
        let result = handle("fs.readFile", &json!({"path": dst})).unwrap();
        assert_eq!(result, json!("move me"));

        cleanup(&dst);
    }

    #[test]
    fn test_missing_params() {
        assert!(handle("fs.readFile", &json!({})).is_err());
        assert!(handle("fs.writeFile", &json!({"path": "/tmp/t"})).is_err());
        assert!(handle("fs.writeFile", &json!({"data": "x"})).is_err());
    }

    #[test]
    fn test_unknown_method() {
        assert!(handle("fs.unknownMethod", &json!({})).is_err());
    }
}
