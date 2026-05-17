/**
 * vokex 框架 - 友好的 API 封装
 *
 * 提供类似 import { os } from "vokex.app"; 的调用方式
 */

import vokexCall from "./apis/vokexCall";
import { events } from "./apis/events";








/**
 * ProxyConfig 接口
 */
export interface ProxyConfig {
  proxyRules: string;
  pacScript?: string;
  proxyBypassRules?: string;
}


// ==============================
// 菜单 API
// ==============================

/**
 * 菜单项类型
 */
export type MenuItemType = 'normal' | 'separator' | 'submenu' | 'checkbox' | 'native';

/**
 * 原生菜单项标签
 */
export type NativeLabel =
  | 'separator' | 'copy' | 'cut' | 'paste' | 'selectAll'
  | 'undo' | 'redo' | 'minimize' | 'maximize' | 'fullscreen'
  | 'hide' | 'hideOthers' | 'showAll' | 'closeWindow'
  | 'quit' | 'about' | 'services' | 'bringAllToFront';

/**
 * 菜单项接口
 */
export interface MenuItem {
  /** 菜单项类型 */
  type?: MenuItemType;
  /** 菜单项 ID（点击事件中返回） */
  id?: string;
  /** 菜单项标签文本 */
  label?: string;
  /** 是否启用 */
  enabled?: boolean;
  /** 是否选中（checkbox 类型） */
  checked?: boolean;
  /** 快捷键，如 "Ctrl+Shift+S" */
  accelerator?: string;
  /** 子菜单项 */
  submenu?: MenuItem[];
  /** 原生菜单项标签（native 类型） */
  nativeLabel?: NativeLabel;
}

/**
 * 底层调用（兼容旧版 API）
 */
export const call = (method: string, args: any[] = []): Promise<any> => {
  return vokexCall(method, args);
};

/**
 * CPU 信息
 */
export interface CpuInfo {
  /** CPU 制造商 */
  manufacturer: string;
  /** CPU 型号 */
  model: string;
  /** 物理核心数 */
  cores: number;
  /** 逻辑处理器数 */
  logicalProcessors: number;
  /** 架构 */
  architecture: string;
}

/**
 * 系统内存信息
 */
export interface MemoryInfo {
  /** 总内存（字节） */
  total: number;
  /** 可用内存（字节） */
  available: number;
  /** 已用内存（字节） */
  used: number;
}

/**
 * 操作系统信息
 */
export interface OsInfo {
  /** 操作系统名称 */
  name: string;
  /** 操作系统版本 */
  version: string;
  /** 平台 */
  platform: string;
  /** 架构 */
  arch: string;
}

export { app } from './apis/app';
export { events } from './apis/events';
export { storage } from './apis/storage';
export { fs, type Stats, type Dirent, type ReadFileOptions, type WriteFileOptions, type RmOptions, type ReaddirOptions, type MkdirOptions, type GlobOptions, type GlobStreamCallbacks } from './apis/fs';
export { shell } from './apis/shell';
export { process } from './apis/process';
export { http, type HttpAPI, type HttpInit } from './apis/http';
export { clipboard } from './apis/clipboard';
export { dialog, type DialogAPI, type MessageBoxOptions, type CustomMessageBoxOptions, type MessageBoxResult, type OpenDialogOptions, type SaveDialogOptions, type ErrorBoxOptions, type FileFilter } from './apis/dialog';
export { notification } from './apis/notification';
export { computer } from './apis/computer';
export { menu } from './apis/menu';
export { browserWindow } from './apis/browserWindow';
export { tray, Tray, type TrayOptions, type TrayEventType } from './apis/tray';
export { shortcut, type ShortcutAPI, type UnregisterFn, type HotKeyInfo, type ShortcutTriggeredEvent, type ShortcutHandler } from './apis/shortcut';
export { path, type PathAPI } from './apis/path';

