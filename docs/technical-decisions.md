# Wind 技术决策文档

本文档记录 Wind 项目在探索阶段做出的所有关键技术决策及理由。

## 产品定位

**Wind 是什么**: 一个轻量级 Windows 文件中心工作台，用 vim 方式操作一切。

**Wind 不是什么**: 不是 IDE（不做代码补全/重构/调试），不是终端模拟器（不自建渲染引擎）。

**类比**: ranger (文件管理) + zathura (预览) + vim (操作方式) + 终端，打包成一个 Windows 原生 GUI 应用。

**核心场景**:
- 打开文件夹，用 vim 键浏览文件
- 按回车打开终端，j/k 切换文件，p 预览
- Markdown / PDF / 图片 / 代码都能预览
- 不需要为看个文件就开 VSCode

---

## 技术栈总览

| 层级 | 技术选型 | 理由 |
|------|----------|------|
| 应用框架 | Tauri 2.0 | 性能好，内存低 (~60-80MB)，启动快 (<1s)，WebView2 是 Windows 自带组件 |
| 前端框架 | Svelte | 编译时框架，接近零运行时开销，学习曲线低 |
| 编辑器 | Neovim (--embed) | 完整 vim 体验，零实现成本，中文输入法成熟方案 |
| 终端 | xterm.js + ConPTY | xterm.js 最成熟的 Web 终端，ConPTY 是 Windows 官方伪终端 API |
| 语法高亮 | Shiki | 使用 VSCode TextMate 语法，高亮质量最高，200+ 语言 |
| Markdown | markdown-it + KaTeX + Mermaid | CommonMark 标准，数学公式，图表渲染 |
| PDF | pdf.js | Mozilla 出品，Firefox 内置，成熟稳定 |
| 开源许可 | MIT 或 Apache 2.0 | 所有核心依赖许可证兼容，最友好 |

---

## 决策 D1: 框架选择 — Tauri 2.0 + Svelte

**选择**: Tauri 2.0 做应用框架，Svelte 做前端 UI

**替代方案对比**:

| 方案 | 内存 | 启动 | 开发效率 | 评估 |
|------|------|------|----------|------|
| Electron + xterm.js | 300MB+ | 慢 | 高 | 太重，排除 |
| Tauri + xterm.js | 30-60MB | 快 | 高 | **选中** |
| Go + Wails | 20-40MB | 很快 | 中 | GUI 生态弱 |
| Rust + iced | 10-20MB | 极快 | 低 | 开发周期太长 |
| C# + WPF | 20-40MB | 快 | 中高 | 跨平台困难 |

**理由**:
- Tauri 2.0: Rust 后端性能好，WebView2 是 Windows 10+ 自带组件
- Svelte: 编译时框架，无虚拟 DOM 开销，代码简洁，适合前端新手

---

## 决策 D2: 编辑器方案 — 嵌入 Neovim

**选择**: Neovim --embed --headless，通过 msgpack-rpc 通信，Canvas 渲染

**替代方案对比**:

| 方案 | vim 完整性 | 实现成本 | 中文支持 | 评估 |
|------|-----------|----------|----------|------|
| 嵌入 Neovim | 100% | 低 | 成熟 | **选中** |
| 自建 vim 状态机 | 追不上 | 极高 | 需自建 | 排除 |
| CodeMirror vim 模式 | ~70% | 中 | 一般 | 不是真 vim |

**理由**:
- 完整 vim 体验：寄存器、宏、补全、搜索、替换、marks、text objects 全部原生
- 中文输入法在 Neovim 已有成熟方案
- 插件生态直接可用
- 一年时间可以专注在"非 vim"的部分

**嵌入方式**:
```
nvim --embed --headless
  │
  ├── stdin  ← msgpack-rpc 请求 (nvim_input, nvim_command...)
  │
  └── stdout → msgpack-rpc 响应 + redraw 通知
```

**UI 协议选项**:
- `ext_linegrid: true` — 行级网格更新（只更新变化的行）
- `ext_multigrid: true` — 多网格支持（浮窗独立 grid）
- `ext_cmdline: true` — 外部命令行（自己决定怎么显示）
- `ext_popupmenu: true` — 外部弹出菜单（自定义补全 UI）
- `ext_tabline: true` — 外部标签栏
- `ext_messages: true` — 外部消息显示

---

## 决策 D3: 终端方案 — xterm.js + ConPTY

**选择**: xterm.js 做终端渲染，通过 ConPTY 连接 Windows Shell

**替代方案**:
- 自建 VT100 渲染引擎: 工作量太大，不值得
- winpty: 已过时，ConPTY 是 Windows 官方替代

**支持的 Shell**:
- cmd
- PowerShell
- Git Bash
- WSL

---

## 决策 D4: 语法高亮 — Shiki

**选择**: Shiki 作为语法高亮引擎

**替代方案对比**:

| 方案 | 高亮质量 | 语言数 | 主题 | 包体积 | 评估 |
|------|----------|--------|------|--------|------|
| Shiki | VSCode 级 | 200+ | VSCode 全部 | ~1MB | **选中** |
| Prism.js | 良好 | 300+ | 自带几套 | ~50KB | 质量稍低 |
| Highlight.js | 良好 | 200+ | 自带几套 | ~40KB | 质量稍低 |
| Tree-sitter | 优秀 | 需逐个装 | 自己配 | ~2MB/WASM | 太重 |

**理由**: Wind 作为"打开文件的轻量替代"，预览效果应该接近 VSCode，Shiki 共享 VSCode 的 TextMate 语法和主题。

---

## 决策 D5: 预览器架构 — 统一插件接口

**选择**: 定义 Previewer 接口，每种文件类型实现一个预览器

**接口设计**:
```typescript
interface Previewer {
  match(filePath: string): boolean;
  render(content: ArrayBuffer | string, container: HTMLElement): void;
  dispose(): void;
}
```

**预览器清单**:

| 预览器 | 匹配格式 | 技术方案 |
|--------|----------|----------|
| TextPreviewer | .js .ts .py .java .go .rs .c .cpp ... | Shiki 语法高亮 |
| MarkdownPreviewer | .md .markdown | markdown-it + KaTeX + Mermaid |
| PdfPreviewer | .pdf | pdf.js Canvas 渲染 |
| ImagePreviewer | .png .jpg .gif .svg .webp | 原生 `<img>` + 缩放平移 |
| JsonPreviewer | .json | 树形视图 + 原始视图 |
| CsvPreviewer | .csv .tsv | 表格视图 + 分页 |
| Fallback | 其他 | 纯文本 或 hex dump |

**大文件处理策略**:
- 文本文件 > 1MB: 只高亮前 N 行，虚拟滚动按需加载
- PDF: 只渲染当前可见页 + 前后各 2 页
- 图片 > 5000px: 降采样显示，提示原图尺寸
- CSV > 1000 行: 分页显示

---

## 决策 D6: 前后端通信 — Tauri IPC

**选择**: 使用 Tauri 内置的 IPC 机制（invoke/emit）

**通信模式**:
- `invoke('command', { params })` — 请求/响应模式（前端 → 后端）
- `emit('event-name', data)` — 事件推送模式（后端 → 前端）

**关键数据流**:

```
场景: 用户在文件树按 j 选中文件
  FileTree → invoke("read_file", { path }) → Rust → 返回内容 → Preview 渲染

场景: 用户在终端输入命令
  xterm.js → invoke("terminal_input", { data }) → ConPTY → emit("terminal_output") → xterm.js

场景: 用户在编辑器按 i
  Canvas → invoke("nvim_input", { keys: "i" }) → Neovim RPC → emit("nvim_redraw") → Canvas 重绘
```

---

## Neovim Canvas 渲染方案

**渲染方式**: HTML Canvas（非 DOM）

**选择 Canvas 的理由**:
- 性能好，大文件滚动流畅
- 字符级精确控制
- DOM 方式在大文件下性能差

**数据结构**:
```
GridManager
  ├── grids: Map<gridId, Grid>
  │     ├── cells: Uint32Array (charCode + hlGroupId)
  │     ├── cursorRow, cursorCol
  │     └── position: {x, y}
  ├── highlightMap: Map<hlId, HighlightAttr>
  │     ├── foreground, background, special (RGB)
  │     └── bold, italic, underline, undercurl...
  └── defaultFg, defaultBg
```

**渲染流程**:
1. 收到 `flush` 事件后触发渲染
2. 遍历每个 grid 的每个 cell
3. 绘制背景色 → 绘制字符 → 绘制装饰线 → 绘制光标
4. CJK 双宽度字符：占 2 列，跳过占位格

**光标样式**:
- normal: █ block, 闪烁
- insert: │ vertical line, 不闪烁
- visual: █ block, 反色选中区域
- replace: _ horizontal line

---

## 中文输入法处理方案

**问题**: IME 中间状态（如拼音 "ni"）不能发给 Neovim，否则会被当作普通按键。

**方案**:
1. `compositionstart` → 进入组合模式，不向 Neovim 发送按键
2. `compositionupdate` → 在 Canvas 光标位置显示预编辑文本（带下划线）
3. `compositionend` → 将最终组合文本发送给 Neovim，清除预编辑显示

**需要测试的输入法**: 搜狗、微软拼音、百度输入法

---

## 面板布局设计

```
┌──────────────┬──────────────────────┬───────────────┐
│              │                      │               │
│  File Tree   │   Neovim Editor      │   Preview     │
│  (vim 导航)   │   (Canvas 渲染)       │   (多格式)     │
│              │                      │               │
├──────────────┴──────────────────────┴───────────────┤
│                                                     │
│   Terminal (xterm.js + ConPTY)                      │
│                                                     │
├─────────────────────────────────────────────────────┤
│  Status Bar: mode | path | encoding                 │
└─────────────────────────────────────────────────────┘
```

**面板切换**: Ctrl+W h/j/k/l（vim 风格）

**路径同步**:
- 文件树选中目录 → 终端可同步 cd
- Neovim 打开文件 → 文件树高亮对应文件
- Neovim 编辑 Markdown → 预览实时更新

---

## 开源相关

**许可证**: MIT 或 Apache 2.0

**依赖许可证兼容性**:
- Tauri: MIT / Apache 2.0
- xterm.js: MIT
- Neovim: Apache 2.0
- pdf.js: Apache 2.0
- Shiki: MIT
- markdown-it: MIT

**开源基础设施** (后续建立):
- GitHub Actions CI/CD
- Issue/PR 模板
- CONTRIBUTING.md
- CHANGELOG.md

---

## 开发路线 (一年)

| 阶段 | 时间 | 内容 |
|------|------|------|
| Phase 1 | Month 1-2 | Tauri 脚手架 + 文件树 + 基础终端 + Neovim 基础嵌入 |
| Phase 2 | Month 3-4 | 预览器 (Shiki/Markdown/图片) + 面板联动 + 鼠标支持 |
| Phase 3 | Month 5-6 | PDF 预览 + 文件搜索 + 多标签 + 状态栏 |
| Phase 4 | Month 7-9 | 分屏布局 + 主题系统 + 中文输入法优化 + 文件操作 |
| Phase 5 | Month 10-12 | 命令面板 + 书签 + Shell 切换 + 打包发布 |

---

## 已知风险

| 风险 | 缓解策略 |
|------|----------|
| Neovim 版本兼容性 | 运行时检测版本和能力，降级处理 |
| 渲染性能 (大文件/快速滚动) | flush 时批量渲染，dirty flag，requestAnimationFrame 节流 |
| 中文输入法兼容性 | 测试主流输入法，预编辑文本显示方案 |
| Neovim 插件冲突 | 使用 Wind 自己的 init.vim，提供"干净模式"和"用户模式" |
| Neovim 进程崩溃 | 监控进程状态，自动恢复，友好错误提示 |
| 前端学习成本 | Svelte 学习曲线低，代码注释充分 |

---

## 参考项目

| 项目 | 语言 | 参考价值 |
|------|------|----------|
| Goneovim | Go + Qt | Neovim 嵌入、多 grid 渲染、浮窗处理 |
| VimR | Swift + macOS | macOS 上的 Neovim GUI，架构清晰 |
| NyaoVim | Electron | Neovim 嵌入思路 |
| neovim/node-client | Node.js | msgpack-rpc 实现参考 |
