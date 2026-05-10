const fs = require('fs');

// 检查原始壳文件
const shellBuf = fs.readFileSync('./prebuilt/win32-x64.exe');
console.log('原始壳文件大小:', shellBuf.length, 'bytes');

// 检查构建后的文件
const buf = fs.readFileSync('./example/release/Vokex Demo.exe');
const size = buf.length;
console.log('构建后文件大小:', size, 'bytes');
console.log('大小差异:', size - shellBuf.length, 'bytes');

// 读取末尾 8 字节偏移量
const offsetBuf = buf.subarray(size - 8, size);
const offset = Number(offsetBuf.readBigUInt64LE());
console.log('偏移量:', offset);

// 计算预期的偏移量
// VOKEX 资源结构: [壳] [MAGIC] [索引长度] [索引] [压缩数据] [偏移量]
// 偏移量应该指向 MAGIC 的位置
const expectedOffset = size - 8; // 偏移量字段本身在最后 8 字节
console.log('实际文件末尾位置:', expectedOffset);

// 检查偏移量位置的 magic
const magicBuf = buf.subarray(offset, offset + 5);
console.log('在偏移量位置读取的 Magic:', magicBuf.toString());
console.log('Magic hex:', Buffer.from(magicBuf).toString('hex'));

// 期望的 MAGIC
const expectedMagic = Buffer.from('VOKEX');
console.log('期望 Magic:', expectedMagic.toString());

// 尝试在文件中搜索 VOKEX
const vokexIndex = buf.indexOf('VOKEX');
if (vokexIndex !== -1) {
  console.log('\n找到 VOKEX 在位置:', vokexIndex);
  console.log('该位置的上下文:');
  console.log('前 20 字节:', buf.subarray(vokexIndex - 20, vokexIndex).toString('hex'));
  console.log('VOKEX 及后 20 字节:', buf.subarray(vokexIndex, vokexIndex + 25).toString('hex'));
} else {
  console.log('\n文件中没有找到 VOKEX 字符串');
}
