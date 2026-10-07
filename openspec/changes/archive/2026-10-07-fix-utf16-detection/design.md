## Context

当前 `decode_text()` 函数（`src-tauri/src/commands/file_io.rs:64-79`）的解码流程：
1. 检查 UTF-8 BOM（`EF BB BF`）→ 有的话跳过
2. 尝试 `String::from_utf8()` → 成功则返回
3. 使用 `chardetng` 检测编码 → `encoding_rs` 解码

问题：UTF-16 LE 文件（BOM: `FF FE`）包含大量 `0x00` 字节，`chardetng` 可能误判为 ISO-8859-1 等单字节编码，解码后文本仍含 `\0`，前端 `loadTextOrBinary()` 检测到 `\0` 后判定为二进制。

## Goals / Non-Goals

**Goals:**
- 正确识别 UTF-16 LE/BE 编码的文本文件
- 将 UTF-16 文件解码为 UTF-8 字符串返回给前端
- 保持对现有 UTF-8、GBK 等编码的兼容性

**Non-Goals:**
- 不处理无 BOM 的 UTF-16 文件（需要启发式检测，复杂度高）
- 不改变前端的二进制检测逻辑
- 不支持 UTF-32 编码

## Decisions

**Decision 1: 在 `decode_text()` 中优先检测 UTF-16 BOM**

在 UTF-8 BOM 检测之后、`String::from_utf8()` 之前，增加 UTF-16 BOM 检测：
- `FF FE` → UTF-16 LE
- `FE FF` → UTF-16 BE

使用 `encoding_rs::UTF_16LE` / `UTF_16BE` 直接解码。

**替代方案考虑**：
- 方案A：让 chardetng 处理 → 失败，chardetng 对 UTF-16 检测不可靠
- 方案B：前端处理 UTF-16 → 不可行，前端无法高效处理二进制转码

**Decision 2: 只处理有 BOM 的 UTF-16**

无 BOM 的 UTF-16 文件极少见，且需要启发式检测（检查是否交替出现 `0x00`），容易误判。只处理有 BOM 的情况即可覆盖 99% 的场景。

## Risks / Trade-offs

- **风险**：`encoding_rs` 的 `decode_without_bom_handling` 是否正确处理 UTF-16 BOM → 需要验证，可能需要手动跳过 BOM 字节
- **权衡**：不支持无 BOM 的 UTF-16 → 可接受，这类文件非常少见