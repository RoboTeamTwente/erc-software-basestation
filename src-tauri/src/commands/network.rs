use tauri::State;

use tokio::sync::Mutex; 
use tokio_util::sync::CancellationToken;

use crate::network::service::UdpService;

use crate::network::dummy::{StreamSpec, stream_table, SimulatorConfig};


pub struct DummyStreamHandle {
    pub token: Mutex<Option<CancellationToken>>,
}

// PING COMMAND
#[tauri::command]
pub async fn send_ping_cmd(
    state: State<'_, UdpService>,
    packet_type: String
) -> Result<(), String> {
    // let socket = state.socket();
    // let target = "127.0.0.1:9000";
    // let packet_type = packet_type.unwrap_or(PingPacketType::Imu);

    // let envelope = build_ping_envelope(packet_type);
    // sender::send_envelope(&socket, target, envelope)
    //     .await
    //     .map_err(|e| e.to_string())
    println!("Ping command received for packet type: {:?}", packet_type);
    Ok(())
}


// STREAM TABLE

pub fn all_stream_specs() -> Vec<StreamSpec> {
    stream_table()
}


// FULL SIMULATOR
#[tauri::command]
pub async fn start_dummy_streams(
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    if let Some(old_token) = handle.token.lock().await.take() {
        old_token.cancel();
    }
    let token = CancellationToken::new();
    *handle.token.lock().await = Some(token.clone());

    crate::network::dummy::spawn_simulator(
        token,
        "127.0.0.1:9000".to_string(),
        SimulatorConfig { jitter_ms: 30, packet_loss: 0.02, record_path: None },
        crate::network::dummy::stream_table(),
    );
    Ok(())
}

#[tauri::command]
pub async fn start_detection_sim(
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    if let Some(old_token) = handle.token.lock().await.take() {
        old_token.cancel();
    }
    let token = CancellationToken::new();
    *handle.token.lock().await = Some(token.clone());

    crate::network::dummy::spawn_simulator(
        token,
        "127.0.0.1:9000".to_string(),
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
    let config = crate::commands::config::RoverConfig { ip: address.clone() };
    crate::commands::config::save_config(&app, &config)?;

    // Update in-memory state
    *state.ip.lock().unwrap() = address;

    // NOTE: The UDP socket itself is already bound — it will use the new
    // address for outgoing packets on the next send automatically since
    // your send commands read from RoverAddress at call time.
    Ok(())
}