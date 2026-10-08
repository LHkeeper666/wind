use std::collections::HashMap;
use std::sync::Mutex;

use log::debug;

static ENCODING_CACHE: Mutex<Option<HashMap<String, Option<&'static encoding_rs::Encoding>>>> =
    Mutex::new(None);

/// Try to detect the encoding for a ZIP archive's entry names.
/// Returns the detected encoding, or None if detection failed.
pub(crate) fn detect_archive_encoding(archive_path: &str, name_bytes: &[u8]) -> Option<&'static encoding_rs::Encoding> {
    // Check cache first
    {
        let cache = ENCODING_CACHE.lock().unwrap();
        if let Some(ref map) = *cache {
            if let Some(cached) = map.get(archive_path) {
                return *cached;
            }
        }
    }

    let encoding = detect_encoding_from_bytes(name_bytes);
    debug!("[archive] encoding detected for {}: {:?}", archive_path, encoding.map(|e| e.name()));
    let mut cache = ENCODING_CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(HashMap::new());
    }
    cache.as_mut().unwrap().insert(archive_path.to_string(), encoding);
    encoding
}

pub(crate) fn detect_encoding_from_bytes(bytes: &[u8]) -> Option<&'static encoding_rs::Encoding> {
    if String::from_utf8(bytes.to_vec()).is_ok() {
        return Some(encoding_rs::UTF_8);
    }

    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    if encoding == encoding_rs::UTF_8 {
        // chardetng defaults to UTF-8 for short/ambiguous input; trust it
        Some(encoding_rs::UTF_8)
    } else {
        Some(encoding)
    }
}

pub(crate) fn decode_name(raw: &[u8], encoding: Option<&'static encoding_rs::Encoding>) -> String {
    match encoding {
        Some(enc) => enc.decode_without_bom_handling(raw).0.into_owned(),
        None => String::from_utf8_lossy(raw).to_string(),
    }
}

pub(crate) fn decode_tar_name(path: &str, entry: &tar::Entry<impl std::io::Read>) -> String {
    let raw = entry.path_bytes();
    let encoding = detect_archive_encoding(path, &raw);
    decode_name(&raw, encoding)
}
