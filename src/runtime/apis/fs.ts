import vokexCall from "./vokexCall";

// ==============================
// 工具函数
// ==============================

const CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/**
 * Base64 字符串 → Uint8Array（高性能实现，避免 atob + charCodeAt 逐字节转换）
 */
function base64ToUint8Array(base64: string): Uint8Array {
  const len = base64.length;
  // 计算实际字节数（去掉 padding）
  const padLen = base64.endsWith("==") ? 2 : base64.endsWith("=") ? 1 : 0;
  const byteLen = ((len * 3) >> 2) - padLen;
  const bytes = new Uint8Array(byteLen);

  let idx = 0;
  let i = 0;
  while (i < len) {
    const a = CHARS.indexOf(base64[i++]);
    const b = CHARS.indexOf(base64[i++]);
    const c = CHARS.indexOf(base64[i++]);
    const d = CHARS.indexOf(base64[i++]);

    bytes[idx++] = (a << 2) | (b >> 4);
    if (idx < byteLen) bytes[idx++] = ((b & 15) << 4) | (c >> 2);
    if (idx < byteLen) bytes[idx++] = ((c & 3) << 6) | d;
  }

  return bytes;
}

/**
 * Uint8Array → Base64 字符串（高性能实现，避免逐字节 btoa）
 */
function uint8ArrayToBase64(bytes: Uint8Array): string {
  let result = "";
  const len = bytes.length;
  for (let i = 0; i < len; i += 3) {
    const b0 = bytes[i];
    const b1 = i + 1 < len ? bytes[i + 1] : 0;
    const b2 = i + 2 < len ? bytes[i + 2] : 0;
    const n = (b0 << 16) | (b1 << 8) | b2;

    result += CHARS[(n >> 18) & 0x3f];
    result += CHARS[(n >> 12) & 0x3f];
    result += i + 1 < len ? CHARS[(n >> 6) & 0x3f] : "=";
    result += i + 2 < len ? CHARS[n & 0x3f] : "=";
  }
  return result;
}

// ==============================
// 类型定义
// ==============================

/** 文件系统 Stats 对象（对齐 Node.js fs.Stats） */
export interface Stats {
  /** 是否是文件 */
  isFile: boolean;
  /** 是否是目录 */
  isDir: boolean;
  /** 是否是符号链接 */
  isSymlink: boolean;
  /** 文件大小（字节） */
  size: number;
  /** 最后访问时间（Unix 毫秒时间戳） */
  atimeMs: number;
  /** 最后修改时间（Unix 毫秒时间戳） */
  mtimeMs: number;
  /** 创建时间（Unix 毫秒时间戳） */
  birthtimeMs: number;
  /** 文件模式（权限位） */
  mode: number;
}

/** 目录项（readdir withFileTypes 模式） */
export interface Dirent {
  /** 文件名 */
  name: string;
  /** 是否是文件 */
  isFile: boolean;
  /** 是否是目录 */
  isDir: boolean;
  /** 是否是符号链接 */
  isSymlink: boolean;
}

/** readFile 选项 */
export interface ReadFileOptions {
  /** 编码方式。为空/null 时返回 Uint8Array（二进制），'base64' 返回 base64 字符串 */
  encoding?: "utf8" | "base64" | "hex" | null;
}

/** writeFile 选项 */
export interface WriteFileOptions {
  /** 写入模式：'w' 覆盖（默认），'a' 追加，'wx' 排他创建（文件已存在则报错） */
  flag?: "w" | "a" | "wx";
  /** 文件权限（仅 Unix 有效） */
  mode?: number;
  /** 数据编码：'utf8'（默认）或 'base64' */
  encoding?: "utf8" | "base64";
}

/** rm 选项 */
export interface RmOptions {
  /** 是否递归删除目录 */
  recursive?: boolean;
  /** 为 true 时，路径不存在不报错 */
  force?: boolean;
}

/** readdir 选项 */
export interface ReaddirOptions {
  /** 为 true 时返回 Dirent 对象数组，否则返回文件名字符串数组 */
  withFileTypes?: boolean;
}

/** mkdir 选项 */
export interface MkdirOptions {
  /** 是否递归创建父目录 */
  recursive?: boolean;
}

/** Glob 选项 */
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

/** 流式 Glob 回调选项 */
export interface GlobStreamCallbacks {
  /** 每找到一个文件时触发 */
  onMatch: (path: string, index: number) => void;
  /** 搜索完成时触发 */
  onDone: (total: number) => void;
  /** 发生错误时触发 */
  onError?: (error: Error) => void;
}

// ==============================
// API 接口定义（函数重载）
// ==============================

export interface FsAPI {
  /**
   * 读取文件内容
   * - 无 encoding / null：返回 Uint8Array（二进制数据）
   * - encoding='utf8'：返回 UTF-8 字符串
   * - encoding='base64'：返回 Base64 字符串
   * - encoding='hex'：返回 Hex 字符串
   */
  readFile(path: string): Promise<Uint8Array>;
  readFile(path: string, options: { encoding: "utf8" }): Promise<string>;
  readFile(path: string, options: { encoding: "base64" }): Promise<string>;
  readFile(path: string, options: { encoding: "hex" }): Promise<string>;
  readFile(path: string, options?: ReadFileOptions): Promise<Uint8Array | string>;

  /**
   * 写入文件
   * - flag='w'（默认）：覆盖写入
   * - flag='a'：追加写入
   * - flag='wx'：排他创建，文件已存在则报错
   * - data 支持 string | Uint8Array
   */
  writeFile(path: string, data: string | Uint8Array, options?: WriteFileOptions): Promise<void>;

  /** 删除文件或目录（对齐 Node.js fs.rm） */
  rm(path: string, options?: RmOptions): Promise<void>;

  /** 读取目录内容 */
  readdir(path: string, options?: ReaddirOptions): Promise<string[] | Dirent[]>;

  /** 创建目录（对齐 Node.js fs.mkdir） */
  mkdir(path: string, options?: MkdirOptions): Promise<void>;

  /** 获取文件/目录信息（对齐 Node.js fs.stat） */
  stat(path: string): Promise<Stats>;

  /** 检查路径是否存在 */
  exists(path: string): Promise<boolean>;

  /** 复制文件（对齐 Node.js fs.copyFile） */
  copyFile(src: string, dest: string): Promise<void>;

  /** 重命名/移动文件（对齐 Node.js fs.rename） */
  rename(oldPath: string, newPath: string): Promise<void>;

  /** 使用 glob 模式搜索文件 */
  glob(options: GlobOptions): Promise<string[]>;

  /**
   * 流式 glob 搜索：边遍历边返回结果，适合大量文件的场景
   * @returns 返回 streamId，可用于后续取消操作
   */
  globStream(options: GlobOptions, callbacks: GlobStreamCallbacks): Promise<string>;
}

// ==============================
// 实现
// ==============================

export const fs: FsAPI = {
  readFile: (path: string, options?: ReadFileOptions): any => {
    const encoding = options?.encoding ?? null;
    // 始终请求 base64 传输（高效 IPC），在 TS 端按需转换
    return vokexCall("fs.readFile", { path, encoding: encoding ?? "base64" }).then(
      (data: string) => {
        if (encoding === "utf8" || encoding === "base64" || encoding === "hex") {
          return data;
        }
        // 无编码：将 base64 静默转为 Uint8Array
        return base64ToUint8Array(data);
      }
    );
  },

  writeFile: (path: string, data: string | Uint8Array, options?: WriteFileOptions): Promise<void> => {
    let sendEncoding = options?.encoding ?? "utf8";
    let sendData: string | number[];

    if (data instanceof Uint8Array) {
      // Uint8Array → base64 传输
      sendData = uint8ArrayToBase64(data);
      sendEncoding = "base64";
    } else {
      sendData = data;
    }

    return vokexCall("fs.writeFile", {
      path,
      data: sendData,
      flag: options?.flag ?? "w",
      encoding: sendEncoding,
    });
  },

  rm: (path: string, options?: RmOptions): Promise<void> =>
    vokexCall("fs.rm", {
      path,
      recursive: options?.recursive ?? false,
      force: options?.force ?? false,
    }),

  readdir: (path: string, options?: ReaddirOptions): Promise<string[] | Dirent[]> =>
    vokexCall("fs.readdir", { path, withFileTypes: options?.withFileTypes ?? false }),

  mkdir: (path: string, options?: MkdirOptions): Promise<void> =>
    vokexCall("fs.mkdir", { path, recursive: options?.recursive ?? false }),

  stat: (path: string): Promise<Stats> =>
    vokexCall("fs.stat", { path }),

  exists: (path: string): Promise<boolean> =>
    vokexCall("fs.exists", { path }),

  copyFile: (src: string, dest: string): Promise<void> =>
    vokexCall("fs.copyFile", { src, dest }),

  rename: (oldPath: string, newPath: string): Promise<void> =>
    vokexCall("fs.rename", { oldPath, newPath }),

  glob: (options: GlobOptions): Promise<string[]> =>
    vokexCall("fs.glob", options),

  globStream: async (options: GlobOptions, callbacks: GlobStreamCallbacks): Promise<string> => {
    const vokex = (window as any).__VOKEX__;
    if (!vokex?.call) {
      console.warn(`[vokex] 此 API 仅在原生模式下可用`);
      return "";
    }

    // 调用 Rust 层获取 streamId
    const result = await vokexCall("fs.globStream", options);
    const streamId = result?.streamId;
    if (!streamId) {
      throw new Error("Failed to create glob stream");
    }

    // 监听数据事件
    const dataEvent = `glob.data.${streamId}`;
    const doneEvent = `glob.done.${streamId}`;

    const onData = (data: any) => {
      callbacks.onMatch(data.path, data.index);
    };

    const onDone = (data: any) => {
      callbacks.onDone(data.total);
      // 清理监听器
      vokex.off(dataEvent, onData);
      vokex.off(doneEvent, onDone);
    };

    vokex.on(dataEvent, onData);
    vokex.on(doneEvent, onDone);

    return streamId;
  },
};
