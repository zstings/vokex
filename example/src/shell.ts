import { shell, fs, app } from "vokex.app";

import { log, clear } from './utils'

document.getElementById("btn-shell-openexternal")?.addEventListener("click", async () => {
  clear();
  log("=== shell.openExternal 测试 ===");
  log("正在用默认浏览器打开 https://github.com...");
  try {
    await shell.openExternal("https://github.com");
    log("✅ 已请求打开链接");
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-shell-openpath")?.addEventListener("click", async () => {
  clear();
  log("=== shell.openPath 测试 ===");
  try {
    const cwd = await app.getPath("cwd");
    log(`正在用文件管理器打开: ${cwd}`);
    await shell.openPath(cwd);
    log("✅ 已请求打开目录");
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-shell-exec-dir")?.addEventListener("click", async () => {
  clear();
  log("=== shell.exec 测试: dir ===");
  try {
    const result = await shell.exec("cmd", ["/C", "dir"], { cwd: "." });
    log(`退出码: ${result.code}`);
    log(`成功: ${result.success}`);
    log("\n输出内容:\n---");
    log(result.stdout);
    log("---");
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-shell-exec-echo")?.addEventListener("click", async () => {
  clear();
  log("=== shell.exec 测试: echo ===");
  try {
    const message = "Hello from Vokex shell API!";
    const result = await shell.exec("cmd", ["/C", "echo", message]);
    log(`退出码: ${result.code}`);
    log(`成功: ${result.success}`);
    log(`输出: ${result.stdout.trim()}`);
    if (result.stderr) {
      log(`错误: ${result.stderr}`);
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-shell-spawn")?.addEventListener("click", async () => {
  clear();
  log("=== shell.spawn 测试: 流式输出 ===");
  log("启动 ping 命令，实时输出结果...\n");

  try {
    const child = await shell.spawn("ping", ["-n", "5", "127.0.0.1"], { cwd: "." });

    log(`进程已启动，PID: ${child.pid}\n`);

    // 监听 stdout 输出
    const unSubStdout = child.onStdout((data) => {
      log(`[stdout] ${data}`);
    });

    // 监听 stderr 输出
    const unSubStderr = child.onStderr((data) => {
      log(`[stderr] ${data}`);
    });

    // 监听进程退出
    child.onExit((code) => {
      log(`\n✅ 进程已退出，退出码: ${code}`);
      unSubStdout();
      unSubStderr();
    });
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-shell-spawn-kill")?.addEventListener("click", async () => {
  clear();
  log("=== shell.spawn + kill 测试 ===");
  log("启动长时间运行的进程，5秒后自动停止...\n");

  try {
    // Windows 上使用 timeout 模拟长时间运行
    const child = await shell.spawn("ping", ["-n", "100", "127.0.0.1"], { cwd: "." });

    log(`进程已启动，PID: ${child.pid}\n`);

    const unSubStdout = child.onStdout((data) => {
      log(`[stdout] ${data}`);
    });

    child.onExit((code) => {
      log(`\n✅ 进程已被杀死，退出码: ${code}`);
      unSubStdout();
    });

    // 5秒后杀死进程
    setTimeout(async () => {
      log("\n⏰ 5秒已到，正在杀死进程...");
      await child.kill();
      log("✅ 进程已终止");
    }, 5000);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-shell-trash")?.addEventListener("click", async () => {
  clear();
  log("=== shell.trashItem 测试 ===");

  const testFile = "trash_test.txt";

  try {
    log(`创建测试文件: ${testFile}`);
    await fs.writeFile(testFile, "这个文件会被移到回收站\nCreated by Vokex shell.trashItem demo");

    const exists = await fs.exists(testFile);
    log(`文件创建成功，存在: ${exists}`);

    log(`\n正在将文件移到回收站: ${testFile}`);
    await shell.trashItem(testFile);
    log("✅ 文件已移到回收站");

    const existsAfter = await fs.exists(testFile);
    log(`移动后文件存在: ${existsAfter}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});
