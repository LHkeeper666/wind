# preview-navigation-commands Specification

## Purpose

定义主预览中通过命令面板控制 Markdown/PDF 大纲与跳转 PDF 页码的行为。

## Requirements

### Requirement: 主预览目录命令

命令面板 SHALL 在当前主预览为 Markdown 或 PDF 且存在大纲时，支持 `:toc` 切换大纲可见性，支持 `:toc open` 显示大纲，支持 `:toc close` 隐藏大纲。显式开关 MUST 是幂等的。

#### Scenario: Markdown 中切换目录
- **WHEN** 当前主预览为有标题大纲的 Markdown，用户执行 `:toc`
- **THEN** 标题大纲的可见性反转

#### Scenario: PDF 中切换目录
- **WHEN** 当前主预览为有书签大纲的 PDF，用户执行 `:toc`
- **THEN** 书签大纲的可见性反转

#### Scenario: 显式打开和关闭目录
- **WHEN** 当前主预览有大纲，用户执行 `:toc open` 或 `:toc close`
- **THEN** 大纲分别显示或隐藏
- **AND** 重复相同命令不反转其状态

#### Scenario: 当前预览没有可用目录
- **WHEN** 当前主预览没有文件、不是 Markdown/PDF，或文件没有可用大纲，用户执行目录命令
- **THEN** 系统显示说明原因的提示，且不改变其他预览的目录状态

### Requirement: PDF 页码跳转命令

命令面板 SHALL 支持 `:page N` 跳转当前主预览 PDF 的第 N 页，并 MUST 保留 `:N` 的相同语义。页码 MUST 为从 1 开始且不超过 PDF 总页数的整数。

#### Scenario: 使用命名命令跳页
- **WHEN** 当前主预览为已加载的 PDF，用户执行 `:page 12`，且文件至少有 12 页
- **THEN** 预览滚动到第 12 页

#### Scenario: 使用已有快捷命令跳页
- **WHEN** 当前主预览为已加载的 PDF，用户执行 `:12`，且文件至少有 12 页
- **THEN** 预览滚动到第 12 页

#### Scenario: 页码无效
- **WHEN** 用户输入零、非整数或超过当前 PDF 总页数的页码
- **THEN** 系统显示有效页码范围提示，且预览位置不变

#### Scenario: 当前预览不是 PDF
- **WHEN** 当前主预览不是已加载 PDF，用户执行 PDF 页码命令
- **THEN** 系统提示该命令仅适用于 PDF 预览，且不改变预览位置

### Requirement: 命令可发现性

命令帮助 SHALL 列出 `:toc`、`:toc open`、`:toc close`、`:page N` 和 `:N` 的用途与适用预览类型。

#### Scenario: 查看命令帮助
- **WHEN** 用户打开命令帮助
- **THEN** 用户可以看到目录控制与 PDF 跳页命令的语法和适用范围
