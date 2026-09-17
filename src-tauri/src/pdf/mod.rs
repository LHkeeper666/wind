use base64::{Engine, engine::general_purpose::STANDARD};
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use tauri::Manager;

// --- Document Cache ---
// Caches the Pdfium instance and loaded PdfDocument objects to avoid
// reloading the PDF file from disk on every page render.
//
// We use raw pointers to work around the borrow checker: PdfDocument<'a>
// borrows from Pdfium (&'a self), but we need to store both in the same
// struct. By storing Pdfium as a raw pointer, we can create &PdfDocument
// without holding &Pdfium through the struct. The Pdfium instance is kept
// alive for the program's lifetime, so the pointer remains valid.
// PdfDocument does NOT implement Drop (no FPDF_CloseDocument call).

struct CachedDoc {
    doc: PdfDocument<'static>,
    last_used: u64,
    /// Keep the bytes alive (PdfDocument may own them via load_pdf_from_byte_vec,
    /// but for load_pdf_from_file the OS keeps the file mapped).
    _bytes: Option<Vec<u8>>,
}

struct PdfDocCache {
    pdfium: *mut Pdfium,
    docs: HashMap<String, CachedDoc>,
    clock: u64,
}

const MAX_CACHED_DOCUMENTS: usize = 4;

// SAFETY: Pdfium is Send+Sync (the `sync` + `thread_safe` features ensure this).
// The raw pointer is only dereferenced while we hold the mutex.
unsafe impl Send for PdfDocCache {}
unsafe impl Sync for PdfDocCache {}

static PDF_CACHE: std::sync::LazyLock<Mutex<Option<PdfDocCache>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

/// Run a closure with a reference to a cached PdfDocument.
/// The document is loaded on first access and cached for subsequent calls.
fn with_cached_doc<T>(
    path: &str,
    app_handle: &tauri::AppHandle,
    f: impl FnOnce(&PdfDocument) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = PDF_CACHE.lock().map_err(|e| format!("Lock: {}", e))?;

    // Initialize cache on first use
    if guard.is_none() {
        let pdfium = load_pdfium(app_handle)?;
        *guard = Some(PdfDocCache {
            pdfium: Box::into_raw(Box::new(pdfium)),
            docs: HashMap::new(),
            clock: 0,
        });
    }

    let cache = guard.as_mut().unwrap();
    cache.clock = cache.clock.wrapping_add(1);

    // Load document if not cached
    if !cache.docs.contains_key(path) {
        if cache.docs.len() >= MAX_CACHED_DOCUMENTS {
            if let Some(oldest_path) = cache.docs
                .iter()
                .min_by_key(|(_, cached)| cached.last_used)
                .map(|(path, _)| path.clone())
            {
                cache.docs.remove(&oldest_path);
            }
        }
        // SAFETY: pdfium pointer is valid — we created it from Box::into_raw
        // and never free it. The cache is behind a Mutex so no concurrent access.
        let pdfium_ref = unsafe { &*cache.pdfium };

        let doc = pdfium_ref
            .load_pdf_from_file(path, None)
            .map_err(|e| format!("Failed to load PDF: {:?}", e))?;

        // SAFETY: We extend the lifetime from the local borrow to 'static.
        // This is sound because:
        // 1. The Pdfium instance lives for the program's lifetime (we never free it)
        // 2. PdfDocument does NOT implement Drop (doesn't call FPDF_CloseDocument)
        // 3. The bindings reference inside PdfDocument points to the same Pdfium
        // 4. All access is serialized through the Mutex
        let doc_static: PdfDocument<'static> = unsafe { std::mem::transmute(doc) };

        cache.docs.insert(path.to_string(), CachedDoc {
            doc: doc_static,
            last_used: cache.clock,
            _bytes: None,
        });
    } else if let Some(cached) = cache.docs.get_mut(path) {
        cached.last_used = cache.clock;
    }

    let cached = cache.docs.get(path).unwrap();
    f(&cached.doc)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdfPageDimensions {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfInfo {
    page_count: u32,
    title: Option<String>,
    author: Option<String>,
    file_size: u64,
    page_dimensions: Vec<PdfPageDimensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfPageResult {
    data: String, // base64-encoded image
    width: u32,
    height: u32,
    format: String, // "jpeg" or "png"
}

const MAX_PDF_TILE_SIZE: u32 = 512;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfTileRequest {
    path: String,
    page: u32,
    scale: f64,
    tile_x: u32,
    tile_y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfTileResult {
    data: String,
    width: u32,
    height: u32,
    format: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PdfTileBounds {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn tile_bounds(
    page_width: f32,
    page_height: f32,
    scale: f64,
    tile_x: u32,
    tile_y: u32,
    requested_width: u32,
    requested_height: u32,
) -> Result<PdfTileBounds, String> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err("Tile scale must be a positive finite value".to_string());
    }
    if requested_width == 0 || requested_height == 0 {
        return Err("Tile dimensions must be positive".to_string());
    }
    if requested_width > MAX_PDF_TILE_SIZE || requested_height > MAX_PDF_TILE_SIZE {
        return Err(format!("Tile dimensions cannot exceed {} pixels", MAX_PDF_TILE_SIZE));
    }

    let rendered_width = (page_width as f64 * scale).ceil() as u32;
    let rendered_height = (page_height as f64 * scale).ceil() as u32;
    if tile_x >= rendered_width || tile_y >= rendered_height {
        return Err("Tile origin is outside the rendered page".to_string());
    }

    Ok(PdfTileBounds {
        x: tile_x,
        y: tile_y,
        width: requested_width.min(rendered_width - tile_x),
        height: requested_height.min(rendered_height - tile_y),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextMatch {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfSearchResult {
    page: u32,
    matches: Vec<TextMatch>,
}

fn load_pdfium(app_handle: &tauri::AppHandle) -> Result<Pdfium, String> {
    let lib_name = if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "linux") {
        "libpdfium.so"
    } else {
        "libpdfium.dylib"
    };

    let mut search_dirs: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(res_dir) = app_handle.path().resource_dir() {
        search_dirs.push(res_dir.join("resources"));
        search_dirs.push(res_dir);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            search_dirs.push(dir.to_path_buf());
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        search_dirs.push(cwd.join("src-tauri").join("bin"));
        search_dirs.push(cwd.join("bin"));
        search_dirs.push(cwd);
    }

    for dir in &search_dirs {
        let lib_path = dir.join(lib_name);
        if lib_path.exists() {
            match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(dir)) {
                Ok(bindings) => {
                    eprintln!("[pdf] Loaded pdfium from: {:?}", dir);
                    return Ok(Pdfium::new(bindings));
                }
                Err(e) => {
                    eprintln!("[pdf] Failed from {:?}: {:?}", dir, e);
                }
            }
        }
    }

    match Pdfium::bind_to_system_library() {
        Ok(bindings) => Ok(Pdfium::new(bindings)),
        Err(e) => Err(format!("Failed to load pdfium: {:?}", e)),
    }
}

#[tauri::command]
pub fn get_pdf_info(path: String, app_handle: tauri::AppHandle) -> Result<PdfInfo, String> {
    let file_path = std::path::Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

    with_cached_doc(&path, &app_handle, |document| {
        let page_count = document.pages().len() as u32;

        let mut page_dimensions = Vec::with_capacity(page_count as usize);
        for i in 0..page_count {
            if let Ok(page) = document.pages().get(i as u16) {
                page_dimensions.push(PdfPageDimensions {
                    width: page.width().value,
                    height: page.height().value,
                });
            }
        }

        let metadata = document.metadata();
        let title = metadata
            .get(PdfDocumentMetadataTagType::Title)
            .map(|t| t.value().to_string());
        let author = metadata
            .get(PdfDocumentMetadataTagType::Author)
            .map(|t| t.value().to_string());

        Ok(PdfInfo {
            page_count,
            title,
            author,
            file_size,
            page_dimensions,
        })
    })
}

#[tauri::command]
pub fn render_pdf_page(
    path: String,
    page: u32,
    scale: Option<f64>,
    app_handle: tauri::AppHandle,
) -> Result<PdfPageResult, String> {
    let t_total = std::time::Instant::now();
    let render_scale = scale.unwrap_or(1.5) as f32;

    // Phase 1: Render under Mutex (fast: 3-20ms)
    let t_render_start = std::time::Instant::now();
    let (rgba_bytes, width, height) = with_cached_doc(&path, &app_handle, |document| {
        let page_count = document.pages().len() as u32;
        if page >= page_count {
            return Err(format!("Page {} out of range (total: {})", page, page_count));
        }

        let pdf_page = document
            .pages()
            .get(page as u16)
            .map_err(|e| format!("Failed to get page {}: {:?}", page, e))?;

        let target_width = (pdf_page.width().value * render_scale).round() as i32;
        let target_height = (pdf_page.height().value * render_scale).round() as i32;

        let config = PdfRenderConfig::new()
            .set_target_width(target_width)
            .set_maximum_height(target_height)
            .clear_before_rendering(true)
            .render_form_data(true)
            .render_annotations(true);

        let bitmap = pdf_page
            .render_with_config(&config)
            .map_err(|e| format!("Failed to render page {}: {:?}", page, e))?;

        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        let rgba = bitmap.as_rgba_bytes();

        Ok((rgba, width, height))
    })?;
    let t_render = t_render_start.elapsed();

    // Phase 2: JPEG encode + base64 OUTSIDE Mutex (fast: ~10-30ms with turbojpeg)
    let t_encode_start = std::time::Instant::now();
    let jpeg_data = encode_jpeg(&rgba_bytes, width, height)?;
    let b64 = STANDARD.encode(&jpeg_data);
    let t_encode = t_encode_start.elapsed();

    eprintln!("[pdf-perf-rs] page={} scale={:.1} render={:.1}ms encode={:.1}ms total={:.1}ms size={}x{} b64={}KB",
        page, render_scale, t_render.as_millis(), t_encode.as_millis(),
        t_total.elapsed().as_millis(), width, height, b64.len() / 1024);

    Ok(PdfPageResult {
        data: b64,
        width,
        height,
        format: "jpeg".to_string(),
    })
}

#[tauri::command]
pub fn render_pdf_tile(
    request: PdfTileRequest,
    app_handle: tauri::AppHandle,
) -> Result<PdfTileResult, String> {
    let t_total = std::time::Instant::now();
    let (rgba_bytes, bounds) = with_cached_doc(&request.path, &app_handle, |document| {
        let page_count = document.pages().len() as u32;
        if request.page >= page_count {
            return Err(format!("Page {} out of range (total: {})", request.page, page_count));
        }

        let pdf_page = document
            .pages()
            .get(request.page as u16)
            .map_err(|e| format!("Failed to get page {}: {:?}", request.page, e))?;
        let bounds = tile_bounds(
            pdf_page.width().value,
            pdf_page.height().value,
            request.scale,
            request.tile_x,
            request.tile_y,
            request.width,
            request.height,
        )?;

        let rendered_width = (pdf_page.width().value as f64 * request.scale).ceil() as i32;
        let rendered_height = (pdf_page.height().value as f64 * request.scale).ceil() as i32;
        let mut bitmap = PdfBitmap::empty(
            bounds.width as i32,
            bounds.height as i32,
            PdfBitmapFormat::default(),
            pdf_page.bindings(),
        )
        .map_err(|e| format!("Failed to allocate PDF tile bitmap: {:?}", e))?;

        let config = PdfRenderConfig::new()
            .set_target_size(rendered_width, rendered_height)
            .translate(
                PdfPoints::new(-(bounds.x as f32 / request.scale as f32)),
                PdfPoints::new(-(bounds.y as f32 / request.scale as f32)),
            )
            .map_err(|e| format!("Failed to configure PDF tile transform: {:?}", e))?
            .clear_before_rendering(true)
            .render_annotations(true);

        pdf_page
            .render_into_bitmap_with_config(&mut bitmap, &config)
            .map_err(|e| format!("Failed to render page {} tile: {:?}", request.page, e))?;

        Ok((bitmap.as_rgba_bytes(), bounds))
    })?;

    let jpeg_data = encode_jpeg(&rgba_bytes, bounds.width, bounds.height)?;
    let data = STANDARD.encode(&jpeg_data);
    eprintln!(
        "[pdf-tile] page={} scale={:.3} tile={}x{}+{},{} output={}x{} total={}ms",
        request.page,
        request.scale,
        request.width,
        request.height,
        request.tile_x,
        request.tile_y,
        bounds.width,
        bounds.height,
        t_total.elapsed().as_millis(),
    );

    Ok(PdfTileResult {
        data,
        width: bounds.width,
        height: bounds.height,
        format: "jpeg".to_string(),
    })
}

fn encode_jpeg(rgba_data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    // Convert RGBA → RGB (drop alpha channel)
    let pixel_count = (width * height) as usize;
    let mut rgb_data = Vec::with_capacity(pixel_count * 3);
    for i in 0..pixel_count {
        let offset = i * 4;
        rgb_data.push(rgba_data[offset]);
        rgb_data.push(rgba_data[offset + 1]);
        rgb_data.push(rgba_data[offset + 2]);
    }

    let image = turbojpeg::Image {
        pixels: rgb_data.as_slice(),
        width: width as usize,
        pitch: width as usize * 3,
        height: height as usize,
        format: turbojpeg::PixelFormat::RGB,
    };

    let mut compressor = turbojpeg::Compressor::new()
        .map_err(|e| format!("JPEG compressor: {}", e))?;
    compressor.set_quality(92)
        .map_err(|e| format!("JPEG quality: {}", e))?;
    compressor.compress_to_vec(image)
        .map_err(|e| format!("JPEG encode: {}", e))
}

#[tauri::command]
pub fn search_pdf_text(
    path: String,
    query: String,
    app_handle: tauri::AppHandle,
) -> Result<Vec<PdfSearchResult>, String> {
    if query.is_empty() {
        return Ok(Vec::new());
    }

    with_cached_doc(&path, &app_handle, |document| {
        let options = PdfSearchOptions::new().match_case(false);
        let mut results = Vec::new();

        for page_index in 0..document.pages().len() {
            let page = document
                .pages()
                .get(page_index as u16)
                .map_err(|e| format!("Page {}: {:?}", page_index, e))?;

            let text_page = page
                .text()
                .map_err(|e| format!("Text {}: {:?}", page_index, e))?;

            let search = text_page.search(&query, &options);
            if let Ok(search) = search {
                let mut matches = Vec::new();
                let mut current = search.find_next();
                while let Some(segments) = current {
                    for segment in segments.iter() {
                        let bounds = segment.bounds();
                        matches.push(TextMatch {
                            x: bounds.left().value as f64,
                            y: bounds.bottom().value as f64,
                            width: bounds.width().value as f64,
                            height: bounds.height().value as f64,
                        });
                    }
                    current = search.find_next();
                }
                if !matches.is_empty() {
                    results.push(PdfSearchResult {
                        page: page_index as u32,
                        matches,
                    });
                }
            }
        }
        Ok(results)
    })
}

// --- Outline / Bookmarks ---

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfOutlineItem {
    pub title: String,
    pub page: u32,
    pub x: f32,
    pub y: f32,
    pub children: Vec<PdfOutlineItem>,
}

fn collect_outline_children(
    bookmark: &PdfBookmark,
    document: &PdfDocument,
) -> Vec<PdfOutlineItem> {
    let mut items = Vec::new();
    if let Some(first_child) = bookmark.first_child() {
        let mut current = Some(first_child);
        while let Some(child) = current {
            let title = child.title().unwrap_or_default();
            let (page, x, y) = extract_dest_coords(&child, document);
            let children = collect_outline_children(&child, document);
            items.push(PdfOutlineItem {
                title,
                page,
                x,
                y,
                children,
            });
            current = child.next_sibling();
        }
    }
    items
}

fn extract_dest_coords(bookmark: &PdfBookmark, _document: &PdfDocument) -> (u32, f32, f32) {
    if let Some(dest) = bookmark.destination() {
        let page = dest.page_index().unwrap_or(0) as u32;
        if let Ok(view) = dest.view_settings() {
            if let PdfDestinationViewSettings::SpecificCoordinatesAndZoom(x, y, _zoom) = view {
                return (
                    page,
                    x.map(|v| v.value).unwrap_or(0.0),
                    y.map(|v| v.value).unwrap_or(0.0),
                );
            }
        }
        return (page, 0.0, 0.0);
    }
    if let Some(action) = bookmark.action() {
        if let Some(local) = action.as_local_destination_action() {
            if let Ok(dest) = local.destination() {
                let page = dest.page_index().unwrap_or(0) as u32;
                if let Ok(view) = dest.view_settings() {
                    if let PdfDestinationViewSettings::SpecificCoordinatesAndZoom(x, y, _zoom) = view
                    {
                        return (
                            page,
                            x.map(|v| v.value).unwrap_or(0.0),
                            y.map(|v| v.value).unwrap_or(0.0),
                        );
                    }
                }
                return (page, 0.0, 0.0);
            }
        }
    }
    (0, 0.0, 0.0)
}

#[tauri::command]
pub fn get_pdf_outline(
    path: String,
    app_handle: tauri::AppHandle,
) -> Result<Vec<PdfOutlineItem>, String> {
    with_cached_doc(&path, &app_handle, |document| {
        let mut items = Vec::new();
        let mut count = 0u32;
        // Walk only top-level bookmarks (siblings of root), not all bookmarks.
        // Using iter() would return ALL bookmarks depth-first, causing children
        // to appear both as top-level items and nested under their parent.
        let mut current = document.bookmarks().root();
        while let Some(bookmark) = current {
            count += 1;
            let title = bookmark.title().unwrap_or_default();
            let (page, x, y) = extract_dest_coords(&bookmark, document);
            let children = collect_outline_children(&bookmark, document);
            eprintln!("[pdf] outline item #{}: title=\"{}\" page={} children={}", count, title, page, children.len());
            items.push(PdfOutlineItem {
                title,
                page,
                x,
                y,
                children,
            });
            current = bookmark.next_sibling();
        }
        eprintln!("[pdf] get_pdf_outline: total {} items", count);
        Ok(items)
    })
}

// --- Link Annotations ---

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfLinkAnnotation {
    pub rect: [f32; 4],
    pub target_page: u32,
    pub target_x: f32,
    pub target_y: f32,
    pub is_external: bool,
    pub url: Option<String>,
}

#[tauri::command]
pub fn get_pdf_page_links(
    path: String,
    page: u32,
    app_handle: tauri::AppHandle,
) -> Result<Vec<PdfLinkAnnotation>, String> {
    with_cached_doc(&path, &app_handle, |document| {
        let page_count = document.pages().len() as u32;
        if page >= page_count {
            return Err(format!("Page {} out of range (total: {})", page, page_count));
        }

        let pdf_page = document
            .pages()
            .get(page as u16)
            .map_err(|e| format!("Failed to get page {}: {:?}", page, e))?;

        let mut links = Vec::new();

        for annot_idx in 0..pdf_page.annotations().len() {
            let annotation = match pdf_page.annotations().get(annot_idx) {
                Ok(a) => a,
                Err(_) => continue,
            };

            if annotation.annotation_type() != PdfPageAnnotationType::Link {
                continue;
            }

            let link_annotation = match annotation.as_link_annotation() {
                Some(la) => la,
                None => continue,
            };

            let bounds = match annotation.bounds() {
                Ok(b) => b,
                Err(_) => continue,
            };

            let link = match link_annotation.link() {
                Ok(l) => l,
                Err(_) => continue,
            };

            if let Some(action) = link.action() {
                if let Some(uri_action) = action.as_uri_action() {
                    if let Ok(uri) = uri_action.uri() {
                        links.push(PdfLinkAnnotation {
                            rect: [
                                bounds.left().value,
                                bounds.bottom().value,
                                bounds.right().value,
                                bounds.top().value,
                            ],
                            target_page: 0,
                            target_x: 0.0,
                            target_y: 0.0,
                            is_external: true,
                            url: Some(uri),
                        });
                        continue;
                    }
                }

                if let Some(local) = action.as_local_destination_action() {
                    if let Ok(dest) = local.destination() {
                        let target_page = dest.page_index().unwrap_or(0) as u32;
                        let (tx, ty) = extract_dest_xy(&dest);
                        links.push(PdfLinkAnnotation {
                            rect: [
                                bounds.left().value,
                                bounds.bottom().value,
                                bounds.right().value,
                                bounds.top().value,
                            ],
                            target_page,
                            target_x: tx,
                            target_y: ty,
                            is_external: false,
                            url: None,
                        });
                        continue;
                    }
                }
            }

            if let Some(dest) = link.destination() {
                let target_page = dest.page_index().unwrap_or(0) as u32;
                let (tx, ty) = extract_dest_xy(&dest);
                links.push(PdfLinkAnnotation {
                    rect: [
                        bounds.left().value,
                        bounds.bottom().value,
                        bounds.right().value,
                        bounds.top().value,
                    ],
                    target_page,
                    target_x: tx,
                    target_y: ty,
                    is_external: false,
                    url: None,
                });
            }
        }

        Ok(links)
    })
}

fn extract_dest_xy(dest: &PdfDestination) -> (f32, f32) {
    if let Ok(view) = dest.view_settings() {
        if let PdfDestinationViewSettings::SpecificCoordinatesAndZoom(x, y, _zoom) = view {
            return (
                x.map(|v| v.value).unwrap_or(0.0),
                y.map(|v| v.value).unwrap_or(0.0),
            );
        }
    }
    (0.0, 0.0)
}

/// Clear the PDF document cache (e.g., when switching files to free memory).
#[tauri::command]
pub fn clear_pdf_cache() -> Result<(), String> {
    if let Ok(mut guard) = PDF_CACHE.lock() {
        if let Some(cache) = guard.as_mut() {
            cache.docs.clear();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_bounds_scale_page_coordinates() {
        let bounds = tile_bounds(1024.0, 1024.0, 2.0, 512, 512, 512, 512).unwrap();
        assert_eq!(bounds, PdfTileBounds { x: 512, y: 512, width: 512, height: 512 });
    }

    #[test]
    fn tile_bounds_clip_right_and_bottom_edges() {
        let bounds = tile_bounds(100.0, 200.0, 2.0, 150, 350, 512, 512).unwrap();
        assert_eq!(bounds, PdfTileBounds { x: 150, y: 350, width: 50, height: 50 });
    }

    #[test]
    fn adjacent_tile_bounds_share_an_edge_without_overlap() {
        let left = tile_bounds(1024.0, 512.0, 1.0, 0, 0, 512, 512).unwrap();
        let right = tile_bounds(1024.0, 512.0, 1.0, 512, 0, 512, 512).unwrap();
        assert_eq!(left.x + left.width, right.x);
        assert_eq!(left.y, right.y);
        assert_eq!(left.height, right.height);
    }
}
