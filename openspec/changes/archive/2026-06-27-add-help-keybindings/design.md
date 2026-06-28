## Context

Wind 的快捷键分散在 4 个组件中：PanelLayout（全局快捷键）、DirectoryPanel（目录导航和 tab 操作）、FloatingTerminal（终端模式切换）、PreviewEditor（编辑器模式切换）。目前没有统一的地方让用户查看所有可用快捷键和命令。

现有命令面板（`:` 触发）已有 commands 数组和自由输入解析（如 `ratio X:Y:Z`、`cd`、`e`），但缺少帮助入口。

## Goals / Non-Goals

**Goals:**
- 提供声明式的快捷键注册表，集中定义所有快捷键和命令
- 提供应用内帮助面板，展示分组的快捷键列表
- 通过 `F1` 快捷键和 `help` 命令两种方式触发

**Non-Goals:**
- 不重构现有 keydown handler 为声明式分发（各组件上下文判断复杂，统一收益不大）
- 不支持用户自定义快捷键
- 不从代码自动收集快捷键（手动同步注册表即可）

## Decisions

### 1. 注册表只用于展示，不用于分发

**选择**: 注册表 `keybindings.ts` 只存储展示用数据（key、context、description），不参与实际的事件分发。

**理由**: 现有 keydown handler 有复杂的上下文判断——Ctrl+W prefix 状态、IME 兼容（event.code vs event.key）、terminal mode 判断（insert/normal）、fullscreen 状态等。将这些逻辑统一到声明式配置中会大幅增加复杂度，且快捷键已经稳定，手动同步成本低。

**替代方案**: 从注册表驱动事件分发。放弃原因：增加复杂度，收益有限。

### 2. HelpOverlay 作为独立组件，overlay 模式

**选择**: HelpOverlay 用 fixed overlay 覆盖全屏，任意键关闭，不抢焦点。

**理由**: 帮助面板是瞬时查看，不需要交互。overlay 模式最简单，不需要路由或状态管理。

### 3. F1 作为全局触发键

**选择**: F1 键触发帮助面板，在 `handleGlobalKeydown` 中处理。

**理由**: F1 是标准帮助键，不与现有快捷键冲突。

### 4. ratio 预设改为 `ratio X:Y:Z` 格式

**选择**: 将预设 `Set Ratio 1:2:2` 改为 `ratio 1:2:2`，与自由输入命令格式一致。

**理由**: 当前预设叫 `Set Ratio` 但自由输入要打 `ratio`，格式不一致容易混淆。统一为 `ratio X:Y:Z` 格式既做预设又做用户提示。

## Risks / Trade-offs

- **注册表与实际快捷键不同步** → 快捷键已稳定，修改时注意更新注册表即可。后续可考虑加测试校验。
- **HelpOverlay 增加组件数量** → 组件很简单（纯展示），维护成本低。
