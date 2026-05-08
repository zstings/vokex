import vokexCall from "./vokexCall";

/**
 * DirEntry 目录项
 */
export interface DirEntry {
  /** 文件名 */
  name: string;
  /** 完整路径 */
  path: string;
  /** 是否是目录 */
  isDir: boolean;
}

/**
 * FileInfo 文件信息
 */
export interface FileInfo {
  /** 是否是文件 */
  isFile: boolean;
  /** 是否是目录 */
  isDir: boolean;
  /** 文件大小（字节） */
  size: number;
  /** 最后修改时间（距现在秒数） */
  modified?: number;
}


/**
 * GlobOptions glob 选项
 */
export interface GlobOptions {
  /** glob 模式，如 '*.txt', '**\/*.js' */
  pattern: string;
  /** 搜索目录，默认当前目录 */
  cwd?: string;
  /** 排除模式列表 */
  ignore?: string[];
  /** 只返回文件，不返回目录 */
  nodir?: boolean;
  /** 返回绝对路径 */
  absolute?: boolean;
  /** 是否包含隐藏文件（以 . 开头） */
  dot?: boolean;
}

/**
 * 文件系统 API 接口
 */
export interface FsAPI {
  /** 读取文本文件 */
  readFile: (path: string) => Promise<string>;
  /** 读取二进制文件 */
  readFileBinary: (path: string) => Promise<string>;
  /** 写入文本文件 */
  writeFile: (path: string, data: string) => Promise<void>;
  /** 追加内容到文件 */
  appendFile: (path: string, data: string) => Promise<void>;
  /** 删除文件 */
  deleteFile: (path: string) => Promise<void>;
  /** 读取目录内容 */
  readDir: (path: string) => Promise<DirEntry[]>;
  /** 创建目录（支持递归创建） */
  createDir: (path: string) => Promise<void>;
  /** 删除目录（支持递归删除） */
  removeDir: (path: string) => Promise<void>;
  /** 获取文件/目录信息 */
  stat: (path: string) => Promise<FileInfo>;
  /** 检查路径是否存在 */
  exists: (path: string) => Promise<boolean>;
  /** 复制文件 */
  copyFile: (source: string, destination: string) => Promise<void>;
  /** 移动/重命名文件 */
  moveFile: (source: string, destination: string) => Promise<void>;
  /** 使用 glob 模式搜索文件 */
  glob: (options: GlobOptions) => Promise<string[]>;
}

/**
 * 文件系统相关 API
 */
export const fs: FsAPI = {
  /** 读取文本文件 */
  readFile: (path: string): Promise<string> => vokexCall('fs.readFile', { path }),

  /** 读取二进制文件 */
  readFileBinary: (path: string): Promise<string> => vokexCall('fs.readFileBinary', { path }),

  /** 写入文本文件 */
  writeFile: (path: string, data: string): Promise<void> => vokexCall('fs.writeFile', { path, data }),

  /** 追加内容到文件 */
  appendFile: (path: string, data: string): Promise<void> => vokexCall('fs.appendFile', { path, data }),

  /** 删除文件 */
  deleteFile: (path: string): Promise<void> => vokexCall('fs.deleteFile', { path }),

  /** 读取目录内容 */
  readDir: (path: string): Promise<DirEntry[]> => vokexCall('fs.readDir', { path }),

  /** 创建目录（支持递归创建） */
  createDir: (path: string): Promise<void> => vokexCall('fs.createDir', { path }),

  /** 删除目录（支持递归删除） */
  removeDir: (path: string): Promise<void> => vokexCall('fs.removeDir', { path }),

  /** 获取文件/目录信息 */
  stat: (path: string): Promise<FileInfo> => vokexCall('fs.stat', { path }),

  /** 检查路径是否存在 */
  exists: (path: string): Promise<boolean> => vokexCall('fs.exists', { path }),

  /** 复制文件 */
  copyFile: (source: string, destination: string): Promise<void> => vokexCall('fs.copyFile', { source, destination }),

  /** 移动/重命名文件 */
  moveFile: (source: string, destination: string): Promise<void> => vokexCall('fs.moveFile', { source, destination }),

  /** 使用 glob 模式搜索文件 */
  glob: (options: GlobOptions): Promise<string[]> => vokexCall('fs.glob', options),
};