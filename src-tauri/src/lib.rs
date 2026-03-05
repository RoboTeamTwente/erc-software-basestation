// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod proto;
mod network;

use std::sync::Mutex;
use tauri::Manager;

use commands::rover_states::RoverState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RoverState {
            drive_manual_mode: Mutex::new(true),
            arm_manual_mode: Mutex::new(true),
            pickup_mode: Mutex::new(false),
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
            commands::controller::pressed_key,
            commands::rover_states::get_state,
            commands::rover_states::set_state,
            commands::network::send_ping_cmd,
        ])
        .setup(|app| {
            if let Err(e) = commands::file_management::ensure_storage_dirs_internal(app.handle()) {
                eprintln!("Failed to ensure storage dirs: {}", e);
            }
            if let Err(e) = commands::checks::clear_cache_on_startup() {
                eprintln!("Failed to clear cache on startup: {}", e);
            }
            // Spawn gstream receiver
            tauri::async_runtime::spawn(async {
                // Import your streaming module
                if let Err(e) = commands::gstreamer::stream().await {
                    eprintln!("MJPEG streaming server error: {}", e);
                }
            });

            // Spawn udp service
            let udp_service = tauri::async_runtime::block_on(async {
                network::service::UdpService::new("0.0.0.0:9000")
                    .await
                    .expect("Failed to start UDP service")
            });

            // Extract socket BEFORE moving service
            let udp_socket = udp_service.socket();

            // Register service so commands can access it
            app.handle().manage(udp_service);

            // Spawn listener and MOVE the socket into it
            tauri::async_runtime::spawn(async move {
                if let Err(e) = network::listener::run_listener(udp_socket).await {
                    eprintln!("UDP listener error: {e}");
                }
            });

            Ok(())
        })

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
