use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// 全局存储缓存，避免每次都读文件
static STORAGE_CACHE: Mutex<Option<HashMap<String, Value>>> = Mutex::new(None);

/// 获取存储文件路径
fn get_storage_path() -> Result<PathBuf, String> {
    let config = crate::app_config::get_config();
    let data_dir = if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        PathBuf::from(local_appdata).join(&config.identifier)
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(format!(".{}", config.identifier))
    } else {
        return Err("Cannot determine data directory".to_string());
    };
    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;
    Ok(data_dir.join("storage.json"))
}

/// 从文件加载存储到缓存
fn load_storage() -> Result<HashMap<String, Value>, String> {
    let path = get_storage_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read storage: {}", e))?;
    let map: HashMap<String, Value> = serde_json::from_str(&content)
        .unwrap_or_else(|_| HashMap::new());
    Ok(map)
}

/// 将缓存写入文件
fn save_storage(map: &HashMap<String, Value>) -> Result<(), String> {
    let path = get_storage_path()?;
    let content = serde_json::to_string_pretty(map)
        .map_err(|e| format!("Failed to serialize storage: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write storage: {}", e))?;
    Ok(())
}

/// 确保缓存已加载
fn ensure_cache() -> Result<(), String> {
    let mut cache = STORAGE_CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(load_storage()?);
    }
    Ok(())
}

#[cfg(test)]
fn reset_storage_cache() {
    let mut cache = STORAGE_CACHE.lock().unwrap();
    *cache = None;
}

pub fn handle(method: &str, params: &Value) -> Result<Value, String> {
    match method {
        "storage.setItem" => {
            let key = params.get("key").and_then(|v| v.as_str())
                .ok_or("Missing 'key' parameter")?;
            let value = params.get("value")
                .ok_or("Missing 'value' parameter")?;
            ensure_cache()?;
            let mut cache = STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_mut() {
                map.insert(key.to_string(), value.clone());
                save_storage(map)?;
            }
            Ok(json!(true))
        }

        "storage.getItem" => {
            let key = params.get("key").and_then(|v| v.as_str())
                .ok_or("Missing 'key' parameter")?;
            ensure_cache()?;
            let cache = STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_ref() {
                Ok(map.get(key).cloned().unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }

        "storage.keys" => {
            ensure_cache()?;
            let cache = STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_ref() {
                let keys: Vec<&String> = map.keys().collect();
                Ok(json!(keys))
            } else {
                Ok(json!([]))
            }
        }

        "storage.has" => {
            let key = params.get("key").and_then(|v| v.as_str())
                .ok_or("Missing 'key' parameter")?;
            ensure_cache()?;
            let cache = STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_ref() {
                Ok(json!(map.contains_key(key)))
            } else {
                Ok(json!(false))
            }
        }

        "storage.removeItem" => {
            let key = params.get("key").and_then(|v| v.as_str())
                .ok_or("Missing 'key' parameter")?;
            ensure_cache()?;
            let mut cache = STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_mut() {
                map.remove(key);
                save_storage(map)?;
            }
            Ok(json!(true))
        }

        "storage.clear" => {
            let mut cache = STORAGE_CACHE.lock().unwrap();
            *cache = Some(HashMap::new());
            save_storage(cache.as_ref().unwrap())?;
            Ok(json!(true))
        }

        _ => Err(format!("Unknown method: {}", method)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn setup() {
        crate::app_config::init_test_config();
        reset_storage_cache();
    }

    fn cleanup_keys(keys: &[&str]) {
        for key in keys {
            let _ = handle("storage.removeItem", &json!({"key": key}));
        }
    }

    #[test]
    fn test_set_and_get_data() {
        setup();
        let key = "t1_user";
        handle("storage.setItem", &json!({"key": key, "value": {"name": "张三", "age": 25}})).unwrap();
        let result = handle("storage.getItem", &json!({"key": key})).unwrap();
        assert_eq!(result["name"], json!("张三"));
        assert_eq!(result["age"], json!(25));
        cleanup_keys(&[key]);
    }

    #[test]
    fn test_get_nonexistent_key() {
        setup();
        let result = handle("storage.getItem", &json!({"key": "t2_no_such_key"})).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_get_keys() {
        setup();
        let keys = ["t3_a", "t3_b"];
        handle("storage.setItem", &json!({"key": keys[0], "value": 1})).unwrap();
        handle("storage.setItem", &json!({"key": keys[1], "value": 2})).unwrap();

        let result = handle("storage.keys", &json!({})).unwrap();
        let all_keys: Vec<String> = result.as_array().unwrap().iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert!(all_keys.contains(&keys[0].to_string()));
        assert!(all_keys.contains(&keys[1].to_string()));
        cleanup_keys(&keys);
    }

    #[test]
    fn test_has() {
        setup();
        let key = "t4_exists";
        handle("storage.setItem", &json!({"key": key, "value": "y"})).unwrap();
        assert!(handle("storage.has", &json!({"key": key})).unwrap().as_bool().unwrap());
        assert!(!handle("storage.has", &json!({"key": "t4_missing"})).unwrap().as_bool().unwrap());
        cleanup_keys(&[key]);
    }

    #[test]
    fn test_remove_data() {
        setup();
        let key = "t5_rm";
        handle("storage.setItem", &json!({"key": key, "value": "bye"})).unwrap();
        assert!(handle("storage.has", &json!({"key": key})).unwrap().as_bool().unwrap());

        handle("storage.removeItem", &json!({"key": key})).unwrap();
        assert!(!handle("storage.has", &json!({"key": key})).unwrap().as_bool().unwrap());
        assert_eq!(handle("storage.getItem", &json!({"key": key})).unwrap(), json!(null));
    }

    #[test]
    fn test_clear() {
        setup();
        let keys = ["t6_k1", "t6_k2"];
        handle("storage.setItem", &json!({"key": keys[0], "value": 1})).unwrap();
        handle("storage.setItem", &json!({"key": keys[1], "value": 2})).unwrap();

        handle("storage.clear", &json!({})).unwrap();

        let result = handle("storage.keys", &json!({})).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 0);
        // 清理可能残留
        cleanup_keys(&keys);
    }

    #[test]
    fn test_overwrite_value() {
        setup();
        let key = "t7_ow";
        handle("storage.setItem", &json!({"key": key, "value": "old"})).unwrap();
        handle("storage.setItem", &json!({"key": key, "value": "new"})).unwrap();
        let result = handle("storage.getItem", &json!({"key": key})).unwrap();
        assert_eq!(result, json!("new"));
        cleanup_keys(&[key]);
    }

    #[test]
    fn test_set_missing_params() {
        setup();
        assert!(handle("storage.setItem", &json!({"key": "x"})).is_err());
        assert!(handle("storage.setItem", &json!({"value": 1})).is_err());
    }
}