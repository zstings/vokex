import { log, clear } from './utils'
import { storage } from "vokex.app";
document.getElementById("btn-storage-set")?.addEventListener("click", async () => {
  clear();
  log("=== storage.setItem() ===");
  try {
    const testData = {
      name: "Vokex",
      version: "0.1.0",
      features: ["desktop", "rust", "typescript"],
      timestamp: Date.now(),
    };
    await storage.setItem("test_key", testData);
    log("✅ 已存储数据:");
    log(`  key: test_key`);
    log(`  value: ${JSON.stringify(testData, null, 2)}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-storage-get")?.addEventListener("click", async () => {
  clear();
  log("=== storage.getItem() ===");
  try {
    const data = await storage.getItem("test_key");
    if (data === null) {
      log("⚠️ 键 test_key 不存在，请先点击「设置存储」");
    } else {
      log("✅ 读取到数据:");
      log(JSON.stringify(data, null, 2));
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-storage-keys")?.addEventListener("click", async () => {
  clear();
  log("=== storage.keys() ===");
  try {
    const keys = await storage.keys();
    log(`存储中共有 ${keys.length} 个键:`);
    keys.forEach((key, i) => log(`  [${i}] ${key}`));
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-storage-has")?.addEventListener("click", async () => {
  clear();
  log("=== storage.has() ===");
  try {
    const exists = await storage.has("test_key");
    log(`键 "test_key" 是否存在: ${exists}`);
    if (!exists) {
      log("提示: 请先点击「设置存储」");
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-storage-remove")?.addEventListener("click", async () => {
  clear();
  log("=== storage.removeItem() ===");
  try {
    const existsBefore = await storage.has("test_key");
    log(`删除前 "test_key" 是否存在: ${existsBefore}`);
    if (existsBefore) {
      await storage.removeItem("test_key");
      const existsAfter = await storage.has("test_key");
      log(`✅ 已删除 "test_key"`);
      log(`删除后 "test_key" 是否存在: ${existsAfter}`);
    } else {
      log("⚠️ 键不存在，无需删除");
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-storage-clear")?.addEventListener("click", async () => {
  clear();
  log("=== storage.clear() ===");
  try {
    const keysBefore = await storage.keys();
    log(`清空前共有 ${keysBefore.length} 个键`);
    await storage.clear();
    const keysAfter = await storage.keys();
    log(`✅ 已清空所有存储`);
    log(`清空后共有 ${keysAfter.length} 个键`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});