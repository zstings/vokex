import vokexCall from "./vokexCall";

/**
 * FileFilter 文件过滤器
 */
export interface FileFilter {
  /** 过滤器名称 */
  name: string;
  /** 文件扩展名列表，`["*"]` 表示所有文件 */
  extensions: string[];
}

/**
 * MessageBoxOptions 标准按钮消息对话框选项
 */
export interface MessageBoxOptions {
  /** 对话框标题 */
  title?: string;
  /** 消息内容 */
  message: string;
  /** 按钮类型 */
  type?: 'none' | 'okCancel' | 'yesNo' | 'yesNoCancel';
  /** 图标类型 */
  icon?: 'info' | 'warning' | 'error';
}

/**
 * CustomMessageBoxOptions 自定义按钮消息对话框选项
 */
export interface CustomMessageBoxOptions {
  /** 对话框标题 */
  title?: string;
  /** 消息内容 */
  message: string;
  /** 自定义按钮标签数组，如 ['确定', '取消', '再想想'] */
  buttons: string[];
  /** 图标类型 */
  icon?: 'info' | 'warning' | 'error';
}

/**
 * MessageBoxResult 标准按钮消息对话框返回值
 */
export interface MessageBoxResult {
  /** 用户点击的按钮 */
  response: 'ok' | 'cancel' | 'yes' | 'no';
  /** 是否取消 */
  cancelled: boolean;
}

/**
 * OpenDialogOptions 打开文件对话框选项
 */
export interface OpenDialogOptions {
  /** 对话框标题 */
  title?: string;
  /** 默认路径（目录或包含文件名的完整路径） */
  defaultPath?: string;
  /** 默认文件名 */
  defaultName?: string;
  /** 是否多选 */
  multiple?: boolean;
  /** 是否允许选择目录 */
  directory?: boolean;
  /** 文件过滤器 */
  filters?: FileFilter[];
}

/**
 * SaveDialogOptions 保存文件对话框选项
 */
export interface SaveDialogOptions {
  /** 对话框标题 */
  title?: string;
  /** 默认路径 */
  defaultPath?: string;
  /** 默认文件名 */
  defaultName?: string;
  /** 文件过滤器 */
  filters?: FileFilter[];
}

/**
 * ErrorBoxOptions 错误对话框选项
 */
export interface ErrorBoxOptions {
  /** 对话框标题 */
  title?: string;
  /** 错误消息 */
  message: string;
}

/**
 * Dialog API 接口
 */
export interface DialogAPI {
  /** 显示消息对话框（自定义按钮），返回用户点击的按钮索引 */
  showMessageBox(options: CustomMessageBoxOptions): Promise<number>;
  /** 显示消息对话框（标准按钮），返回用户点击的按钮 */
  showMessageBox(options: MessageBoxOptions): Promise<MessageBoxResult>;

  /** 显示错误对话框 */
  showErrorBox(options: ErrorBoxOptions): Promise<void>;

  /** 显示打开文件对话框（多选），返回文件路径数组或 null */
  showOpenDialog(options: OpenDialogOptions & { multiple: true }): Promise<string[] | null>;
  /** 显示打开文件对话框（单选/默认），返回文件路径或 null */
  showOpenDialog(options?: OpenDialogOptions): Promise<string | null>;

  /** 显示保存文件对话框 */
  showSaveDialog(options?: SaveDialogOptions): Promise<string | null>;

  /** 确认对话框，返回用户是否确认 */
  confirm(options: Omit<MessageBoxOptions, 'type'>): Promise<boolean>;
  /** 信息提示对话框（无需处理返回值） */
  info(options: Omit<MessageBoxOptions, 'type' | 'icon'>): Promise<void>;
  /** 错误提示对话框（无需处理返回值） */
  error(options: ErrorBoxOptions): Promise<void>;
}

/**
 * 自动注入当前窗口 ID，确保在 window 未完全初始化时的健壮性
 */
function withWindowId(options?: Record<string, any>): Record<string, any> {
  const vokex = (window as any).__VOKEX__;
  const windowId = vokex?.__windowId__ ?? 0;
  return { ...(options || {}), windowId };
}

/**
 * 原生对话框 API
 */
export const dialog: DialogAPI = {
  showMessageBox: ((options: any) =>
    vokexCall('dialog.showMessageBox', withWindowId(options))
  ) as DialogAPI['showMessageBox'],

  showErrorBox: (options: ErrorBoxOptions): Promise<void> =>
    vokexCall('dialog.showErrorBox', withWindowId(options)),

  showOpenDialog: ((options?: any) =>
    vokexCall('dialog.showOpenDialog', withWindowId(options))
  ) as DialogAPI['showOpenDialog'],

  showSaveDialog: (options?: SaveDialogOptions): Promise<string | null> =>
    vokexCall('dialog.showSaveDialog', withWindowId(options)),

  confirm: async (options: Omit<MessageBoxOptions, 'type'>): Promise<boolean> => {
    const result: MessageBoxResult = await vokexCall('dialog.showMessageBox', withWindowId({
      ...options,
      type: 'okCancel',
    }));
    return !result.cancelled;
  },

  info: async (options: Omit<MessageBoxOptions, 'type' | 'icon'>): Promise<void> => {
    await vokexCall('dialog.showMessageBox', withWindowId({
      ...options,
      type: 'none',
      icon: 'info',
    }));
  },

  error: async (options: ErrorBoxOptions): Promise<void> => {
    await vokexCall('dialog.showErrorBox', withWindowId(options));
  },
};
