use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// 全局加密存储缓存
static SAFE_STORAGE_CACHE: Mutex<Option<HashMap<String, Value>>> = Mutex::new(None);

/// 加密文件格式：nonce(12字节) + ciphertext + tag(16字节)
/// 存储为二进制文件，结构简单高效
const NONCE_LEN: usize = 12;

/// 获取加密存储文件路径
fn get_safe_storage_path() -> Result<PathBuf, String> {
    let config = crate::app_config::get_config();
    let data_dir = if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        PathBuf::from(local_appdata).join(&config.identifier)
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(format!(".{}", config.identifier))
    } else {
        return Err("Cannot determine data directory".to_string());
    };
    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;
    Ok(data_dir.join("safeStorage.json.enc"))
}

/// 获取或创建主密钥
///
/// 首次启动时生成 32 字节随机密钥，存储到系统安全区（Windows 凭据管理器/macOS 钥匙串）。
/// 后续启动直接从系统安全区读取。
#[cfg(not(test))]
fn get_or_create_master_key() -> Result<Vec<u8>, String> {
    let config = crate::app_config::get_config();
    let service_name = format!("{}.safeStorage", config.identifier);
    let entry = keyring::Entry::new(&service_name, "master_key")
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    // 尝试从系统安全区读取
    match entry.get_password() {
        Ok(encoded) => {
            // Base64 解码
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(&encoded)
                .map_err(|e| format!("Failed to decode master key: {}", e))
        }
        Err(keyring::Error::NoEntry) => {
            // 首次启动，生成新密钥
            let mut key = vec![0u8; 32];
            rand::Rng::fill(&mut rand::thread_rng(), &mut key[..]);

            // Base64 编码后存储到系统安全区
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&key);
            entry
                .set_password(&encoded)
                .map_err(|e| format!("Failed to save master key to keyring: {}", e))?;

            Ok(key)
        }
        Err(e) => Err(format!("Failed to read master key from keyring: {}", e)),
    }
}

/// 测试环境使用固定密钥（避免 keyring 并发问题）
#[cfg(test)]
fn get_or_create_master_key() -> Result<Vec<u8>, String> {
    // 测试用固定密钥，保证并行测试的一致性
    Ok(vec![0x42u8; 32])
}

/// 创建 AES-256-GCM 加密器
fn create_cipher(key: &[u8]) -> Result<Aes256Gcm, String> {
    use aes_gcm::Key;
    let key = Key::<Aes256Gcm>::from_slice(key);
    Ok(Aes256Gcm::new(key))
}

/// 加密数据
fn encrypt(cipher: &Aes256Gcm, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    // 生成随机 nonce
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::Rng::fill(&mut rand::thread_rng(), &mut nonce_bytes[..]);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // 拼接：nonce + ciphertext（包含 tag）
    let mut result = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// 解密数据
fn decrypt(cipher: &Aes256Gcm, data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < NONCE_LEN {
        return Err("Invalid encrypted data: too short".to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// 从加密文件加载存储到缓存
fn load_safe_storage() -> Result<HashMap<String, Value>, String> {
    let path = get_safe_storage_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let encrypted_data = match fs::read(&path) {
        Ok(data) => data,
        Err(_) => return Ok(HashMap::new()), // 文件可能被并发删除
    };

    if encrypted_data.is_empty() {
        return Ok(HashMap::new());
    }

    let key = get_or_create_master_key()?;
    let cipher = create_cipher(&key)?;
    let plaintext = decrypt(&cipher, &encrypted_data)?;

    let map: HashMap<String, Value> = serde_json::from_slice(&plaintext)
        .map_err(|e| format!("Failed to parse safe storage data: {}", e))?;
    Ok(map)
}

/// 将缓存加密写入文件
fn save_safe_storage(map: &HashMap<String, Value>) -> Result<(), String> {
    let path = get_safe_storage_path()?;
    let plaintext = serde_json::to_vec_pretty(map)
        .map_err(|e| format!("Failed to serialize safe storage: {}", e))?;

    let key = get_or_create_master_key()?;
    let cipher = create_cipher(&key)?;
    let encrypted_data = encrypt(&cipher, &plaintext)?;

    fs::write(&path, encrypted_data)
        .map_err(|e| format!("Failed to write safe storage file: {}", e))?;
    Ok(())
}

/// 确保缓存已加载
fn ensure_cache() -> Result<(), String> {
    let mut cache = SAFE_STORAGE_CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(load_safe_storage()?);
    }
    Ok(())
}

/// 重置缓存（测试用）
#[cfg(test)]
fn reset_safe_storage_cache() {
    let mut cache = SAFE_STORAGE_CACHE.lock().unwrap();
    *cache = None;
}

/// 处理 safeStorage API 请求
pub fn handle(method: &str, params: &Value) -> Result<Value, String> {
    match method {
        "safeStorage.setData" => {
            let key = params
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'key' parameter")?;
            let value = params
                .get("value")
                .ok_or("Missing 'value' parameter")?;
            ensure_cache()?;
            let mut cache = SAFE_STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_mut() {
                map.insert(key.to_string(), value.clone());
                save_safe_storage(map)?;
            }
            Ok(json!(true))
        }

        "safeStorage.getData" => {
            let key = params
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'key' parameter")?;
            ensure_cache()?;
            let cache = SAFE_STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_ref() {
                Ok(map.get(key).cloned().unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }

        "safeStorage.getKeys" => {
            ensure_cache()?;
            let cache = SAFE_STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_ref() {
                let keys: Vec<&String> = map.keys().collect();
                Ok(json!(keys))
            } else {
                Ok(json!([]))
            }
        }

        "safeStorage.has" => {
            let key = params
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'key' parameter")?;
            ensure_cache()?;
            let cache = SAFE_STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_ref() {
                Ok(json!(map.contains_key(key)))
            } else {
                Ok(json!(false))
            }
        }

        "safeStorage.removeData" => {
            let key = params
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'key' parameter")?;
            ensure_cache()?;
            let mut cache = SAFE_STORAGE_CACHE.lock().unwrap();
            if let Some(map) = cache.as_mut() {
                map.remove(key);
                save_safe_storage(map)?;
            }
            Ok(json!(true))
        }

        "safeStorage.clear" => {
            let mut cache = SAFE_STORAGE_CACHE.lock().unwrap();
            *cache = Some(HashMap::new());
            save_safe_storage(cache.as_ref().unwrap())?;
            Ok(json!(true))
        }

        _ => Err(format!("Unknown method: {}", method)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serial_test::serial;

    fn setup() {
        crate::app_config::init_test_config();
        reset_safe_storage_cache();
        // 清理测试文件，避免旧数据干扰
        let _ = fs::remove_file(get_safe_storage_path().unwrap());
        reset_safe_storage_cache();
    }

    fn cleanup_keys(keys: &[&str]) {
        for key in keys {
            let _ = handle("safeStorage.removeData", &json!({"key": key}));
        }
    }

    #[test]
    #[serial]
    fn test_encrypt_decrypt_roundtrip() {
        let key = vec![0u8; 32]; // 测试用固定密钥
        let cipher = create_cipher(&key).unwrap();
        let plaintext = b"Hello, World! This is a test message.";

        let encrypted = encrypt(&cipher, plaintext).unwrap();
        let decrypted = decrypt(&cipher, &encrypted).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    #[serial]
    fn test_encrypt_decrypt_json() {
        let key = vec![1u8; 32];
        let cipher = create_cipher(&key).unwrap();
        let data = json!({"token": "abc123", "expires": 3600});
        let plaintext = serde_json::to_vec(&data).unwrap();

        let encrypted = encrypt(&cipher, &plaintext).unwrap();
        let decrypted = decrypt(&cipher, &encrypted).unwrap();
        let result: Value = serde_json::from_slice(&decrypted).unwrap();

        assert_eq!(result, data);
    }

    #[test]
    #[serial]
    fn test_decrypt_tampered_data_fails() {
        let key = vec![0u8; 32];
        let cipher = create_cipher(&key).unwrap();
        let plaintext = b"secret data";

        let mut encrypted = encrypt(&cipher, plaintext).unwrap();
        // 篡改密文
        if encrypted.len() > NONCE_LEN + 1 {
            encrypted[NONCE_LEN + 1] ^= 0xFF;
        }

        let result = decrypt(&cipher, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_set_and_get_data() {
        setup();
        let key = "t1_api_key";
        let value = json!({"provider": "openai", "key": "sk-xxx"});

        handle("safeStorage.setData", &json!({"key": key, "value": value}))
            .unwrap();

        let result = handle("safeStorage.getData", &json!({"key": key})).unwrap();
        assert_eq!(result["provider"], json!("openai"));
        assert_eq!(result["key"], json!("sk-xxx"));

        cleanup_keys(&[key]);
    }

    #[test]
    #[serial]
    fn test_get_nonexistent_key() {
        setup();
        let result = handle("safeStorage.getData", &json!({"key": "nonexistent"})).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    #[serial]
    fn test_has() {
        setup();
        let key = "t3_exists";
        handle("safeStorage.setData", &json!({"key": key, "value": "secret"})).unwrap();

        assert!(handle("safeStorage.has", &json!({"key": key})).unwrap().as_bool().unwrap());
        assert!(!handle("safeStorage.has", &json!({"key": "t3_missing"})).unwrap().as_bool().unwrap());

        cleanup_keys(&[key]);
    }

    #[test]
    #[serial]
    fn test_remove_data() {
        setup();
        let key = "t4_rm";
        handle("safeStorage.setData", &json!({"key": key, "value": "bye"})).unwrap();
        assert!(handle("safeStorage.has", &json!({"key": key})).unwrap().as_bool().unwrap());

        handle("safeStorage.removeData", &json!({"key": key})).unwrap();
        assert!(!handle("safeStorage.has", &json!({"key": key})).unwrap().as_bool().unwrap());
        assert_eq!(handle("safeStorage.getData", &json!({"key": key})).unwrap(), json!(null));
    }

    #[test]
    #[serial]
    fn test_clear() {
        setup();
        let keys = ["t5_k1", "t5_k2"];
        handle("safeStorage.setData", &json!({"key": keys[0], "value": 1})).unwrap();
        handle("safeStorage.setData", &json!({"key": keys[1], "value": 2})).unwrap();

        handle("safeStorage.clear", &json!({})).unwrap();

        let result = handle("safeStorage.getKeys", &json!({})).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 0);
    }

    #[test]
    #[serial]
    fn test_overwrite_value() {
        setup();
        let key = "t6_ow";
        handle("safeStorage.setData", &json!({"key": key, "value": "old"})).unwrap();
        handle("safeStorage.setData", &json!({"key": key, "value": "new"})).unwrap();
        let result = handle("safeStorage.getData", &json!({"key": key})).unwrap();
        assert_eq!(result, json!("new"));
        cleanup_keys(&[key]);
    }

    #[test]
    #[serial]
    fn test_data_persists_after_reload() {
        setup();
        let key = "t7_persist";
        handle("safeStorage.setData", &json!({"key": key, "value": "persist_me"})).unwrap();

        // 模拟重新加载
        reset_safe_storage_cache();

        let result = handle("safeStorage.getData", &json!({"key": key})).unwrap();
        assert_eq!(result, json!("persist_me"));

        cleanup_keys(&[key]);
    }
}
