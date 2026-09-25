# Split Archive Module — Handoff

## 模块现状

`src-tauri/src/archive/mod.rs` — 1663 行，7 个逻辑区域。

## 内部依赖图

```
mod.rs (公共 API: list/extract/compress/delete/rename/add/write/create)
  ├─→ ArchiveFormat::from_path() — 格式判断
  ├─→ password::resolve_archive_password() — 密码解析
  └─→ 按格式分发到 zip/tar/tar_gz/seven_z 子模块

zip.rs
  ├─→ password::{remember_archive_password, password_required_error, password_incorrect_error}
  ├─→ encoding::{detect_archive_encoding, decode_name, ZipCdEntry, parse_zip_central_dir, find_eocd, read_cd_info}
  └─→ shared::{normalize_internal, collect_entries_at_path, matches_internal_path, is_dir_in_entries}

tar.rs
  ├─→ encoding::{detect_archive_encoding, decode_name, decode_tar_name}
  └─→ shared::{normalize_internal, collect_entries_at_path, matches_internal_path, is_dir_in_entries}

seven_z.rs
  ├─→ password::{remember_archive_password, password_required_error, password_incorrect_error}
  └─→ shared::{normalize_internal, collect_entries_at_path, matches_internal_path, is_dir_in_entries}
```

## 拆分方案：5 个子模块

### 1. `archive/password.rs` (~60 行)

**提取内容：**
- `ARCHIVE_PASSWORD_CACHE` 静态变量
- `archive_cache_key()`
- `cached_archive_password()`
- `store_archive_password()`
- `resolve_archive_password()` — pub(crate)
- `remember_archive_password()` — pub(crate)
- `password_required_error()` — pub(crate)
- `password_incorrect_error()` — pub(crate)

**不包含：**
- `ENCODING_CACHE` — 属于 encoding.rs
- `map_zip_error()` / `is_zip_password_error()` — 属于 zip.rs（格式特定）
- `map_7z_error()` — 属于 seven_z.rs（格式特定）
- `zip_entry_requires_password()` / `validate_zip_password()` / `read_zip_entry()` — 属于 zip.rs（ZIP 特定密码操作）

**理由：** 密码缓存和错误消息是跨格式共享的（ZIP 和 7z 都用），但格式特定的错误映射函数仅被对应格式模块使用，应跟随格式模块走。

### 2. `archive/encoding.rs` (~140 行)

**提取内容：**
- `ENCODING_CACHE` 静态变量
- `detect_archive_encoding()` — pub(crate)
- `detect_encoding_from_bytes()` — pub(crate)
- `decode_name()` — pub(crate)
- `decode_tar_name()` — pub(crate)（TAR/TAR.GZ 共用）

**不包含：**
- `ZipCdEntry` 结构体 — 属于 zip.rs
- `parse_zip_central_dir()` / `find_eocd()` / `read_cd_info()` — 属于 zip.rs（ZIP 中心目录解析是 ZIP 格式特有逻辑）

**理由：** 编码检测是纯通用能力，ZIP 和 TAR 都依赖。但 ZIP 中心目录解析是 ZIP 格式的二进制协议细节，应该跟着 ZIP 走。

### 3. `archive/zip.rs` (~560 行)

**提取内容：**
- `ZipCdEntry` 结构体
- `parse_zip_central_dir()` / `find_eocd()` / `read_cd_info()`
- `is_zip_password_error()` / `map_zip_error()`
- `zip_entry_requires_password()` / `validate_zip_password()` / `read_zip_entry()`
- `list_zip_entries()` / `read_zip_file()` / `extract_zip_files()` / `extract_zip_all()`
- `delete_zip_entries()` / `rename_zip_entry()` / `add_files_to_zip()` / `write_zip_file()` / `create_zip_entry()`
- `compress_files()` / `add_dir_to_zip()`

**pub(crate) 接口：**
```rust
pub(crate) fn list_entries(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String>
pub(crate) fn read_file(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<u8>, String>
pub(crate) fn extract_files(path: &str, paths: &[String], dest: &str, password: Option<&str>) -> Result<(), String>
pub(crate) fn extract_all(path: &str, dest: &str, password: Option<&str>) -> Result<u64, String>
pub(crate) fn delete_entries(path: &str, paths: &[String]) -> Result<(), String>
pub(crate) fn rename_entry(path: &str, old: &str, new: &str) -> Result<(), String>
pub(crate) fn add_files(path: &str, sources: &[String], internal: &str) -> Result<(), String>
pub(crate) fn write_file(path: &str, internal: &str, content: &[u8]) -> Result<(), String>
pub(crate) fn create_entry(path: &str, internal: &str, is_dir: bool) -> Result<(), String>
pub(crate) fn compress_files(sources: &[String], dest: &str) -> Result<u64, String>
```

### 4. `archive/tar.rs` (~300 行，合并 TAR + TAR.GZ)

**提取内容：**
- `list_tar_entries()` / `read_tar_file()` / `extract_tar_files()` / `extract_tar_all()`
- `list_tar_gz_entries()` / `read_tar_gz_file()` / `extract_tar_gz_files()` / `extract_tar_gz_all()`

**pub(crate) 接口：**
```rust
pub(crate) fn list_entries(path: &str, internal: &str) -> Result<Vec<FileEntry>, String>
pub(crate) fn list_gz_entries(path: &str, internal: &str) -> Result<Vec<FileEntry>, String>
pub(crate) fn read_file(path: &str, internal: &str) -> Result<Vec<u8>, String>
pub(crate) fn read_gz_file(path: &str, internal: &str) -> Result<Vec<u8>, String>
pub(crate) fn extract_files(path: &str, paths: &[String], dest: &str) -> Result<(), String>
pub(crate) fn extract_gz_files(path: &str, paths: &[String], dest: &str) -> Result<(), String>
pub(crate) fn extract_all(path: &str, dest: &str) -> Result<u64, String>
pub(crate) fn extract_gz_all(path: &str, dest: &str) -> Result<u64, String>
```

**理由：** TAR 和 TAR.GZ 的代码几乎完全相同（仅初始化时多一层 GzDecoder），合并为一个模块避免重复。内部可以提取共用的 `list_entries_impl<R: Read>(archive, internal)` 泛型函数。

### 5. `archive/seven_z.rs` (~155 行)

**提取内容：**
- `make_7z_password()`
- `map_7z_error()`
- `list_7z_entries()` / `read_7z_file()` / `extract_7z_files()` / `extract_7z_all()`

**pub(crate) 接口：**
```rust
pub(crate) fn list_entries(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String>
pub(crate) fn read_file(path: &str, internal: &str, password: Option<&str>) -> Result<Vec<u8>, String>
pub(crate) fn extract_files(path: &str, paths: &[String], dest: &str, password: Option<&str>) -> Result<(), String>
pub(crate) fn extract_all(path: &str, dest: &str, password: Option<&str>) -> Result<u64, String>
```

### 6. `archive/shared.rs` (~125 行)

**提取内容：**
- `normalize_internal()` — pub(crate)
- `matches_internal_path()` — pub(crate)
- `is_dir_in_entries()` — pub(crate)
- `collect_entries_at_path()` — pub(crate)

**理由：** 这 4 个函数被 ZIP、TAR、TAR.GZ 共同依赖，放在独立模块避免循环依赖。

## mod.rs 拆分后保留内容 (~120 行)

```rust
mod password;
mod encoding;
mod shared;
mod zip;
mod tar;
mod seven_z;

use crate::FileEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    SevenZ,
}

impl ArchiveFormat {
    pub fn from_path(path: &str) -> Option<Self> { ... }
    pub fn supports_write(&self) -> bool { ... }
}

// 公共 API — 纯分发层，不含业务逻辑
pub fn list_entries(archive_path: &str, internal_path: &str, password: Option<String>) -> Result<Vec<FileEntry>, String> {
    let format = ArchiveFormat::from_path(archive_path)
        .ok_or_else(|| format!("Unsupported archive format: {}", archive_path))?;
    let password = password::resolve_archive_password(archive_path, password);
    match format {
        ArchiveFormat::Zip => zip::list_entries(archive_path, internal_path, password.as_deref()),
        ArchiveFormat::Tar => tar::list_entries(archive_path, internal_path),
        ArchiveFormat::TarGz => tar::list_gz_entries(archive_path, internal_path),
        ArchiveFormat::SevenZ => seven_z::list_entries(archive_path, internal_path, password.as_deref()),
    }
}
// ... 其余 8 个公共 API 同理
```

## 拆分后行数预估

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | ~120 | ArchiveFormat + 公共 API 分发 |
| `password.rs` | ~60 | 密码缓存、解析、错误消息 |
| `encoding.rs` | ~140 | 编码检测、解码 |
| `shared.rs` | ~125 | 路径规范化、条目收集 |
| `zip.rs` | ~560 | ZIP 读写操作 + 中心目录解析 |
| `tar.rs` | ~300 | TAR + TAR.GZ 读操作 |
| `seven_z.rs` | ~155 | 7z 读操作 |
| **合计** | ~1460 | - |

## 关键设计决策

### Q: 密码管理能否提取为 `archive/password.rs`？
**A: 可以，但只提取缓存/解析/错误消息。** 格式特定的密码函数（`map_zip_error`、`zip_entry_requires_password`、`validate_zip_password`、`read_zip_entry`、`make_7z_password`、`map_7z_error`）留在各自格式模块中，因为它们仅被单一格式使用且依赖格式特定的类型（`zip::result::ZipError`、`sevenz_rust::Error`）。

### Q: ZIP 和 7z 的实现能否分别提取？
**A: 可以，无阻碍。** ZIP 模块最大（~560 行），但自包含。唯一需要外部依赖的是 `encoding` 和 `shared`，都是单向依赖。

### Q: TAR 和 TAR.GZ 是否应该分开？
**A: 不应该。** 两者的代码几乎完全相同（约 95% 重复），仅在 `File::open` 后多一层 `GzDecoder`。合并为一个模块，内部用泛型或两个入口函数消除重复。

### Q: 共享类型放在哪里？
**A: `ArchiveFormat` 留在 `mod.rs`。** 它是公共 API 的一部分，所有外部调用者通过 `mod.rs` 的公共函数间接使用。如果未来需要外部直接使用 `ArchiveFormat`，也自然从 `mod.rs` 导出。

### Q: `ZipCdEntry` 放哪里？
**A: `zip.rs`。** 它是 ZIP 中心目录解析的内部结构体，仅被 `parse_zip_central_dir()` 生产、被 `list/read/extract_zip_*` 消费，全部在 ZIP 模块内部。

## 实施顺序

1. **`shared.rs`** — 无内部依赖，最先提取
2. **`password.rs`** — 无内部依赖
3. **`encoding.rs`** — 无内部依赖
4. **`tar.rs`** — 依赖 shared + encoding
5. **`zip.rs`** — 依赖 shared + encoding + password
6. **`seven_z.rs`** — 依赖 shared + password
7. **`mod.rs`** — 改为分发层，导入子模块

每步完成后运行 `cargo check` 验证编译通过。

## 外部调用者

检查 `lib.rs` 和其他模块中对 `crate::archive::*` 的调用，确保公共 API 签名不变。由于 `mod.rs` 保留所有 `pub` 函数，外部调用者无需修改。

## 风险

- **encoding 缓存的 `&'static` 生命周期：** `ENCODING_CACHE` 存储 `&'static encoding_rs::Encoding`，跨模块移动不影响，因为 `encoding_rs` 的编码引用本身就是 `'static`。
- **tar.rs 泛型化：** TAR 和 TAR.GZ 合并时需注意 `tar::Archive<File>` vs `tar::Archive<GzDecoder<File>>` 的泛型约束，确保 `decode_tar_name` 的 `impl Read` bound 能同时适用。
