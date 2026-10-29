use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::fs;
use std::path::Path;

use log::{debug, error, info, warn};

use crate::tool_cache;

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("Failed to open: {}", e))
}

#[tauri::command]
pub fn open_with_dialog(path: String) -> Result<(), String> {
    tool_cache::background_command("rundll32")
        .args(["shell32.dll,OpenAs_RunDLL", &path])
        .spawn()
        .map_err(|e| format!("Failed to open with dialog: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    let bytes = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(decode_text(&bytes))
}

#[tauri::command]
pub fn read_file_partial(path: String, max_bytes: u64) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    use std::io::Read;
    let mut file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut buffer = vec![0u8; max_bytes as usize];
    let bytes_read = file
        .read(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    buffer.truncate(bytes_read);

    Ok(decode_text(&buffer))
}

/// Decode bytes to String: UTF-16 BOM first, then strict UTF-8, then chardetng + encoding_rs
pub fn decode_text(bytes: &[u8]) -> String {
    // UTF-16 LE BOM (FF FE)
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let (decoded, _) = encoding_rs::UTF_16LE.decode_without_bom_handling(&bytes[2..]);
        return decoded.into_owned();
    }
    // UTF-16 BE BOM (FE FF)
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let (decoded, _) = encoding_rs::UTF_16BE.decode_without_bom_handling(&bytes[2..]);
        return decoded.into_owned();
    }
    // Strip UTF-8 BOM if present (EF BB BF)
    let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return s;
    }
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    let (decoded, _had_errors) = encoding.decode_without_bom_handling(bytes);
    decoded.into_owned()
}

#[derive(Debug, Serialize)]
pub struct ImageThumbnail {
    pub data: String,
    pub width: u32,
    pub height: u32,
    pub original_size: u64,
    pub is_thumbnail: bool,
}

#[tauri::command]
pub fn read_binary_file(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    let bytes = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(STANDARD.encode(bytes))
}

#[tauri::command]
pub fn read_binary_file_partial(path: String, max_bytes: u64) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    use std::io::Read;
    let mut file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut buffer = vec![0u8; max_bytes as usize];
    let bytes_read = file
        .read(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    buffer.truncate(bytes_read);

    Ok(STANDARD.encode(buffer))
}

// Thumbnail: scale to max 1200px on the longest side. 4/5 layout on 1920px≃1500px panel.
const THUMBNAIL_MAX_SIDE: u32 = 1200;
// Files smaller than 512KB are fast to transfer — skip thumbnail & send original quality.
const THUMBNAIL_SKIP_SIZE: u64 = 512 * 1024;
// JPEG re-encode quality after scaling (85 = good balance, avoids double-compression artifacts).
const THUMBNAIL_JPEG_QUALITY: u8 = 85;

#[tauri::command]
pub async fn read_image_thumbnail(path: String) -> Result<ImageThumbnail, String> {
    let file_path = Path::new(&path).to_path_buf();

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if file_path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path));
    }

    tokio::task::spawn_blocking(move || {
        let original_size = fs::metadata(&file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let file_data = fs::read(&file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let is_actual_jpeg = file_data.len() >= 2 && file_data[0] == 0xFF && file_data[1] == 0xD8;
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let is_jpeg_ext = ext == "jpg" || ext == "jpeg";

        info!("[thumbnail] Processing: {} ({} bytes, ext={}, actual_jpeg={})", path, original_size, ext, is_actual_jpeg);

        if is_actual_jpeg {
            let t0 = std::time::Instant::now();

            let mut decompressor = turbojpeg::Decompressor::new()
                .map_err(|e| {
                    error!("[thumbnail] Failed to create decompressor: {}", e);
                    format!("Failed to create decompressor: {}", e)
                })?;
            let header = decompressor.read_header(&file_data)
                .map_err(|e| {
                    error!("[thumbnail] Failed to read JPEG headers {}: {}", path, e);
                    format!("Failed to read JPEG headers: {}", e)
                })?;
            let orig_w = header.width as u32;
            let orig_h = header.height as u32;
            let max_side = orig_w.max(orig_h);

            debug!("[thumbnail] JPEG dimensions: {}x{}, max_side={}", orig_w, orig_h, max_side);

            if max_side <= THUMBNAIL_MAX_SIDE || original_size <= THUMBNAIL_SKIP_SIZE {
                info!("[thumbnail] Skip thumbnail ({}x{}, {}B)", orig_w, orig_h, original_size);
                return Ok(ImageThumbnail {
                    data: String::new(),
                    width: orig_w,
                    height: orig_h,
                    original_size,
                    is_thumbnail: false,
                });
            }

            let t1 = std::time::Instant::now();
            let rgb_pixels: Vec<u8> = match turbojpeg::decompress(&file_data, turbojpeg::PixelFormat::RGB) {
                Ok(image) => image.pixels,
                Err(e) => {
                    warn!("[thumbnail] turbojpeg failed ({}), falling back to image crate", e);
                    let img = image::load_from_memory(&file_data)
                        .map_err(|e2| format!("Failed to decode JPEG: {} (turbojpeg: {})", e2, e))?;
                    img.to_rgb8().into_raw()
                }
            };
            let t2 = std::time::Instant::now();

            let ratio = THUMBNAIL_MAX_SIDE as f64 / max_side as f64;
            let final_w = (orig_w as f64 * ratio).round() as u32;
            let final_h = (orig_h as f64 * ratio).round() as u32;

            let src_image = fast_image_resize::images::Image::from_vec_u8(
                orig_w,
                orig_h,
                rgb_pixels,
                fast_image_resize::PixelType::U8x3,
            ).map_err(|e| format!("Failed to create source image: {}", e))?;

            let mut dst_image = fast_image_resize::images::Image::new(
                final_w,
                final_h,
                fast_image_resize::PixelType::U8x3,
            );

            let mut resizer = fast_image_resize::Resizer::new();
            #[cfg(target_arch = "x86_64")]
            unsafe {
                resizer.set_cpu_extensions(fast_image_resize::CpuExtensions::Avx2);
            }
            let options = fast_image_resize::ResizeOptions::new()
                .resize_alg(fast_image_resize::ResizeAlg::Nearest);
            resizer.resize(&src_image, &mut dst_image, &options)
                .map_err(|e| format!("Failed to resize image: {}", e))?;
            let t3 = std::time::Instant::now();

            let mut buf: Vec<u8> = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, THUMBNAIL_JPEG_QUALITY);
            let resized_img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
                image::ImageBuffer::from_raw(final_w, final_h, dst_image.into_vec())
                    .ok_or("Failed to create resized image buffer")?;
            resized_img.write_with_encoder(encoder)
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
            let t4 = std::time::Instant::now();

            debug!("[turbojpeg+fast-resize] {}x{} -> {}x{}, read={}ms decode={}ms resize={}ms encode={}ms total={}ms",
                orig_w, orig_h, final_w, final_h,
                (t1-t0).as_millis(), (t2-t1).as_millis(), (t3-t2).as_millis(), (t4-t3).as_millis(), (t4-t0).as_millis());

            Ok(ImageThumbnail {
                data: STANDARD.encode(&buf),
                width: final_w,
                height: final_h,
                original_size,
                is_thumbnail: true,
            })
        } else {
            if is_jpeg_ext && !is_actual_jpeg {
                warn!("[thumbnail] Warning: {} has .jpg extension but is not JPEG format", path);
            }

            let format = image::guess_format(&file_data)
                .map_err(|e| {
                    error!("[thumbnail] Failed to guess format {}: {}", path, e);
                    format!("Failed to guess image format: {}", e)
                })?;
            debug!("[thumbnail] Detected format: {:?}", format);

            let reader = image::ImageReader::new(std::io::Cursor::new(&file_data))
                .with_guessed_format()
                .map_err(|e| {
                    error!("[thumbnail] Failed to create reader {}: {}", path, e);
                    format!("Failed to create image reader: {}", e)
                })?;
            let (orig_w, orig_h) = reader.into_dimensions()
                .map_err(|e| {
                    error!("[thumbnail] Failed to read dimensions {}: {}", path, e);
                    format!("Failed to read image dimensions: {}", e)
                })?;
            let max_side = orig_w.max(orig_h);

            debug!("[thumbnail] Non-JPEG dimensions: {}x{}, max_side={}", orig_w, orig_h, max_side);

            if max_side <= THUMBNAIL_MAX_SIDE || original_size <= THUMBNAIL_SKIP_SIZE {
                info!("[thumbnail] Skip thumbnail ({}x{}, {}B)", orig_w, orig_h, original_size);
                return Ok(ImageThumbnail {
                    data: String::new(),
                    width: orig_w,
                    height: orig_h,
                    original_size,
                    is_thumbnail: false,
                });
            }

            let img = image::load_from_memory(&file_data)
                .map_err(|e| {
                    error!("[thumbnail] Failed to load image {}: {}", path, e);
                    format!("Failed to load image: {}", e)
                })?;
            let rgb_img = img.to_rgb8();

            let ratio = THUMBNAIL_MAX_SIDE as f64 / max_side as f64;
            let final_w = (orig_w as f64 * ratio).round() as u32;
            let final_h = (orig_h as f64 * ratio).round() as u32;

            let src_image = fast_image_resize::images::Image::from_vec_u8(
                orig_w,
                orig_h,
                rgb_img.into_raw(),
                fast_image_resize::PixelType::U8x3,
            ).map_err(|e| format!("Failed to create source image: {}", e))?;

            let mut dst_image = fast_image_resize::images::Image::new(
                final_w,
                final_h,
                fast_image_resize::PixelType::U8x3,
            );

            let mut resizer = fast_image_resize::Resizer::new();
            #[cfg(target_arch = "x86_64")]
            unsafe {
                resizer.set_cpu_extensions(fast_image_resize::CpuExtensions::Avx2);
            }
            let options = fast_image_resize::ResizeOptions::new()
                .resize_alg(fast_image_resize::ResizeAlg::Nearest);
            resizer.resize(&src_image, &mut dst_image, &options)
                .map_err(|e| format!("Failed to resize image: {}", e))?;

            let mut buf: Vec<u8> = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, THUMBNAIL_JPEG_QUALITY);
            let resized_img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
                image::ImageBuffer::from_raw(final_w, final_h, dst_image.into_vec())
                    .ok_or("Failed to create resized image buffer")?;
            resized_img.write_with_encoder(encoder)
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

            Ok(ImageThumbnail {
                data: STANDARD.encode(&buf),
                width: final_w,
                height: final_h,
                original_size,
                is_thumbnail: true,
            })
        }
    })
    .await
    .map_err(|e| format!("Image processing task failed: {}", e))?
}

#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    let file_path = Path::new(&path);
    let operation_id = next_legacy_write_id();
    let before = file_path.metadata().ok().and_then(|metadata| text_save_version(&metadata).ok());
    log::warn!("[text-save] id={} entry=legacy-write-file path={} before_size={} before_version={} bytes={} result=requested",
        operation_id, path, before.as_ref().map(|v| v.size.to_string()).unwrap_or_else(|| "missing".into()),
        before.as_ref().map(|v| v.modified_ns.as_str()).unwrap_or("missing"), content.len());

    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                log::error!("[text-save] id={} result=io_error stage=create_parent error={}", operation_id, e);
                format!("Failed to create parent directories: {}", e)
            })?;
        }
    }

    let result = fs::write(file_path, content);
    let after = file_path.metadata().ok().and_then(|metadata| text_save_version(&metadata).ok());
    match result {
        Ok(()) => {
            log::warn!("[text-save] id={} result=success after_size={} after_version={}", operation_id,
                after.as_ref().map(|v| v.size.to_string()).unwrap_or_else(|| "unknown".into()),
                after.as_ref().map(|v| v.modified_ns.as_str()).unwrap_or("unknown"));
            Ok(())
        }
        Err(e) => {
            log::error!("[text-save] id={} result=io_error stage=write after_size={} after_version={} error={}", operation_id,
                after.as_ref().map(|v| v.size.to_string()).unwrap_or_else(|| "unknown".into()),
                after.as_ref().map(|v| v.modified_ns.as_str()).unwrap_or("unknown"), e);
            Err(format!("Failed to write file: {}", e))
        }
    }
}

fn next_legacy_write_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    format!("legacy-{}-{}", std::process::id(), NEXT_ID.fetch_add(1, Ordering::Relaxed))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextSaveVersion {
    pub size: u64,
    pub modified: u64,
    pub modified_ns: String,
}

fn text_save_version(metadata: &fs::Metadata) -> Result<TextSaveVersion, String> {
    let duration = metadata.modified()
        .map_err(|e| format!("无法读取文件修改时间: {e}"))?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("文件修改时间无效: {e}"))?;
    Ok(TextSaveVersion {
        size: metadata.len(),
        modified: duration.as_secs(),
        modified_ns: duration.as_nanos().to_string(),
    })
}

fn text_save_rejection(current: &TextSaveVersion, expected_size: u64,
    expected_modified_ns: &str, pending_bytes: usize, allow_empty: bool) -> Option<&'static str> {
    if expected_modified_ns.is_empty() || current.size != expected_size || current.modified_ns != expected_modified_ns {
        return Some("文件已被外部修改，保存已取消；请先重新加载并处理冲突");
    }
    if pending_bytes == 0 && current.size > 0 && !allow_empty {
        return Some("空内容缺少有效编辑来源，保存已取消；请重新加载文件");
    }
    None
}

#[tauri::command]
pub fn save_text_file(
    path: String, content: String, expected_size: u64, expected_modified_ns: String,
    allow_empty: bool, entry: String, operation_id: String,
) -> Result<TextSaveVersion, String> {
    use std::io::Write;
    let bytes = content.as_bytes();
    let file_path = Path::new(&path);
    log::info!("[text-save] id={} entry={} path={} expected_size={} expected_version={} bytes={} empty_source={} result=requested",
        operation_id, entry, path, expected_size, expected_modified_ns, bytes.len(),
        if allow_empty { "edited" } else { "loaded" });
    let mut file = fs::OpenOptions::new().write(true).open(file_path)
        .map_err(|e| { log::error!("[text-save] id={} result=io_error reason=open error={}", operation_id, e); format!("无法打开文件: {e}") })?;
    let current_metadata = file.metadata().map_err(|e| {
        log::error!("[text-save] id={} result=io_error stage=metadata_before error={}", operation_id, e);
        format!("无法读取文件信息: {e}")
    })?;
    let current = text_save_version(&current_metadata).map_err(|e| {
        log::error!("[text-save] id={} result=io_error stage=version_before error={}", operation_id, e);
        e
    })?;
    log::info!("[text-save] id={} before_size={} before_version={} bytes={} version_check=start", operation_id, current.size, current.modified_ns, bytes.len());
    if let Some(reason) = text_save_rejection(&current, expected_size, &expected_modified_ns, bytes.len(), allow_empty) {
        let conflict = current.size != expected_size || current.modified_ns != expected_modified_ns || expected_modified_ns.is_empty();
        log::warn!("[text-save] id={} result={} before_size={} before_version={} version_check={} reason={}", operation_id,
            if conflict { "conflict" } else { "rejected" }, current.size, current.modified_ns,
            if conflict { "failed" } else { "passed" },
            if conflict { "disk_version_changed" } else { "unverified_empty" });
        return Err(reason.to_string());
    }
    log::info!("[text-save] id={} stage=truncate_start version_check=passed", operation_id);
    file.set_len(0).map_err(|e| {
        log::error!("[text-save] id={} result=io_error stage=truncate after_size={} error={}", operation_id,
            file.metadata().map(|m| m.len().to_string()).unwrap_or_else(|_| "unknown".into()), e);
        format!("无法清空文件: {e}")
    })?;
    log::info!("[text-save] id={} stage=truncate_done after_size=0", operation_id);
    file.write_all(bytes).map_err(|e| {
        log::error!("[text-save] id={} result=io_error stage=write after_size={} error={}", operation_id,
            file.metadata().map(|m| m.len().to_string()).unwrap_or_else(|_| "unknown".into()), e);
        format!("写入文件失败: {e}")
    })?;
    let updated_metadata = file.metadata().map_err(|e| {
        log::error!("[text-save] id={} result=unknown_written stage=metadata_after bytes={} error={}", operation_id, bytes.len(), e);
        format!("无法读取保存后版本: {e}")
    })?;
    let updated = text_save_version(&updated_metadata).map_err(|e| {
        log::error!("[text-save] id={} result=unknown_written stage=version_after bytes={} error={}", operation_id, bytes.len(), e);
        e
    })?;
    log::info!("[text-save] id={} result=success version_check=passed after_size={} after_version={}", operation_id, updated.size, updated.modified_ns);
    Ok(updated)
}

#[cfg(test)]
mod text_save_tests {
    use super::{text_save_rejection, TextSaveVersion};

    #[test]
    fn rejects_same_second_version_changes_and_unverified_empty_content() {
        let current = TextSaveVersion { size: 4, modified: 123, modified_ns: "123000000002".into() };
        assert!(text_save_rejection(&current, 4, "123000000001", 4, true).unwrap().contains("外部修改"));
        assert!(text_save_rejection(&current, 4, "123000000002", 0, false).unwrap().contains("空内容"));
        assert!(text_save_rejection(&current, 4, "123000000002", 0, true).is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_utf16_le_with_bom() {
        // "hello" in UTF-16 LE with BOM: FF FE 68 00 65 00 6C 00 6C 00 6F 00
        let bytes: &[u8] = &[0xFF, 0xFE, 0x68, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00];
        let result = decode_text(bytes);
        assert_eq!(result, "hello");
        assert!(!result.contains('\0'));
    }

    #[test]
    fn decode_utf16_be_with_bom() {
        // "hello" in UTF-16 BE with BOM: FE FF 00 68 00 65 00 6C 00 6C 00 6F
        let bytes: &[u8] = &[0xFE, 0xFF, 0x00, 0x68, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F];
        let result = decode_text(bytes);
        assert_eq!(result, "hello");
        assert!(!result.contains('\0'));
    }

    #[test]
    fn decode_utf8_with_bom() {
        let bytes: &[u8] = &[0xEF, 0xBB, 0xBF, 0x68, 0x65, 0x6C, 0x6C, 0x6F];
        let result = decode_text(bytes);
        assert_eq!(result, "hello");
    }

    #[test]
    fn decode_utf8_no_bom() {
        let bytes: &[u8] = b"hello world";
        let result = decode_text(bytes);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn decode_utf16_le_chinese() {
        // "你好" in UTF-16 LE with BOM
        let bytes: &[u8] = &[0xFF, 0xFE, 0x60, 0x4F, 0x7D, 0x59];
        let result = decode_text(bytes);
        assert_eq!(result, "你好");
        assert!(!result.contains('\0'));
    }
}
