import vokexCall from "./vokexCall";

/**
 * SafeStorage API 接口
 *
 * 安全的持久化键值对存储，数据在本地加密保存（AES-256-GCM）。
 * 适用于存储敏感数据如 API 密钥、用户 Token 等。
 */
export interface SafeStorageAPI {
    /** 存储数据 */
    setItem: (key: string, value: any) => Promise<void>;
    /** 读取数据 */
    getItem: (key: string) => Promise<any>;
    /** 删除指定键 */
    removeItem: (key: string) => Promise<void>;
    /** 清空所有存储 */
    clear: () => Promise<void>;
    /** 获取所有键名 */
    keys: () => Promise<string[]>;
    /** 检查键是否存在 */
    has: (key: string) => Promise<boolean>;
}

/**
 * 安全的持久化键值对存储 API
 *
 * 数据通过 AES-256-GCM 加密后存储在本地文件中，密钥由系统安全区（Windows 凭据管理器/macOS 钥匙串）保护。
 */
export const safeStorage: SafeStorageAPI = {
    /** 存储数据 */
    setItem: (key: string, value: any): Promise<void> =>
        vokexCall('safeStorage.setData', { key, value }),

    /** 读取数据 */
    getItem: (key: string): Promise<any> =>
        vokexCall('safeStorage.getData', { key }),

    /** 删除指定键 */
    removeItem: (key: string): Promise<void> =>
        vokexCall('safeStorage.removeData', { key }),

    /** 清空所有存储 */
    clear: (): Promise<void> =>
        vokexCall('safeStorage.clear', {}),

    /** 获取所有键名 */
    keys: (): Promise<string[]> =>
        vokexCall('safeStorage.getKeys', {}),

    /** 检查键是否存在 */
    has: (key: string): Promise<boolean> =>
        vokexCall('safeStorage.has', { key }),
};
