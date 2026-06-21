## 1. 基础设施准备

- [x] 1.1 安装 CodeMirror 6 依赖（codemirror, @codemirror/lang-javascript, @codemirror/lang-python, @codemirror/lang-html, @codemirror/lang-css, @codemirror/lang-json, @codemirror/theme-one-dark）
- [x] 1.2 创建 layout store（src/lib/stores/layout.ts），管理三列比例、当前路径、选中文件等状态
- [x] 1.3 创建 DirectoryPanel.svelte 组件骨架
- [x] 1.4 创建 PreviewEditor.svelte 组件骨架
- [x] 1.5 创建 FullscreenEditor.svelte 组件骨架
- [x] 1.6 创建 FloatingTerminal.svelte 组件骨架

## 2. 三列布局重构

- [x] 2.1 重构 PanelLayout.svelte，使用 CSS Grid 实现三列布局
- [x] 2.2 实现列比例调整（拖拽分隔线）
- [x] 2.3 实现列比例命令调整（:ratio 1:2:2）
- [x] 2.4 实现列焦点切换（h/l 键）
- [x] 2.5 添加列面板标题（Parent Directory, Current Directory, Preview/Editor）

## 3. 目录面板实现

- [x] 3.1 实现 DirectoryPanel 的目录内容显示
- [x] 3.2 实现父目录列（左列）显示父目录内容
- [x] 3.3 实现当前目录列（中列）显示当前目录内容
- [x] 3.4 实现目录联动导航（选中文件夹时更新三列）
- [x] 3.5 实现目录内容缓存
- [x] 3.6 实现文件选择高亮
- [x] 3.7 实现 vim 风格键盘导航（j/k/gg/G）
- [x] 3.8 实现 Enter 键打开文件/进入目录
- [x] 3.9 实现 R 键刷新目录

## 4. 预览/编辑器实现

- [x] 4.1 集成 PreviewRouter 到 PreviewEditor
- [x] 4.2 实现预览模式（Shiki 语法高亮）
- [x] 4.3 集成 CodeMirror 6 编辑器
- [x] 4.4 实现 Enter 键切换到编辑模式
- [x] 4.5 实现 Escape 键退出编辑模式
- [x] 4.6 实现文件修改状态跟踪
- [x] 4.7 实现 Ctrl+S 保存文件
- [x] 4.8 实现语言自动检测和语法高亮

## 5. 全屏编辑器实现

- [x] 5.1 实现 FullscreenEditor 浮层（覆盖整个窗口）
- [x] 5.2 实现 e 键打开全屏编辑器
- [x] 5.3 实现两次 Escape 关闭全屏编辑器
- [x] 5.4 实现全屏编辑器与右列的状态同步
- [x] 5.5 实现关闭按钮
- [x] 5.6 实现 Ctrl+S 保存并关闭

## 6. 浮动终端实现

- [x] 6.1 重构 TerminalPanel 为 FloatingTerminal
- [x] 6.2 实现默认隐藏
- [x] 6.3 实现 Ctrl+` 切换显示/隐藏
- [x] 6.4 实现从底部滑出的动画
- [x] 6.5 实现终端高度调整（拖拽）
- [x] 6.6 实现 shell 切换按钮（PowerShell, CMD, Git Bash）
- [x] 6.7 实现 vim 模式切换（Insert/Normal）
- [x] 6.8 实现焦点管理（打开时自动聚焦，关闭时恢复焦点）

## 7. 集成和优化

- [x] 7.1 更新 Status Bar 显示（当前路径、活动面板）
- [x] 7.2 更新命令面板（添加新命令）
- [x] 7.3 更新文件搜索（Ctrl+P）集成到新布局
- [x] 7.4 处理旧组件（FileTreePanel, EditorPanel, TerminalPanel）的废弃
- [x] 7.5 测试所有快捷键
- [x] 7.6 性能优化（目录缓存、CodeMirror 按需加载）
- [x] 7.7 更新 CLAUDE.md 文档
