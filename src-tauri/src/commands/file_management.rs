use tauri::{AppHandle, Manager};
use std::fs;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

// Take a reference instead of moving
pub fn ensure_storage_dirs_internal(app: &AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let tasks_dir = app_data_dir.join("tasks");
    let images_dir = app_data_dir.join("images");
    let maps_dir = app_data_dir.join("maps");

    std::fs::create_dir_all(&tasks_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&maps_dir).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn save_task_file(app: AppHandle, file_name: String, data: Vec<u8>, directory: String) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let save_in_dir = app_data_dir.join(&directory);
    
    println!("Saving in directory: {}", save_in_dir.display());

    let file_path = save_in_dir.join(file_name);

    fs::write(file_path, data).map_err(|e| e.to_string())?;
    println!("Task file saved successfully.");
    Ok(())
}

#[tauri::command]
pub fn list_task_files(app: AppHandle, directory: String) -> Result<Vec<String>, String> {
    let chosen_dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join(&directory);


    if !chosen_dir.exists() {
        return Ok(vec![]);
    }

    let entries = fs::read_dir(chosen_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    println!("Files in tasks directory: {:?}", entries);
    println!("Listed task files successfully.");
    Ok(entries)
}

#[tauri::command]
pub fn delete_all_task_files(app: AppHandle, directory: String) -> Result<(), String> {
    let chosen_dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join(&directory);

    if chosen_dir.exists() {
        fs::remove_dir_all(&chosen_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&chosen_dir).map_err(|e| e.to_string())?;
    }

    println!("All {} files deleted successfully.", &directory);
    Ok(())
}

#[tauri::command]
pub fn read_task_file(app: AppHandle, file_name: String) -> Result<Vec<u8>, String> {
    let tasks_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("tasks");

    let file_path = tasks_dir.join(file_name);

    let data = std::fs::read(file_path).map_err(|e| e.to_string())?;
    Ok(data)
}


// Add map file to map folder
#[tauri::command]
pub fn import_map_file(app: AppHandle, directory: String) -> Result<(), String> {
    let chosen_file_dir = PathBuf::from(&directory);
    let destination_dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("maps");

    // Make sure the file exists
    if !chosen_file_dir.exists() {
        return Err(format!("File does not exist: {}", directory));
    }

    // Create the "maps" folder if it doesn't exist
    if !destination_dir.exists() {
        fs::create_dir_all(&destination_dir).map_err(|e| e.to_string())?;
    }

    // Build the destination file path (keep the original file name)
    let file_name = chosen_file_dir
        .file_name()
        .ok_or("Invalid file name".to_string())?;
    let destination_file = destination_dir.join(file_name);

    // Copy the file
    fs::copy(&chosen_file_dir, &destination_file).map_err(|e| e.to_string())?;

    Ok(())
}

//Get home directory
#[tauri::command]
pub fn get_app_dir(app: AppHandle) -> Result<String, String> {
    let app_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?;

    Ok(app_dir.to_string_lossy().to_string())
}

//Save image from video (for Science task)
#[tauri::command]
pub async fn save_snapshot(app: AppHandle, port: String) -> Result<(), String> {
    use tokio::fs;
    use std::io::Read;

    let images_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("images");

    if fs::metadata(&images_dir).await.is_err() {
        fs::create_dir_all(&images_dir)
            .await
            .map_err(|e| format!("Failed to create images directory: {}", e))?;
    }

    let url = format!("http://localhost:{}", port);

    // Spawn blocking to safely fetch the first JPEG frame
    let jpeg_data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
        let mut resp = reqwest::blocking::get(&url).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        let mut temp = [0u8; 1024]; // read in chunks
        let mut found_start = false;

        while let Ok(n) = resp.read(&mut temp) {
            if n == 0 {
                break;
            }

            buffer.extend_from_slice(&temp[..n]);

            if !found_start {
                if let Some(pos) = buffer.windows(2).position(|w| w == [0xFF, 0xD8]) {
                    buffer = buffer[pos..].to_vec(); // discard anything before JPEG start
                    found_start = true;
                } else {
                    buffer.clear();
                }
            }

            if found_start && buffer.windows(2).any(|w| w == [0xFF, 0xD9]) {
                break; // stop after first JPEG frame
            }

            // Safety: limit max size to 5MB to avoid huge buffers
            if buffer.len() > 5_000_000 {
                return Err("Frame too large".into());
            }
        }

        if buffer.is_empty() {
            return Err("No JPEG frame found".into());
        }

        Ok(buffer)
    })
    .await
    .map_err(|e| e.to_string())??;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let file_path = images_dir.join(format!("snapshot_{}.jpg", timestamp)); // ---------> TODO: change naming to match task and sample

    let mut file = fs::File::create(file_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&jpeg_data)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}
