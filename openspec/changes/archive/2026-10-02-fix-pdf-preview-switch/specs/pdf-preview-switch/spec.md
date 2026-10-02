## ADDED Requirements

### Requirement: PDF preview SHALL update when selected file changes

当用户在目录面板中选中不同的 PDF 文件时，预览面板 SHALL 显示新选中 PDF 的内容，而非保留旧 PDF 的渲染结果。

#### Scenario: Switch between PDFs with same page count

- **WHEN** 用户选中 PDF A（5 页），等待预览渲染完成
- **AND** 用户选中 PDF B（也是 5 页）
- **THEN** 预览面板 SHALL 显示 PDF B 的内容，不残留 PDF A 的 tile canvas

#### Scenario: Switch between PDFs with different page count

- **WHEN** 用户选中 PDF A（3 页），等待预览渲染完成
- **AND** 用户选中 PDF B（10 页）
- **THEN** 预览面板 SHALL 显示 PDF B 的 10 页内容

#### Scenario: Rapid PDF switching

- **WHEN** 用户快速连续选中 PDF A、PDF B、PDF C
- **THEN** 预览面板 SHALL 最终显示 PDF C 的内容，不出现混合渲染结果