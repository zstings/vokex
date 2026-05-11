import { shortcut, type UnregisterFn } from "vokex.app";
import { log, clear } from './utils';

// 存储注销函数，用于单个注销
const unbindMap = new Map<string, UnregisterFn>();

// ==================== 注册快捷键 ====================

document.getElementById("btn-shortcut-register-1")?.addEventListener("click", async () => {
  clear();
  log("=== 注册 Ctrl+Shift+S ===");
  try {
    const unbind = await shortcut.register("Ctrl+Shift+S", (ev) => {
      log(`[${ev.accelerator}] 被按下`);
    });
    unbindMap.set("Ctrl+Shift+S", unbind);
    log("注册成功，试试按下 Ctrl+Shift+S");
  } catch (e: any) {
    log(`注册失败: ${e.message}`);
  }
});

document.getElementById("btn-shortcut-register-2")?.addEventListener("click", async () => {
  clear();
  log("=== 注册 Alt+F5 ===");
  try {
    const unbind = await shortcut.register("Alt+F5", (ev) => {
      log(`[${ev.accelerator}] 被按下`);
    });
    unbindMap.set("Alt+F5", unbind);
    log("注册成功，试试按下 Alt+F5");
  } catch (e: any) {
    log(`注册失败: ${e.message}`);
  }
});

document.getElementById("btn-shortcut-register-3")?.addEventListener("click", async () => {
  clear();
  log("=== 注册 Ctrl+Alt+Space ===");
  try {
    const unbind = await shortcut.register("Ctrl+Alt+Space", (ev) => {
      log(`[${ev.accelerator}] 被按下`);
    });
    unbindMap.set("Ctrl+Alt+Space", unbind);
    log("注册成功，试试按下 Ctrl+Alt+Space");
  } catch (e: any) {
    log(`注册失败: ${e.message}`);
  }
});

// ==================== 批量注册 ====================

document.getElementById("btn-shortcut-register-all")?.addEventListener("click", async () => {
  clear();
  log("=== 批量注册 ===");
  try {
    const unbind = await shortcut.registerAll({
      "Ctrl+Shift+1": (ev) => log(`[${ev.accelerator}] 批量-1 触发`),
      "Ctrl+Shift+2": (ev) => log(`[${ev.accelerator}] 批量-2 触发`),
      "Ctrl+Shift+3": (ev) => log(`[${ev.accelerator}] 批量-3 触发`),
    });
    unbindMap.set("batch", unbind);
    log("批量注册成功: Ctrl+Shift+1 / 2 / 3");
  } catch (e: any) {
    log(`注册失败: ${e.message}`);
  }
});

// ==================== 注销快捷键 ====================

document.getElementById("btn-shortcut-unregister-1")?.addEventListener("click", async () => {
  clear();
  log("=== 注销 Ctrl+Shift+S ===");
  const unbind = unbindMap.get("Ctrl+Shift+S");
  if (!unbind) {
    log("该快捷键未注册");
    return;
  }
  try {
    await unbind();
    unbindMap.delete("Ctrl+Shift+S");
    log("注销成功");
  } catch (e: any) {
    log(`注销失败: ${e.message}`);
  }
});

// ==================== 批量注销 ====================

document.getElementById("btn-shortcut-unregister-all")?.addEventListener("click", async () => {
  clear();
  log("=== 注销所有快捷键 ===");
  try {
    await shortcut.unregisterAll();
    unbindMap.clear();
    log("所有快捷键已注销");
  } catch (e: any) {
    log(`注销失败: ${e.message}`);
  }
});

// ==================== 查询 ====================

document.getElementById("btn-shortcut-is-registered")?.addEventListener("click", async () => {
  clear();
  log("=== 查询注册状态 ===");
  const targets = ["Ctrl+Shift+S", "Alt+F5", "Ctrl+Alt+Space", "Ctrl+Q"];
  for (const acc of targets) {
    const ok = await shortcut.isRegistered(acc);
    log(`  ${acc}: ${ok ? "已注册" : "未注册"}`);
  }
});

// ==================== 列出所有 ====================

document.getElementById("btn-shortcut-list")?.addEventListener("click", async () => {
  clear();
  log("=== 已注册快捷键列表 ===");
  try {
    const list = await shortcut.list();
    if (list.length === 0) {
      log("  (无)");
    } else {
      list.forEach((item) => {
        log(`  ID=${item.id}  ${item.accelerator}`);
      });
    }
  } catch (e: any) {
    log(`查询失败: ${e.message}`);
  }
});
