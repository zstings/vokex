//! 危险 API 分类定义
//! 
//! 维护危险 API 列表。默认安全，只标记真正危险的 API。
//! 新增 API 默认可用，除非主动标记为危险。

/// 危险 API 列表
/// 
/// 这些 API 具有高风险性，远端页面需要显式授权才能使用
const DANGEROUS_API_LIST: &[&str] = &[
    // 文件系统 - 读写
    "fs.readFile",
    "fs.readFileBinary",
    "fs.writeFile",
    "fs.appendFile",
    // 文件系统 - 删除/移动
    "fs.deleteFile",
    "fs.createDir",
    "fs.removeDir",
    "fs.copyFile",
    "fs.moveFile",
    // Shell
    "shell.exec",
    "shell.spawn",
    "shell.kill",
    "shell.openPath",
    "shell.trashItem",
    // 进程
    "process.kill",
    // 保存对话框
    "dialog.showSaveDialog",
];

/// 判断 API 是否为危险 API
#[inline]
pub fn is_dangerous_api(method: &str) -> bool {
    DANGEROUS_API_LIST.contains(&method)
}
