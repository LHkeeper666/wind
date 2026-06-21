## Why

Windows 用户日常浏览和操作文件时面临一个尴尬的中间地带：VSCode 太重（300-500MB 内存，启动慢），记事本太简陋，Windows 自带终端不方便浏览和预览文件。需要一个轻量级的、以文件为中心的工作台，用 vim 方式操作一切——浏览文件、编辑文本、预览文档、使用终端，都在一个应用里完成。

## What Changes

- 创建一个全新的 Tauri 2.0 桌面应用项目（Rust 后端 + Svelte 前端）
- 实现 vim 驱动的文件树浏览器，支持 j/k/gg/G/Enter/h/l 等键绑定和鼠标操作
- 嵌入 Neovim 作为编辑器后端，通过 msgpack-rpc 通信，Canvas 渲染，支持中文输入法
- 集成 xterm.js 终端，通过 ConPTY 连接 cmd/powershell/git-bash/wsl
- 实现多格式文件预览：语法高亮（Shiki）、Markdown 渲染（markdown-it + KaTeX + Mermaid）、PDF（pdf.js）、图片、JSON 树形视图、CSV 表格
- 建立预览器插件系统，统一 Previewer 接口，可扩展新格式
- 实现面板联动：文件树 ↔ 编辑器 ↔ 预览 ↔ 终端的路径同步和状态联动
- 全局 vim 操作模式：面板切换、命令面板、模糊搜索都用 vim 键绑定

## Capabilities

### New Capabilities

- `file-tree`: 文件树浏览器组件，vim 键绑定导航，文件过滤搜索，文件操作（复制/移动/删除/重命名），鼠标支持
- `neovim-editor`: Neovim 嵌入编辑器，msgpack-rpc 通信，Canvas 渲染引擎，键盘输入处理，中文输入法支持，多 Grid 和浮窗渲染
- `terminal`: 终端组件，xterm.js + ConPTY，多 Shell 支持（cmd/powershell/git-bash/wsl）
- `file-preview`: 文件预览系统，语法高亮（Shiki）、Markdown 渲染、PDF 预览（pdf.js）、图片预览、JSON/CSV 数据预览，统一 Previewer 插件接口
- `panel-layout`: 面板布局系统，分屏支持，面板切换，路径同步，编辑器 ↔ 预览联动
- `global-vim-mode`: 全局 vim 操作层，命令面板，模糊搜索，快捷键管理

### Modified Capabilities

（无，这是全新项目）

## Impact

- **新建项目**：从零创建 Tauri 2.0 项目，包含 Rust 后端和 Svelte 前端
- **核心依赖**：Tauri 2.0, Svelte, xterm.js, Neovim (外部进程), Shiki, markdown-it, KaTeX, Mermaid, pdf.js
- **系统要求**：Windows 10+（依赖 WebView2 和 ConPTY），需要 Neovim 已安装并在 PATH 中
- **许可证**：MIT 或 Apache 2.0，所有核心依赖许可证兼容
- **架构**：Tauri IPC 前后端通信，Rust 管理 Neovim 进程和 ConPTY，前端负责 UI 渲染
