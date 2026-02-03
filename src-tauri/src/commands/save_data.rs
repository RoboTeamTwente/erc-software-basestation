use serde_json::map;
use tauri::{AppHandle, Manager};
use std::fs;
use std::path::Path;

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


// Map file
#[tauri::command]
pub fn import_map_file(app: AppHandle, source_path: String) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let maps_dir = app_data_dir.join("maps");

    let source = Path::new(&source_path);

    let file_name = source
        .file_name()
        .ok_or("Invalid file name".to_string())?;

    let destination = maps_dir.join(file_name);

    fs::copy(source, destination).map_err(|e| e.to_string())?;

    println!("Map file imported successfully.");
    Ok(())
}
