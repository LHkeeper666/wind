## Context

当前预览系统有三层过滤（PreviewEditor → PreviewRouter → TextPreviewer），TextPreviewer 使用白名单只匹配约 50 个扩展名，导致大量文本文件无法预览。二进制文件则完全不显示。

PreviewEditor.svelte 中已有 `isTextFile()` 函数用黑名单排除二进制格式，逻辑上应该放行所有文本文件，但 TextPreviewer 的白名单又把它们挡回去了。

## Goals / Non-Goals

**Goals:**
- 所有非特殊处理文件都能预览（文本或 hex dump）
- 二进制文件显示 hex dump 以识别文件类型
- 大文件分段读取，不阻塞 UI
- 无扩展名文件当文本处理，已知文件名做语法高亮

**Non-Goals:**
- 不实现二进制文件编辑
- 不添加 hex dump 的编辑/修改功能
- 不修改其他 previewer 的匹配逻辑

## Decisions

### D1: TextPreviewer.match() 始终返回 true

**选择**: 删除白名单过滤，match() 直接返回 true

**理由**: PreviewRouter 中 TextPreviewer 排在最后，前面 6 个 previewer 已经拦截了各自负责的文件类型。TextPreviewer 应该是真正的 fallback，不应该再拒绝任何文件。

**替代方案**: 扩大白名单 → 维护成本高，永远追不上所有文本扩展名

### D2: 二进制文件检测策略 — 扩展名黑名单 + null byte 检测

**选择**: 先检查扩展名黑名单（exe/dll/png/mp3 等），再检查内容前 8KB 是否含 `\0`

**理由**: 扩展名检测快且准确，null byte 检测兜底处理无扩展名或未知扩展名的二进制文件。只检查前 8KB 避免读大文件。

**替代方案**: 纯扩展名检测 → 漏掉无扩展名二进制文件；纯内容检测 → 对大文件慢

### D3: 大文件阈值 200KB，只读头部

**选择**: 文件超过 200KB 时，只读前 200KB 显示，并在底部显示截断提示

**理由**: 200KB 足以覆盖 99% 的预览场景（配置、代码、日志开头）。Shiki 已有 200KB 的高亮跳过阈值，保持一致。

**实现**: 新增 Rust 命令 `read_file_partial(path, max_bytes)`，返回 `String`

### D4: Hex dump 格式 — 经典 16 字节/行

**选择**:
```
00000000  7f 45 4c 46 02 01 01 00  00 00 00 00 00 00 00 00  |.ELF............|
```

**理由**: 经典 hex dump 格式，开发者熟悉，可读性好。最多显示 64KB（约 4096 行），防止 hex dump 过大。

### D5: 特殊文件名映射

**选择**: 在 TextPreviewer 的 getLanguage() 中增加文件名到语言的映射

**映射表**:
- `Makefile`, `CMakeLists.txt` → makefile
- `Dockerfile` → dockerfile
- `Vagrantfile` → ruby
- `Gemfile`, `Rakefile` → ruby
- `LICENSE`, `README`, `CHANGELOG`, `CONTRIBUTING` → text (无高亮)
- `.env.example`, `.env.local` → bash

## Risks / Trade-offs

- **[二进制误判]** 某些含 `\0` 的文本文件（如 UTF-16 编码）会被判为二进制 → 可接受，UTF-16 文件在开发场景少见
- **[Hex dump 性能]** 64KB 二进制 → ~4096 行 HTML，渲染可接受 → 超过 64KB 截断
- **[Shiki 对未知语言]** 不认识的扩展名 fallback 到 `text`，无语法高亮 → 可接受，纯文本仍然可读
