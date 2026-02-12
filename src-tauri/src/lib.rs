// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;

use std::sync::Mutex;
use commands::maps::MapState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(MapState {
            displayed_map_name: Mutex::new(String::new()),
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::file_management::save_task_file,
            commands::file_management::list_task_files,
            commands::file_management::delete_all_task_files,
            commands::file_management::delete_one_file,
            commands::file_management::read_task_file,
            commands::file_management::import_map_file,
            commands::file_management::get_app_dir,
            commands::file_management::save_snapshot,
            commands::checks::ping,
            commands::checks::clear_cache,
            commands::maps::selected_map_to_backend,
            commands::maps::selected_map_from_backend,
        ])
        .setup(|app| {
            if let Err(e) = commands::file_management::ensure_storage_dirs_internal(app.handle()) {
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
