/**
 * vokex 框架 - Vite 插件
 *
 * 在 Vite 构建完成后，自动将前端资源嵌入到预编译壳中
 *
 * 用法：
 * ```ts
 * // vite.config.ts
 * import { vokexPlugin } from "vokex/vite-plugin";
 *
 * export default defineConfig({
 *   plugins: [
 *     vue(),
 *     vokexPlugin({
 *       name: "我的应用",
 *       window: { width: 1200, height: 800 },
 *     })
 *   ]
 * });
 * ```
 */

import type { Plugin, ResolvedConfig } from "vite";
import { resolve, dirname } from "path";
import { existsSync, writeFileSync, mkdirSync, rmSync, cpSync, readdirSync, unlinkSync, copyFileSync, readFileSync } from "fs";
import { createRequire } from "module";
import { fileURLToPath } from "url";
import { spawn, type ChildProcess } from "child_process";

/** 获取当前文件的目录 */
function getCurrentDir(): string {
  return dirname(fileURLToPath(import.meta.url));
}

/** Vite 插件配置 */
export interface VokexPluginOptions {
  /** 应用名称 */
  name: string;
  /** 应用标识符，用于存储用户数据目录 (e.g. com.example.myapp) */
  identifier: string;
  /** 应用图标路径 */
  icon: string;
  /** 窗口配置 */
  window: {
    title: string;
    width: number;
    height: number;
    minWidth?: number;
    minHeight?: number;
    resizable?: boolean;
    fullscreen?: boolean;
    maximized?: boolean;
    transparent?: boolean;
    decorations?: boolean;
    alwaysOnTop?: boolean;
    center?: boolean;
    /** 应用图标路径 */
    icon?: string;
  };
  /** 应用版本号 */
  version: string;
  /** 输出路径（完整路径，默认为 release/应用名.exe） */
  outputDir?: string;
  /** 自定义壳二进制路径（默认使用内置预编译壳） */
  shellPath?: string;
  /** 是否在开发模式下跳过构建 */
  skipInDev?: boolean;
  /** 是否显示详细日志 */
  verbose?: boolean;
  /** 是否开启调试 */
  devtools?: boolean;
  /** 是否开启window.open拦截
   *  默认拦截不处理
   *  value 0 默认  1 新窗口打开  2 默认浏览器打开
   *  width、heiught value为1时 配置窗口大小 默认 800 * 600
   */
  new_window?: {
    value: 0 | 1| 2,
    width?: number,
    heiught?: number,
  }
  /** 安全配置 */
  security?: {
    /** 允许远端页面调用的危险 API 列表（支持通配符如 computer.*） */
    allowed_remote_apis?: string[];
    /** 是否允许加载远端页面 */
    allow_remote_pages?: boolean;
  }
}

/** 获取预编译壳路径 */
function getPrebuiltShellPath(): string {
  const ext = process.platform === "win32" ? ".exe" : "";
  const fileName = `${process.platform}-${process.arch}${ext}`;
  const currentDir = getCurrentDir();
  const path = resolve(currentDir, `../../prebuilt/${fileName}`);
  if (existsSync(path)) return path;

  throw new Error(
    `找不到预编译壳二进制文件。\n` +
    `请确保依赖安装网站或支持当前平台的预编译壳，或使用 shellPath 选项指定路径。`
  );
}

/** 动态加载 embed 模块 */
async function loadEmbedModule() {
  const require = createRequire(import.meta.url);
  const currentDir = getCurrentDir();
  const embedPath = resolve(currentDir, "../build/embed.js");
  if (existsSync(embedPath)) {
    return require(embedPath);
  }
  // 备用路径
  const fallbackPath = resolve(currentDir, "../../dist/build/embed.js");
  if (existsSync(fallbackPath)) {
    return require(fallbackPath);
  }
  throw new Error(`找不到 embed 模块: ${embedPath}`);
}

export function vokexPlugin(options: VokexPluginOptions): Plugin {
  let config: ResolvedConfig;
  let isDev = false;
  let shellChild: ChildProcess | null = null;

  

  // 获取输出路径
  const getOutputPath = () => {
    if (options.outputDir) {
      return resolve(options.outputDir);
    }
    const outputName = options.name + (process.platform === "win32" ? ".exe" : "");
    return resolve(process.cwd(), "release", outputName);
  };

  // 获取输入目录
  const getInputDir = () => resolve(process.cwd(), config?.build?.outDir || "dist");

  // 启动壳（开发模式）
  function startShell(devUrl: string) {
    // 将public直接复制到壳所在的位置 改名 为 devDist
    const publicDir = resolve(process.cwd(), "public");
    const shellPath = getPrebuiltShellPath();
    const shellDir = dirname(shellPath);
    const entries = readdirSync(shellDir, { withFileTypes: true });
    // 删除除 exe 以外的所有文件和目录
    for (const entry of entries) {
      if (entry.name.includes('-x64')) continue;
      const fullPath = resolve(shellDir, entry.name);
      if (entry.isDirectory()) {
        rmSync(fullPath, { recursive: true, force: true });
      } else {
        unlinkSync(fullPath);
      }
    }
    // 把 public 里的内容复制到壳目录
    const publicEntries = readdirSync(publicDir, { withFileTypes: true });
    for (const entry of publicEntries) {
      const src = resolve(publicDir, entry.name);
      const dest = resolve(shellDir, entry.name);
      if (entry.isDirectory()) {
        cpSync(src, dest, { recursive: true });
      } else {
        copyFileSync(src, dest);
      }
    }

    

    if (!existsSync(shellPath)) {
      console.error(`[vokex] 壳文件不存在: ${shellPath}`);
      console.error(`[vokex] 请先编译壳: cd shell && cargo build --release`);
      return;
    }

    console.log(`[vokex] 启动壳，加载: ${devUrl}`);
    
    const shell = spawn(shellPath, ['--env_dev'], { 
      stdio: ["ignore", "pipe", "pipe"],
      detached: true 
    });
    shellChild = shell;
    shell.unref();

    // 捕获输出
    shell.stdout?.on("data", (data) => {
      console.log(`[shell] ${data.toString().trim()}`);
    });
    shell.stderr?.on("data", (data) => {
      console.error(`[shell] ${data.toString().trim()}`);
    });

    // 壳关闭时退出进程
    shell.on("close", (code) => {
      console.log(`[vokex] 壳已退出，退出码: ${code}`);
      process.exit(code || 0);
    });

    // 壳启动错误
    shell.on("error", (err) => {
      console.error(`[vokex] 壳启动失败: ${err.message}`);
    });

    // Ctrl+C 时杀壳
    process.on("SIGINT", () => {
      if (shellChild && !shellChild.killed) {
        shellChild.kill();
      }
      process.exit(0);
    });
  }

  // 执行构建
  async function doBuild() {
    const inputDir = getInputDir();
    const shellPath = getPrebuiltShellPath();
    const outputPath = getOutputPath();

    if (!existsSync(inputDir)) {
      console.warn(`[vokex] 构建产物目录不存在: ${inputDir}`);
      return;
    }

    try {
      const { build } = await loadEmbedModule();
      const result = await build({
        inputDir,
        shellPath,
        outputPath,
        verbose: options.verbose,
      });

      console.log(`[vokex:build] OUTPUT_DIR=${result.outputPath}`);
    } catch (error: any) {
      console.error(`[vokex] ❌ 构建失败: ${error.message}`);
      if (options.verbose) {
        console.error(error.stack);
      }
      throw error;
    }
  }

  return {
    name: "vokex-plugin",

    config(_, env) {
      isDev = env.command === "serve";
    },

    configResolved(resolvedConfig) {
      config = resolvedConfig;

      // 插件运行的目录如果没有public就创建一个
      // 将 options 写入到 public 目录里 文件名叫 vokex-config.json。
      const publicDir = resolve(process.cwd(), "public");
      if (!existsSync(publicDir)) {
        mkdirSync(publicDir, { recursive: true });
      }
      const configPath = resolve(publicDir, "vokex-config.json");
      writeFileSync(configPath, JSON.stringify(options, null, 2), "utf-8");
    },

    configureServer(server) {
      if (!isDev) return;
      // 开发模式下，Vite 服务器启动后启动壳
      server.httpServer?.once("listening", () => {
        const address = server.httpServer?.address();
        if (address && typeof address === "object" && "port" in address) {
          const port = address.port;
          const devUrl = `http://localhost:${port}`;
          console.log(`[vokex] Vite 开发服务器已启动: ${devUrl}`);
          //配置写入url链接
          const publicDir = resolve(process.cwd(), "public");
          const configPath = resolve(publicDir, "vokex-config.json");
          const vokexConfig = JSON.parse(readFileSync(configPath, "utf-8"));
          vokexConfig.dev_url = devUrl;
          writeFileSync(configPath, JSON.stringify(vokexConfig, null, 2), "utf-8");
          // 启动壳
          startShell(devUrl);
        }
      });
    },

    async closeBundle() {
      // 开发模式下跳过
      if (isDev && options.skipInDev !== false) {
        console.log("[vokex] 开发模式，跳过原生构建");
        return;
      }
      // 执行构建
      await doBuild();
    },
  };
}

// 默认导出
export default vokexPlugin;
