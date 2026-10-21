## Context

比例命令目前在 `CommandPalette.svelte` 内联解析，只识别 `:`，并用 `p > 0` 拒绝 0；布局切换在 `keyboard-shortcuts.svelte.ts` 与 `PanelLayout.svelte` 各有一份 `togglePreviewLayout`，底层由 `layout.ts` 的 `expandPreview/collapsePreview` 和 `originalRatios` 支撑。`columnWidths` 把比例换算成百分比后直接写入 grid 的 `fr` 轨道，因此 0 值在渲染层已经可用（现有两列模式就是 `[0,1,4]`）。

本 change 只调整命令语法、布局状态机和焦点行为，不引入可配置布局系统。

## Goals / Non-Goals

**Goals:**

- 支持 `:` 与 `.` 两种分隔符，语义完全等价。
- 用同一套模型表达三列、两列、一列布局，并让 `:ratio 0:0:1`、`:ratio preview` 与快捷键结果一致。
- 循环切换依赖可见列数而不是精确数值，非常规零位（如 `1:0:4`）也能自然参与循环。
- 0 宽列不会被键盘焦点命中，避免焦点落在不可见面板。

**Non-Goals:**

- 不支持小数、负数、百分比、1 段或 2 段简写。
- 不记忆用户自定义的两列/一列比例；循环始终回到固定预设。
- 不新增单个面板放大、上下分栏或用户可配置预设。
- 不修改 `FullscreenEditor`；该组件由活跃 change `remove-fullscreen-text-editor` 退役。
- 不调整比例持久化字段的结构；继续使用 `columnRatios`、`originalRatios`、`previewExpanded`。

## Decisions

### 1. 解析规则

新增 `src/lib/utils/ratio.ts`，导出纯函数供命令面板和测试使用：

```
parseRatioCommand(input): 
  | { kind: 'preset'; preset: 'default' | 'dual' | 'preview' }
  | { kind: 'ratios'; ratios: [number, number, number] }
  | { kind: 'error'; message: string }
```

规则：

- `ratio` → `default`；`ratio default` → `default`；`ratio dual` → `dual`；`ratio preview` → `preview`。
- 数值形式使用正则 `^(\d+)([:.])(\d+)\2(\d+)$`，捕获同一分隔符并要求三次出现一致。
  - 该约束使 `1:2.2` 因分隔符混用而失败，避免被误读为 `1:2:2`。
- 每段必须是非负整数，`a + b + c > 0`。
- `a === 0 && b === 0` 时归一化为 `[0, 0, 1]`。
- 其他零位组合保留原值，例如 `0:2:1`、`1:0:4`、`0:1:0`。

非法输入统一返回 `error`，命令面板显示 toast：`Invalid ratio. Usage: :ratio X:Y:Z`，布局保持不变。

### 2. 布局模式由可见列数推导

不新增持久化字段。定义 `visibleColumnCount(ratios)`：

| 非零列数 | 模式 | 循环下一态 |
| --- | --- | --- |
| 3 | 三列 | `[0,1,4]` |
| 2 | 两列 | `[0,0,1]` |
| 1 | 一列 | 最近的三列比例 |

`previewExpanded` 继续保留，值恒等于“可见列数 < 3”，现有依赖它的自动打开文件、TOC 焦点和按钮样式无需重写。

### 3. 三列快照与循环

- `setRatios()` 在以下两种情况更新 `originalRatios`：
  - 当前布局有 3 个可见列时，记录本次比例（覆盖拖拽调整或 `:ratio` 设置）；
  - 从 3 个可见列切换到更少列时，先记录切换前的比例。
- `cycleLayout()`：
  - 3 列 → `setRatios([0, 1, 4])`
  - 2 列 → `setRatios([0, 0, 1])`
  - 1 列 → `setRatios(originalRatios)`；若快照不是 3 列布局则回退 `[1, 1, 3]`
- `expandPreview()` 收敛为进入两列预设，`collapsePreview()` 收敛为恢复三列快照，保留现有调用方。

两列 `[0,1,4]` 和一列 `[0,0,1]` 为固定预设，不按级别保存自定义值。用户执行 `:ratio 0:2:1` 后循环离开再回到两列，结果是 `[0,1,4]`。

### 4. 隐藏列的焦点处理

- 新增可见性判断：某列的 ratio 大于 0 才可聚焦。
- `handleSwitchPanel` 在 parent/current/preview 之间按方向查找下一个可见列；方向尽头没有可见列时保持不动。
- 布局变更后若 `activeColumn` 已不可见，按 `current → preview → parent` 的优先级选择最近可见列并调用 `focusPanel`。
- 标签页恢复时执行同样的校正，避免恢复出“焦点在 0 宽列”的状态。
- 现有两处 `handleSwitchPanel` 实现保持行为一致；本 change 不强制合并两份实现。

### 5. 两处布局入口收敛

- `keyboard-shortcuts.svelte.ts` 的 `togglePreviewLayout` 改名为 `cyclePreviewLayout` 并调用 `layout.cycleLayout()`。
- `PanelLayout.svelte` 的按钮回调改为同一 store 方法。
- `PreviewEditor` 按钮标题从 Expand/Collapse 改为描述循环的文案。
- `Ctrl+W m` 已存在于 `VimOverlay` 的放行列表；`FullscreenEditor` 不修改。PDF TOC 聚焦时是否放行 `Ctrl+W m` 按验证结果处理，不扩大到其他按键。

## Risks / Trade-offs

- **自定义两列比例不保留** → 符合本次简化目标；如需保留再扩展为分级快照。
- **非常规零位产生不可见面板** → 焦点规则已覆盖；工具栏或状态栏仍可能显示被隐藏的列名，属于可接受的既有 UI 行为。
- **`previewExpanded` 与比例短暂不同步** → 所有写入统一经过 `setRatios` 与 `cycleLayout`，避免直接赋值；标签页恢复时以 `columnRatios` 为准重新推导。
- **0 宽列旁仍保留 4px 分隔轨道** → 与现有两列模式一致，不在本 change 中调整 grid 模板。
