## Context

Wind 是一个 vim 驱动的 Windows 文件工作站，当前图片预览只在预览区以缩略图方式展示。用户需要全屏查看原图，并在目录中的图片间快速切换。

现有架构关键点：
- 预览系统基于 `Previewer` 接口（match/render/dispose），`ImagePreviewer` 负责图片渲染
- `FullscreenEditor.svelte` 是全屏覆盖层的现有模式（fixed + z-index:1000 + 半透明背景）
- 键盘分发采用捕获阶段全局监听 → 面板本地 `onkeydown` 的分层架构
- `PanelLayout` 持有 `currentDirectoryPanel` 引用，可通过方法获取面板数据
- 文件 I/O 统一通过 Tauri `invoke()` 调用 Rust 命令

## Goals / Non-Goals

**Goals:**
- 新增独立的全屏图片查看器组件，复用 `E` 快捷键打开
- 支持 `j/k` 切换目录中的图片文件
- 支持 `h/l` 缩放、`Ctrl+j/k/h/l` 平移
- 默认 fit-to-screen 模式，最小缩放 0.1

**Non-Goals:**
- 不做图片编辑功能
- 不做图片拖拽排序
- 不做多图对比/并排查看
- 不修改现有 `ImagePreviewer` 的缩略图预览逻辑

## Decisions

### 1. 组件独立性：新建 `FullscreenImageViewer.svelte`，不复用 `FullscreenEditor`

**决定**：独立组件，与 FullscreenEditor 同级。

**理由**：两者职责完全不同 — 编辑器基于 CodeMirror，查看器基于图片缩放/平移。统一为一个组件会增加耦合度，且共享代码极少（仅 overlay 样式）。

**替代方案**：提取公共 `FullscreenOverlay` 壳组件，内部根据类型渲染编辑器或查看器。被否决，因为壳组件的抽象收益不大。

### 2. E 键路由：在 `handleFullscreenEditor` 中按文件类型分支

**决定**：在 `PanelLayout.handleFullscreenEditor()` 中判断 `isImageFile(selectedFile)`，图片走 viewer，文本走 editor。

**理由**：一处判断，两处入口（DirectoryPanel 和 PreviewEditor 的 E 键）都汇聚到这个函数，无需修改两个面板的键盘处理。

### 3. 图片列表获取：DirectoryPanel 暴露 `getImageFiles()` 方法

**决定**：在 DirectoryPanel 上添加 `export function getImageFiles()`，返回当前目录中过滤后的图片文件列表。

**理由**：DirectoryPanel 已经持有 `files` 数组和目录加载逻辑，不需要重复加载。通过 `bind:this` 引用调用即可。

**替代方案**：在 viewer 中独立加载目录。被否决，因为会重复一次 `invoke('read_directory')` 调用，且需要传入 `currentPath`。

### 4. 缩放模型：CSS transform + JS 状态

**决定**：
- `scale` 状态变量，初始为 fit-to-screen 比例
- `translateX/translateY` 状态变量，初始为 0
- CSS: `transform: scale(${scale}) translate(${tx}px, ${ty}px)`
- `transform-origin: center center` 以图片中心为锚点

**理由**：纯 CSS transform 性能好，GPU 加速。与现有 ImagePreviewer 的 `object-fit: contain` 思路一致。

**替代方案**：使用 canvas 绘制。被否决，复杂度高且对 GIF 动画不友好。

### 5. 图片加载：复用 `read_binary_file` Tauri 命令

**决定**：全屏查看器直接调用 `invoke('read_binary_file')` 加载原图，不走缩略图路径。

**理由**：全屏查看的目的是看原图细节，缩略图没有意义。

## Risks / Trade-offs

- **大图片内存** → 原图全量加载到内存，超大图片（如 100MB+ PSD）可能 OOM。缓解：当前只支持 png/jpg/jpeg/gif/svg/webp/bmp/ico，这些格式通常不会极端大。
- **GIF 动画** → blob URL + `<img>` 标签天然支持 GIF 动画帧，无需特殊处理。
- **缩放边界** → scale 最小 0.1，最大不设上限（用户可以无限放大看像素）。不自动关闭。
