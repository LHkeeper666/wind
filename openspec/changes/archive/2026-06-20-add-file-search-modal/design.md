## Context

Wind 是一个基于 Tauri 2.0 的文件工作站。FileTreePanel 目前只有简单的内联过滤（按 `/` 触发），不支持正则表达式和递归搜索。用户需要一个类似 Telescope 的浮窗搜索组件。

现有代码结构：
- 后端：`src-tauri/src/lib.rs` - Tauri 命令
- 前端：`src/lib/components/FileTreePanel.svelte` - 文件树面板

## Goals / Non-Goals

**Goals:**
- 支持正则表达式的浮窗文件搜索
- rg (ripgrep) 优先 + Rust 原生回退
- j/k 键盘导航搜索结果
- 实时搜索进度显示
- 鼠标点击和快捷键 `/` 触发

**Non-Goals:**
- 不实现目录跳转功能
- 不修改现有的目录浏览逻辑
- 不添加文件内容搜索（只搜文件名）

## Decisions

### 1. rg 调用方式：Rust Command vs Tauri Shell 插件

**选择：Rust `std::process::Command`**

理由：
- 安全性：不需要启用 shell 插件
- 性能：后端执行，不阻塞 UI
- 流式返回：可通过事件发送进度
- 错误处理更完善

### 2. 搜索回退策略

**选择：rg 优先，自动回退 Rust 原生**

流程：
1. 检测 `rg --version` 是否可用
2. 可用则调用 `rg --files --regex`
3. 不可用则用 `walkdir` + `regex` 递归
4. 右下角提示用户安装 rg（仅首次）

### 3. 移除内联搜索

**选择：移除，用浮窗完全替代**

理由：
- 浮窗功能更强（正则、rg、相对路径）
- 避免快捷键冲突
- 代码更简洁

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| rg 未安装导致搜索慢 | 右下角提示安装，Rust 原生仍可用 |
| 大目录搜索慢 | 限制最大结果数 50，显示进度 |
| 正则表达式复杂导致性能问题 | 前端防抖 300ms，后端超时保护 |
