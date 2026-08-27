## Context

Markdown 预览由 `MarkdownPreviewer` 类实现，使用 markdown-it 解析，Shiki 高亮代码，KaTeX 渲染数学公式。当前渲染管线为：

```
text → markdown-it.render() → HTML → 后处理（Shiki/Mermaid/KaTeX）→ DOM
```

两个改进点：
1. **Frontmatter**: 目前无任何处理，YAML 元数据被渲染为 `<hr>` + 普通段落
2. **缩进表格**: 标准 markdown 表格要求行首无空白，缩进表格被当作普通文本

## Goals / Non-Goals

**Goals:**
- YAML frontmatter 解析为灰色小字元数据行，始终显示，不折叠
- 缩进表格（tab 或空格缩进）被正确识别为表格，渲染后保持视觉缩进
- 无 frontmatter 的文件零额外开销

**Non-Goals:**
- 不支持 frontmatter 折叠/展开交互
- 不修改 markdown-it 内部的表格解析规则
- 不处理非表格的缩进块（如缩进代码块保持原行为）

## Decisions

### 1. Frontmatter 解析方案

**选择**: 使用 `gray-matter` npm 包

**替代方案**:
- 手动正则 + 内部 YAML 解析: 减少依赖但增加维护成本，对嵌套 YAML 处理不完善
- 不引入依赖，只做简单 key: value 解析: 无法处理嵌套对象/数组/list 等复杂 frontmatter

**理由**: `gray-matter` 是 Node.js 生态解析 frontmatter 的标准工具，体积小（~2KB gzipped），能正确处理所有 YAML 类型。解析失败时回退到原样渲染，不会崩溃。

### 2. Frontmatter 渲染样式

**选择**: 灰色小字键值对行，底部虚线分隔

```
┌──────────────────────────────────────────────┐
│  title: My Document                          │
│  date: 2024-01-01                            │
│  tags: [wind, terminal]                      │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ │
│  # 正文标题                                   │
└──────────────────────────────────────────────┘
```

- 字号 0.82em，透明度 0.75
- key 使用 accent 色，value 使用 secondary 色
- 嵌套值用 `JSON.stringify` 展平
- 底部虚线分隔正文

### 3. 缩进表格处理方案

**选择**: 预处理阶段检测并 strip 前导空白，渲染后用 `margin-left` 包裹

**流程**:
```
原始文本 → 检测缩进表格块 → 记录缩进量 → strip 空白 → markdown-it 渲染 → 包裹 margin-left div
```

**替代方案**:
- 修改 markdown-it 内部 table 规则: 侵入性强，升级 markdown-it 可能冲突
- 使用第三方插件: 引入不必要的其他功能
- 仅 strip 不还原缩进: 用户要求保持视觉缩进

**理由**: 预处理 + 后处理是最低侵入的方式，不影响 markdown-it 的正常行为。利用已有的 `data-line` 属性做行号映射，无需额外追踪。

### 4. 缩进表格检测算法

```
遍历每行 i:
  if line[i] 匹配 表格行模式 (前导空白 + |...|...|):
    if line[i+1] 匹配 分隔行模式 (前导空白 + |---|...|) 且缩进量相同:
      → 检测到表格块
      → 记录 indentMap[line i] = 缩进宽度
      → 收集所有后续同缩进量的表格行
      → 对所有表格行 strip 前导空白
      → i 跳到表格块之后
  else:
    → 保留原行
```

表格行模式: `/^(\s*)\|([^|\n]+\|)+[^|\n]*\|?\s*$/`
分隔行模式: `/^(\s*)\|([-: |]+)\|?\s*$/`

### 5. 缩进宽度计算

- 空格: 1 空格 = 1ch
- Tab: 1 tab = 4ch（等宽字体标准）
- 最终 `margin-left` 使用 `ch` 单位，在等宽字体下精确对齐

## Risks / Trade-offs

- **gray-matter 解析失败**: try-catch 包裹，解析失败则跳过 frontmatter，原样渲染
- **缩进表格误判**: 严格要求分隔行存在 + 缩进量一致，单行 `|` 代码不会被误判
- **parseHeadings 行号偏移**: frontmatter 被移除后，heading 的 `data-line` 需要加上 frontmatter 行数
- **缩进表格在 blockquote 内**: 不处理，保持原样。仅处理正文中的缩进表格