//! 危险 API 分类定义
//! 
//! 将 API 分为三类：
//! - 危险 API (DANGEROUS_APIS): 远端默认禁用
//! - 受限 API (RESTRICTED_APIS): 远端需要配置
//! - 安全 API (SAFE_APIS): 远端可用

use std::collections::HashSet;
use std::sync::OnceLock;

/// 简单的 Lazy 包装器，基于 OnceLock 实现
/// 
/// 用于替代 once_cell::Lazy，只在首次访问时初始化静态数据
macro_rules! lazy_static {
    ($name:ident: $t:ty = $init:expr) => {
        static $name: OnceLock<$t> = OnceLock::new();
        fn $name() -> &'static $t {
            $name.get_or_init(|| $init)
        }
    };
}

lazy_static! {
    DANGEROUS_APIS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        // 文件系统 - 读写操作
        set.insert("fs.readFile");
        set.insert("fs.readFileBinary");
        set.insert("fs.writeFile");
        set.insert("fs.appendFile");
        // 文件系统 - 删除操作
        set.insert("fs.deleteFile");
        set.insert("fs.createDir");
        set.insert("fs.removeDir");
        set.insert("fs.copyFile");
        set.insert("fs.moveFile");
        // Shell - 系统命令
        set.insert("shell.execCommand");
        set.insert("shell.openPath");
        set.insert("shell.trashItem");
        // 进程 - 终止操作
        set.insert("process.kill");
        // 对话框 - 保存操作
        set.insert("dialog.showSaveDialog");
        set
    };
}

lazy_static! {
    RESTRICTED_APIS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        // 文件系统 - 目录操作
        set.insert("fs.readDir");
        set.insert("fs.stat");
        set.insert("fs.exists");
        // HTTP 请求
        set.insert("http.request");
        set.insert("http.get");
        set.insert("http.post");
        set.insert("http.put");
        set.insert("http.delete");
        // 剪贴板
        set.insert("clipboard.writeText");
        set
    };
}

lazy_static! {
    SAFE_APIS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        // 应用信息 - 只读
        set.insert("app.quit");
        set.insert("app.exit");
        set.insert("app.restart");
        set.insert("app.getAppPath");
        set.insert("app.getPath");
        set.insert("app.getVersion");
        set.insert("app.getName");
        set.insert("app.getIdentifier");
        set.insert("app.getLocale");
        set.insert("app.getPid");
        set.insert("app.getArgv");
        set.insert("app.getEnv");
        set.insert("app.getPlatform");
        set.insert("app.getArch");
        set.insert("app.requestSingleInstanceLock");
        // 系统信息 - 只读
        set.insert("computer.getCpuInfo");
        set.insert("computer.getOsInfo");
        set.insert("computer.getDisplays");
        set.insert("computer.getMousePosition");
        set.insert("computer.getKeyboardLayout");
        // 通知 - 用户主动触发
        set.insert("notification.show");
        // 对话框 - 只显示信息
        set.insert("dialog.showMessageBox");
        set.insert("dialog.showErrorBox");
        set.insert("dialog.showOpenDialog");
        // 剪贴板 - 只读
        set.insert("clipboard.readText");
        set.insert("clipboard.clear");
        set
    };
}

/// 判断 API 是否为危险 API
#[inline]
pub fn is_dangerous_api(method: &str) -> bool {
    DANGEROUS_APIS().contains(method)
}

/// 判断 API 是否为受限 API
#[inline]
pub fn is_restricted_api(method: &str) -> bool {
    RESTRICTED_APIS().contains(method)
}

/// 判断 API 是否为安全 API
#[inline]
pub fn is_safe_api(method: &str) -> bool {
    SAFE_APIS().contains(method)
}

/// 判断远端页面是否允许调用某个 API
/// 
/// 规则：
/// - 安全 API：始终允许
/// - 受限 API：始终允许
/// - 危险 API：需要在 allowed_apis 中显式列出
pub fn is_api_allowed_for_remote(method: &str, allowed_apis: &HashSet<&str>) -> bool {
    if is_safe_api(method) {
        return true;
    }
    if is_restricted_api(method) {
        return true;
    }
    allowed_apis.contains(method)
}
