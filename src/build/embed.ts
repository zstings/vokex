/**
 * vokex 框架 - 资源嵌入构建器
 *
 * 将前端构建产物嵌入到预编译壳二进制文件中
 */

import { readFile, readdir, writeFile, mkdir } from "fs/promises";
import { existsSync } from "fs";
import { resolve, relative } from "path";
import { createHash } from "crypto";
import { createDeflate } from "zlib";
import { injectIcon } from "./icon-embed.js";

/** 魔数：VOKEX */
const MAGIC = Buffer.from("VOKEX");
/** 索引长度字段大小（4字节） */
const INDEX_LENGTH_SIZE = 4;
/** 偏移量字段大小（8字节） */
const OFFSET_SIZE = 8;

/** 嵌入结果 */
export interface BuildResult {
  /** 输出文件路径 */
  outputPath: string;
  /** 原始资源大小 */
  originalSize: number;
  /** 压缩后大小 */
  compressedSize: number;
  /** 最终文件大小 */
  finalSize: number;
  /** 文件数量 */
  fileCount: number;
  /** 压缩比 */
  compressionRatio: number;
}

/** 验证结果 */
export interface ValidateResult {
  /** 是否有效 */
  valid: boolean;
  /** 应用信息 */
  info?: {
    /** 文件数量 */
    fileCount: number;
    /** 原始大小 */
    originalSize: number;
  };
}

/**
 * 扫描目录获取所有文件
 */
async function scanDir(
  dir: string,
  baseDir: string = dir
): Promise<Map<string, { size: number; data: Buffer }>> {
  const files = new Map<string, { size: number; data: Buffer }>();

  async function scan(currentDir: string) {
    const entries = await readdir(currentDir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = resolve(currentDir, entry.name);
      const relativePath = relative(baseDir, fullPath).replace(/\\/g, "/");

      if (entry.isDirectory()) {
        await scan(fullPath);
      } else if (entry.isFile()) {
        const data = await readFile(fullPath);
        files.set(relativePath, { size: data.length, data });
      }
    }
  }

  await scan(dir);
  return files;
}

/**
 * 计算 SHA256 哈希
 */
function sha256(data: Buffer): string {
  return createHash("sha256").update(data).digest("hex");
}

/**
 * 压缩数据
 */
async function compress(data: Buffer): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    const deflate = createDeflate();

    deflate.on("data", (chunk) => chunks.push(chunk));
    deflate.on("end", () => resolve(Buffer.concat(chunks)));
    deflate.on("error", reject);

    deflate.write(data);
    deflate.end();
  });
}

/**
 * 验证二进制文件
 */
export function validate(filePath: string): ValidateResult {
  try {
    const buffer = require("fs").readFileSync(filePath);
    const fileSize = buffer.length;

    if (fileSize < OFFSET_SIZE) {
      return { valid: false };
    }

    // 读取偏移量
    const offsetBuffer = buffer.subarray(fileSize - OFFSET_SIZE, fileSize);
    const offset = offsetBuffer.readBigUInt64LE();

    if (offset >= fileSize) {
      return { valid: false };
    }

    // 验证魔数
    const magicBuffer = buffer.subarray(Number(offset), Number(offset) + MAGIC.length);
    if (!magicBuffer.equals(MAGIC)) {
      return { valid: false };
    }

    // 读取索引长度
    const indexLengthBuffer = buffer.subarray(
      Number(offset) + MAGIC.length,
      Number(offset) + MAGIC.length + INDEX_LENGTH_SIZE
    );
    const indexLength = indexLengthBuffer.readUInt32LE();

    // 读取索引
    const indexStart = Number(offset) + MAGIC.length + INDEX_LENGTH_SIZE;
    const indexEnd = indexStart + indexLength;
    const indexBuffer = buffer.subarray(indexStart, indexEnd);
    const index = JSON.parse(indexBuffer.toString());

    // 计算资源大小
    let totalSize = 0;
    for (const [_, offsets] of Object.entries(index)) {
      const [start, end] = offsets as [number, number];
      totalSize += end - start;
    }

    return {
      valid: true,
      info: {
        fileCount: Object.keys(index).length,
        originalSize: totalSize,
      },
    };
  } catch {
    return { valid: false };
  }
}

/**
 * 构建嵌入文件
 */
export async function build(options: {
  /** 输入目录 */
  inputDir: string;
  /** 壳二进制路径 */
  shellPath: string;
  /** 输出路径 */
  outputPath: string;
  /** 应用图标路径（相对于输入目录的 PNG 文件） */
  iconPath?: string;
  /** 是否显示详细日志 */
  verbose?: boolean;
}): Promise<BuildResult> {
  const {
    inputDir,
    shellPath,
    outputPath,
    verbose = false,
  } = options;

  if (verbose) console.log(`[vokex] 读取壳二进制: ${shellPath}`);
  const shellBuffer = await readFile(shellPath);

  if (verbose) console.log(`[vokex] 扫描资源目录: ${inputDir}`);
  const files = await scanDir(inputDir);

  // 按文件名排序以确保一致性
  const sortedFiles = Array.from(files.entries()).sort(([a], [b]) =>
    a.localeCompare(b)
  );

  if (verbose)
    console.log(
      `[vokex] 找到 ${files.size} 个文件，总大小 ${(
        files.size /
        1024
      ).toFixed(1)} KB`
    );

  // 生成索引，特殊 key 存储应用配置
  const index: Record<string, any> = {};
  let currentOffset = 0;
  const rawChunks: Buffer[] = [];

  for (const [filePath, { size, data }] of sortedFiles) {
    index[filePath] = [currentOffset, currentOffset + size];
    currentOffset += size;
    rawChunks.push(data);
  }

  const indexJson = JSON.stringify(index);
  const indexBuffer = Buffer.from(indexJson);
  const indexLengthBuffer = Buffer.alloc(INDEX_LENGTH_SIZE);
  indexLengthBuffer.writeUInt32LE(indexBuffer.length);

  const offsetBuffer = Buffer.alloc(OFFSET_SIZE);
  offsetBuffer.writeBigUInt64LE(BigInt(shellBuffer.length));

  // 合并所有文件数据
  if (verbose) console.log(`[vokex] 压缩资源...`);
  const rawData = Buffer.concat(rawChunks);
  const compressedData = await compress(rawData);

  if (verbose) {
    console.log(`[vokex] 原始: ${(rawData.length / 1024).toFixed(1)} KB`);
    console.log(`[vokex] 压缩后: ${(compressedData.length / 1024).toFixed(1)} KB`);
    console.log(
      `[vokex] 压缩率: ${((1 - compressedData.length / rawData.length) * 100).toFixed(1)}%`
    );
  }

  // 构建资源尾部
  const tail = Buffer.concat([
    MAGIC,
    indexLengthBuffer,
    indexBuffer,
    compressedData,
    offsetBuffer,
  ]);

  // 确保输出目录存在
  const outputDir = resolve(outputPath, "..");
  if (!existsSync(outputDir)) {
    await mkdir(outputDir, { recursive: true });
  }

  // 写入输出文件
  await writeFile(outputPath, Buffer.concat([shellBuffer, tail]));

  // 注入应用图标到 PE 资源段（仅 Windows）
  if (process.platform === "win32" && options.iconPath) {
    const iconRelPath = options.iconPath.replace(/\\/g, "/");
    const iconEntry = files.get(iconRelPath);
    if (iconEntry) {
      const ok = injectIcon(outputPath, iconEntry.data);
      if (ok && verbose) {
        console.log(`[vokex]    图标: 已注入 ${iconRelPath}`);
      }
    } else if (verbose) {
      console.warn(`[vokex] 图标文件未在构建产物中找到: ${iconRelPath}`);
    }
  }

  console.log(`[vokex] ✅ 构建成功!`);
  console.log(`[vokex]    文件数: ${sortedFiles.length}`);
  console.log(
    `[vokex]    资源: ${(rawData.length / 1024).toFixed(1)} KB → ${(
      compressedData.length /
      1024
    ).toFixed(1)} KB (${((1 - compressedData.length / rawData.length) * 100).toFixed(0)}% 压缩)`
  );
  console.log(
    `[vokex]    输出: ${outputPath} (${(shellBuffer.length / 1024 / 1024).toFixed(2)} MB)`
  );

  return {
    outputPath,
    originalSize: rawData.length,
    compressedSize: compressedData.length,
    finalSize: shellBuffer.length + tail.length,
    fileCount: sortedFiles.length,
    compressionRatio: 1 - compressedData.length / rawData.length,
  };
}
