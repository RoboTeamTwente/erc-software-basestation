// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::save_data::save_task_file,
            commands::save_data::list_task_files,
            commands::save_data::delete_all_task_files,
            commands::save_data::read_task_file,
            commands::save_data::import_map_file,
            commands::checks::ping,
        ])
        .setup(|app| {
            if let Err(e) = commands::save_data::ensure_storage_dirs_internal(app.handle()) {
                eprintln!("Failed to ensure storage dirs: {}", e);
            }
            // Spawn gstream receiver
            tauri::async_runtime::spawn(async {
                // Import your streaming module
                if let Err(e) = commands::gstreamer::stream().await {
                    eprintln!("MJPEG streaming server error: {}", e);
                }
            });

            Ok(())
        })

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
