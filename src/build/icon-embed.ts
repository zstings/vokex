/**
 * vokex 框架 - PE 资源注入模块
 *
 * 使用 resedit 库将图标和版本信息注入到 Windows PE 可执行文件的资源段中。
 */

import { readFileSync, writeFileSync } from "fs";
import { NtExecutable, NtExecutableResource, Data, Resource } from "resedit";

/** 版本信息配置 */
export interface VersionInfoOptions {
  /** 应用名称 */
  name: string;
  /** 应用版本号 (如 "1.0.0" 或 "1.0.0.0") */
  version: string;
  /** 应用标识符 */
  identifier?: string;
}

/**
 * 将 .ico 图标注入到 Windows PE 可执行文件中。
 * @returns 是否成功
 */
export function injectIcon(exePath: string, iconBuffer: Buffer): boolean {
  try {
    const exeBuf = readFileSync(exePath);
    console.log(`[vokex:icon] 原始文件大小: ${exeBuf.length} bytes`);

    // Buffer → ArrayBuffer
    const ab = exeBuf.buffer.slice(
      exeBuf.byteOffset,
      exeBuf.byteOffset + exeBuf.byteLength,
    );

    const exe = NtExecutable.from(ab);

    // 保存 PE 尾部追加的额外数据（VOKEX 资源嵌入在 PE 之后）
    const extraData = exe.getExtraData();
    console.log(`[vokex:icon] 额外数据: ${extraData ? '有' : '无'}`);

    const res = NtExecutableResource.from(exe);

    const iconFile = Data.IconFile.from(iconBuffer);
    if (!iconFile.icons.length) {
      console.warn("[vokex:icon] ICO 文件中没有图标数据");
      return false;
    }

    Resource.IconGroupEntry.replaceIconsForResource(
      res.entries,
      1,     // icon group ID
      0x409, // lang: en-US
      iconFile.icons.map((i) => i.data),
    );

    res.outputResource(exe);

    // 确保额外数据不丢失
    if (extraData) {
      exe.setExtraData(extraData);
    }

    const newBuf = Buffer.from(exe.generate());
    console.log(`[vokex:icon] 生成文件大小: ${newBuf.length} bytes`);
    writeFileSync(exePath, newBuf);
    return true;
  } catch (err: any) {
    console.warn(`[vokex:icon] 图标注入失败: ${err.message}`);
    return false;
  }
}

/**
 * 解析版本号字符串为 MS/LS 格式（Windows VERSIONINFO 要求）
 * "1.2.3.4" → { ms: 0x00010002, ls: 0x00030004 }
 */
function parseVersion(version: string): { ms: number; ls: number } {
  const parts = version.split(".").map(Number);
  const major = parts[0] || 0;
  const minor = parts[1] || 0;
  const patch = parts[2] || 0;
  const build = parts[3] || 0;
  return {
    ms: ((major & 0xFFFF) << 16) | (minor & 0xFFFF),
    ls: ((patch & 0xFFFF) << 16) | (build & 0xFFFF),
  };
}

/**
 * 将版本信息注入到 Windows PE 可执行文件中。
 * @returns 是否成功
 */
export function injectVersionInfo(exePath: string, options: VersionInfoOptions): boolean {
  try {
    const exeBuf = readFileSync(exePath);

    // Buffer → ArrayBuffer
    const ab = exeBuf.buffer.slice(
      exeBuf.byteOffset,
      exeBuf.byteOffset + exeBuf.byteLength,
    );

    const exe = NtExecutable.from(ab);

    // 保存 PE 尾部追加的额外数据
    const extraData = exe.getExtraData();

    const res = NtExecutableResource.from(exe);

    // 读取现有的版本信息（如果有的话）
    const existingVersions = Resource.VersionInfo.fromEntries(res.entries);

    // 解析版本号
    const version = parseVersion(options.version);
    const versionStr = options.version.includes(".")
      ? options.version.split(".").concat(Array(4).fill("0")).slice(0, 4).join(".")
      : `${options.version}.0.0.0`;

    // 使用现有版本信息或创建新的
    const lang = { lang: 0x0409, codepage: 1200 }; // en-US, Unicode
    let versionInfo: Resource.VersionInfo;

    if (existingVersions.length > 0) {
      versionInfo = existingVersions[0];
    } else {
      versionInfo = Resource.VersionInfo.create(lang.lang, {}, []);
    }

    // 设置固定版本信息
    versionInfo.fixedInfo.fileVersionMS = version.ms;
    versionInfo.fixedInfo.fileVersionLS = version.ls;
    versionInfo.fixedInfo.productVersionMS = version.ms;
    versionInfo.fixedInfo.productVersionLS = version.ls;
    versionInfo.fixedInfo.fileOS = Resource.VersionFileOS.NT_Windows32;
    versionInfo.fixedInfo.fileType = Resource.VersionFileType.App;

    // 设置字符串值
    const strings: Resource.VersionStringValues = {
      FileDescription: options.name,
      ProductName: options.name,
      FileVersion: versionStr,
      ProductVersion: versionStr,
      OriginalFilename: `${options.name}.exe`,
      InternalName: options.name,
    };

    if (options.identifier) {
      strings.CompanyName = options.identifier;
      strings.LegalCopyright = `Copyright © ${new Date().getFullYear()} ${options.name}`;
    }

    versionInfo.setStringValues(lang, strings, true);
    versionInfo.outputToResourceEntries(res.entries);

    res.outputResource(exe);

    // 确保额外数据不丢失
    if (extraData) {
      exe.setExtraData(extraData);
    }

    const newBuf = Buffer.from(exe.generate());
    writeFileSync(exePath, newBuf);
    return true;
  } catch (err: any) {
    console.warn(`[vokex:version] 版本信息注入失败: ${err.message}`);
    return false;
  }
}
