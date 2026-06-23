use base64::{Engine, engine::general_purpose::STANDARD};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static FFMPEG_CHECKED: AtomicBool = AtomicBool::new(false);
static FFMPEG_AVAILABLE: AtomicBool = AtomicBool::new(false);
static FFMPEG_CHECK_LOCK: Mutex<()> = Mutex::new(());

// --- Video HTTP streaming server state ---

struct VideoServerState {
    stop_flag: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

static VIDEO_SERVER: Mutex<Option<VideoServerState>> = Mutex::new(None);

// --- Thumbnail types and commands ---

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoThumbnail {
    pub data: String,        // base64-encoded JPEG
    pub width: u32,
    pub height: u32,
    pub duration_seconds: f64,
    pub file_size: u64,
}

fn check_ffmpeg() -> bool {
    if FFMPEG_CHECKED.load(Ordering::Relaxed) {
        return FFMPEG_AVAILABLE.load(Ordering::Relaxed);
    }
    let _lock = FFMPEG_CHECK_LOCK.lock().unwrap();
    if FFMPEG_CHECKED.load(Ordering::Relaxed) {
        return FFMPEG_AVAILABLE.load(Ordering::Relaxed);
    }
    let available = std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    FFMPEG_AVAILABLE.store(available, Ordering::Relaxed);
    FFMPEG_CHECKED.store(true, Ordering::Relaxed);
    available
}

fn parse_meta_from_stderr(stderr: &str) -> (f64, u32, u32) {
    let mut duration: f64 = 0.0;
    let mut width: u32 = 0;
    let mut height: u32 = 0;

    if let Some(caps) = Regex::new(r"Duration:\s*(\d+):(\d+):(\d+\.\d+)")
        .ok()
        .and_then(|re| re.captures(stderr))
    {
        let h: f64 = caps.get(1).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
        let m: f64 = caps.get(2).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
        let s: f64 = caps.get(3).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
        duration = h * 3600.0 + m * 60.0 + s;
    }

    if let Some(caps) = Regex::new(r"(\d{2,5})x(\d{2,5})")
        .ok()
        .and_then(|re| {
            let stream_section = stderr.split("Stream #").skip(1).find(|s| s.contains("Video:"));
            if let Some(section) = stream_section {
                re.captures(section)
            } else {
                re.captures(stderr)
            }
        })
    {
        width = caps.get(1).map(|m| m.as_str().parse().unwrap_or(0)).unwrap_or(0);
        height = caps.get(2).map(|m| m.as_str().parse().unwrap_or(0)).unwrap_or(0);
    }

    (duration, width, height)
}

#[tauri::command]
pub fn get_video_thumbnail(path: String) -> Result<VideoThumbnail, String> {
    let file_path = std::path::Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    if !check_ffmpeg() {
        return Err("ffmpeg not found. Run 'winget install ffmpeg' to install.".to_string());
    }

    let file_size = fs::metadata(file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let path_clone = path.clone();

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.args([
        "-ss", "10",
        "-i", &path_clone,
        "-frames:v", "1",
        "-f", "image2pipe",
        "-v", "quiet",
        "-c:v", "mjpeg",
        "-vf", "scale='if(gte(iw,ih),400,-2):if(gte(ih,iw),400,-2):flags=neighbor'",
        "-",
    ]);

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    let mut stdout = child.stdout.take().unwrap();
    let handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).map(|_| buf)
    });

    let jpeg_data = match handle.join() {
        Ok(Ok(buf)) => buf,
        Ok(Err(e)) => return Err(format!("Failed to read ffmpeg output: {}", e)),
        Err(_) => return Err("ffmpeg process panicked".to_string()),
    };

    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait on ffmpeg: {}", e))?;

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let (duration, mut width, mut height) = parse_meta_from_stderr(&stderr_str);

    let jpeg_data = if jpeg_data.is_empty() || jpeg_data.len() < 100 {
        let mut cmd2 = std::process::Command::new("ffmpeg");
        cmd2.args([
            "-ss", "1",
            "-i", &path,
            "-frames:v", "1",
            "-f", "image2pipe",
            "-v", "quiet",
            "-c:v", "mjpeg",
            "-vf", "scale='if(gte(iw,ih),400,-2):if(gte(ih,iw),400,-2):flags=neighbor'",
            "-",
        ]);

        let output2 = cmd2.output()
            .map_err(|e| format!("Failed to run ffmpeg retry: {}", e))?;

        let stderr2 = String::from_utf8_lossy(&output2.stderr);
        if duration == 0.0 {
            let (d2, w2, h2) = parse_meta_from_stderr(&stderr2);
            if d2 > 0.0 { width = w2; height = h2; }
        }

        output2.stdout
    } else {
        jpeg_data
    };

    if jpeg_data.is_empty() {
        return Err("Failed to extract video frame: empty output".to_string());
    }

    Ok(VideoThumbnail {
        data: STANDARD.encode(&jpeg_data),
        width,
        height,
        duration_seconds: duration,
        file_size,
    })
}

// --- Video HTTP streaming server ---

fn guess_mime(path: &str) -> &'static str {
    let ext = path.split('.').last().map(|e| e.to_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "mp4" => "video/mp4",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "wmv" => "video/x-ms-wmv",
        "flv" => "video/x-flv",
        "webm" => "video/webm",
        "m4v" => "video/x-m4v",
        "mpg" | "mpeg" => "video/mpeg",
        "ts" => "video/mp2t",
        _ => "application/octet-stream",
    }
}

// Cap single chunk size to avoid reading the entire file at once.
// The browser issues follow-up Range requests for subsequent chunks.
const MAX_CHUNK_SIZE: u64 = 8 * 1024 * 1024; // 8 MB

struct RangeHeader {
    start: u64,
    end: u64, // inclusive
}

fn parse_range_header(header: &str, file_size: u64) -> Option<RangeHeader> {
    let re = Regex::new(r"bytes=(\d+)-(\d*)").ok()?;
    let caps = re.captures(header)?;
    let start: u64 = caps.get(1)?.as_str().parse().ok()?;
    let end_str = caps.get(2).map(|m| m.as_str());

    if start >= file_size {
        return None;
    }

    // Calculate end byte (inclusive)
    let requested_end = if let Some(s) = end_str {
        if s.is_empty() {
            // Open-ended: "bytes=N-" → cap to chunk size
            None
        } else {
            Some(s.parse::<u64>().ok()?.min(file_size - 1))
        }
    } else {
        // No end at all (shouldn't happen with valid Range, but handle gracefully)
        None
    };

    let end = if let Some(req_end) = requested_end {
        // Closed range: respect but still cap to avoid one giant read
        req_end.min(start + MAX_CHUNK_SIZE - 1)
    } else {
        // Open-ended: cap to chunk size
        (start + MAX_CHUNK_SIZE - 1).min(file_size - 1)
    };

    if start > end {
        return None;
    }

    Some(RangeHeader { start, end })
}

fn run_video_server(file_path: String, stop_flag: Arc<AtomicBool>, port: u16) {
    let server = match tiny_http::Server::http(format!("127.0.0.1:{}", port)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[video-server] Failed to start server: {}", e);
            return;
        }
    };

    eprintln!("[video-server] Started on port {} serving: {}", port, file_path);

    loop {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        let request = match server.recv_timeout(Duration::from_millis(200)) {
            Ok(Some(req)) => req,
            Ok(None) => continue,
            Err(e) => {
                eprintln!("[video-server] recv error: {}", e);
                break;
            }
        };

        let method = request.method();
        if method != &tiny_http::Method::Get && method != &tiny_http::Method::Head {
            let _ = request.respond(tiny_http::Response::empty(405));
            continue;
        }

        let file_size = match fs::metadata(&file_path) {
            Ok(m) => m.len(),
            Err(_) => {
                let _ = request.respond(tiny_http::Response::empty(404));
                break;
            }
        };

        let mime = guess_mime(&file_path);

        // Check for Range header
        let range = request.headers().iter()
            .find(|h| h.field.equiv("Range"))
            .and_then(|h| parse_range_header(h.value.as_str(), file_size));

        if let Some(r) = range {
            let chunk_size = (r.end - r.start + 1) as usize;
            let mut file = match fs::File::open(&file_path) {
                Ok(f) => f,
                Err(_) => {
                    let _ = request.respond(tiny_http::Response::empty(500));
                    break;
                }
            };

            if file.seek(SeekFrom::Start(r.start)).is_err() {
                let _ = request.respond(tiny_http::Response::empty(500));
                break;
            }

            let mut buf = vec![0u8; chunk_size];
            if file.read_exact(&mut buf).is_err() {
                let _ = request.respond(tiny_http::Response::empty(500));
                break;
            }

            let content_range = format!("bytes {}-{}/{}", r.start, r.end, file_size);
            let response = tiny_http::Response::new(
                206.into(),
                vec![
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &mime.as_bytes()[..]).unwrap(),
                    tiny_http::Header::from_bytes(&b"Content-Range"[..], &content_range.as_bytes()[..]).unwrap(),
                    tiny_http::Header::from_bytes(&b"Content-Length"[..], &chunk_size.to_string().as_bytes()[..]).unwrap(),
                    tiny_http::Header::from_bytes(&b"Accept-Ranges"[..], &b"bytes"[..]).unwrap(),
                    tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
                ],
                buf.as_slice(),
                Some(buf.len()),
                None,
            );
            let _ = request.respond(response);
        } else {
            // No Range header — serve first chunk as 206 to keep memory bounded.
            // Browsers issue Range for <video> anyway; this handles edge cases.
            let chunk_end = (MAX_CHUNK_SIZE - 1).min(file_size - 1);
            let chunk_size = (chunk_end + 1) as usize;
            let mut file = match fs::File::open(&file_path) {
                Ok(f) => f,
                Err(_) => {
                    let _ = request.respond(tiny_http::Response::empty(500));
                    break;
                }
            };

            let mut buf = vec![0u8; chunk_size];
            if file.read_exact(&mut buf).is_err() {
                let _ = request.respond(tiny_http::Response::empty(500));
                break;
            }

            let content_range = format!("bytes 0-{}/{}", chunk_end, file_size);
            let response = tiny_http::Response::new(
                206.into(),
                vec![
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &mime.as_bytes()[..]).unwrap(),
                    tiny_http::Header::from_bytes(&b"Content-Range"[..], &content_range.as_bytes()[..]).unwrap(),
                    tiny_http::Header::from_bytes(&b"Content-Length"[..], &chunk_size.to_string().as_bytes()[..]).unwrap(),
                    tiny_http::Header::from_bytes(&b"Accept-Ranges"[..], &b"bytes"[..]).unwrap(),
                    tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
                ],
                buf.as_slice(),
                Some(buf.len()),
                None,
            );
            let _ = request.respond(response);
        }
    }

    eprintln!("[video-server] Stopped");
}

#[tauri::command]
pub fn start_video_server(path: String) -> Result<String, String> {
    let file_path = std::path::Path::new(&path);

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    // Stop any existing server first
    {
        let mut guard = VIDEO_SERVER.lock().unwrap();
        if let Some(state) = guard.take() {
            state.stop_flag.store(true, Ordering::Relaxed);
            if let Some(handle) = state.handle {
                let _ = handle.join();
            }
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();
    let path_clone = path.clone();

    // Bind to a random port
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to bind: {}", e))?;
    let port = listener.local_addr()
        .map_err(|e| format!("Failed to get port: {}", e))?
        .port();
    drop(listener); // Release the port so tiny_http can bind it

    let handle = std::thread::spawn(move || {
        run_video_server(path_clone, stop_flag_clone, port);
    });

    let url = format!("http://127.0.0.1:{}/video", port);

    let mut guard = VIDEO_SERVER.lock().unwrap();
    *guard = Some(VideoServerState {
        stop_flag,
        handle: Some(handle),
    });

    eprintln!("[video-server] Returning URL: {}", url);
    Ok(url)
}

#[tauri::command]
pub fn stop_video_server() -> Result<(), String> {
    let mut guard = VIDEO_SERVER.lock().unwrap();
    if let Some(state) = guard.take() {
        state.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = state.handle {
            let _ = handle.join();
        }
    }
    Ok(())
}
