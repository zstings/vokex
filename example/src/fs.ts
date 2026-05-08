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
      await fs.createDir(testDir);
    } else {
      log(`2. 目录已存在: ${testDir}`);
    }

    log(`3. 写入文件: ${testFile}`);
    const content = `这是一个测试文件\n创建时间: ${new Date().toString()}\n来自 Vokex fs API`;
    await fs.writeFile(testFile, content);

    log(`4. 读取文件内容:`);
    const readContent = await fs.readFile(testFile);
    log(`---\n${readContent}\n---`);

    log(`5. 复制文件到: ${copyFile}`);
    await fs.copyFile(testFile, copyFile);

    log(`6. 读取目录内容: ${testDir}`);
    const entries = await fs.readDir(testDir);
    log(`目录包含 ${entries.length} 个条目:`);
    entries.forEach(entry => {
      log(`  ${entry.isDir ? "📁" : "📄"} ${entry.name}`);
    });

    log(`7. 获取文件信息: ${testFile}`);
    const stat = await fs.stat(testFile);
    log(`  是否文件: ${stat.isFile}`);
    log(`  是否目录: ${stat.isDir}`);
    log(`  文件大小: ${stat.size} 字节`);

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
    const content = await fs.readFile(`${appPath}\\test_demo\\test.txt`);
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
    const entries = await fs.readDir(appPath);
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
    log(`  modified: ${stat.modified} seconds ago`);
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
      await fs.deleteFile(fileName);
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
      await fs.removeDir(dirName);
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
    const data = await fs.readFileBinary(appPath + "/test_demo/test.txt");
    const bytes = Uint8Array.from(atob(data), c => c.charCodeAt(0));
    log(`读取成功，字节长度: ${bytes.length}`);
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

    await fs.appendFile(fileName, appendContent);
    log(`✅ 已追加内容到: ${fileName}`);

    const statAfter = await fs.stat(fileName);
    log(`追加后大小: ${statAfter.size} 字节`);
    log(`增加了 ${statAfter.size - statBefore.size} 字节`);

    const fullContent = await fs.readFile(fileName);
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
      await fs.createDir("test_demo");
      await fs.writeFile(src, "这是要被重命名的文件\n");
    }

    const destExistsBefore = await fs.exists(dest);
    log(`目标文件已存在: ${destExistsBefore}`);

    await fs.moveFile(src, dest);
    log(`✅ 已移动/重命名: ${src} -> ${dest}`);

    const srcExistsAfter = await fs.exists(src);
    const destExistsAfter = await fs.exists(dest);
    log(`源文件现在存在: ${srcExistsAfter}`);
    log(`目标文件现在存在: ${destExistsAfter}`);

    if (destExistsAfter) {
      const content = await fs.readFile(dest);
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
      await fs.createDir(testDir);
      for (let i = 1; i <= 3; i++) {
        await fs.writeFile(`${testDir}\\file${i}.txt`, `内容 ${i}`);
      }
      await fs.createDir(`${testDir}\\subdir`);
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