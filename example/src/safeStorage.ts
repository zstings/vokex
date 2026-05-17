import { log, clear } from './utils'
import { safeStorage } from "vokex.app";

document.getElementById("btn-safe-storage-set")?.addEventListener("click", async () => {
  clear();
  log("=== safeStorage.setItem() ===");
  try {
    const sensitiveData = {
      provider: "openai",
      apiKey: "sk-xxxxxxxxxxxxxxxxxxxx",
      model: "gpt-4",
      timestamp: Date.now(),
    };
    await safeStorage.setItem("api_config", sensitiveData);
    log("✅ 已加密存储敏感数据:");
    log(`  key: api_config`);
    log(`  value: ${JSON.stringify(sensitiveData, null, 2)}`);
    log("");
    log("数据已通过 AES-256-GCM 加密存储在本地文件中");
    log("密钥由系统安全区（Windows 凭据管理器/macOS 钥匙串）保护");
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-safe-storage-get")?.addEventListener("click", async () => {
  clear();
  log("=== safeStorage.getItem() ===");
  try {
    const data = await safeStorage.getItem("api_config");
    if (data === null) {
      log("⚠️ 键 api_config 不存在，请先点击「加密存储」");
    } else {
      log("✅ 解密读取到数据:");
      log(JSON.stringify(data, null, 2));
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-safe-storage-keys")?.addEventListener("click", async () => {
  clear();
  log("=== safeStorage.keys() ===");
  try {
    const keys = await safeStorage.keys();
    log(`安全存储中共有 ${keys.length} 个键:`);
    keys.forEach((key, i) => log(`  [${i}] ${key}`));
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-safe-storage-has")?.addEventListener("click", async () => {
  clear();
  log("=== safeStorage.has() ===");
  try {
    const exists = await safeStorage.has("api_config");
    log(`键 "api_config" 是否存在: ${exists}`);
    if (!exists) {
      log("提示: 请先点击「加密存储」");
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-safe-storage-remove")?.addEventListener("click", async () => {
  clear();
  log("=== safeStorage.removeItem() ===");
  try {
    const existsBefore = await safeStorage.has("api_config");
    log(`删除前 "api_config" 是否存在: ${existsBefore}`);
    if (existsBefore) {
      await safeStorage.removeItem("api_config");
      const existsAfter = await safeStorage.has("api_config");
      log("✅ 已删除 api_config");
      log(`删除后 "api_config" 是否存在: ${existsAfter}`);
    } else {
      log("⚠️ 键不存在，无需删除");
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-safe-storage-clear")?.addEventListener("click", async () => {
  clear();
  log("=== safeStorage.clear() ===");
  try {
    const keysBefore = await safeStorage.keys();
    log(`清空前共有 ${keysBefore.length} 个键`);
    await safeStorage.clear();
    const keysAfter = await safeStorage.keys();
    log("✅ 已清空所有安全存储");
    log(`清空后共有 ${keysAfter.length} 个键`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});
