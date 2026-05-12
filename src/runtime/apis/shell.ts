import vokexCall from "./vokexCall";

/**
 * ExecOptions 执行命令选项
 */
export interface ExecOptions {
  /** 工作目录 */
  cwd?: string;
  /** 环境变量 */
  env?: Record<string, string>;
}

/**
 * ExecResult 命令执行结果
 */
export interface ExecResult {
  /** 退出码 */
  code: number | null;
  /** 标准输出 */
  stdout: string;
  /** 标准错误 */
  stderr: string;
  /** 是否成功（退出码为 0） */
  success: boolean;
}

/**
 * ShellProcess 流式进程控制器
 *
 * @example
 * ```typescript
 * // 启动实时编译任务
 * const child = await shell.spawn("npm", ["run", "dev"], { cwd: "./project" });
 *
 * // 监听 stdout 输出
 * const unSub = child.onStdout((data) => {
 *   console.log(`[编译日志]: ${data}`);
 * });
 *
 * // 监听 stderr 输出
 * child.onStderr((data) => {
 *   console.error(`[错误]: ${data}`);
 * });
 *
 * // 监听进程退出
 * child.onExit((code) => {
 *   console.log(`进程退出，退出码: ${code}`);
 * });
 *
 * // 5秒后停止监听并杀死进程
 * setTimeout(async () => {
 *   unSub(); // 停止前端监听
 *   await child.kill(); // 杀死后端进程
 *   console.log("已停止编译");
 * }, 5000);
 * ```
 */
export interface ShellProcess {
  /** 进程 ID */
  pid: number;
  /**
   * 监听 stdout 输出
   * @param cb 回调函数，每行数据触发一次
   * @returns 取消监听函数
   */
  onStdout: (cb: (data: string) => void) => () => void;
  /**
   * 监听 stderr 输出
   * @param cb 回调函数，每行数据触发一次
   * @returns 取消监听函数
   */
  onStderr: (cb: (data: string) => void) => () => void;
  /**
   * 监听进程退出
   * @param cb 回调函数，接收退出码
   * @returns 取消监听函数
   */
  onExit: (cb: (code: number | null) => void) => () => void;
  /**
   * 杀死进程
   * 使用 taskkill /F /T /PID（Windows）或 kill（Unix）强制终止进程树
   */
  kill: () => Promise<void>;
}

/**
 * Shell API 接口
 */
export interface ShellAPI {
  /** 用系统默认浏览器打开 URL */
  openExternal: (url: string) => Promise<void>;
  /** 用系统默认程序打开文件/目录 */
  openPath: (path: string) => Promise<void>;
  /**
   * 一次性执行程序并返回完整结果
   *
   * @example
   * ```typescript
   * const result = await shell.exec("git", ["status"], { cwd: "./my-repo" });
   * console.log(result.stdout);
   * ```
   */
  exec: (program: string, args?: string[], options?: ExecOptions) => Promise<ExecResult>;
  /**
   * 启动流式执行进程，返回进程控制器
   *
   * @example
   * ```typescript
   * const child = await shell.spawn("npm", ["run", "dev"]);
   * child.onStdout((line) => console.log(line));
   * // 稍后...
   * await child.kill();
   * ```
   */
  spawn: (program: string, args?: string[], options?: ExecOptions) => Promise<ShellProcess>;
  /** 将文件移到回收站 */
  trashItem: (path: string) => Promise<void>;
}

/**
 * 注册 shell 流式事件监听
 * @param eventPrefix 事件前缀（如 "shell.stdout.123"）
 * @param pid 进程 ID
 * @param cb 回调函数
 * @returns 取消监听函数
 */
function onShellEvent(eventPrefix: string, pid: number, cb: (data: any) => void): () => void {
  const eventName = `${eventPrefix}.${pid}`;
  // 使用 window.__VOKEX__ 的事件系统
  const vokex = (window as any).__VOKEX__;
  if (vokex && vokex.on) {
    vokex.on(eventName, cb);
    return () => vokex.off(eventName, cb);
  }
  return () => {};
}

/**
 * 创建 ShellProcess 对象
 */
function createShellProcess(pid: number): ShellProcess {
  return {
    pid,
    onStdout: (cb: (data: string) => void) => onShellEvent("shell.stdout", pid, cb),
    onStderr: (cb: (data: string) => void) => onShellEvent("shell.stderr", pid, cb),
    onExit: (cb: (code: number | null) => void) => onShellEvent("shell.exit", pid, (data: any) => {
      // 后端发送 {code, success} 对象，提取 code 字段
      cb(typeof data === 'object' && data !== null ? data.code : data);
    }),
    kill: () => vokexCall('shell.kill', { pid }),
  };
}

/**
 * 系统命令与外部程序相关 API
 *
 * @example
 * ```typescript
 * // 一次性执行
 * const result = await shell.exec("git", ["status"]);
 * console.log(result.stdout);
 *
 * // 流式执行
 * const child = await shell.spawn("npm", ["run", "dev"], { cwd: "./project" });
 * child.onStdout((data) => console.log(data));
 * child.onExit((code) => console.log("退出:", code));
 * // 稍后杀死进程
 * await child.kill();
 * ```
 */
export const shell: ShellAPI = {
  /**
   * 用系统默认浏览器打开 URL
   * @param url 需要打开的 URL
   */
  openExternal: (url: string): Promise<void> => vokexCall('shell.openExternal', { url }),

  /**
   * 用系统默认程序打开文件/目录
   * @param path 文件或目录路径
   */
  openPath: (path: string): Promise<void> => vokexCall('shell.openPath', { path }),

  /**
   * 一次性执行程序并返回完整结果
   *
   * 安全设计：必须指定程序名和参数数组，禁止字符串拼接
   *
   * @param program 程序名（通过系统 PATH 查找）
   * @param args 参数数组
   * @param options 选项（可选）
   * @returns 执行结果，包含 code、stdout、stderr、success
   */
  exec: (program: string, args: string[] = [], options?: ExecOptions): Promise<ExecResult> =>
    vokexCall('shell.exec', { program, args, ...options }),

  /**
   * 启动流式执行进程，返回进程控制器
   *
   * 返回的 ShellProcess 对象提供事件监听方法，可实时获取 stdout/stderr 输出。
   * 进程在后台运行，不会阻塞前端。
   *
   * @param program 程序名（通过系统 PATH 查找）
   * @param args 参数数组
   * @param options 选项（可选）
   * @returns 进程控制器，包含 pid、onStdout、onStderr、onExit、kill 方法
   */
  spawn: async (program: string, args: string[] = [], options?: ExecOptions): Promise<ShellProcess> => {
    const result: { pid: number } = await vokexCall('shell.spawn', { program, args, ...options });
    return createShellProcess(result.pid);
  },

  /**
   * 将文件移到回收站
   * @param path 要移动的文件路径
   */
  trashItem: (path: string): Promise<void> => vokexCall('shell.trashItem', { path }),
};
