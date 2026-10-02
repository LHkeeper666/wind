## Context

Wind 的解压模块（`src-tauri/src/archive/`）目前支持 ZIP、TAR、TAR.GZ、7z 四种格式。每种格式有独立的实现模块（`zip.rs`、`tar.rs`、`seven_z.rs`），通过 `mod.rs` 的 `ArchiveFormat` enum 统一分发。

RAR 是 Windows 生态中最常见的压缩格式之一，用户经常需要打开 `.rar` 文件。RAR 格式是专利格式，只能读取不能创建。

分卷 RAR 有两种命名规范：
- RAR5 新格式：`archive.part1.rar`、`archive.part2.rar`、...
- RAR4 旧格式：`archive.rar`、`archive.r00`、`archive.r01`、...

## Goals / Non-Goals

**Goals:**
- 支持 RAR4/RAR5 格式的只读操作（列表、预览、解压）
- 支持分卷 RAR，用户双击任意分卷时自动跳转到第一卷打开
- 支持密码保护的 RAR 档案，复用现有密码提示流程
- 分卷文件在目录面板中全部显示，不做特殊标记

**Non-Goals:**
- 不支持创建 RAR 档案（`supports_write()` 返回 false）
- 不支持修改 RAR 档案内的文件
- 不支持 RAR 恢复记录功能

## Decisions

### Decision 1: 使用 `unrar` crate

**选择**: `unrar` crate（muja/unrar.rs），绑定官方 libunrar C++ 库

**备选方案**:
- `unrar_rs` - 纯 Rust 实现，较新，API 不够成熟
- 系统命令 `unrar.exe` - 依赖外部工具，用户体验差
- `compress-tools` (libarchive) - 过重，RAR5 支持不完整

**理由**: `unrar` crate 成熟稳定，原生支持分卷 RAR、密码、RAR5，API 风格与现有 `sevenz-rust` 类似（迭代条目模式），易于集成。

### Decision 2: 分卷 RAR 的处理方式

**选择**: 使用 `unrar` crate 的 `as_first_part()` 方法自动跳转到第一卷

**实现**:
```rust
// 用户双击 archive.part3.rar
let archive = unrar::Archive::new("archive.part3.rar")
    .as_first_part();  // 自动变成 archive.part1.rar
// 从第一卷开始正常读取，库内部自动遍历所有分卷
```

**分卷识别**:
- `.partN.rar` 模式：通过 `unrar::Archive::is_multipart()` 和 `first_part()` 处理
- `.rar/.rXX` 模式：`unrar` crate 内部处理

**前端行为**: 分卷文件全部显示，不做特殊标记。用户双击任意分卷，后端自动找到第一卷开始读取。

### Decision 3: 前端格式识别

**选择**: 在 `getArchiveFormat()` 中添加 `.rar` 识别，扩展 `ArchiveFormat` 类型

**实现**:
```typescript
// layout.ts
export type ArchiveFormat = 'zip' | 'tar' | 'tar.gz' | '7z' | 'rar';

// archive-browser.ts
if (lower.endsWith('.rar') || lower.match(/\.r\d{2}$/) || lower.match(/\.part\d+\.rar$/)) return 'rar';
```

`stripArchiveExtension()` 需要处理 `.rar` 和 `.partN.rar` 两种情况。

### Decision 4: RAR 密码支持

**选择**: 复用现有密码提示流程，后端使用 `unrar::Archive::with_password()`

**实现**: `rar.rs` 中的错误处理映射 `unrar` 的密码错误到现有的 `password_required_error` / `password_incorrect_error`，与 ZIP/7z 行为一致。

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|-----|-----|-----|
| C++ 编译依赖 | Windows 上需要 MSVC 工具链 | 已有 Tauri 构建环境覆盖，`unrar` crate 在 Windows 上有预编译支持 |
| 分卷缺失 | 用户只复制了部分分卷 | `unrar` 会报错，前端显示友好错误提示 |
| 二进制体积增加 | 静态链接 libunrar 约 1-2MB | 可接受，功能价值大于体积成本 |
| 文件名编码 | RAR4 可能使用非 UTF-8 编码 | RAR5 默认 UTF-8；RAR4 需要额外处理，可参考现有 `encoding.rs` 方案 |
| `unrar` crate 不支持随机访问 | 只能顺序扫描条目 | 与现有 `sevenz-rust` 模式一致，不影响用户体验 |

## Migration Plan

无需迁移。纯增量改动：
1. 添加 `unrar` 依赖到 `Cargo.toml`
2. 新增 `rar.rs` 模块
3. 更新 `mod.rs` 分发逻辑
4. 更新前端类型和识别逻辑

## Open Questions

- `unrar` crate 在 Windows 上的编译是否需要额外配置（如 cmake）？需要在实际编译时验证。