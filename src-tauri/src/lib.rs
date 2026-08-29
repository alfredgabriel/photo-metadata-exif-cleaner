use std::fs;
use std::io::Cursor;
use std::path::Path;
use bytes::Bytes;
use img_parts::{jpeg::Jpeg, ImageEXIF};
use serde::{Deserialize, Serialize};
use exif::{In, Tag, Reader};

#[derive(Serialize, Deserialize)]
pub struct ExifTag {
    pub key: String,
    pub value: String,
}

#[tauri::command]
fn read_exif(file_path: String) -> Result<Vec<ExifTag>, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist".into());
    }

    let mut tags: Vec<ExifTag> = Vec::new();

    // File size
    if let Ok(meta) = fs::metadata(path) {
        let size = meta.len();
        let size_str = if size >= 1_048_576 {
            format!("{:.1} MB", size as f64 / 1_048_576.0)
        } else {
            format!("{:.0} KB", size as f64 / 1024.0)
        };
        tags.push(ExifTag { key: "file_size".to_string(), value: size_str });
    }

    let input = fs::read(path).map_err(|e| e.to_string())?;
    let mut cursor = Cursor::new(&input);

    match Reader::new().read_from_container(&mut cursor) {
        Ok(exif) => {
            let field_map = vec![
                (Tag::Make, "make"),
                (Tag::Model, "model"),
                (Tag::DateTime, "datetime"),
                (Tag::DateTimeOriginal, "datetime_original"),
                (Tag::Software, "software"),
                (Tag::ImageWidth, "width"),
                (Tag::ImageLength, "height"),
                (Tag::GPSLatitude, "gps_latitude"),
                (Tag::GPSLongitude, "gps_longitude"),
                (Tag::GPSAltitude, "gps_altitude"),
                (Tag::FocalLength, "focal_length"),
                (Tag::FNumber, "aperture"),
                (Tag::ExposureTime, "exposure_time"),
                (Tag::ISOSpeed, "iso"),
                (Tag::LensModel, "lens_model"),
                (Tag::Copyright, "copyright"),
                (Tag::Artist, "author"),
            ];

            for (tag, key) in field_map {
                if let Some(field) = exif.get_field(tag, In::PRIMARY) {
                    let val = field.display_value().with_unit(&exif).to_string();
                    if !val.is_empty() && val != "\"\"" {
                        tags.push(ExifTag {
                            key: key.to_string(),
                            value: val,
                        });
                    }
                }
            }

            if tags.len() == 1 {
                // Only file size added, no EXIF found
                tags.push(ExifTag {
                    key: "has_exif".to_string(),
                    value: "false".to_string(),
                });
            }
        }
        Err(_) => {
            tags.push(ExifTag {
                key: "has_exif".to_string(),
                value: "false".to_string(),
            });
        }
    }

    Ok(tags)
}

#[tauri::command]
fn clean_exif(file_paths: Vec<String>) -> Result<Vec<String>, String> {
    let mut cleaned_files = Vec::new();

    for file_path in file_paths {
        let path = Path::new(&file_path);
        if !path.exists() {
            continue;
        }

        let parent = path.parent().unwrap_or(Path::new(""));
        let cleaned_dir = parent.join("Cleaned");

        if !cleaned_dir.exists() {
            if let Err(e) = fs::create_dir_all(&cleaned_dir) {
                return Err(format!("Failed to create Cleaned directory: {}", e));
            }
        }

        let file_name = path.file_name().unwrap().to_string_lossy();
        let out_path = cleaned_dir.join(format!("{}", file_name));
        let input = fs::read(path).map_err(|e| e.to_string())?;

        if let Ok(mut jpeg) = Jpeg::from_bytes(Bytes::from(input)) {
            jpeg.set_exif(None);
            let mut out = fs::File::create(&out_path).map_err(|e| e.to_string())?;
            jpeg.encoder().write_to(&mut out).map_err(|e| e.to_string())?;
            cleaned_files.push(out_path.to_string_lossy().into_owned());
        }
    }

    if cleaned_files.is_empty() {
        return Err("No valid JPEG files processed.".into());
    }

    Ok(cleaned_files)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![read_exif, clean_exif])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
