# Python Completion

Python 第三方库 API 发现与 CodeMirror 补全集成。

## Requirements

### Requirement: 自动发现已安装的第三方包

Rust 后端 SHALL 通过 `pip list --format=json` 获取当前 Python 环境中已安装的第三方包名称和版本号，过滤掉标准库和本地包。

#### Scenario: 扫描安装的包

- **GIVEN** 用户的 Python 环境已安装 numpy、pandas、requests
- **WHEN** 前端调用 `scan_python_completions` 命令
- **THEN** 返回包含 `numpy`、`pandas`、`requests` 的包列表（含版本号）

### Requirement: 提取包的公开 API 和函数签名

Rust 后端 SHALL 对每个第三方包，通过 Python 子进程执行 `import + dir() + inspect.signature()` 提取公开成员（非 `_` 前缀），包括函数名、签名和类型信息，以 JSON 格式返回。

#### Scenario: 提取 numpy 的公开 API

- **GIVEN** 包名为 `numpy`
- **WHEN** 后端执行 API 提取
- **THEN** 返回 JSON 数组，包含 `{"name": "array", "signature": "(object, dtype=None, ...)", "type": "builtin_function_or_method"}` 等条目

### Requirement: API 缓存到本地磁盘

后端 SHALL 将提取的包 API 数据以 JSON 文件缓存到 `%APPDATA%/wind/python-completions/` 目录，缓存 key 为 `{python_exe_hash}_{package_name}_{package_version}.json`。后续请求 SHALL 优先读取缓存，仅在缓存不存在或版本不匹配时重新提取。

#### Scenario: 缓存命中

- **GIVEN** numpy 1.26.0 的 API 缓存文件已存在
- **WHEN** 前端请求 numpy 的 API
- **THEN** 直接返回缓存数据，不执行 Python 子进程

### Requirement: import 语句解析

前端 CompletionSource SHALL 解析当前文档的 import 语句，构建 alias → package_name 的映射表。

- `import numpy` → `{ numpy: "numpy" }`
- `import numpy as np` → `{ np: "numpy" }`
- `from pandas import DataFrame` → `{ DataFrame: "pandas" }`

#### Scenario: 解析 import 别名

- **GIVEN** Python 文件顶部有 `import numpy as np`
- **WHEN** 用户输入 `np.` 并触发补全
- **THEN** 补全列表显示 numpy 包的公开 API

### Requirement: CodeMirror CompletionSource 补全

前端 SHALL 在 Python 文件中注册自定义 CompletionSource，当光标前的 identifier 匹配已知 import 别名时，调用后端获取对应包的 API 并以补全列表形式展示。补全条目 SHALL 显示函数签名。

#### Scenario: numpy 函数补全

- **GIVEN** Python 文件中有 `import numpy`
- **WHEN** 用户输入 `numpy.` 并触发 autocomplete
- **THEN** 补全列表显示 numpy 的函数（array、zeros、linspace 等），含签名
