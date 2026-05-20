
import { dialog, http } from "vokex.app";
import { clear, log } from "./utils";

// ─── http.get ─────────────────────────────────────────────

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

// ─── http.post ────────────────────────────────────────────

document.getElementById("btn-http-post")?.addEventListener("click", async () => {
  clear();
  log("=== http.post() ===");
  log("POST: https://jsonplaceholder.typicode.com/posts");
  try {
    // 直接传入纯对象，自动 JSON.stringify + Content-Type: application/json
    const data = { title: 'foo', body: 'bar', userId: 1 };
    log(`请求体: ${JSON.stringify(data, null, 2)}`);
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

// ─── http.put ─────────────────────────────────────────────

document.getElementById("btn-http-put")?.addEventListener("click", async () => {
  clear();
  log("=== http.put() ===");
  log("PUT: https://jsonplaceholder.typicode.com/posts/1");
  try {
    const data = { id: 1, title: 'updated', body: 'new content', userId: 1 };
    log(`请求体: ${JSON.stringify(data, null, 2)}`);
    const response = await http.put("https://jsonplaceholder.typicode.com/posts/1", data);
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

// ─── http.delete ──────────────────────────────────────────

document.getElementById("btn-http-delete")?.addEventListener("click", async () => {
  clear();
  log("=== http.delete() ===");
  log("DELETE: https://jsonplaceholder.typicode.com/posts/1");
  try {
    const response = await http.delete("https://jsonplaceholder.typicode.com/posts/1");
    log(`状态码: ${response.status}`);
    log(`状态文本: ${response.statusText}`);
    log(`成功: ${response.ok}`);
    log(`\n响应体 (json):`);
    const json = await response.json();
    log(JSON.stringify(json, null, 2));
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

// ─── http.fetch（通用请求）─────────────────────────────────

document.getElementById("btn-http-request")?.addEventListener("click", async () => {
  clear();
  log("=== http.fetch() PATCH ===");
  log("PATCH: https://jsonplaceholder.typicode.com/posts/1");
  try {
    const response = await http.fetch("https://jsonplaceholder.typicode.com/posts/1", {
      method: "PATCH",
      headers: {
        "X-Custom-Header": "vokex-demo",
        "Accept": "application/json",
      },
      body: { title: "patched" },
    });
    log(`状态码: ${response.status}`);
    log(`状态文本: ${response.statusText}`);
    log(`成功: ${response.ok}`);
    log(`\n响应头:`);
    response.headers.forEach((value, key) => {
      log(`  ${key}: ${value}`);
    });
    log(`\n响应体 (json):`);
    const json = await response.json();
    log(JSON.stringify(json, null, 2));
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

// ─── 自定义请求头 ─────────────────────────────────────────

document.getElementById("btn-http-headers")?.addEventListener("click", async () => {
  clear();
  log("=== 自定义请求头 ===");
  log("GET: https://httpbin.org/headers");
  try {
    const response = await http.get("https://httpbin.org/headers", {
      headers: {
        "X-Custom-Header": "Hello-Vokex",
        "X-Request-Id": "12345",
        "Accept-Language": "zh-CN",
      },
    });
    log(`状态码: ${response.status}`);
    log(`成功: ${response.ok}`);
    log(`\n服务端收到的请求头:`);
    const json = await response.json();
    log(JSON.stringify(json, null, 2));
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

// ─── 超时测试 ─────────────────────────────────────────────

document.getElementById("btn-http-timeout")?.addEventListener("click", async () => {
  clear();
  log("=== 超时测试 ===");
  log("请求一个 5 秒延迟的接口，但超时设为 2 秒");
  // try {
  //   const response = await http.get("https://httpbin.org/delay/5", {
  //     timeout: 2,
  //   });
  // } catch (error: any) {
  //   log(`预期的超时错误: ${error.message}`);
  // }
  http.get("https://httpbin.org/delay/5", {
    timeout: 2,
  }).catch((error: any) => {
    log(`预期的超时错误: ${error.message}`);
  });
  dialog.info({
    title: 'timeout test',
    message: '立即弹出了，没有等待 2 秒',
  });
});

// ─── 错误处理 ─────────────────────────────────────────────

document.getElementById("btn-http-error")?.addEventListener("click", async () => {
  clear();
  log("=== 错误处理 ===");
  try {
    log("--- 404 测试 ---");
    const resp404 = await http.get("https://httpbin.org/status/404");
    log(`状态码: ${resp404.status}`);
    log(`ok: ${resp404.ok}`);
    log(`状态文本: ${resp404.statusText}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }

  try {
    log("\n--- 500 测试 ---");
    const resp500 = await http.get("https://httpbin.org/status/500");
    log(`状态码: ${resp500.status}`);
    log(`ok: ${resp500.ok}`);
    log(`状态文本: ${resp500.statusText}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }

  try {
    log("\n--- 连接失败测试 ---");
    await http.get("http://127.0.0.1:1/");
  } catch (error: any) {
    log(`预期的连接错误: ${error.message}`);
  }
});

// ─── http.fetch（标准 fetch 兼容）──────────────────────────

document.getElementById("btn-vokex-fetch")?.addEventListener("click", async () => {
  clear();
  log("=== http.fetch() ===");
  log("标准 fetch 兼容模式");
  log("GET: https://jsonplaceholder.typicode.com/posts/1");
  try {
    const response = await http.fetch("https://jsonplaceholder.typicode.com/posts/1");
    log(`类型: ${response.constructor.name}`);
    log(`状态码: ${response.status}`);
    log(`状态文本: ${response.statusText}`);
    log(`ok: ${response.ok}`);
    log(`Content-Type: ${response.headers.get('content-type')}`);
    log(`\n响应体 (text):`);
    const text = await response.text();
    log(text.substring(0, 500));
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

// ─── http.fetch POST ──────────────────────────────────────

document.getElementById("btn-vokex-fetch-post")?.addEventListener("click", async () => {
  clear();
  log("=== http.fetch() POST ===");
  log("POST: https://jsonplaceholder.typicode.com/posts");
  try {
    const response = await http.fetch("https://jsonplaceholder.typicode.com/posts", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ title: "http.fetch", body: "标准 fetch 兼容", userId: 1 }),
    });
    log(`状态码: ${response.status}`);
    log(`ok: ${response.ok}`);
    log(`\n响应体 (json):`);
    const json = await response.json();
    log(JSON.stringify(json, null, 2));
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

// ─── SSE 流式请求 ─────────────────────────────────────────

document.getElementById("btn-http-sse")?.addEventListener("click", async () => {
  clear();
  log("=== SSE 流式请求 ===");
  log("使用 http.fetch + stream: true");
  log("请求: https://httpbin.org/stream/5");
  log("（每行实时推送，模拟大模型打字机效果）\n");

  try {
    const response = await http.fetch("https://httpbin.org/stream/5", {
      stream: true,
    });

    log(`状态码: ${response.status}`);
    log(`ok: ${response.ok}`);
    log(`Content-Type: ${response.headers.get('content-type')}`);
    log(`\n开始接收流式数据...\n`);

    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let chunkCount = 0;

    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        log(`\n✅ 流结束，共接收 ${chunkCount} 个数据块`);
        break;
      }
      chunkCount++;
      const text = decoder.decode(value, { stream: true });
      log(`[chunk ${chunkCount}] ${text.trimEnd()}`);
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

// ─── SSE 流式 + 大模型模拟 ────────────────────────────────

document.getElementById("btn-http-sse-llm")?.addEventListener("click", async () => {
  clear();
  log("=== SSE 大模型流式输出模拟 ===");
  log("模拟场景：调用大模型 API，逐 token 输出");
  log("请求: https://httpbin.org/stream/20\n");

  try {
    const response = await http.fetch("https://httpbin.org/stream/20", {
      stream: true,
      timeout: 30,
    });

    log(`状态码: ${response.status}`);
    log(`\n逐行输出:\n`);

    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let lineBuffer = "";

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      lineBuffer += decoder.decode(value, { stream: true });
      const lines = lineBuffer.split("\n");
      lineBuffer = lines.pop() || "";

      for (const line of lines) {
        if (line.trim()) {
          log(line.trim());
        }
      }
    }

    if (lineBuffer.trim()) {
      log(lineBuffer.trim());
    }

    log("\n✅ 流式输出完成");
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

// ─── Headers 不区分大小写 ────────────────────────────────

document.getElementById("btn-http-headers-class")?.addEventListener("click", async () => {
  clear();
  log("=== Headers 不区分大小写 ===");
  try {
    const response = await http.get("https://jsonplaceholder.typicode.com/todos/1");

    log("以下写法都等价:");
    log(`  headers.get('Content-Type')     = ${response.headers.get('Content-Type')}`);
    log(`  headers.get('content-type')     = ${response.headers.get('content-type')}`);
    log(`  headers.get('CONTENT-TYPE')     = ${response.headers.get('CONTENT-TYPE')}`);
    log(`  headers.has('content-type')     = ${response.headers.has('content-type')}`);
    log(`  headers.has('not-exist')        = ${response.headers.has('not-exist')}`);

    log(`\n遍历所有响应头:`);
    response.headers.forEach((value, key) => {
      log(`  ${key}: ${value}`);
    });
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

// ─── Response.clone 演示 ─────────────────────────────────

document.getElementById("btn-http-clone")?.addEventListener("click", async () => {
  clear();
  log("=== Response.clone() ===");
  log("克隆必须在读取 body 之前调用\n");
  try {
    const response = await http.get("https://jsonplaceholder.typicode.com/todos/1");

    // 先克隆，再分别读取
    const cloned = response.clone();
    const text1 = await response.text();
    const text2 = await cloned.text();

    log(`原始 (text):`);
    log(text1.substring(0, 100) + "...");
    log(`bodyUsed: ${response.bodyUsed}`);

    log(`\n克隆 (text):`);
    log(text2.substring(0, 100) + "...");
    log(`bodyUsed: ${cloned.bodyUsed}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});
