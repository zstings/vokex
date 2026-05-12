use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, LazyLock};
use std::io::{BufRead, BufReader};
use std::thread;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 全局进程管理器，存储正在运行的子进程
static PROCESS_MANAGER: LazyLock<Arc<Mutex<ProcessManager>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(ProcessManager::new()))
});

/// 进程管理器
struct ProcessManager {
    processes: HashMap<u32, Arc<Mutex<ChildProcess>>>,
    next_pid: u32,
}

/// 子进程信息
struct ChildProcess {
    child: Option<std::process::Child>,
}

impl ProcessManager {
    fn new() -> Self {
        Self {
            processes: HashMap::new(),
            next_pid: 1,
        }
    }

    fn register(&mut self, child: std::process::Child) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;

        let process = Arc::new(Mutex::new(ChildProcess {
            child: Some(child),
        }));

        self.processes.insert(pid, process);
        pid
    }

    fn get(&self, pid: u32) -> Option<Arc<Mutex<ChildProcess>>> {
        self.processes.get(&pid).cloned()
    }

    fn remove(&mut self, pid: u32) {
        self.processes.remove(&pid);
    }
}

/// 用系统默认程序打开 URL
fn open_external(url: &str) -> Result<(), String> {
    open::that(url).map_err(|e| format!("Failed to open URL: {}", e))
}

/// 用系统默认程序打开文件/目录
fn open_path(path: &str) -> Result<(), String> {
    open::that(path).map_err(|e| format!("Failed to open path: {}", e))
}

/// 构建 Command 实例，应用 cwd 和 env 参数
fn build_command(program: &str, args: &[String], cwd: Option<&str>, env: Option<&Value>) -> Command {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    if let Some(env_vars) = env {
        if let Some(obj) = env_vars.as_object() {
            for (key, val) in obj {
                if let Some(s) = val.as_str() {
                    cmd.env(key, s);
                }
            }
        }
    }

    // Windows: 隐藏控制台窗口，确保 IO 能被正确重定向到管道
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd
}

/// 执行命令并等待完成（一次性执行）
fn exec_command(program: &str, args: &[String], cwd: Option<&str>, env: Option<&Value>) -> Result<Value, String> {
    let mut cmd = build_command(program, args, cwd, env);

    let output = cmd.output()
        .map_err(|e| format!("Failed to execute program '{}': {}", program, e))?;

    Ok(json!({
        "code": output.status.code().unwrap_or(-1),
        "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
        "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
        "success": output.status.success()
    }))
}

/// 启动流式执行进程
fn spawn_command(
    program: &str,
    args: &[String],
    cwd: Option<&str>,
    env: Option<&Value>,
    window_id: u32,
) -> Result<u32, String> {
    let mut cmd = build_command(program, args, cwd, env);

    // 配置管道捕获 stdout 和 stderr
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn program '{}': {}", program, e))?;

    // 获取 stdout 和 stderr 的管道句柄
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    // 注册进程到管理器
    let pid = {
        let mut manager = PROCESS_MANAGER.lock().unwrap();
        manager.register(child)
    };

    // 获取进程引用，用于后续 kill 操作
    let process_ref = {
        let manager = PROCESS_MANAGER.lock().unwrap();
        manager.get(pid).ok_or("Failed to get process reference")?
    };

    // 启动 stdout 读取线程
    // 注意：必须通过 emit_via_proxy 将事件投递到主线程执行，因为 WebView.evaluate_script 只能在主线程调用
    let process_ref_clone = process_ref.clone();
    let pid_clone = pid;
    thread::spawn(move || {
        use std::io::Read;
        let mut reader = BufReader::new(stdout);
        let mut buf = [0u8; 4096];
        let mut remainder = String::new();

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF，流关闭
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    let combined = format!("{}{}", remainder, chunk);
                    let mut lines: Vec<&str> = combined.split('\n').collect();
                    // 最后一个元素可能是不完整的行，存入 remainder
                    remainder = lines.pop().unwrap_or("").to_string();
                    for line in lines {
                        let line = line.trim_end_matches('\r');
                        if !line.is_empty() {
                            crate::ipc::emit_via_proxy(
                                window_id,
                                format!("shell.stdout.{}", pid_clone),
                                json!(line),
                            );
                        }
                    }
                }
                Err(_) => break,
            }
        }

        // 处理剩余数据
        if !remainder.is_empty() {
            crate::ipc::emit_via_proxy(
                window_id,
                format!("shell.stdout.{}", pid_clone),
                json!(remainder),
            );
        }

        // stdout 流已关闭，说明进程已退出，此时获取退出码
        let exit_code = {
            let mut proc = process_ref_clone.lock().unwrap();
            if let Some(ref mut child) = proc.child {
                let code = child.wait().ok().map(|s| (s.code(), s.success()));
                proc.child = None;
                code
            } else {
                None
            }
        };

        if let Some((code, success)) = exit_code {
            crate::ipc::emit_via_proxy(
                window_id,
                format!("shell.exit.{}", pid_clone),
                json!({
                    "code": code,
                    "success": success
                }),
            );
        }

        // 清理进程管理器中的记录
        PROCESS_MANAGER.lock().unwrap().remove(pid_clone);
    });

    // 启动 stderr 读取线程
    thread::spawn(move || {
        use std::io::Read;
        let mut reader = BufReader::new(stderr);
        let mut buf = [0u8; 4096];
        let mut remainder = String::new();

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    let combined = format!("{}{}", remainder, chunk);
                    let mut lines: Vec<&str> = combined.split('\n').collect();
                    remainder = lines.pop().unwrap_or("").to_string();
                    for line in lines {
                        let line = line.trim_end_matches('\r');
                        if !line.is_empty() {
                            crate::ipc::emit_via_proxy(
                                window_id,
                                format!("shell.stderr.{}", pid_clone),
                                json!(line),
                            );
                        }
                    }
                }
                Err(_) => break,
            }
        }

        if !remainder.is_empty() {
            crate::ipc::emit_via_proxy(
                window_id,
                format!("shell.stderr.{}", pid_clone),
                json!(remainder),
            );
        }
    });

    Ok(pid)
}

/// 窗口关闭时清理所有子进程
pub fn cleanup_all() {
    let pids: Vec<u32> = {
        let manager = PROCESS_MANAGER.lock().unwrap();
        manager.processes.keys().copied().collect()
    };
    for pid in pids {
        let _ = kill_process(pid);
    }
}

/// 杀死进程
fn kill_process(pid: u32) -> Result<(), String> {
    let process = {
        let manager = PROCESS_MANAGER.lock().unwrap();
        manager.get(pid)
    };

    if let Some(process) = process {
        let mut proc = process.lock().unwrap();
        if let Some(ref mut child) = proc.child {
            // 在 Windows 上使用 taskkill 杀死进程树（防止残留子进程）
            #[cfg(target_os = "windows")]
            {
                let mut kill_cmd = Command::new("taskkill");
                kill_cmd.args(["/F", "/T", "/PID", &child.id().to_string()]);
                kill_cmd.creation_flags(CREATE_NO_WINDOW);
                let _ = kill_cmd.output();
            }
            // 在 Unix 系统上使用 kill
            #[cfg(not(target_os = "windows"))]
            {
                let _ = child.kill();
            }
            let _ = child.wait();
            proc.child = None;
        }
        // 清理进程管理器
        PROCESS_MANAGER.lock().unwrap().remove(pid);
        Ok(())
    } else {
        Err(format!("Process with pid {} not found", pid))
    }
}

pub fn handle(method: &str, params: &Value, window_id: u32) -> Result<Value, String> {
    match method {
        "shell.openExternal" => {
            let url = params.get("url").and_then(|v| v.as_str())
                .ok_or("Missing 'url' parameter")?;
            open_external(url)?;
            Ok(json!(true))
        }

        "shell.openPath" => {
            let path = params.get("path").and_then(|v| v.as_str())
                .ok_or("Missing 'path' parameter")?;
            open_path(path)?;
            Ok(json!(true))
        }

        "shell.exec" => {
            let program = params.get("program").and_then(|v| v.as_str())
                .ok_or("Missing 'program' parameter")?;
            let args: Vec<String> = params.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_default();
            let cwd = params.get("cwd").and_then(|v| v.as_str());
            let env = params.get("env");
            exec_command(program, &args, cwd, env)
        }

        "shell.spawn" => {
            let program = params.get("program").and_then(|v| v.as_str())
                .ok_or("Missing 'program' parameter")?;
            let args: Vec<String> = params.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_default();
            let cwd = params.get("cwd").and_then(|v| v.as_str());
            let env = params.get("env");
            let pid = spawn_command(program, &args, cwd, env, window_id)?;
            Ok(json!({ "pid": pid }))
        }

        "shell.kill" => {
            let pid = params.get("pid").and_then(|v| v.as_u64())
                .ok_or("Missing 'pid' parameter")? as u32;
            kill_process(pid)?;
            Ok(json!(true))
        }

        "shell.trashItem" => {
            let path = params.get("path").and_then(|v| v.as_str())
                .ok_or("Missing 'path' parameter")?;
            trash::delete(path)
                .map_err(|e| format!("Failed to trash item: {}", e))?;
            Ok(json!(true))
        }

        _ => Err(format!("Unknown method: {}", method)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_exec_echo() {
        let (program, args, expected) = if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), "echo".to_string(), "hello".to_string()], "hello")
        } else {
            ("echo", vec!["hello".to_string()], "hello")
        };
        let result = exec_command(program, &args, None, None).unwrap();
        assert_eq!(result["code"], json!(0));
        assert!(result["success"].as_bool().unwrap());
        let stdout = result["stdout"].as_str().unwrap();
        assert!(stdout.contains(expected), "stdout should contain '{}', got: '{}'", expected, stdout);
    }

    #[test]
    fn test_exec_failure() {
        let (program, args) = if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), "exit".to_string(), "1".to_string()])
        } else {
            ("sh", vec!["-c".to_string(), "exit 1".to_string()])
        };
        let result = exec_command(program, &args, None, None).unwrap();
        assert!(!result["success"].as_bool().unwrap());
    }

    #[test]
    fn test_exec_with_cwd() {
        let tmp = std::env::temp_dir().to_string_lossy().to_string();
        let (program, args) = if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), "cd".to_string()])
        } else {
            ("pwd", vec![])
        };
        let result = exec_command(program, &args, Some(&tmp), None).unwrap();
        assert_eq!(result["code"], json!(0));
    }

    #[test]
    fn test_unknown_method() {
        assert!(handle("shell.unknownMethod", &json!({}), 1).is_err());
    }
}
