import { tray, type Tray } from "vokex.app";
import { log, clear } from './utils';

let currentTray: Tray | null = null;

document.getElementById("btn-tray-create")?.addEventListener("click", async () => {
  clear();
  log("=== 创建系统托盘 ===");

  try {
    currentTray = await tray.create({
      icon: "icon/icon.ico",
      tooltip: "Vokex 托盘示例",
      title: "Vokex Demo",
      menu: [
        { id: "show", label: "显示窗口" },
        { id: "hide", label: "隐藏窗口" },
        { type: "separator" },
        { id: "info", label: "查看信息" },
        { id: "quit", label: "退出" },
      ],
    });

    log(`托盘创建成功，ID: ${currentTray.getId()}`);
  } catch (e: any) {
    log(`创建失败: ${e.message}`);
  }
});

document.getElementById("btn-tray-set-tooltip")?.addEventListener("click", async () => {
  clear();
  log("=== 设置托盘提示 ===");

  if (!currentTray) {
    log("请先创建托盘");
    return;
  }

  try {
    const text = `Vokex - ${new Date().toLocaleTimeString()}`;
    await currentTray.setToolTip(text);
    log(`提示已更新: ${text}`);
  } catch (e: any) {
    log(`失败: ${e.message}`);
  }
});

document.getElementById("btn-tray-set-menu")?.addEventListener("click", async () => {
  clear();
  log("=== 更新托盘菜单 ===");

  if (!currentTray) {
    log("请先创建托盘");
    return;
  }

  try {
    await currentTray.setMenu([
      { id: "refresh", label: "刷新" },
      { id: "about", label: "关于" },
      { type: "separator" },
      { id: "quit", label: "退出应用" },
    ]);
    log("托盘菜单已更新");
  } catch (e: any) {
    log(`失败: ${e.message}`);
  }
});

document.getElementById("btn-tray-destroy")?.addEventListener("click", async () => {
  clear();
  log("=== 销毁托盘 ===");

  if (!currentTray) {
    log("请先创建托盘");
    return;
  }

  try {
    await currentTray.destroy();
    log("托盘已销毁");
    currentTray = null;
  } catch (e: any) {
    log(`失败: ${e.message}`);
  }
});

document.getElementById("btn-tray-listen")?.addEventListener("click", () => {
  clear();
  log("=== 监听托盘事件 ===");

  if (!currentTray) {
    log("请先创建托盘");
    return;
  }

  currentTray.on("click", (data) => {
    log(`托盘被点击 [${data.button}] - 时间: ${new Date().toLocaleTimeString()}`);
  });

  currentTray.on("double-click", (data) => {
    log(`托盘被双击 [${data.button}] - 时间: ${new Date().toLocaleTimeString()}`);
  });

  log("已注册 click 和 double-click 监听器");
  log("试试点击系统托盘图标");
});
