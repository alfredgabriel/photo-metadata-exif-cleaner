use std::fs;
use std::path::Path;
use bytes::Bytes;
use img_parts::{jpeg::Jpeg, ImageEXIF};

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

        // Create 'Cleaned' directory if it doesn't exist
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
        .invoke_handler(tauri::generate_handler![clean_exif])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
