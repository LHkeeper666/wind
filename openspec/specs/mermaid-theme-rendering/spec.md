## Purpose

确保 Markdown 预览中的 Mermaid 图表始终使用与应用主题一致的配色，并在主题切换后及时更新。

## Requirements

### Requirement: Mermaid 图表匹配应用主题
系统 SHALL 根据应用当前 `data-theme` 为 Markdown 预览中的 Mermaid 图表选择对应主题；未显式设为浅色时 SHALL 使用深色主题。

#### Scenario: 深色模式首次打开图表
- **WHEN** 用户在深色模式下打开含 Mermaid 图表的 Markdown 文件
- **THEN** 图表使用 Mermaid 深色主题渲染，连线与背景可辨识

#### Scenario: 浅色模式首次打开图表
- **WHEN** 用户在浅色模式下打开含 Mermaid 图表的 Markdown 文件
- **THEN** 图表使用 Mermaid 浅色主题渲染

### Requirement: 主题切换更新已有图表
系统 SHALL 在应用主题切换后更新已渲染的 Mermaid 图表，包括保留在其他预览标签中的图表，并 SHALL 防止过期异步结果覆盖当前主题。

#### Scenario: 当前预览切换主题
- **WHEN** 用户在显示 Mermaid 图表的 Markdown 预览中切换深浅主题
- **THEN** 图表按新主题重新渲染，预览滚动位置保持不变

#### Scenario: 非活动标签中的图表
- **WHEN** 用户切换主题后返回先前保留的 Markdown 预览标签
- **THEN** 该标签中的 Mermaid 图表使用当前主题

#### Scenario: 快速连续切换
- **WHEN** 用户在 Mermaid 渲染尚未完成时再次切换主题
- **THEN** 最终显示的图表与最新主题一致

### Requirement: Mermaid 缓存按主题隔离
系统 SHALL 将主题作为 Mermaid SVG 缓存键的一部分，且 SHALL 仅复用与当前主题一致的 SVG。

#### Scenario: 同主题缓存命中
- **WHEN** 同一 Mermaid 源码在相同主题下再次预览
- **THEN** 系统可以复用该主题的缓存 SVG

#### Scenario: 切换主题后再次预览
- **WHEN** 同一 Mermaid 源码曾在另一主题下缓存，用户在当前主题下预览
- **THEN** 系统不得复用另一主题的 SVG
