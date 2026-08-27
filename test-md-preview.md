---
title: Test Document
date: 2026-08-27
author: LHkeeper
tags: [markdown, preview, test]
status: draft
---

# Frontmatter 测试

这段文字上方应该显示灰色小字的元数据行，而不是 --- 水平线。

## 缩进表格测试

### 2空格缩进表格

  | Name | Age | Role |
  |------|-----|------|
  | Alice | 25 | Developer |
  | Bob | 30 | Designer |

### 4空格缩进表格

    | Item | Price | Qty |
    |------|-------|-----|
    | Apple | 5.00 | 10 |
    | Banana | 3.50 | 20 |

### 1 tab 缩进表格

	| Column A | Column B | Column C |
	|----------|----------|----------|
	| Value 1 | Value 2 | Value 3 |
	| Value 4 | Value 5 | Value 6 |

### 对比：普通表格（无缩进）

| Feature | Status |
|---------|--------|
| Frontmatter | ✓ |
| Indented tables | ✓ |

## 普通内容

这段文字应该正常渲染，上面的表格应该保持视觉缩进。