import vokexCall from "./vokexCall";

export interface PathAPI {
  /** 将多个路径片段拼接成一个完整路径，自动处理不同系统的斜杠 */
  join(...paths: string[]): Promise<string>;

  /** 将路径或路径片段序列解析为绝对路径 */
  resolve(...paths: string[]): Promise<string>;

  /** 规范化路径，处理 '..' 和 '.' */
  normalize(path: string): Promise<string>;

  /** 返回路径的最后一部分（文件名），可过滤后缀 */
  basename(path: string, suffix?: string): string;

  /** 返回路径的目录名 */
  dirname(path: string): string;

  /** 返回路径的扩展名 */
  extname(path: string): string;

  /** 平台特定的路径分隔符 (Windows: '\\', POSIX: '/') */
  readonly sep: string;
}

function parsePath(p: string): { root: string; dir: string; base: string; ext: string; name: string } {
  const isSep = (c: string) => c === '/' || c === '\\';

  // Parse root
  let root = "";
  if (p.length >= 1 && isSep(p[0])) {
    if (p.length >= 2 && isSep(p[1])) {
      // UNC: "//server/share"
      if (p.length >= 3 && !isSep(p[2])) {
        const serverEnd = p.indexOf('/', 3) !== -1 ? p.indexOf('/', 3) : p.indexOf('\\', 3);
        if (serverEnd !== -1) {
          const shareEnd = p.indexOf('/', serverEnd + 1) !== -1 ? p.indexOf('/', serverEnd + 1) : p.indexOf('\\', serverEnd + 1);
          root = shareEnd !== -1 ? p.substring(0, shareEnd) : p;
        } else {
          root = p;
        }
      } else {
        root = p.substring(0, 2);
      }
    } else {
      root = p.substring(0, 1);
    }
  } else if (p.length >= 2 && p[1] === ':') {
    // Drive letter: "C:" or "C:\"
    root = p.length >= 3 && isSep(p[2]) ? p.substring(0, 3) : p.substring(0, 2);
  }

  // Find base start (after last separator)
  let baseStart = root.length;
  let hasTrailingSep = false;
  if (p.length > root.length) {
    const lastChar = p[p.length - 1];
    if (isSep(lastChar)) {
      hasTrailingSep = true;
      // Skip trailing separators
      let end = p.length - 1;
      while (end > root.length && isSep(p[end])) end--;
      // Find last separator before trailing ones
      let lastSep = -1;
      for (let i = end; i >= root.length; i--) {
        if (isSep(p[i])) { lastSep = i; break; }
      }
      baseStart = lastSep !== -1 ? lastSep + 1 : root.length;
    } else {
      let lastSep = -1;
      for (let i = p.length - 1; i >= root.length; i--) {
        if (isSep(p[i])) { lastSep = i; break; }
      }
      baseStart = lastSep !== -1 ? lastSep + 1 : root.length;
    }
  }

  const base = hasTrailingSep ? "" : p.substring(baseStart);
  const dir = baseStart > root.length
    ? p.substring(0, baseStart).replace(/[/\\]+$/, "")
    : baseStart > 0 ? root : "";

  // Parse ext and name from base
  const lastDot = base.lastIndexOf(".");
  if (lastDot <= 0) {
    return { root, dir, base, ext: "", name: base };
  }
  return { root, dir, base, ext: base.substring(lastDot), name: base.substring(0, lastDot) };
}

function posixDirname(p: string): string {
  if (p.length === 0) return ".";

  const isSep = (c: string) => c === '/' || c === '\\';
  // Remove trailing separators
  let end = p.length;
  while (end > 1 && isSep(p[end - 1])) end--;

  // Find last separator
  let lastSep = -1;
  for (let i = end - 1; i >= 0; i--) {
    if (isSep(p[i])) { lastSep = i; break; }
  }

  if (lastSep === -1) return ".";
  if (lastSep === 0) return p[0] === '/' || p[0] === '\\' ? p[0] : ".";

  // Collapse consecutive separators
  let dirEnd = lastSep;
  while (dirEnd > 0 && isSep(p[dirEnd - 1])) dirEnd--;

  return dirEnd === 0 ? p[0] : p.substring(0, dirEnd);
}

function posixBasename(p: string, suffix?: string): string {
  if (p.length === 0) return "";

  const isSep = (c: string) => c === '/' || c === '\\';

  // Remove trailing separators
  let end = p.length;
  while (end > 1 && isSep(p[end - 1])) end--;

  // Find last separator
  let start = 0;
  for (let i = end - 1; i >= 0; i--) {
    if (isSep(p[i])) { start = i + 1; break; }
  }

  let name = p.substring(start, end);
  if (suffix && name.endsWith(suffix) && name.length > suffix.length) {
    name = name.substring(0, name.length - suffix.length);
  }
  return name;
}

function posixExtname(p: string): string {
  const base = posixBasename(p);
  if (base.length === 0) return "";

  if (base === "." || base === "..") return "";
  if (base[0] === "." && base.indexOf(".", 1) === -1) return "";

  const lastDot = base.lastIndexOf(".");
  if (lastDot <= 0) return "";
  return base.substring(lastDot);
}

let _cachedSep: string | undefined;

// Eagerly fetch the platform separator when the module loads.
// By the time user code runs after app.ready, _cachedSep will be populated.
vokexCall("path.getSep", {}).then((s: string) => {
  if (s) _cachedSep = s;
});

export const path: PathAPI = {
  join: (...paths: string[]): Promise<string> => vokexCall("path.join", { paths }),
  resolve: (...paths: string[]): Promise<string> => vokexCall("path.resolve", { paths }),
  normalize: (p: string): Promise<string> => vokexCall("path.normalize", { path: p }),
  basename: (p: string, suffix?: string): string => posixBasename(p, suffix),
  dirname: (p: string): string => posixDirname(p),
  extname: (p: string): string => posixExtname(p),
  get sep(): string {
    if (_cachedSep) return _cachedSep;
    // Fallback for browser dev mode (no native shell)
    return navigator.platform?.startsWith("Win") ? "\\" : "/";
  },
};
