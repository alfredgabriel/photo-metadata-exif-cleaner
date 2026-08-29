use std::fs;
use std::io::Cursor;
use std::path::Path;
use bytes::Bytes;
use img_parts::{jpeg::Jpeg, png::Png, ImageEXIF};
use serde::{Deserialize, Serialize};
use exif::Reader;
use base64::{Engine, engine::general_purpose::STANDARD};

#[derive(Serialize, Deserialize, Clone)]
pub struct ExifTag {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    pub tags: Vec<ExifTag>,
    pub file_size: String,
    pub preview_b64: String,
    pub mime: String,
    pub format: String,
}

#[tauri::command]
fn read_exif(file_path: String) -> Result<FileInfo, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist".into());
    }
    let input = fs::read(path).map_err(|e| e.to_string())?;

    // File size
    let size = input.len() as u64;
    let file_size = if size >= 1_048_576 {
        format!("{:.1} MB", size as f64 / 1_048_576.0)
    } else {
        format!("{:.0} KB", size as f64 / 1024.0)
    };

    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "tif" | "tiff" => "image/tiff",
        _ => "image/jpeg",
    }.to_string();

    // Preview (limit to 4MB)
    let preview_data = if input.len() > 4_000_000 {
        input[..4_000_000].to_vec()
    } else {
        input.clone()
    };
    let preview_b64 = STANDARD.encode(&preview_data);

    // Read EXIF
    let mut tags: Vec<ExifTag> = Vec::new();
    let mut cursor = Cursor::new(&input);
    match Reader::new().read_from_container(&mut cursor) {
        Ok(exif) => {
            for field in exif.fields() {
                let value = field.display_value().with_unit(&exif).to_string();
                if !value.is_empty() && value != "\"\"" {
                    tags.push(ExifTag {
                        key: format!("{}", field.tag),
                        value,
                    });
                }
            }
        }
        Err(_) => {}
    }

    Ok(FileInfo { tags, file_size, preview_b64, mime, format: ext })
}

#[tauri::command]
fn clean_exif(file_paths: Vec<String>) -> Result<String, String> {
    let mut cleaned_count = 0;
    let mut output_dir = String::new();

    for file_path in &file_paths {
        let path = Path::new(file_path);
        if !path.exists() { continue; }

        let parent = path.parent().unwrap_or(Path::new(""));
        let cleaned_dir = parent.join("Cleaned");
        if !cleaned_dir.exists() {
            fs::create_dir_all(&cleaned_dir).map_err(|e| e.to_string())?;
        }
        if output_dir.is_empty() {
            output_dir = cleaned_dir.to_string_lossy().into_owned();
        }

        let file_name = path.file_name().unwrap().to_string_lossy();
        let out_path = cleaned_dir.join(format!("{}", file_name));
        let input = fs::read(path).map_err(|e| e.to_string())?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        let cleaned: Option<Vec<u8>> = match ext.as_str() {
            "jpg" | "jpeg" => {
                if let Ok(mut jpeg) = Jpeg::from_bytes(Bytes::from(input)) {
                    jpeg.set_exif(None);
                    let mut buf = Vec::new();
                    jpeg.encoder().write_to(&mut buf).map_err(|e| e.to_string())?;
                    Some(buf)
                } else { None }
            }
            "png" => {
                if let Ok(mut png) = Png::from_bytes(Bytes::from(input)) {
                    png.set_exif(None);
                    let mut buf = Vec::new();
                    png.encoder().write_to(&mut buf).map_err(|e| e.to_string())?;
                    Some(buf)
                } else { None }
            }
            _ => Some(input),
        };

        if let Some(data) = cleaned {
            fs::write(&out_path, data).map_err(|e| e.to_string())?;
            cleaned_count += 1;
        }
    }

    if cleaned_count == 0 {
        return Err("No files were processed.".into());
    }
    Ok(output_dir)
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&path).spawn().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![read_exif, clean_exif, open_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}