import { events } from "./events";
import vokexCall from "./vokexCall";

/**
 * 窗口选项
 */
export interface WindowOptions {
    /** 窗口标题 */
    title?: string;
    /** 窗口宽度 */
    width?: number;
    /** 窗口高度 */
    height?: number;
    /** 窗口 X 坐标 */
    x?: number;
    /** 窗口 Y 坐标 */
    y?: number;
    /** 是否可调整大小 */
    resizable?: boolean;
    /** 是否最小化 */
    minimizable?: boolean;
    /** 是否最大化 */
    maximizable?: boolean;
    /** 是否可关闭 */
    closable?: boolean;
    /** 是否置顶 */
    alwaysOnTop?: boolean;
    /** 是否全屏 */
    fullscreen?: boolean;
    /** 是否显示在任务栏 */
    skipTaskbar?: boolean;
    /** 窗口透明度 (0.0 - 1.0) [Windows] [macOS] */
    opacity?: number;
    /** 背景色 */
    backgroundColor?: string;
    /** 最小宽度 */
    minWidth?: number;
    /** 最小高度 */
    minHeight?: number;
    /** 最大宽度 */
    maxWidth?: number;
    /** 最大高度 */
    maxHeight?: number;
    /** 窗口图标路径 [Windows] [Linux] */
    icon?: string;
    /** 是否显示 */
    show?: boolean;
    /** 是否居中 */
    center?: boolean;
    /** 加载的 URL */
    url?: string;
    /** 窗口主题 */
    theme?: 'light' | 'dark' | 'system';
}

/**
 * 窗口信息
 */
export interface WindowInfo {
    /** 窗口 ID */
    id: number;
    /** 窗口标题 */
    title: string;
    /** 窗口宽度 */
    width: number;
    /** 窗口高度 */
    height: number;
    /** 窗口 X 坐标 */
    x: number;
    /** 窗口 Y 坐标 */
    y: number;
    /** 是否最大化 */
    is_maximized: boolean;
    /** 是否最小化 */
    is_minimized: boolean;
    /** 是否全屏 */
    is_fullscreen: boolean;
    /** 是否聚焦 */
    is_focused: boolean;
    /** 是否可见 */
    is_visible: boolean;
}

/**
 * 窗口事件类型（系统窗口事件，由 Rust 层通过 window.event 通道发送）
 */
export type WindowEventType =
    | 'close'
    | 'resize'
    | 'move'
    | 'minimize'
    | 'maximize'
    | 'restore'
    | 'focus'
    | 'blur'
    | 'enter-full-screen'
    | 'leave-full-screen';

/**
 * 窗口消息事件类型（由其他窗口通过 sendMessage 发送，或自定义事件）
 */
export type WindowMessageEventType = 'window.message' | string;

/**
 * BrowserWindow 实例类
 */
export class BrowserWindow {
    private id: number;
    private eventListeners: Map<string, Set<(data: any) => void>> = new Map();

    constructor(id: number) {
        this.id = id;
        this.setupEventListeners();
    }

    /** 获取窗口 ID */
    getId(): number {
        return this.id;
    }

    /** 关闭窗口 */
    close(): Promise<void> {
        return vokexCall('browserWindow.close', { id: this.id });
    }

    /** 显示窗口 */
    show(): Promise<void> {
        return vokexCall('browserWindow.show', { id: this.id });
    }

    /** 隐藏窗口 */
    hide(): Promise<void> {
        return vokexCall('browserWindow.hide', { id: this.id });
    }

    /** 最小化窗口 */
    minimize(): Promise<void> {
        return vokexCall('browserWindow.minimize', { id: this.id });
    }

    /** 最大化窗口 */
    maximize(): Promise<void> {
        return vokexCall('browserWindow.maximize', { id: this.id });
    }

    /** 取消最大化 */
    unmaximize(): Promise<void> {
        return vokexCall('browserWindow.unmaximize', { id: this.id });
    }

    /** 恢复窗口 */
    restore(): Promise<void> {
        return vokexCall('browserWindow.restore', { id: this.id });
    }

    /** 聚焦窗口 */
    focus(): Promise<void> {
        return vokexCall('browserWindow.focus', { id: this.id });
    }

    /** 取消聚焦 */
    blur(): Promise<void> {
        return vokexCall('browserWindow.blur', { id: this.id });
    }

    /** 是否最大化 */
    isMaximized(): Promise<boolean> {
        return vokexCall('browserWindow.isMaximized', { id: this.id });
    }

    /** 是否最小化 */
    isMinimized(): Promise<boolean> {
        return vokexCall('browserWindow.isMinimized', { id: this.id });
    }

    /** 是否全屏 */
    isFullScreen(): Promise<boolean> {
        return vokexCall('browserWindow.isFullScreen', { id: this.id });
    }

    /** 是否聚焦 */
    isFocused(): Promise<boolean> {
        return vokexCall('browserWindow.isFocused', { id: this.id });
    }

    /** 是否可见 */
    isVisible(): Promise<boolean> {
        return vokexCall('browserWindow.isVisible', { id: this.id });
    }

    /** 是否可调整大小 */
    isResizable(): Promise<boolean> {
        return vokexCall('browserWindow.isResizable', { id: this.id });
    }

    /** 设置全屏状态 */
    setFullScreen(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.setFullScreen', { id: this.id, flag });
    }

    /** 设置窗口标题 */
    setTitle(title: string): Promise<void> {
        return vokexCall('browserWindow.setTitle', { id: this.id, title });
    }

    /** 获取窗口标题 */
    getTitle(): Promise<string> {
        return vokexCall('browserWindow.getTitle', { id: this.id });
    }

    /** 设置窗口大小 */
    setSize(width: number, height: number): Promise<void> {
        return vokexCall('browserWindow.setSize', { id: this.id, width, height });
    }

    /** 获取窗口大小 */
    getSize(): Promise<[number, number]> {
        return vokexCall('browserWindow.getSize', { id: this.id });
    }

    /** 设置最小窗口尺寸 */
    setMinimumSize(width: number, height: number): Promise<void> {
        return vokexCall('browserWindow.setMinimumSize', { id: this.id, width, height });
    }

    /** 设置最大窗口尺寸 */
    setMaximumSize(width: number, height: number): Promise<void> {
        return vokexCall('browserWindow.setMaximumSize', { id: this.id, width, height });
    }

    /** 设置窗口是否可调整大小 */
    setResizable(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.setResizable', { id: this.id, flag });
    }

    /** 设置窗口是否置顶 */
    setAlwaysOnTop(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.setAlwaysOnTop', { id: this.id, flag });
    }

    /** 设置窗口位置 */
    setPosition(x: number, y: number): Promise<void> {
        return vokexCall('browserWindow.setPosition', { id: this.id, x, y });
    }

    /** 获取窗口位置 */
    getPosition(): Promise<[number, number]> {
        return vokexCall('browserWindow.getPosition', { id: this.id });
    }

    /** 窗口居中 */
    center(): Promise<void> {
        return vokexCall('browserWindow.center', { id: this.id });
    }

    /** 设置窗口透明度 (0.0 - 1.0) [Windows] [macOS] */
    setOpacity(opacity: number): Promise<void> {
        return vokexCall('browserWindow.setOpacity', { id: this.id, opacity });
    }

    /** 设置窗口背景色 */
    setBackgroundColor(color: string): Promise<void> {
        return vokexCall('browserWindow.setBackgroundColor', { id: this.id, color });
    }

    /** 设置窗口图标 [Windows] [Linux] */
    setIcon(icon: string): Promise<void> {
        return vokexCall('browserWindow.setIcon', { id: this.id, icon });
    }

    /** 加载本地 HTML 文件 */
    loadFile(path: string): Promise<void> {
        return vokexCall('browserWindow.loadFile', { id: this.id, path });
    }

    /** 加载远程 URL */
    loadURL(url: string): Promise<void> {
        return vokexCall('browserWindow.loadURL', { id: this.id, url });
    }

    /** 重新加载当前页面 */
    reload(): Promise<void> {
        return vokexCall('browserWindow.reload', { id: this.id });
    }

    /** 设置任务栏进度条 (0.0 - 1.0, -1 隐藏) [Windows] [macOS] */
    setProgressBar(progress: number): Promise<void> {
        return vokexCall('browserWindow.setProgressBar', { id: this.id, progress });
    }

    /** 设置是否在任务栏中显示 [Windows] */
    setSkipTaskbar(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.setSkipTaskbar', { id: this.id, flag });
    }

    /** 闪烁任务栏图标提示用户 [Windows] */
    flashTaskbar(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.flashTaskbar', { id: this.id, flag });
    }

    /** 截取窗口内容 */
    // TODO: Rust端未实现，后续可能会实现
    // capturePage(): Promise<string> {
    //     return vokexCall('browserWindow.capturePage', { id: this.id });
    // }

    /** 获取窗口缩放因子 */
    scaleFactor(): Promise<number> {
        return vokexCall('browserWindow.scaleFactor', { id: this.id });
    }

    /** 获取窗口客户区位置 */
    getInnerPosition(): Promise<{ x: number; y: number }> {
        return vokexCall('browserWindow.innerPosition', { id: this.id });
    }

    /** 获取窗口外部大小（含边框）[通用] */
    getOuterSize(): Promise<{ width: number; height: number }> {
        return vokexCall('browserWindow.outerSize', { id: this.id });
    }

    /** 是否可最小化 */
    isMinimizable(): Promise<boolean> {
        return vokexCall('browserWindow.isMinimizable', { id: this.id });
    }

    /** 设置是否可最小化 */
    setMinimizable(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.setMinimizable', { id: this.id, flag });
    }

    /** 是否可最大化 */
    isMaximizable(): Promise<boolean> {
        return vokexCall('browserWindow.isMaximizable', { id: this.id });
    }

    /** 设置是否可最大化 */
    setMaximizable(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.setMaximizable', { id: this.id, flag });
    }

    /** 是否可关闭 */
    isClosable(): Promise<boolean> {
        return vokexCall('browserWindow.isClosable', { id: this.id });
    }

    /** 设置是否可关闭 */
    setClosable(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.setClosable', { id: this.id, flag });
    }

    /** 是否有窗口装饰（边框）[通用] */
    isDecorated(): Promise<boolean> {
        return vokexCall('browserWindow.isDecorated', { id: this.id });
    }

    /** 设置窗口装饰 */
    setDecorated(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.setDecorated', { id: this.id, flag });
    }

    /** 设置窗口置底 */
    setAlwaysOnBottom(flag: boolean): Promise<void> {
        return vokexCall('browserWindow.setAlwaysOnBottom', { id: this.id, flag });
    }

    /** 请求用户关注 */
    requestUserAttention(level?: 'normal' | 'informational' | 'critical'): Promise<void> {
        return vokexCall('browserWindow.requestUserAttention', { id: this.id, level: level || 'normal' });
    }

    /** 设置内容保护（防截图）[通用] */
    setContentProtection(enabled: boolean): Promise<void> {
        return vokexCall('browserWindow.setContentProtection', { id: this.id, enabled });
    }

    /** 设置窗口是否在所有工作区可见 */
    setVisibleOnAllWorkspaces(visible: boolean): Promise<void> {
        return vokexCall('browserWindow.setVisibleOnAllWorkspaces', { id: this.id, visible });
    }

    /** 设置光标图标 */
    setCursorIcon(icon: string): Promise<void> {
        return vokexCall('browserWindow.setCursorIcon', { id: this.id, icon });
    }

    /** 设置光标位置 */
    setCursorPosition(x: number, y: number): Promise<void> {
        return vokexCall('browserWindow.setCursorPosition', { id: this.id, x, y });
    }

    /** 设置光标抓取 */
    setCursorGrab(grab: boolean): Promise<void> {
        return vokexCall('browserWindow.setCursorGrab', { id: this.id, grab });
    }

    /** 设置光标是否可见 */
    setCursorVisible(visible: boolean): Promise<void> {
        return vokexCall('browserWindow.setCursorVisible', { id: this.id, visible });
    }

    /** 设置窗口主题 */
    setTheme(theme: 'light' | 'dark' | 'system'): Promise<void> {
        return vokexCall('browserWindow.setTheme', { id: this.id, theme });
    }

    /** 获取窗口主题 */
    getTheme(): Promise<'light' | 'dark'> {
        return vokexCall('browserWindow.getTheme', { id: this.id });
    }

    /** 窗口直接通信 */
    sendMessage(message: any, targetWindow: BrowserWindow): Promise<void> {
        return vokexCall('browserWindow.sendMessage', {
            fromId: this.id,
            targetId: targetWindow.getId(),
            message
        });
    }

    /** 监听窗口事件（支持系统事件和消息事件）[通用] */
    on(event: WindowEventType | WindowMessageEventType, callback: (data?: any) => void): () => void {
        if (!this.eventListeners.has(event)) {
            this.eventListeners.set(event, new Set());
        }
        this.eventListeners.get(event)!.add(callback);

        // 返回取消订阅函数
        return () => {
            this.eventListeners.get(event)?.delete(callback);
        };
    }

    /** 取消监听窗口事件 */
    off(event: WindowEventType | WindowMessageEventType, callback: (data?: any) => void): void {
        this.eventListeners.get(event)?.delete(callback);
    }

    /** 设置事件监听器 */
    private setupEventListeners(): void {
        // 监听来自后端的窗口系统事件（close/resize/move 等）
        events.on('window.event', (data: { windowId: number; event: any }) => {
            if (data.windowId === this.id) {
                const eventType = data.event.type as WindowEventType;
                const listeners = this.eventListeners.get(eventType);
                if (listeners) {
                    listeners.forEach(callback => {
                        try {
                            callback(data.event);
                        } catch (e) {
                            console.error(`Error in window event listener for ${eventType}:`, e);
                        }
                    });
                }
            }
        });

        // 监听来自其他窗口的消息事件（window.message）
        events.on('window.message', (data: { from: number; message: any }) => {
            const listeners = this.eventListeners.get('window.message');
            if (listeners) {
                listeners.forEach(callback => {
                    try { callback(data); }
                    catch (e) { console.error('Error in window.message listener:', e); }
                });
            }
        });
    }
}

/**
 * BrowserWindow 静态方法
 */
export const browserWindow = {
    /**
     * 创建新窗口
     * @param options 窗口选项
     * @returns BrowserWindow 实例
     */
    async create(options: WindowOptions = {}): Promise<BrowserWindow> {
        const result = await vokexCall('browserWindow.create', { ...options });
        return new BrowserWindow(result.id);
    },

    /**
     * 获取所有窗口
     * @returns 窗口信息数组
     */
    getAll(): Promise<WindowInfo[]> {
        return vokexCall('browserWindow.getAll', {});
    },

    /**
     * 获取当前聚焦的窗口
     * @returns 窗口信息或 null
     */
    getFocused(): Promise<WindowInfo | null> {
        return vokexCall('browserWindow.getFocused', {});
    },

    /**
     * 根据 ID 获取窗口
     * @param id 窗口 ID
     * @returns BrowserWindow 实例或 null
     */
    async getById(id: number): Promise<BrowserWindow | null> {
        const info = await vokexCall('browserWindow.getById', { id });
        if (info) {
            return new BrowserWindow(id);
        }
        return null;
    },

    /**
     * 根据 ID 获取窗口
     * @param id 窗口 ID
     * @returns BrowserWindow 实例或 null
     */
    async getWindow(id: number): Promise<BrowserWindow | null> {
        const info = await vokexCall('browserWindow.getById', { id });
        if (info) {
            return new BrowserWindow(id);
        }
        return null;
    },
    /**
     * 获取当前窗口（JS 代码所在的窗口）[通用]
     * @returns BrowserWindow 实例或 null
     */
    getCurrentWindow(): BrowserWindow | null {
        const windowId = (window as any).__VOKEX__?.__windowId__;
        if (windowId) {
            return new BrowserWindow(windowId);
        }
        return null;
    },

    /**
     * 获取当前聚焦的窗口
     * @returns BrowserWindow 实例或 null
     */
    async getFocusedWindow(): Promise<BrowserWindow | null> {
        const info = await vokexCall('browserWindow.getFocused', {});
        if (info && info.id !== undefined) {
        return new BrowserWindow(info.id);
        }
        return null;
    },
};
