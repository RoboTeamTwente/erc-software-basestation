use std::fs;
use tauri::path::BaseDirectory;


#[tauri::command]
pub fn ping() {
    println!("PING FROM RUST");
}


#[tauri::command]
pub async fn clear_cache() -> Result<(), String> {
    // Get the cache directory using dirs crate
    let cache_dir = match BaseDirectory::Cache {
        BaseDirectory::Cache => {
            dirs::cache_dir().ok_or("Failed to get cache directory")?
        }
        _ => return Err("Invalid BaseDirectory".into()),
    };

    // Optional: append your app-specific folder
    let app_cache_dir = cache_dir.join("base_station"); 

    println!("Cache directory: {:?}", &app_cache_dir);

    if app_cache_dir.exists() {
        // Iterate over each entry in the folder and remove it
        for entry in fs::read_dir(&app_cache_dir)
            .map_err(|e| format!("Failed to read cache directory: {}", e))? 
        {
            let entry = entry.map_err(|e| format!("Failed to read cache entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .map_err(|e| format!("Failed to remove directory {:?}: {}", path, e))?;
            } else {
                fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove file {:?}: {}", path, e))?;
            }
        }

        println!("Cache contents cleared successfully.");
    } else {
        println!("Cache directory does not exist, nothing to clear.");
    }

    Ok(())
}

/// Clear only the contents of the cache folder (for setup)
pub fn clear_cache_on_startup() -> Result<(), String> {
    // Get the system cache directory
    let cache_dir = match BaseDirectory::Cache {
        BaseDirectory::Cache => {
            dirs::cache_dir().ok_or("Failed to get cache directory")?
        }
        _ => return Err("Invalid BaseDirectory".into()),
    };

    // Append your app-specific folder
    let app_cache_dir = cache_dir.join("my_app_name"); 

    println!("Cache directory: {:?}", &app_cache_dir);

    if app_cache_dir.exists() {
        for entry in fs::read_dir(&app_cache_dir)
            .map_err(|e| format!("Failed to read cache directory: {}", e))? 
        {
            let entry = entry.map_err(|e| format!("Failed to read cache entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .map_err(|e| format!("Failed to remove directory {:?}: {}", path, e))?;
            } else {
                fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove file {:?}: {}", path, e))?;
            }
        }
        println!("Cache contents cleared successfully on startup.");
    } else {
        println!("Cache directory does not exist, nothing to clear.");
    }

    Ok(())
}
