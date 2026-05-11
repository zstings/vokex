import vokexCall from "./vokexCall";
import { events } from "./events";

/**
 * 快捷键触发事件数据
 */
export interface ShortcutTriggeredEvent {
  /** 热键 ID（底层标识） */
  id: number;
  /** 加速器字符串 */
  accelerator: string;
}

/**
 * 已注册快捷键信息
 */
export interface HotKeyInfo {
  /** 热键 ID（底层标识） */
  id: number;
  /** 加速器字符串 */
  accelerator: string;
}

/** 注销函数类型 */
export type UnregisterFn = () => Promise<void>;

/** 快捷键回调函数类型 */
export type ShortcutHandler = (event: ShortcutTriggeredEvent) => void;

/**
 * 内部映射：HotKey ID → 回调函数
 * TS 侧维护，Rust 只发送带 ID 的事件
 */
const handlerMap = new Map<number, ShortcutHandler>();

// 全局事件分发：Rust 侧触发时统一走这里，按 ID 查找并执行对应回调
events.on("shortcut.triggered", (ev: ShortcutTriggeredEvent) => {
  const handler = handlerMap.get(ev.id);
  if (handler) {
    handler(ev);
  }
});

/**
 * Shortcut API 接口
 */
export interface ShortcutAPI {
  /**
   * 注册全局快捷键
   * @param accelerator - 加速器字符串，如 "Ctrl+Shift+A"、"CmdOrCtrl+S"、"Alt+F4"
   * @param handler - 触发时的回调函数
   * @returns 注销函数，调用后自动注销该快捷键并移除回调
   */
  register: (accelerator: string, handler: ShortcutHandler) => Promise<UnregisterFn>;

  /**
   * 批量注册多个快捷键
   * @param bindings - 加速器到回调的映射，如 { "Ctrl+S": save, "Ctrl+N": create }
   * @returns 注销函数，调用后注销这一批所有快捷键
   */
  registerAll: (bindings: Record<string, ShortcutHandler>) => Promise<UnregisterFn>;

  /** 注销所有已注册的全局快捷键 */
  unregisterAll: () => Promise<void>;

  /**
   * 查询指定加速器是否已注册
   * @param accelerator - 加速器字符串
   */
  isRegistered: (accelerator: string) => Promise<boolean>;

  /** 列出所有已注册的快捷键 */
  list: () => Promise<HotKeyInfo[]>;
}

/**
 * 全局快捷键 API
 *
 * 基于系统级全局热键，即使应用不在前台也能响应。
 * 触发时直接调用注册时传入的回调函数。
 *
 * 支持的修饰键：Ctrl、Shift、Alt、Super（Win/Cmd）
 * 支持的按键：A-Z、F1-F24、Space、Enter、Escape、数字 0-9 等
 */
export const shortcut: ShortcutAPI = {
  async register(accelerator: string, handler: ShortcutHandler): Promise<UnregisterFn> {
    const id: number = await vokexCall("shortcut.register", { accelerator });
    handlerMap.set(id, handler);
    // 闭包自销毁：同时清理 Map 和 Rust 侧注册
    return async () => {
      handlerMap.delete(id);
      await vokexCall("shortcut.unregister", { id });
    };
  },

  async registerAll(bindings: Record<string, ShortcutHandler>): Promise<UnregisterFn> {
    const unbinds: UnregisterFn[] = [];
    for (const [accelerator, handler] of Object.entries(bindings)) {
      unbinds.push(await shortcut.register(accelerator, handler));
    }
    return async () => {
      await Promise.all(unbinds.map((fn) => fn()));
    };
  },

  async unregisterAll(): Promise<void> {
    handlerMap.clear();
    const list: HotKeyInfo[] = await vokexCall("shortcut.list", {});
    await Promise.all(
      list.map((item) => vokexCall("shortcut.unregister", { id: item.id }))
    );
  },

  isRegistered(accelerator: string): Promise<boolean> {
    return vokexCall("shortcut.isRegistered", { accelerator });
  },

  list(): Promise<HotKeyInfo[]> {
    return vokexCall("shortcut.list", {});
  },
};
