## Context

Windows 用户缺少一个轻量级的、以文件为中心的工作台应用。现有方案要么太重（VSCode 300-500MB），要么太简陋（记事本），要么不方便文件浏览和预览（Windows Terminal）。Wind 旨在填补这个空白，提供一个 vim 驱动的文件工作台，集成文件树、编辑器、终端和预览器。

技术约束：
- Windows 10+ 平台（依赖 WebView2 和 ConPTY）
- 需要 Neovim 已安装并在 PATH 中
- 目标内存占用 < 100MB，启动时间 < 1 秒
- 开源项目，MIT 许可证

## Goals / Non-Goals

**Goals:**
- 提供 vim 原生体验的文件浏览、编辑、终端和预览
- 性能优先：低内存占用，快速启动
- 可扩展的预览器插件系统
- 支持中文输入法的 vim 操作
- 面板联动：文件树、编辑器、预览、终端之间无缝切换

**Non-Goals:**
- 不是 IDE，不做代码补全、重构、调试
- 不做远程 SSH 连接（v1 不考虑）
- 不做插件系统（v1 只做预览器插件）
- 不做移动端或 Linux/macOS 支持（v1 只做 Windows）
- 不自建终端渲染引擎（用 xterm.js）

## Decisions

### D1: 框架选择 — Tauri 2.0 + Svelte

**选择**: Tauri 2.0 做应用框架，Svelte 做前端 UI

**理由**:
- Tauri 2.0: Rust 后端性能好，WebView2 是 Windows 自带组件无需额外安装，内存占用 ~30-60MB
- Svelte: 编译时框架，接近零运行时开销，学习曲线低，代码简洁
- 替代方案：Electron（太重，300MB+）、Go+Wails（GUI 生态弱）、Rust+iced（开发周期长）

### D2: 编辑器方案 — 嵌入 Neovim

**选择**: Neovim --embed --headless，通过 msgpack-rpc 通信，Canvas 渲染

**理由**:
- 完整 vim 体验，零实现成本（寄存器、宏、补全、搜索、插件全部原生支持）
- 中文输入法在 Neovim 已有成熟方案
- 替代方案：自己实现 vim 状态机（工作量巨大且不完整）、CodeMirror（不是真 vim）

**关键实现**:
- UI 协议：ext_linegrid + ext_multigrid + ext_cmdline + ext_popupmenu
- Canvas 渲染引擎：字符网格 + 高亮属性映射 + 光标绘制
- 键盘输入：DOM KeyboardEvent → Neovim 键序列转换
- 中文输入法：CompositionEvent 处理，预编辑文本显示

### D3: 终端方案 — xterm.js + ConPTY

**选择**: xterm.js 做终端渲染，通过 ConPTY 连接 Windows Shell

**理由**:
- xterm.js 是最成熟的 Web 终端组件，VT100 兼容性好
- ConPTY 是 Windows 官方伪终端 API，比 winpty 更稳定
- 替代方案：自己渲染 VT100 转义序列（工作量太大）

### D4: 语法高亮 — Shiki

**选择**: Shiki 作为语法高亮引擎

**理由**:
- 使用 VSCode 的 TextMate 语法，高亮质量最高
- 支持 200+ 语言，VSCode 全部主题可用
- 替代方案：Prism.js（质量稍低）、Highlight.js（质量稍低）、Tree-sitter（太重）

### D5: 预览器架构 — 统一插件接口

**选择**: 定义 Previewer 接口，每种文件类型实现一个预览器

**理由**:
- 可扩展：新增格式只需实现接口
- 统一入口：PreviewRouter 根据文件类型分发
- 替代方案：单一大函数处理所有格式（不可维护）

**接口设计**:
```
interface Previewer {
  match(filePath: string): boolean;
  render(content: ArrayBuffer | string, container: HTMLElement): void;
  dispose(): void;
}
```

### D6: 前后端通信 — Tauri IPC

**选择**: 使用 Tauri 内置的 IPC 机制（invoke/emit）

**理由**:
- 类型安全，性能好
- 支持请求/响应和事件推送两种模式
- 替代方案：WebSocket（额外复杂度）、共享内存（太底层）

## Risks / Trade-offs

**R1: Neovim 版本兼容性**
→ 风险：不同版本 UI 协议有差异
→ 缓解：运行时检测 Neovim 版本和能力，降级处理；测试 0.9+ 版本

**R2: 渲染性能**
→ 风险：快速滚动或大文件时 Canvas 渲染卡顿
→ 缓解：只在 flush 时批量渲染，dirty flag 避免重复渲染，requestAnimationFrame 节流

**R3: 中文输入法兼容性**
→ 风险：不同输入法（搜狗、微软、百度）行为不同
→ 缓解：测试主流输入法，预编辑文本显示方案，提供输入法配置选项

**R4: Neovim 插件冲突**
→ 风险：用户已有 Neovim 配置可能破坏 UI
→ 缓解：Wind 使用自己的 init.vim/init.lua，提供"干净模式"和"用户模式"切换

**R5: Neovim 进程崩溃**
→ 风险：Neovim 进程意外退出
→ 缓解：监控进程状态，崩溃后自动恢复，显示友好错误提示

**R6: 前端学习成本**
→ 风险：用户不熟悉前端技术
→ 缓解：Svelte 学习曲线低，提供详细注释和文档

## Open Questions

1. 项目是否需要支持 WSL 文件系统？（/mnt/c/... 路径）
2. 文件树是否需要支持 git 状态显示？（modified/added/deleted 标记）
3. 是否需要支持多窗口（多个 Wind 实例）？
