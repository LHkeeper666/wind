## Context

Wind 是一个基于 Tauri 2.0 的 vim 风格文件工作站。当前架构：
- 前端 Svelte 5 + runes，后端 Rust
- Tab 系统通过 `stores/tabs.ts` 管理，每个 tab 独立保存路径、光标、终端状态
- 文件操作后端已有 `copy_file`、`delete_file`、`rename_file`、`create_file`
- DirectoryPanel 是单选模式，通过 `selectedIndex` 追踪
- PanelLayout 是全局键盘事件入口，处理 t-prefix、Ctrl+W-prefix 等

## Goals / Non-Goals

**Goals:**
- 实现 yazi 风格的 y/x/p 跨 tab 文件复制/剪切
- 支持多选（Space toggle + v 全选）
- 同盘移动使用原子 rename，跨盘使用 copy+delete
- 粘贴冲突时弹出自定义 modal 确认
- 状态栏显示 clipboard 状态，支持 `:clip`/`:clear` 命令

**Non-Goals:**
- 不实现拖拽文件操作
- 不实现跨窗口（不同 Wind 实例）文件传输
- 不实现进度条（大文件复制时的进度反馈留到后续）
- 不实现撤销操作（undo）

## Decisions

### 1. Clipboard Store 设计

**选择**: 独立的 Svelte store（`stores/clipboard.ts`），全局单例

**理由**: clipboard 是跨 tab 共享状态，不属于任何单个 tab。用独立 store 保持关注点分离，tabs store 不需要感知 clipboard。

**替代方案**: 把 clipboard 放进 tabs store → 增加耦合，不自然

```typescript
// stores/clipboard.ts
interface ClipboardEntry {
  path: string;
  name: string;
  is_dir: boolean;
}

interface ClipboardState {
  entries: ClipboardEntry[];
  operation: 'copy' | 'cut' | null;
}
```

### 2. 多选状态管理

**选择**: DirectoryPanel 内部维护 `selectedPaths: Set<string>`，通过 `$effect` 同步到 clipboard

**理由**: 多选是面板级状态，不需要全局共享。y/x 时将选中项推入 clipboard store。

**选中逻辑**:
- 无多选时 y/x → 取光标所在文件
- 有多选时 y/x → 取所有选中文件
- 操作后清空多选状态

### 3. Rust move_file 实现

**选择**: 新增 `move_file` Tauri command，内部判断同盘/跨盘

```rust
fn move_file(source: String, destination: String) -> Result<(), String> {
    let src_drive = source[..1].to_uppercase();
    let dst_drive = destination[..1].to_uppercase();
    if src_drive == dst_drive {
        fs::rename(&source, &destination)  // 原子操作
    } else {
        copy_file(&source, &destination)?;  // 复制
        delete_file(&source)                 // 删除原文件
    }
}
```

**理由**: `fs::rename` 在同盘是原子的，速度快。跨盘 rename 在 Windows 上会失败，必须 copy+delete。

### 4. 粘贴冲突处理

**选择**: 自定义 modal 组件（`ConfirmModal.svelte`），支持三个选项

**流程**:
```
paste 逐个文件:
  ├─ 目标不存在 → 直接执行
  └─ 目标已存在 → 弹 modal
      ├─ [O]verwrite → 删除目标后执行
      ├─ [S]kip → 跳过
      └─ [A]bort → 中止剩余
```

**理由**: 原生 `confirm()` 与应用主题不一致。自定义 modal 保持 UI 统一。

### 5. 快捷键绑定位置

**选择**: y/x/Space/v 在 DirectoryPanel.svelte 的 `handleKeydown` 中处理，p 在 PanelLayout.svelte 的全局键盘处理中处理

**理由**: y/x/Space/v 只在目录面板有意义。p 需要在全局处理，因为用户可能在切换 tab 后（焦点在 current panel）直接按 p。

### 6. 剪切文件视觉标记

**选择**: 被剪切的文件添加 `cut` CSS class，样式为半透明 + 左侧 `x` 标记

**理由**: 简单直观，不引入额外的 DOM 结构。paste 执行后自动清除标记。

## Risks / Trade-offs

**[大文件粘贴性能]** → 当前不做进度反馈，简单阻塞。后续可改为异步 + 进度条。

**[跨盘剪切失败]** → copy 成功但 delete 失败时，目标会有重复文件。Mitigation: 先 copy，验证成功后再 delete，delete 失败时保留源文件并提示。

**[剪切标记持久性]** → 剪切标记只存在于前端内存，刷新页面后丢失。这是可接受的，因为 clipboard 本身就是临时状态。
