/**
 * vokex 框架 - PE 图标注入模块
 *
 * 使用 resedit 库将 .ico 图标注入到 Windows PE 可执行文件的资源段中。
 */

import { readFileSync, writeFileSync } from "fs";
import { NtExecutable, NtExecutableResource, Data, Resource } from "resedit";

/**
 * 将 .ico 图标注入到 Windows PE 可执行文件中。
 * @returns 是否成功
 */
export function injectIcon(exePath: string, iconBuffer: Buffer): boolean {
  try {
    const exeBuf = readFileSync(exePath);

    // Buffer → ArrayBuffer
    const ab = exeBuf.buffer.slice(
      exeBuf.byteOffset,
      exeBuf.byteOffset + exeBuf.byteLength,
    );

    const exe = NtExecutable.from(ab);

    // 保存 PE 尾部追加的额外数据（VOKEX 资源嵌入在 PE 之后）
    const extraData = exe.getExtraData();

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
    writeFileSync(exePath, newBuf);
    return true;
  } catch (err: any) {
    console.warn(`[vokex:icon] 图标注入失败: ${err.message}`);
    return false;
  }
}
