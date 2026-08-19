## Context

`PreviewEditor` 当前只有一个 `EditorView`。Tab 切换时先缓存内容和光标、将 mode 降为 `global-normal`，恢复目标 Tab 后销毁旧实例，并在动画帧中初始化新的 CodeMirror 实例。初始化包含 Vim 扩展、语言扩展、补全、主题、行号和观察器，全部运行在 WebView 主线程。

`PanelLayout.restoreTabContent()` 也在同一个动画帧中发起目录同步并恢复 DOM 焦点。窗口回焦时 `focusPanel()` 再额外等待一个动画帧。交互焦点、编辑器重建和可延后的目录刷新相互竞争，导致输入事件排队。

## Goals / Non-Goals

**Goals:**

- 已进入编辑模式的 Tab 之间切换时不销毁或重建目标 Tab 的 CodeMirror 编辑器。
- 在 Tab 切换和窗口回焦后，让正确的 Vim 输入目标先获得焦点。
- 保持现有内容、未保存状态、光标、滚动位置、Vim normal/insert 模式和 overlay 行为。
- 保持目录版本同步语义，同时使其不阻塞交互焦点恢复。

**Non-Goals:**

- 不为未曾编辑过的 Tab 预创建编辑器。
- 不修改 CodeMirror Vim 键位、文件加载协议、后端命令或 Tab 持久化格式。
- 不保证极大文件首次创建编辑器时没有初始化成本。

## Decisions

### Decision 1: 使用按 Tab 隔离的编辑器会话

为每个进入 `editor-normal` 或 `editor-insert` 的 Tab 建立 `EditorSession`，包含 `EditorView`、专属宿主容器、文件路径和会话级资源。会话容器持续挂载在编辑器区域；激活 Tab 的容器可交互，非激活容器通过 `display: none` 隐藏。切换回已有会话时只切换容器可见性、恢复焦点和必要的 `requestMeasure()`，不重建扩展或文档。

选择该方案而非缓存 `EditorState` 后重建 `EditorView`，因为后者仍要在切换路径创建 DOM、Vim 扩展和视图测量，不能消除交互卡顿。也不保留所有预览 DOM 的可见状态，以免改变当前预览缓存策略与内存边界。

### Decision 2: 显式区分活动会话与后台会话

仅活动 `EditorSession` 参与全局 mode、overlay、剪贴板桥接和 ResizeObserver；后台会话不得接收焦点或共享观察器回调。保存 Tab 状态时从该 Tab 会话读取光标与滚动位置；关闭 Tab 或文件路径失配时销毁对应会话及其容器，避免资源泄漏。

这延续“每 Tab 缓存状态”的语义，但将缓存从序列化快照升级为可复用的前端会话。保留现有缓存快照作为未创建会话、文件失效和关闭编辑器时的回退路径。

### Decision 3: 焦点优先，非关键同步后置

Tab 切换路径在 Svelte DOM 更新完成后立即把焦点交给目标：normal 模式为稳定 overlay，insert 模式为目标 `EditorView`。窗口重获焦点使用同一个无延迟焦点入口，并只在没有有效面板焦点时执行。

目录版本同步移出恢复焦点的动画帧，改为在焦点提交后异步排队。同步仍通过现有协调器、版本比较和错误重试完成，不再处于按键/鼠标首个响应的关键路径。

不使用任意固定 `setTimeout`，因为它只能掩盖竞态并引入设备相关延迟。

### Decision 4: 通过可观测指标验证而非硬编码时限

在开发构建中以 `performance.mark/measure` 包围 Tab 会话激活、焦点提交、目录同步排队和首次会话创建，并在开发控制台输出长于一帧的阶段。验收以“目标焦点在同一交互循环可用、缓存会话切换不调用 `initEditor`”为功能断言；性能数据用于回归检查，不把易受机器影响的毫秒阈值写入运行时代码。

## Risks / Trade-offs

- [Risk] 多个编辑会话增加内存占用 → Mitigation: 只按需创建，Tab 关闭或文件不再匹配时立即销毁，会话数量上限自然受 Tab 数限制。
- [Risk] 隐藏 CodeMirror 容器测量过期 → Mitigation: 激活后调用一次 `requestMeasure()`，但不等待其完成再交付焦点。
- [Risk] 后台会话仍响应全局观察器 → Mitigation: 会话激活时绑定，失活时解绑或在回调中校验活动会话 ID。
- [Risk] 目录同步延后短暂显示旧列表 → Mitigation: 保留版本检查与加载状态；同步只后移调度，不丢弃。
- [Risk] 焦点修改影响 overlay/IME → Mitigation: 复用既有单一焦点所有者与 overlay 稳定 DOM 约束，覆盖 normal、insert 和窗口回焦场景。

## Migration Plan

1. 实现会话容器和会话生命周期，并保留现有单实例路径作为临时回退。
2. 将 Tab 保存/恢复接入会话激活，再移除旧的“切换即销毁重建”分支。
3. 调整焦点与目录同步调度，使用开发性能标记验证。
4. 运行 `npx svelte-check` 并手工验证 Vim Tab 切换、窗口回焦、关闭 Tab 和文件切换。

回滚策略为恢复单一 `EditorView` 生命周期；不涉及数据迁移。

## Open Questions

- 需要在实现前确认最大同时打开 Tab 数是否有产品级上限；当前按 Tab 缓存能接受典型桌面工作负载。
