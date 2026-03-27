use tauri::Manager;

#[tauri::command]
pub fn load_model(app: tauri::AppHandle, path: String) -> Result<Vec<u8>, String> {
    let filename = std::path::Path::new(&path)
        .file_name()
        .ok_or("Invalid path")?;

    let resource_path = if cfg!(debug_assertions) {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("models")
            .join(filename)
    } else {
        app.path()
            .resource_dir()
            .map_err(|e| format!("Could not get resource dir: {}", e))?
            .join("models")
            .join(filename)
    };

    println!("Loading model from: {:?}", resource_path);

    std::fs::read(&resource_path)
        .map_err(|e| format!("Failed to read {:?}: {}", resource_path, e))
}

#[tauri::command]
pub fn debug_resource_dir(app: tauri::AppHandle) -> String {
    let dir = app.path().resource_dir().unwrap();
    let entries = std::fs::read_dir(&dir)
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| format!("{:?}", e.path())).collect::<Vec<_>>())
        .unwrap_or_default();
    format!("resource_dir: {:?}\ncontents: {:#?}", dir, entries)
}