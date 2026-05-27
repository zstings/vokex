import vokexCall from "./vokexCall";
/**
 * Storage API 接口
 */
export interface StorageAPI {
    /** 存储数据 */
    setItem: (key: string, value: any) => Promise<void>;
    /** 读取数据 */
    getItem: (key: string) => Promise<any>;
    /** 获取所有键名 */
    keys: () => Promise<string[]>;
    /** 检查键是否存在 */
    has: (key: string) => Promise<boolean>;
    /** 删除指定键 */
    removeItem: (key: string) => Promise<void>;
    /** 清空所有存储 */
    clear: () => Promise<void>;
}

/**
 * 持久化键值对存储 API
 */
export const storage: StorageAPI = {
    /** 存储数据 */
    setItem: (key: string, value: any): Promise<void> => vokexCall('storage.setData', { key, value }),

    /** 读取数据 */
    getItem: (key: string): Promise<any> => vokexCall('storage.getData', { key }),

    /** 获取所有键名 */
    keys: (): Promise<string[]> => vokexCall('storage.getKeys', {}),

    /** 检查键是否存在 */
    has: (key: string): Promise<boolean> => vokexCall('storage.has', { key }),

    /** 删除指定键 */
    removeItem: (key: string): Promise<void> => vokexCall('storage.removeData', { key }),

    /** 清空所有存储 */
    clear: (): Promise<void> => vokexCall('storage.clear', {}),
};