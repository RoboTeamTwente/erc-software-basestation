// Serves the .glb file directly from disk, bypassing WebKit's HTTP loader
// which is where the WebKit bug (WebLoaderStrategy.cpp:618) originates.
 
#[tauri::command]
pub fn load_model(path: String) -> Result<Vec<u8>, String> {
    // CARGO_MANIFEST_DIR is src-tauri/, so .parent() gives the project root.
    // path is e.g. "models/your-model.glb", resolving to static/models/your-model.glb
    let full_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("static")
        .join(&path);
 
    println!("Loading model from: {:?}", full_path);
    std::fs::read(&full_path).map_err(|e| format!("Failed to read {:?}: {}", full_path, e))
}