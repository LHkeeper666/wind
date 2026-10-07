## 1. 修改 decode_text() 函数

- [x] 1.1 在 `decode_text()` 中 UTF-8 BOM 检测之后、`String::from_utf8()` 之前，增加 UTF-16 LE BOM (`FF FE`) 检测，使用 `encoding_rs::UTF_16LE` 解码
- [x] 1.2 增加 UTF-16 BE BOM (`FE FF`) 检测，使用 `encoding_rs::UTF_16BE` 解码
- [x] 1.3 确保解码后跳过 BOM 字节，返回纯文本内容

## 2. 测试验证

- [x] 2.1 创建 UTF-16 LE 编码的测试文件，验证 `decode_text()` 正确解码且不含 `\0`
- [x] 2.2 创建 UTF-16 BE 编码的测试文件，验证 `decode_text()` 正确解码
- [x] 2.3 验证 UTF-8 和 GBK 编码文件的行为不受影响