# editor-viewport-cursor Specification

## Purpose
TBD - created by archiving change editor-cursor-at-viewport. Update Purpose after archive.
## Requirements
### Requirement: Editor initializes at viewport position

系统 SHALL 在预览视图进入编辑器模式时将光标定位到当前视口可见的第一行，而非文件开头。

#### Scenario: Inline editor from scrolled preview
- **WHEN** 用户在预览面板用 j/k 向下滚动若干行，然后按 e 键
- **THEN** 编辑器光标定位到与预览视口顶部对应的行号附近

#### Scenario: Fullscreen editor from scrolled preview
- **WHEN** 用户在预览面板用 j/k 向下滚动若干行，然后按 E 键
- **THEN** 全屏编辑器光标定位到与预览视口顶部对应的行号附近

#### Scenario: Editor from unscrolled preview
- **WHEN** 用户在预览视口顶部（未滚动）按 e 键
- **THEN** 编辑器光标定位在第 1 行（文件开头），行为与修改前一致

### Requirement: Line calculation uses actual DOM line height

系统 SHALL 通过预览区域实际 DOM 元素的计算样式获取行高，而非使用硬编码值。

#### Scenario: Normal zoom level
- **WHEN** 缩放级别为 100%
- **THEN** 行高从 previewContainer 内第一个 code 或 pre 元素的 computed lineHeight 获取

#### Scenario: Zoomed viewport
- **WHEN** 用户调整了面板缩放级别（Ctrl+=）
- **THEN** 行高计算随缩放比例自动适配，光标定位仍然准确

