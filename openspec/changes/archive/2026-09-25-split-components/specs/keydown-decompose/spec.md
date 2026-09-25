# Spec: DirectoryPanel handleKeydown 内部拆分

## 目标

将 DirectoryPanel 的 `handleKeydown`（425 行）拆分为多个辅助函数，不涉及组件边界变化。

## 拆分方案

```typescript
function handleKeydown(event: KeyboardEvent): void {
  const key = event.key;
  const ctrl = event.ctrlKey || event.metaKey;

  // 优先处理：搜索模式下的按键
  if (isSearchModalOpen) return;

  // 优先处理：输入对话框模式下的按键
  if (inputVisible) return;

  // 优先处理：删除确认模式下的按键
  if (showDeleteConfirm) return;

  // 归档模式专用按键
  if (isArchiveMode) {
    handleArchiveKey(event);
    return;
  }

  // 项目树模式专用按键
  if (projectMode) {
    if (handleTreeKey(event)) return;
  }

  // 排序/过滤按键
  if (handleSortFilterKey(event)) return;

  // 文件操作按键
  if (handleOperationKey(event)) return;

  // 导航按键（兜底）
  handleNavigationKey(event);
}
```

### `handleNavigationKey(event)`
处理：j/k/gg/G/Enter/h/l/H/M/L/Tab/Backspace/Escape

### `handleOperationKey(event)`
处理：d/D/y/x/p/r/a/A/i/I/o/O/c/C（文件删除/复制/剪切/粘贴/重命名/新建等）

### `handleSortFilterKey(event)`
处理：s（排序切换）/S（排序方向）/o（目录优先）/f（过滤）/F（清除过滤）

### `handleTreeKey(event)`
处理：项目树专用的展开/折叠/递归展开等按键。返回 `boolean` 表示是否已处理。

### `handleArchiveKey(event)`
处理：归档模式下的提取/删除/重命名/退出等按键。

## 关键约束

1. 按键优先级必须与现有行为完全一致
2. `sortPrefixPending` 的双击排序逻辑必须在拆分后保持正确
3. 归档模式和普通模式的按键不能互相干扰
4. 项目树模式下部分按键行为不同（如 Enter 展开而非打开）

## 验证标准

1. 所有现有键盘快捷键行为不变
2. 归档模式下的快捷键正确
3. 项目树模式下的快捷键正确
4. 排序前缀双击（如 `ss` 按名称排序）正常工作
5. 输入对话框模式下键盘事件正确拦截