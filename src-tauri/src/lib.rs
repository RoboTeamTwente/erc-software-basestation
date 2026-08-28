// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod proto;
mod network;

use std::sync::Mutex;
use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;

use commands::rover_states::RoverState;
use crate::network::service::UdpService;
use commands::network::DummyStreamHandle;

pub struct RoverAddress {
    pub ip: Mutex<String> 
}

pub struct UdpServiceHandle {
    pub service: Arc<TokioMutex<UdpService>>,
    pub restart_token: Arc<TokioMutex<CancellationToken>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RoverState {
            manual_mode: Mutex::new(true),
            pickup_mode: Mutex::new(false),
            braked: Mutex::new(false),
        })
        .manage(RoverAddress {
            ip: Mutex::new("127.0.0.1:9000".into()), // temporary default
        })

        //Plugins must be loaded here
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
            commands::file_management::save_snapshot,
            commands::checks::ping,
            commands::checks::clear_cache,
            commands::rover_states::get_state,
            commands::rover_states::set_state,
            commands::rover_commands::request_coordinates,
            commands::rover_commands::request_weight,
            commands::rover_commands::request_measurement,
            commands::rover_commands::send_pixel,
            commands::rover_commands::select_object,
            commands::network::send_ping_cmd,
            commands::network::start_dummy_streams,
            commands::network::stop_dummy_streams,
            commands::network::start_detection_sim,
            commands::network::get_rover_address,
            commands::network::set_rover_address,
            commands::network::set_local_port,
            commands::load_model::load_model,
            commands::load_model::debug_resource_dir,
            commands::map_commands::render_map,
            commands::map_commands::pixel_to_world,
            commands::stage::go_on_stage,
            commands::stage::stop_going_on_stage,
            commands::stage::set_rover_profile,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            #[cfg(target_os = "windows")]
            std::env::set_var("GST_PLUGIN_PATH", "C:\\gstreamer\\1.0\\msvc_x86_64\\bin");

            if let Err(e) = commands::file_management::ensure_storage_dirs_internal(app.handle()) {
                eprintln!("Failed to ensure storage dirs: {}", e);
            }

            if let Err(e) = commands::checks::clear_cache_on_startup() {
                eprintln!("Failed to clear cache on startup: {}", e);
            }

            let config = commands::config::load_config(app.handle());
            *app.state::<RoverAddress>().ip.lock().unwrap() = config.ip.clone();

            // block_on because setup is sync but UdpService::new is async
            let udp_service = tauri::async_runtime::block_on(async {
                network::service::UdpService::new(config.local_port)
                    .await
                    .expect("Failed to start UDP service")
            });

            let udp_socket = udp_service.socket();
            let cancel_token = CancellationToken::new();

            app.handle().manage(UdpServiceHandle {
                service: Arc::new(TokioMutex::new(udp_service)),
                restart_token: Arc::new(TokioMutex::new(cancel_token.clone())),
            });
            app.handle().manage(DummyStreamHandle {
                token: TokioMutex::new(None),
            });

            let listener_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                network::listener::run_listener(udp_socket, cancel_token, listener_handle).await;
            });

            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::gstreamer::stream(app_handle).await {
                    eprintln!("MJPEG streaming server error: {}", e);
                }
            });

            commands::controller::start_controller_listener(app.handle().clone());

            Ok(())
        })

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
