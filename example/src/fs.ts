import { app, fs } from "vokex.app";

import { log, clear } from './utils'

document.getElementById("btn-fs-demo")?.addEventListener("click", async () => {
  clear();
  log("=== 文件系统完整演示 ===");

  try {
    const appPath = await app.getAppPath();
    const testDir = `${appPath}\\test_demo`;
    const testFile = `${testDir}\\test.txt`;
    const copyFile = `${testDir}\\test_copy.txt`;
    log(`1. 检查目录是否存在: ${testDir}`);
    const dirExists = await fs.exists(testDir);
    if (!dirExists) {
      log(`2. 创建目录: ${testDir}`);
      await fs.mkdir(testDir);
    } else {
      log(`2. 目录已存在: ${testDir}`);
    }

    log(`3. 写入文件: ${testFile}`);
    const content = `这是一个测试文件\n创建时间: ${new Date().toString()}\n来自 Vokex fs API`;
    await fs.writeFile(testFile, content);

    log(`4. 读取文件内容:`);
    const readContent = await fs.readFile(testFile, { encoding: 'utf8' });
    log(`---\n${readContent}\n---`);

    log(`5. 复制文件到: ${copyFile}`);
    await fs.copyFile(testFile, copyFile);

    log(`6. 读取目录内容: ${testDir}`);
    const entries = await fs.readdir(testDir, { withFileTypes: true }) as import("vokex.app").Dirent[];
    log(`目录包含 ${entries.length} 个条目:`);
    entries.forEach(entry => {
      log(`  ${entry.isDir ? "📁" : "📄"} ${entry.name}`);
    });

    log(`7. 获取文件信息: ${testFile}`);
    const stat = await fs.stat(testFile);
    log(`  是否文件: ${stat.isFile}`);
    log(`  是否目录: ${stat.isDir}`);
    log(`  文件大小: ${stat.size} 字节`);
    log(`  修改时间: ${new Date(stat.mtimeMs).toLocaleString()}`);

    log("\n✅ 演示完成！所有操作成功");
    log(`   测试文件位置: ${testFile}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-fs-read")?.addEventListener("click", async () => {
  clear();
  log("=== 读取文件测试 ===");
  log("尝试读取 test.txt...");

  try {
    const appPath = await app.getAppPath();
    const content = await fs.readFile(`${appPath}\\test_demo\\test.txt`, { encoding: 'utf8' });
    log(`文件内容 (前 300 字符):\n---\n${content.slice(0, 300)}...\n---`);
    log(`文件总长度: ${content.length} 字符`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
    log("提示: 请确保在正确的工作目录运行");
  }
});

document.getElementById("btn-fs-write")?.addEventListener("click", async () => {
  clear();
  log("=== 写入文件测试 ===");
  const appPath = await app.getAppPath();
  const fileName = `${appPath}/test_demo/test_${Date.now()}.txt`;
  const content = `Hello from Vokex!\nTimestamp: ${Date.now()}\n这是通过 fs.writeFile 写入的文件。`;

  try {
    await fs.writeFile(fileName, content);
    log(`✅ 文件已写入: ${fileName}`);
    log(`文件内容:\n---\n${content}\n---`);

    const exists = await fs.exists(fileName);
    log(`文件存在: ${exists}`);

    const stat = await fs.stat(fileName);
    log(`文件大小: ${stat.size} 字节`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-fs-readdir")?.addEventListener("click", async () => {
  clear();
  log("=== 读取目录测试 ===");
  log("当前目录内容:");

  try {
    const appPath = await app.getAppPath();
    const entries = await fs.readdir(appPath, { withFileTypes: true }) as import("vokex.app").Dirent[];
    entries.forEach(entry => {
      const icon = entry.isDir ? "📁" : "📄";
      log(`  ${icon} ${entry.name}`);
    });
    log(`\n总共 ${entries.length} 个条目`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-fs-stat")?.addEventListener("click", async () => {
  clear();
  log("=== 文件信息测试 ===");

  try {
    const appPath = await app.getAppPath();
    const stat = await fs.stat(appPath + "/test_demo/test.txt");
    log(`test_demo/test.txt:`);
    log(`  isFile: ${stat.isFile}`);
    log(`  isDir: ${stat.isDir}`);
    log(`  size: ${stat.size} bytes`);
    log(`  mtimeMs: ${new Date(stat.mtimeMs).toLocaleString()}`);
    log(`  atimeMs: ${new Date(stat.atimeMs).toLocaleString()}`);
    log(`  birthtimeMs: ${new Date(stat.birthtimeMs).toLocaleString()}`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-fs-copy")?.addEventListener("click", async () => {
  clear();
  log("=== 复制文件测试 ===");

  const appPath = await app.getAppPath();
  const src = appPath + "/test_demo/test.txt";
  const dest = appPath + "/test_demo/test.txt.copy";

  try {
    await fs.copyFile(src, dest);
    log(`✅ 已复制 ${src} -> ${dest}`);
    const exists = await fs.exists(dest);
    log(`目标文件存在: ${exists}`);
    const stat = await fs.stat(dest);
    log(`目标文件大小: ${stat.size} bytes`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-fs-delete")?.addEventListener("click", async () => {
  clear();
  log("=== 删除文件测试 ===");

  const appPath = await app.getAppPath();
  const fileName = appPath + "/test_demo/test.txt";

  try {
    const existsBefore = await fs.exists(fileName);
    log(`删除前文件存在: ${existsBefore}`);

    if (existsBefore) {
      await fs.rm(fileName);
      log(`✅ 已删除文件: ${fileName}`);

      const existsAfter = await fs.exists(fileName);
      log(`删除后文件存在: ${existsAfter}`);
    } else {
      log(`⚠️ 文件不存在: ${fileName}`);
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-fs-rmdir")?.addEventListener("click", async () => {
  clear();
  log("=== 删除目录测试 ===");

  const appPath = await app.getAppPath();
  const dirName = appPath + "/test_demo";

  try {
    const existsBefore = await fs.exists(dirName);
    log(`删除前目录存在: ${existsBefore}`);

    if (existsBefore) {
      await fs.rm(dirName, { recursive: true });
      log(`✅ 已删除目录: ${dirName} (递归删除所有内容)`);

      const existsAfter = await fs.exists(dirName);
      log(`删除后目录存在: ${existsAfter}`);
    } else {
      log(`⚠️ 目录不存在: ${dirName}`);
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-fs-read-binary")?.addEventListener("click", async () => {
  clear();
  log("=== 读取二进制文件测试 ===");
  log("尝试读取 test.txt...");

  try {
    const appPath = await app.getAppPath();
    // 无 encoding 时直接返回 Uint8Array，无需手动转换
    const bytes = await fs.readFile(appPath + "/test_demo/test.txt");
    log(`读取成功，类型: ${bytes instanceof Uint8Array ? 'Uint8Array' : typeof bytes}`);
    log(`字节长度: ${bytes.length}`);
    log(`前 10 字节: [${Array.from(bytes.slice(0, 10)).join(', ')}]`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
    log("提示: 请确保在正确的工作目录运行");
  }
});

document.getElementById("btn-fs-append")?.addEventListener("click", async () => {
  clear();
  log("=== 追加内容测试 ===");

  const appPath = await app.getAppPath();
  const fileName = appPath + "/test_demo/test.txt";
  const appendContent = `\n[追加] 这是追加的一行\n时间戳: ${Date.now()}\n`;

  try {
    const exists = await fs.exists(fileName);
    if (!exists) {
      log(`⚠️ 文件不存在，先创建文件: ${fileName}`);
      await fs.writeFile(fileName, "初始内容\n");
    }

    const statBefore = await fs.stat(fileName);
    log(`追加前大小: ${statBefore.size} 字节`);

    await fs.writeFile(fileName, appendContent, { flag: 'a' });
    log(`✅ 已追加内容到: ${fileName}`);

    const statAfter = await fs.stat(fileName);
    log(`追加后大小: ${statAfter.size} 字节`);
    log(`增加了 ${statAfter.size - statBefore.size} 字节`);

    const fullContent = await fs.readFile(fileName, { encoding: 'utf8' });
    log(`\n完整内容:\n---\n${fullContent}\n---`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-fs-move")?.addEventListener("click", async () => {
  clear();
  log("=== 移动/重命名文件测试 ===");

  const appPath = await app.getAppPath();
  const src = appPath + "/test_demo/test.txt";
  const dest = appPath + "/test_demo/test_renamed.txt";

  try {
    const srcExists = await fs.exists(src);
    if (!srcExists) {
      log(`⚠️ 源文件不存在: ${src}`);
      log("先创建源文件...");
      await fs.mkdir("test_demo", { recursive: true });
      await fs.writeFile(src, "这是要被重命名的文件\n");
    }

    const destExistsBefore = await fs.exists(dest);
    log(`目标文件已存在: ${destExistsBefore}`);

    await fs.rename(src, dest);
    log(`✅ 已移动/重命名: ${src} -> ${dest}`);

    const srcExistsAfter = await fs.exists(src);
    const destExistsAfter = await fs.exists(dest);
    log(`源文件现在存在: ${srcExistsAfter}`);
    log(`目标文件现在存在: ${destExistsAfter}`);

    if (destExistsAfter) {
      const content = await fs.readFile(dest, { encoding: 'utf8' });
      log(`\n目标文件内容:\n---\n${content}\n---`);
    }
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});

document.getElementById("btn-fs-glob")?.addEventListener("click", async () => {
  clear();
  log("=== Glob 文件搜索测试 (glob crate) ===");

  const appPath = await app.getAppPath();
  const testDir = `${appPath}\\test_demo`;

  try {
    const dirExists = await fs.exists(testDir);
    if (!dirExists) {
      log(`⚠️ 测试目录不存在，先创建...`);
      await fs.mkdir(testDir, { recursive: true });
      for (let i = 1; i <= 3; i++) {
        await fs.writeFile(`${testDir}\\file${i}.txt`, `内容 ${i}`);
      }
      await fs.mkdir(`${testDir}\\subdir`);
      await fs.writeFile(`${testDir}\\subdir\\nested.js`, `nested`);
      await fs.writeFile(`${testDir}\\subdir\\data.json`, `{"test": true}`);
      await fs.writeFile(`${testDir}\\.hidden.txt`, `隐藏文件`);
    }

    log(`1. 搜索 *.txt (单通配符):`);
    const txtFiles = await fs.glob({ pattern: "*.txt", cwd: testDir });
    txtFiles.forEach(f => log(`   📄 ${f}`));

    log(`\n2. 搜索 **/*.js (递归):`);
    const jsFiles = await fs.glob({ pattern: "**/*.js", cwd: testDir });
    jsFiles.forEach(f => log(`   📄 ${f}`));

    log(`\n3. 搜索 **/* (递归所有，不含隐藏文件):`);
    const allFiles = await fs.glob({ pattern: "**/*", cwd: testDir });
    log(`   找到 ${allFiles.length} 个文件/目录`);

    log(`\n4. 搜索 **/* (含隐藏文件 dot=true):`);
    const withDot = await fs.glob({ pattern: "**/*", cwd: testDir, dot: true });
    log(`   找到 ${withDot.length} 个文件/目录`);

    log(`\n5. 搜索 **/* (只返回文件 nodir=true):`);
    const onlyFiles = await fs.glob({ pattern: "**/*", cwd: testDir, nodir: true });
    log(`   找到 ${onlyFiles.length} 个文件`);

    log(`\n6. 搜索 **/* (排除 *.txt ignore):`);
    const noTxt = await fs.glob({ pattern: "**/*", cwd: testDir, ignore: ["*.txt"] });
    log(`   找到 ${noTxt.length} 个文件/目录`);

    log(`\n✅ Glob 测试完成`);
  } catch (error: any) {
    log(`❌ 错误: ${error.message}`);
  }
});



// 假设你已经通过 window.vokex.fs 暴露了相关接口
async function testGitScanSpeed() {

  // 建议从用户目录或特定项目盘符开始，避免全盘扫描权限问题
  const scanRoot = "F:/";

  console.log(`[Test] 开始扫描: ${scanRoot}`);
  const startTime = performance.now();

  try {
    // 修改点：只传一个对象，并将 pattern 写入其中
    const projects = await fs.glob({
      pattern: "**/.git", // 这里的 pattern 必须在对象内
      cwd: scanRoot,
      ignore: [
        "**/node_modules/**",
        "**/target/**",
        "**/dist/**",
        "**/.git/**",      // 必须加上这个，防止进入 .git 文件夹内部扫描数千个 object 文件
        "**/build/**",
        "**/.cache/**",    // 很多工具（如 Rust, PNPM）的缓存
        "**/AppData/**",   // 如果你扫描的是用户目录，这里面有海量小文件
        "**/Library/**",   // macOS 环境同理
        "**/System32/**",  // 防止扫描到系统盘核心目录
        "**/.idea/**",     // JetBrains 配置文件
        "**/.vscode/**"    // VS Code 配置文件
      ],
      dot: true // .git 是隐藏目录，建议开启 dot 选项
    });

    const duration = performance.now() - startTime;

    console.log("---------------------------------------");
    console.log(`扫描完成！耗时: ${(duration / 1000).toFixed(2)} 秒`);
    console.log(`找到项目数: ${projects.length}`);

    // 映射出项目根目录
    const projectPaths = projects.map(p => p.replace(/[\\/].git$/, ''));
    console.log("项目路径列表:", projectPaths);

  } catch (err) {
    // 如果还是报错，说明 Rust 后端的 GlobOptions 结构体定义非常严格
    console.error("扫描失败，请检查参数格式:", err);
  }
}

// 流式 glob 测试 - 边扫描边返回结果
async function testGitScanStream() {
  const scanRoot = "F:/";

  console.log(`[Stream] 开始流式扫描: ${scanRoot}`);
  const startTime = performance.now();
  let matchCount = 0;

  try {
    const streamId = await fs.globStream(
      {
        pattern: "**/.git",
        cwd: scanRoot,
        ignore: [
          "**/node_modules/**",
          "**/target/**",
          "**/dist/**",
          "**/.git/**",      // 必须加上这个，防止进入 .git 文件夹内部扫描数千个 object 文件
          "**/build/**",
          "**/.cache/**",    // 很多工具（如 Rust, PNPM）的缓存
          "**/AppData/**",   // 如果你扫描的是用户目录，这里面有海量小文件
          "**/Library/**",   // macOS 环境同理
          "**/System32/**",  // 防止扫描到系统盘核心目录
          "**/.idea/**",     // JetBrains 配置文件
          "**/.vscode/**"    // VS Code 配置文件
        ],
        dot: true
      },
      {
        onMatch: (path, index) => {
          matchCount++;
          // 实时显示找到的项目
          const projectPath = path.replace(/[\\/].git$/, '');
          console.log(`[${index}] ${projectPath}`);
        },
        onDone: (total) => {
          const duration = performance.now() - startTime;
          console.log("---------------------------------------");
          console.log(`流式扫描完成！耗时: ${(duration / 1000).toFixed(2)} 秒`);
          console.log(`找到项目数: ${total}`);
        },
        onError: (error) => {
          console.error("扫描错误:", error);
        }
      }
    );

    console.log(`Stream ID: ${streamId}`);

  } catch (err) {
    console.error("扫描失败:", err);
  }
}

window.testGitScanSpeed = testGitScanSpeed;
window.testGitScanStream = testGitScanStream;
