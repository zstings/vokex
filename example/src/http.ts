
import { http } from "vokex.app";
import { clear, log } from "./utils";

document.getElementById("btn-http-get")?.addEventListener("click", async () => {
  clear();
  log("=== http.get() ===");
  log("请求: https://jsonplaceholder.typicode.com/todos/1");
  try {
    const response = await http.get("https://jsonplaceholder.typicode.com/todos/1");
    log(`状态码: ${response.status}`);
    log(`状态文本: ${response.statusText}`);
    log(`成功: ${response.ok}`);
    log(`Content-Type: ${response.headers.get('content-type')}`);
    log(`\n响应体 (text):`);
    log(await response.text());
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-http-post")?.addEventListener("click", async () => {
  clear();
  log("=== http.post() ===");
  log("POST: https://jsonplaceholder.typicode.com/posts");
  try {
    // 直接传入纯对象，自动 JSON.stringify + Content-Type: application/json
    const data = { title: 'foo', body: 'bar', userId: 1 };
    log(`数据: ${JSON.stringify(data, null, 2)}`);
    const response = await http.post("https://jsonplaceholder.typicode.com/posts", data);
    log(`\n状态码: ${response.status}`);
    log(`状态文本: ${response.statusText}`);
    log(`成功: ${response.ok}`);
    log(`\n响应体 (json):`);
    const json = await response.json();
    log(JSON.stringify(json, null, 2));
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});
