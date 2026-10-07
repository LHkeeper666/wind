# UTF-16 Text Detection

Provides correct decoding of UTF-16 LE/BE encoded text files in the Rust backend.

## Requirements

### Requirement: UTF-16 LE 文件正确解码
系统 SHALL 识别 UTF-16 LE BOM（`FF FE`）并将文件内容解码为 UTF-8 字符串返回给前端。

#### Scenario: 打开 UTF-16 LE 编码的文本文件
- **WHEN** 用户打开一个以 `FF FE` BOM 开头的 UTF-16 LE 编码文本文件
- **THEN** 系统返回的字符串不包含 `\0` 字节，前端正常显示为文本内容

### Requirement: UTF-16 BE 文件正确解码
系统 SHALL 识别 UTF-16 BE BOM（`FE FF`）并将文件内容解码为 UTF-8 字符串返回给前端。

#### Scenario: 打开 UTF-16 BE 编码的文本文件
- **WHEN** 用户打开一个以 `FE FF` BOM 开头的 UTF-16 BE 编码文本文件
- **THEN** 系统返回的字符串不包含 `\0` 字节，前端正常显示为文本内容

### Requirement: 保持现有编码兼容性
系统 SHALL 保持对 UTF-8、GBK 等现有编码的正确处理。

#### Scenario: 打开 UTF-8 编码文件
- **WHEN** 用户打开一个 UTF-8 编码的文本文件
- **THEN** 系统正常解码并返回文本内容，行为与修改前一致

#### Scenario: 打开 GBK 编码文件
- **WHEN** 用户打开一个 GBK 编码的文本文件
- **THEN** 系统通过 chardetng 检测并正确解码，行为与修改前一致