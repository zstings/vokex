import { dialog, fs } from "vokex.app";
import { clear, log } from "./utils";

// ─── 标准消息对话框 ──────────────────────────────────────────────────

document.querySelector('#btn-dialog-msg')?.addEventListener('click', async () => {
    clear();
    log("=== 消息对话框（标准按钮）===");
    try {
        const result = await dialog.showMessageBox({
            title: '确认操作',
            message: '确定要执行此操作吗？',
            type: 'okCancel',
            icon: 'warning'
        });
        log(`点击: ${result.response}, 取消: ${result.cancelled}`);
    } catch (error: any) {
        log(`错误: ${error.message}`);
    }
});

// ─── 自定义按钮对话框 ────────────────────────────────────────────────

document.querySelector('#btn-dialog-custom')?.addEventListener('click', async () => {
    clear();
    log("=== 消息对话框（自定义按钮）===");
    try {
        const index = await dialog.showMessageBox({
            title: '选择操作',
            message: '请选择你要执行的操作：',
            buttons: ['保存', '不保存', '取消'],
            icon: 'warning'
        });
        console.log(index);
        const labels = ['保存', '不保存', '取消'];
        log(`点击了第 ${index} 个按钮: ${labels[index]}`);
    } catch (error: any) {
        log(`错误: ${error.message}`);
    }
});

// ─── confirm 快捷方法 ────────────────────────────────────────────────

document.querySelector('#btn-dialog-confirm')?.addEventListener('click', async () => {
    clear();
    log("=== confirm() 快捷方法 ===");
    try {
        const confirmed = await dialog.confirm({
            message: '确定要删除这个文件吗？',
            title: '删除确认'
        });
        log(confirmed ? '用户确认了操作' : '用户取消了操作');
    } catch (error: any) {
        log(`错误: ${error.message}`);
    }
});

// ─── info / error 快捷方法 ───────────────────────────────────────────

document.querySelector('#btn-dialog-info')?.addEventListener('click', async () => {
    clear();
    log("=== info() 快捷方法 ===");
    try {
        await dialog.info({ message: '操作已成功完成！' });
        log('信息提示已关闭');
    } catch (error: any) {
        log(`错误: ${error.message}`);
    }
});

document.querySelector('#btn-dialog-error')?.addEventListener('click', async () => {
    clear();
    log("=== error() 快捷方法 ===");
    try {
        await dialog.error({ title: '错误', message: '操作失败，请重试。' });
        log('错误提示已关闭');
    } catch (error: any) {
        log(`错误: ${error.message}`);
    }
});

// ─── 打开文件（单选）─────────────────────────────────────────────────

document.querySelector('#btn-dialog-open')?.addEventListener('click', async () => {
    clear();
    log("=== 打开文件（单选）===");
    try {
        const result = await dialog.showOpenDialog({
            title: '选择文件',
            defaultPath: 'C:\\',
            filters: [
                { name: '文本文件', extensions: ['txt', 'md'] },
                { name: '所有文件', extensions: ['*'] }
            ]
        });
        log(result ? `选择: ${result}` : '已取消');
    } catch (error: any) {
        log(`错误: ${error.message}`);
    }
});

// ─── 打开文件（多选）─────────────────────────────────────────────────

document.querySelector('#btn-dialog-open-multi')?.addEventListener('click', async () => {
    clear();
    log("=== 打开文件（多选）===");
    try {
        const result = await dialog.showOpenDialog({
            title: '选择多个文件',
            multiple: true,
            filters: [
                { name: '图片', extensions: ['png', 'jpg', 'gif'] },
                { name: '所有文件', extensions: ['*'] }
            ]
        });
        if (result) {
            log(`选择了 ${result.length} 个文件：`);
            result.forEach((p, i) => log(`  [${i}] ${p}`));
        } else {
            log('已取消');
        }
    } catch (error: any) {
        log(`错误: ${error.message}`);
    }
});

// ─── 选择文件夹 ──────────────────────────────────────────────────────

document.querySelector('#btn-dialog-dir')?.addEventListener('click', async () => {
    clear();
    log("=== 选择文件夹 ===");
    try {
        const result = await dialog.showOpenDialog({
            title: '选择文件夹',
            defaultPath: 'C:\\',
            directory: true,
        });
        log(result ? `选择: ${result}` : '已取消');
    } catch (error: any) {
        log(`错误: ${error.message}`);
    }
});

// ─── 保存文件 ────────────────────────────────────────────────────────

document.querySelector('#btn-dialog-save')?.addEventListener('click', async () => {
    clear();
    log("=== 保存文件 ===");
    try {
        const filePath = await dialog.showSaveDialog({
            title: '保存文件',
            defaultPath: 'C:\\',
            defaultName: 'output.txt',
            filters: [{ name: '文本', extensions: ['txt'] }]
        });
        if (filePath) {
            await fs.writeFile(filePath, 'Hello from Vokex!');
            log(`文件已保存: ${filePath}`);
        } else {
            log('已取消');
        }
    } catch (error: any) {
        log(`错误: ${error.message}`);
    }
});
