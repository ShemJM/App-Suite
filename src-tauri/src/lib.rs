use chrono::NaiveDateTime;
use exif::{In, Reader, Tag, Value};
use serde::Serialize;
use std::io::BufReader;
use std::path::Path;
use std::time::UNIX_EPOCH;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use walkdir::WalkDir;

#[derive(Serialize)]
pub struct PhotoEntry {
    path: String,
    filename: String,
    /// Unix timestamp in milliseconds (EXIF date preferred, mtime fallback)
    timestamp: i64,
}

fn is_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref(),
        Some(
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic" | "heif" | "tiff" | "tif" | "bmp"
                | "avif"
        )
    )
}

fn exif_timestamp(path: &Path) -> Option<i64> {
    let file = std::fs::File::open(path).ok()?;
    let exif = Reader::new()
        .read_from_container(&mut BufReader::new(file))
        .ok()?;

    let field = exif
        .get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .or_else(|| exif.get_field(Tag::DateTime, In::PRIMARY))?;

    if let Value::Ascii(ref v) = field.value {
        let raw = v.first()?;
        let s = std::str::from_utf8(raw).ok()?;
        // EXIF format: "2023:06:15 14:30:00"
        let dt = NaiveDateTime::parse_from_str(s.trim(), "%Y:%m:%d %H:%M:%S").ok()?;
        Some(dt.and_utc().timestamp_millis())
    } else {
        None
    }
}

fn mtime_timestamp(path: &Path) -> Option<i64> {
    let ms = std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis();
    Some(ms as i64)
}

#[tauri::command]
fn scan_directory(path: String) -> Result<Vec<PhotoEntry>, String> {
    let mut photos: Vec<PhotoEntry> = WalkDir::new(&path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_image(e.path()))
        .filter_map(|e| {
            let p = e.path();
            let timestamp = exif_timestamp(p).or_else(|| mtime_timestamp(p)).unwrap_or(0);
            Some(PhotoEntry {
                path: p.to_string_lossy().into_owned(),
                filename: e.file_name().to_string_lossy().into_owned(),
                timestamp,
            })
        })
        .collect();

    // Newest first
    photos.sort_unstable_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(photos)
}

#[tauri::command]
async fn open_app_window(
    app: AppHandle,
    label: String,
    title: String,
    route: String,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(&label) {
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, &label, WebviewUrl::App(route.into()))
        .title(title)
        .inner_size(width, height)
        .min_inner_size(900.0, 600.0)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![open_app_window, scan_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
