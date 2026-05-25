use tauri::State;

use tokio::sync::Mutex; 
use tokio_util::sync::CancellationToken;
use crate::UdpServiceHandle;

use crate::network::service::UdpService;

use crate::network::dummy::SimulatorConfig;


pub struct DummyStreamHandle {
    pub token: Mutex<Option<CancellationToken>>,
}

// PING COMMAND
#[tauri::command]
pub async fn send_ping_cmd(
    state: State<'_, UdpServiceHandle>,
) -> Result<(), String> {
    let service: tokio::sync::MutexGuard<'_, UdpService> =
        state.service.lock().await;
    let _socket = service.socket();

    

    Ok(())
}


// FULL SIMULATOR
#[tauri::command]
pub async fn start_dummy_streams(
    app: tauri::AppHandle,
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    println!("[sim] start_detection_sim called");

    if let Some(old_token) = handle.token.lock().await.take() {
        println!("[sim] Cancelling old token"); 
        old_token.cancel();
    }
    let token = CancellationToken::new();
    *handle.token.lock().await = Some(token.clone());

    let target = crate::commands::config::load_config(&app).ip;

    crate::network::dummy::spawn_simulator(
        token,
        target,
        SimulatorConfig { jitter_ms: 30, packet_loss: 0.02, record_path: None },
        crate::network::dummy::stream_table(),
    );
    Ok(())
}


#[tauri::command]
pub async fn start_detection_sim(
    app: tauri::AppHandle,
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    if let Some(old_token) = handle.token.lock().await.take() {
        old_token.cancel();
    }
    let token = CancellationToken::new();
    *handle.token.lock().await = Some(token.clone());

    let target = crate::commands::config::load_config(&app).ip;

    crate::network::dummy::spawn_simulator(
        token,
        target,
        SimulatorConfig { jitter_ms: 0, packet_loss: 0.0, record_path: None },
        crate::network::dummy::detection_only_stream(),
    );
    println!("Detection-only simulator started");
    Ok(())
}

//STOP SIMULATOR
#[tauri::command]
pub async fn stop_dummy_streams(
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    if let Some(token) = handle.token.lock().await.take() {
        token.cancel();
        println!("Dummy simulator stop signalled");
    }
    Ok(())
}

#[tauri::command]
pub async fn get_rover_address(app: tauri::AppHandle) -> Result<String, String> {
    let config = crate::commands::config::load_config(&app);
    Ok(config.ip)
}

#[tauri::command]
pub async fn set_rover_address(
    app: tauri::AppHandle,
    address: String,
    state: tauri::State<'_, crate::RoverAddress>,
) -> Result<(), String> {
    // Basic validation
    address.parse::<std::net::SocketAddr>()
        .map_err(|_| "Invalid address format. Use IP:PORT (e.g. 192.168.1.10:9000)".to_string())?;

    // Persist to disk
    let mut config = crate::commands::config::load_config(&app);
    config.ip = address.clone();
    crate::commands::config::save_config(&app, &config)?;

    // Update in-memory state
    *state.ip.lock().unwrap() = address;

    // NOTE: The UDP socket itself is already bound — it will use the new
    // address for outgoing packets on the next send automatically since
    // your send commands read from RoverAddress at call time.
    Ok(())
}

#[tauri::command]
pub async fn set_local_port(
    app: tauri::AppHandle,
    port: u16,
    handle: tauri::State<'_, UdpServiceHandle>,
) -> Result<(), String> {
    // Persist
    let mut config = crate::commands::config::load_config(&app);
    config.local_port = port;
    crate::commands::config::save_config(&app, &config)?;

    // Cancel the existing listener
    let old_token: tokio::sync::MutexGuard<'_, CancellationToken> =
        handle.restart_token.lock().await;
    old_token.cancel();
    drop(old_token);

    // Create new service bound to new port
    let new_service = crate::network::service::UdpService::new(port)
        .await
        .map_err(|e| e.to_string())?;

    let new_socket = new_service.socket();

    // Replace the service
    *handle.service.lock().await = new_service;

    // New cancellation token for the new listener
    let new_token = CancellationToken::new();
    *handle.restart_token.lock().await = new_token.clone();

    // Spawn new listener
    let listener_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::network::listener::run_listener(new_socket, new_token, listener_handle).await;
    });

    println!("Rebound UDP listener to port {}", port);
    Ok(())
}