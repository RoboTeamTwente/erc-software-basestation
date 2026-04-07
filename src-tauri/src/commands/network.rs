use tauri::State;

use std::sync::{Arc, Mutex};

use crate::network::service::UdpService;
use crate::network::dummy::{self, SimulatorConfig};
use crate::network::sender;

use crate::proto::packets::*;

pub struct DummyStreamHandle {
    pub cancel: Mutex<Option<Arc<Mutex<bool>>>>,
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
use crate::network::dummy::{StreamSpec, stream_table};

pub fn all_stream_specs() -> Vec<StreamSpec> {
    stream_table()
}


// FULL SIMULATOR
#[tauri::command]
pub async fn start_dummy_streams(
    udp: State<'_, UdpService>,
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    stop_dummy_streams(handle.clone()).await?;

    let socket = udp.socket();
    let cancel = Arc::new(Mutex::new(false));

    *handle.cancel.lock().unwrap() = Some(cancel.clone());

    let config = crate::network::dummy::SimulatorConfig {
        jitter_ms: 30,
        packet_loss: 0.02,
        record_path: None,
        replay_path: None,
    };

    let addr = "127.0.0.1:9000".to_string();

    tokio::spawn(async move {
        let _ = crate::network::dummy::run_simulator(
            socket,
            addr,
            cancel,
            config,
        )
        .await;
    });

    println!("Async multi-stream simulator started");
    Ok(())
}

// SINGLE STREAM (for debug)
#[tauri::command]
pub async fn start_dummy_imu_stream(
    udp: State<'_, UdpService>,
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    stop_dummy_streams(handle.clone()).await?;

    let socket = udp.socket();
    let cancel = Arc::new(Mutex::new(false));

    *handle.cancel.lock().unwrap() = Some(cancel.clone());

    let config = crate::network::dummy::SimulatorConfig {
        jitter_ms: 10,
        packet_loss: 0.0,
        record_path: None,
        replay_path: None,
    };

    tokio::spawn(async move {
        let _ = crate::network::dummy::run_simulator(
            socket,
            "127.0.0.1:9000".to_string(),
            cancel,
            config,
        )
        .await;
    });

    println!("IMU-only simulator started");
    Ok(())
}


//STOP SIMULATOR
#[tauri::command]
pub async fn stop_dummy_streams(
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    if let Some(cancel) = handle.cancel.lock().unwrap().take() {
        *cancel.lock().unwrap() = true;
        println!("Dummy simulator stop signalled");
    }
    Ok(())
}