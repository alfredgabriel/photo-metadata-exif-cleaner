use std::fs;
use std::path::Path;
use bytes::Bytes;
use img_parts::{jpeg::Jpeg, ImageEXIF};

#[tauri::command]
fn clean_exif(file_path: String, output_dir: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist".into());
    }

    let file_name = path.file_name().unwrap().to_string_lossy();
    let out_path = Path::new(&output_dir).join(format!("cleaned_{}", file_name));

    let input = fs::read(path).map_err(|e| e.to_string())?;
    
    if let Ok(mut jpeg) = Jpeg::from_bytes(Bytes::from(input)) {
        jpeg.set_exif(None);
        let mut out = fs::File::create(&out_path).map_err(|e| e.to_string())?;
        jpeg.encoder().write_to(&mut out).map_err(|e| e.to_string())?;
        return Ok(out_path.to_string_lossy().into_owned());
    }
    
    Err("Unsupported file format or not a JPEG".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![clean_exif])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
