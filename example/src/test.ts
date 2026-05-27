
import { dialog, http, storage, safeStorage } from "vokex.app";
import { clear, log } from "./utils";

// ─── 竞态测试 ─────────────────────────────────────────────

document.getElementById("btn-test-http-request")?.addEventListener("click", async () => {
  clear();
  log("=== 网络请求+弹窗 ===");
  log("请求一个 5 秒延迟的接口");
  http.get("https://httpbin.org/delay/10", { timeout: 5 }).then(res => {
    log('立即打印了接口响应:' + JSON.stringify(res));
  }).catch((error: any) => {
    log(`预期的超时错误: ${error.message}`);
  });
  dialog.info({
    title: 'timeout test',
    message: '立即弹出了，没有等待接口响应',
  });
});

document.getElementById("btn-test-http-storage")?.addEventListener("click", async () => {
  clear();
  log("=== 网络请求+存储 ===");
  log("请求一个 5 秒延迟的接口");
  http.get("https://httpbin.org/delay/5").then(() => {
    log('5秒后打印了接口响应');
  });
  storage.setItem("timeoutTest", {
    title: 'timeout test',
    message: '立即存储了，没有等待接口响应',
  }).then(async () => {
    log("立即存储了，没有等待接口响应:" + new Date().toLocaleTimeString());
    const data = await storage.getItem("timeoutTest");
    log('立即存储的数据:' + JSON.stringify(data));
  });
  safeStorage.setItem("timeoutTestSafe", {
    title: 'timeout test safe',
    message: '立即存储了，没有等待接口响应',
  }).then(async () => {
    log("立即存储了，没有等待接口响应:" + new Date().toLocaleTimeString());
    const data = await safeStorage.getItem("timeoutTestSafe");
    log('立即存储的数据:' + JSON.stringify(data));
  });
  log("能立即打印:" + new Date().toLocaleTimeString());
});
