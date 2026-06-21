use base64::{Engine, engine::general_purpose::STANDARD};
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfInfo {
    page_count: u32,
    title: Option<String>,
    author: Option<String>,
    file_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfPageResult {
    data: String, // base64-encoded PNG
    width: u32,
    height: u32,
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

fn load_pdfium() -> Result<Pdfium, String> {
    let lib_name = if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "linux") {
        "libpdfium.so"
    } else {
        "libpdfium.dylib"
    };

    // Search directories in priority order
    // pdfium_platform_library_name_at_path(dir) joins dir + lib_name,
    // so we pass DIRECTORY paths, not file paths.
    let mut search_dirs: Vec<std::path::PathBuf> = Vec::new();

    // 1. Next to the executable (production + cargo build)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            search_dirs.push(dir.to_path_buf());
        }
    }

    // 2. In the src-tauri/bin directory (development)
    if let Ok(cwd) = std::env::current_dir() {
        search_dirs.push(cwd.join("src-tauri").join("bin"));
        search_dirs.push(cwd.join("bin"));
        search_dirs.push(cwd);
    }

    // Try each directory
    for dir in &search_dirs {
        let lib_path = dir.join(lib_name);
        if lib_path.exists() {
            eprintln!("[pdf] Trying to load pdfium from: {:?}", lib_path);
            match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(dir)) {
                Ok(bindings) => {
                    eprintln!("[pdf] Successfully loaded pdfium from: {:?}", dir);
                    return Ok(Pdfium::new(bindings));
                }
                Err(e) => {
                    eprintln!("[pdf] Failed to load pdfium from {:?}: {:?}", dir, e);
                }
            }
        }
    }

    // Fallback: try system library
    eprintln!("[pdf] Trying system library...");
    match Pdfium::bind_to_system_library() {
        Ok(bindings) => Ok(Pdfium::new(bindings)),
        Err(e) => Err(format!(
            "Failed to load pdfium. Searched dirs: {:?}. Error: {:?}",
            search_dirs.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            e
        )),
    }
}

#[tauri::command]
pub fn get_pdf_info(path: String) -> Result<PdfInfo, String> {
    let file_path = std::path::Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    let file_size = fs::metadata(file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let pdfium = load_pdfium()?;
    let document = pdfium
        .load_pdf_from_file(&path, None)
        .map_err(|e| format!("Failed to load PDF: {:?}", e))?;

    let page_count = document.pages().len() as u32;

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
    })
}

#[tauri::command]
pub fn render_pdf_page(path: String, page: u32, scale: Option<f64>) -> Result<PdfPageResult, String> {
    let pdfium = load_pdfium()?;
    let document = pdfium
        .load_pdf_from_file(&path, None)
        .map_err(|e| format!("Failed to load PDF: {:?}", e))?;

    let page_count = document.pages().len() as u32;
    if page >= page_count {
        return Err(format!("Page {} out of range (total: {})", page, page_count));
    }

    let pdf_page = document
        .pages()
        .get(page as u16)
        .map_err(|e| format!("Failed to get page {}: {:?}", page, e))?;

    let render_scale = scale.unwrap_or(1.5) as f32;

    // Calculate target pixel dimensions
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

    // Get RGBA bytes and encode to PNG
    let rgba_data = bitmap.as_rgba_bytes();
    let png_data = encode_png(&rgba_data, width, height)?;

    Ok(PdfPageResult {
        data: STANDARD.encode(&png_data),
        width,
        height,
    })
}

fn encode_png(rgba_data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    use image::ImageEncoder;
    encoder
        .write_image(
            rgba_data,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    Ok(buf)
}

#[tauri::command]
pub fn search_pdf_text(path: String, query: String) -> Result<Vec<PdfSearchResult>, String> {
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let pdfium = load_pdfium()?;
    let document = pdfium
        .load_pdf_from_file(&path, None)
        .map_err(|e| format!("Failed to load PDF: {:?}", e))?;

    let options = PdfSearchOptions::new().match_case(false);
    let mut results = Vec::new();

    for page_index in 0..document.pages().len() {
        let page = document
            .pages()
            .get(page_index as u16)
            .map_err(|e| format!("Failed to get page {}: {:?}", page_index, e))?;

        let text_page = page
            .text()
            .map_err(|e| format!("Failed to extract text from page {}: {:?}", page_index, e))?;

        // Use pdfium's built-in search
        let search = text_page.search(&query, &options);
        if let Ok(search) = search {
            let mut matches = Vec::new();

            // Iterate through all search results
            let mut current = search.find_next();
            while let Some(segments) = current {
                // Each search result may span multiple segments
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
}
