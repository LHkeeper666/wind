## 1. 后端：添加搜索命令

- [x] 1.1 在 Cargo.toml 添加 regex 和 walkdir 依赖
- [x] 1.2 实现 `is_rg_available()` 检测函数
- [x] 1.3 实现 `search_with_rg()` 函数（调用 rg --files --regex）
- [x] 1.4 实现 `search_with_rust()` 函数（walkdir + regex 递归）
- [x] 1.5 实现 `search_files` Tauri 命令（自动选择 rg 或原生实现）

## 2. 前端：创建 SearchModal 组件

- [x] 2.1 创建 `src/lib/components/SearchModal.svelte` 基本结构
- [x] 2.2 实现搜索输入框（支持正则、实时验证、300ms 防抖）
- [x] 2.3 实现结果列表显示（相对路径、最多 50 条）
- [x] 2.4 实现搜索进度显示
- [x] 2.5 实现 j/k 键盘导航
- [x] 2.6 实现 Enter 选中文件并打开
- [x] 2.7 实现 Escape 关闭浮窗
- [x] 2.8 实现 rg 未安装提示（右下角 toast）

## 3. 集成：修改 FileTreePanel

- [x] 3.1 移除 FileTreePanel 内联搜索相关代码
- [x] 3.2 添加搜索框触发器（鼠标可点击）
- [x] 3.3 集成 SearchModal 组件
- [x] 3.4 `/` 快捷键触发浮窗
