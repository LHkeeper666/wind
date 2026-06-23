# Wind

Windows 平台下 Vim 风格的文件工作站，基于 Tauri 2.0（Rust 后端 + Svelte 5 前端）。

受 [Yazi](https://github.com/sachinsenal0x64/yazi) 启发，Wind 将三栏文件浏览器、嵌入式终端、代码编辑器和多格式预览融合为纯键盘驱动的工作流。

## 功能特性

- **三栏布局** — 父目录 | 当前目录 | 预览/编辑器，支持 `:ratio X:Y:Z` 调节比例
- **Vim 快捷键** — `j/k` 导航、`gg/G`、`h/l` 面板切换、终端 Normal/Insert 模式
- **文件预览** — 文本（Shiki 语法高亮）、Markdown（KaTeX + Mermaid）、图片（turbojpeg）、PDF、视频（ffmpeg 缩略图 + HTTP 流播放）、JSON、压缩包（zip）、目录
- **全屏查看** — 图片查看器（缩放/平移/旋转）、PDF 查看器（文本搜索）、视频播放器
- **浮动终端** — 基于 ConPTY，支持 PowerShell、CMD、Git Bash，带 Shell 集成（OSC 133/OSC 7）
- **多标签** — Yazi 风格 `t` 前缀快捷键管理标签页
- **代码编辑器** — CodeMirror 6 + Vim 模式，15+ 语言语法高亮，全屏编辑
- **命令面板** — `:` 唤起，`:cd <路径>`、`:e <路径>`、`:ratio X:Y:Z`，Tab 补全路径
- **文件搜索** — `Ctrl+P` 模糊搜索当前目录
- **内容缩放** — `Ctrl+=`/`Ctrl+-`/`Ctrl+0` 和 `Ctrl+滚轮` 缩放面板内容
- **主题系统** — 明暗主题切换，Gruvbox 风格设计
- **中文输入法** — 完整支持 IME 模式下的键盘导航

## 环境要求

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/) stable
- [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（Windows）

## 快速开始

```bash
# 安装依赖
npm install

# 启动开发
npm run tauri dev

# 生产构建
npm run tauri build
```

## 快捷键

### 全局
| 按键 | 功能 |
|---|---|
| `Ctrl+\`` | 切换浮动终端 |
| `:` | 打开命令面板 |
| `Ctrl+P` | 文件搜索 |

### 目录面板
| 按键 | 功能 |
|---|---|
| `j` / `k` | 向下 / 向上导航 |
| `gg` / `G` | 跳到顶部 / 底部 |
| `Enter` | 打开文件或进入目录 |
| `R` | 刷新目录列表 |
| `h` | 切换到左侧面板（父目录） |
| `l` | 切换到右侧面板（预览） |
| `Ctrl+W h/j/k/l` | 面板导航 |

### 浮动终端
| 按键 | 功能 |
|---|---|
| `Ctrl+\`` | 切换终端 |
| `Escape` | 切换到 Normal 模式 |
| `i` | 切换到 Insert 模式 |

### 预览 / 编辑器面板
| 按键 | 功能 |
|---|---|
| `Enter` | 进入编辑模式 |
| `e` | 打开全屏编辑器 |
| `Escape` | 退出编辑模式 |
| `Ctrl+S` | 保存文件 |

### 全屏编辑器
| 按键 | 功能 |
|---|---|
| `Escape` ×2 | 关闭编辑器 |
| `Ctrl+S` | 保存并关闭 |
| `Ctrl+Q` | 不保存关闭 |

### 多标签（Yazi 风格）
| 按键 | 功能 |
|---|---|
| `t n` | 新建标签 |
| `t x` | 关闭标签 |
| `t 1`–`9` | 按数字切换到对应标签 |
| `t [` / `t ]` | 上一个 / 下一个标签 |

### 内容缩放
| 按键 | 功能 |
|---|---|
| `Ctrl+=` | 放大 |
| `Ctrl+-` | 缩小 |
| `Ctrl+0` | 重置缩放 |
| `Ctrl+滚轮` | 放大/缩小 |

## 命令面板

| 命令 | 说明 |
|---|---|
| `:cd <路径>` | 切换目录（`.` 当前目录、`..` 上一级、无参数为家目录） |
| `:e <路径>` | 打开文件或目录 |
| `:ratio X:Y:Z` | 设置三栏宽度比例 |
| `Tab` | 自动补全路径（`cd`：仅目录，`e`：文件 + 目录） |

## Shell 集成

终端支持 OSC 133（命令追踪）和 OSC 7（目录同步）。要启用可视化提示（退出码、当前目录），需安装 Prompt 框架：

**PowerShell**（Oh My Posh）：
```powershell
winget install JanDeDobbeleer.OhMyPosh
# 添加到 $PROFILE：
oh-my-posh init pwsh --config "$env:POSH_THEMES_PATH\paradox.omp.json" | Invoke-Expression
```

**Bash / Git Bash**（Starship）：
```bash
curl -sS https://starship.rs/install.sh | sh
# 添加到 ~/.bashrc：
eval "$(starship init bash)"
```

## 技术栈

| 层级 | 技术 |
|---|---|
| 桌面框架 | Tauri 2.0 |
| 前端 | SvelteKit + Svelte 5 (runes) + TypeScript |
| 终端 | ConPTY + xterm.js |
| 代码编辑器 | CodeMirror 6 + @replit/codemirror-vim |
| PDF | pdfjs-dist + pdfium-render |
| 图片处理 | turbojpeg + fast_image_resize |
| Markdown | markdown-it + KaTeX + Mermaid |
| 语法高亮 | Shiki |

## 项目结构

```
src/                         # Svelte 前端
  lib/components/            # UI 组件
  lib/previewers/            # 文件预览系统（文本、图片、PDF、视频等）
  lib/stores/                # 状态管理
  routes/                    # SvelteKit 路由
src-tauri/                   # Rust 后端
  src/lib.rs                 # Tauri 命令（文件操作、终端、Neovim）
  src/terminal/mod.rs        # ConPTY 终端实现
  src/neovim/mod.rs          # 嵌入式 Neovim（msgpack-rpc）
```

## License

MIT
