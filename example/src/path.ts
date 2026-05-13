import { log, clear } from './utils'
import { path } from "vokex.app";

document.getElementById("btn-path-join")?.addEventListener("click", async () => {
  clear();
  log("=== path.join() ===");
  try {
    const r1 = await path.join("users", "admin", "config.json");
    log(`join("users", "admin", "config.json")`);
    log(`  → ${r1}`);

    const r2 = await path.join("/home", "./user", "../admin", "file.txt");
    log(`join("/home", "./user", "../admin", "file.txt")`);
    log(`  → ${r2}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-path-resolve")?.addEventListener("click", async () => {
  clear();
  log("=== path.resolve() ===");
  try {
    const r1 = await path.resolve("./temp");
    log(`resolve("./temp")`);
    log(`  → ${r1}`);

    const r2 = await path.resolve("/absolute", "relative", "file.txt");
    log(`resolve("/absolute", "relative", "file.txt")`);
    log(`  → ${r2}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-path-normalize")?.addEventListener("click", async () => {
  clear();
  log("=== path.normalize() ===");
  try {
    const r1 = await path.normalize("/home/user/../admin/./config.json");
    log(`normalize("/home/user/../admin/./config.json")`);
    log(`  → ${r1}`);

    const r2 = await path.normalize("foo//bar/../baz");
    log(`normalize("foo//bar/../baz")`);
    log(`  → ${r2}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-path-basename")?.addEventListener("click", () => {
  clear();
  log("=== path.basename() [同步] ===");
  log(`basename("/home/user/file.txt")       → ${path.basename("/home/user/file.txt")}`);
  log(`basename("/home/user/file.txt", ".txt") → ${path.basename("/home/user/file.txt", ".txt")}`);
  log(`basename("index.test.js")             → ${path.basename("index.test.js")}`);
  log(`basename("/home/user/")               → ${path.basename("/home/user/")}`);
});

document.getElementById("btn-path-dirname")?.addEventListener("click", () => {
  clear();
  log("=== path.dirname() [同步] ===");
  log(`dirname("/home/user/file.txt")  → ${path.dirname("/home/user/file.txt")}`);
  log(`dirname("/home/user/")          → ${path.dirname("/home/user/")}`);
  log(`dirname("file.txt")             → ${path.dirname("file.txt")}`);
  log(`dirname("/")                    → ${path.dirname("/")}`);
});

document.getElementById("btn-path-extname")?.addEventListener("click", () => {
  clear();
  log("=== path.extname() [同步] ===");
  log(`extname("index.html")      → ${path.extname("index.html")}`);
  log(`extname("index.test.js")   → ${path.extname("index.test.js")}`);
  log(`extname(".gitignore")      → ${path.extname(".gitignore")}`);
  log(`extname("file.tar.gz")     → ${path.extname("file.tar.gz")}`);
  log(`extname("README")          → ${path.extname("README")}`);
  log(`extname("")                → "${path.extname("")}"`);
});

document.getElementById("btn-path-sep")?.addEventListener("click", () => {
  clear();
  log("=== path.sep ===");
  log(`当前平台路径分隔符: "${path.sep}"`);
  log(`示例: users${path.sep}admin${path.sep}config.json`);
});
