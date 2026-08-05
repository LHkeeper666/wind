## Context

Wind 使用 CodeMirror 6 + `@replit/codemirror-vim` 作为编辑器引擎。语言扩展通过 `src/lib/utils/language.ts` 的 `getLanguage()` 函数按扩展名加载。`basicSetup` 包含默认的 `indentUnit`（2 空格）和 `autocompletion`。

Python 补全目前只有 `@codemirror/lang-python` 提供的关键字补全，不支持第三方库。

## Goals / Non-Goals

**Goals:**
- 自动缩进（回车换行后的缩进）统一为 4 空格
- Python 第三方包的顶层 API 出现在 CodeMirror autocomplete 中，含函数签名
- 解析 `import X as Y` / `from X import A, B` 语句，支持别名补全
- 补全缓存到本地磁盘，按 virtual env 隔离，避免重复扫描

**Non-Goals:**
- 不做类型推断（`x = np.array(...)` 后 `x.` 无法补全 np.ndarray 的方法）
- 不做 LSP 集成
- 不修改 Tab 补全行为（保持 Enter 接受补全）
- 不修改引号渲染样式
- 不修改 `t` 前缀快捷键在编辑器内的行为

## Decisions

### 1. 自动缩进：`indentUnit.of('    ')`

**决定**：在 CodeMirror extensions 数组中添加 `indentUnit.of('    ')`，需要从 `@codemirror/language` 导入 `indentUnit`。

**理由**：CodeMirror 6 的默认 indentUnit 为 `"  "`（2 空格），Python 和大多数语言的风格指南推荐 4 空格。设置后同时影响：
- 自动缩进（回车后下一行的缩进）
- `=` 命令（vim 的 `=` 自动格式化/缩进）
- `>` / `<` 缩进命令（当前硬编码 4 空格，保持不变）

**注意**：Tab 键已经是自定义 keymap 硬编码 `'    '`，不受此修改影响。

### 2. Python 补全后端：Rust 子进程调用 Python

**决定**：Rust 后端提供两个 Tauri 命令：

```
Rust 后端
├── scan_python_completions(python_exe: Option<String>)
│   ├── 运行 pip/pip3 list --format=json
│   ├── 过滤本地包（排除 site-packages 之外的）
│   ├── 对每个包运行 python_script.extract_api(package_name)
│   └── 返回 HashMap<String, PackageApi>
│
└── python_script.extract_api(package_name):
    import json, inspect, importlib
    pkg = importlib.import_module(package_name)
    members = []
    for name in dir(pkg):
        if name.startswith('_'): continue
        obj = getattr(pkg, name)
        sig = None
        if callable(obj):
            try: sig = str(inspect.signature(obj))
            except: pass
        members.append({"name": name, "signature": sig, "type": type(obj).__name__})
    print(json.dumps(members))
```

**缓存策略**：
- 缓存目录：`%APPDATA%/wind/python-completions/`
- 缓存 key：`{python_exe_hash}_{package_name}_{package_version}.json`
- 首次打开 Python 文件时异步加载，不阻塞编辑器初始化

**理由**：`dir()` + `inspect.signature()` 是最轻量的方式，不需要外部依赖。扫描 numpy 约 1-2 秒，之后读缓存 < 10ms。

### 3. 前端 CompletionSource：import 语句解析

**决定**：实现自定义 CodeMirror `CompletionSource`，在 `@codemirror/autocomplete` 中注册。

```
补全流程：
1. 用户输入 word. (如 numpy., pd.)
2. CompletionSource 被触发
3. 解析当前文档的 import 语句，构建 alias → package_name 映射
   - import numpy → { numpy: "numpy" }
   - import numpy as np → { np: "numpy" }
   - from pandas import DataFrame → { DataFrame: "pandas" }
4. 识别 word 是否为已知 alias
5. 如果是 → invoke scan_python_completions 获取该包的 API
6. 返回 Completion[] 列表，含 label + detail(signature)
```

**理由**：不做类型推断的前提，import 解析是回答"补全哪些 API"的最直接方式。正则解析 import 语句覆盖 90% 的使用场景。

### 4. 不在后端启动 Python 守护进程

**决定**：每次都通过 `std::process::Command` 调用 `python -c "..."`，不维护常驻 Python 进程。

**理由**：
- 实现简单，无生命周期管理问题
- 首次扫描有延迟，但缓存后不再调用
- 用户可能切换 Python 环境，每次都指定 python_exe 更灵活

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 扫描大型包（numpy/pandas）时 Python 进程启动 + import 耗时长 | 异步执行，不阻塞 UI；完成后写入缓存 |
| Python 命令输出可能无法解析（非标准 pip/path） | 默认 `python`，允许用户通过命令参数指定 python 路径 |
| 包名包含连字符（scikit-learn）在 `import` 中为下划线 | `importlib.import_module(pkg.replace('-', '_'))` |
| 部分包 import 有副作用（如 matplotlib 弹出窗口） | 设置 `MATPLOTLIBBACKEND=Agg` 环境变量 + 超时保护 |
