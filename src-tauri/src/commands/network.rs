use tauri::State;

use crate::network::service::UdpService;
use crate::network::sender;
use crate::proto::packets::SensorBoardImuInfo;
use std::sync::{Arc, Mutex};

// Holds a cancel handle for the running dummy stream, if any.
// Stored as Tauri state so commands can stop a previous stream before starting a new one.
pub struct DummyStreamHandle(pub Mutex<Option<Arc<Mutex<bool>>>>);

#[tauri::command]
pub async fn send_ping_cmd(
    state: State<'_, UdpService>,
) -> Result<(), String> {
    println!("Called ping command");

    let socket = state.socket();

    let packet = SensorBoardImuInfo {
        accel_x: 0.0,
        accel_y: 0.0,
        accel_z: 0.0,
        gyro_x: 0.0,
        gyro_y: 0.0,
        gyro_z: 0.5,
        mag_x: 0.0,
        mag_y: 0.0,
        mag_z: 0.0,
        is_calibrated: false,
        state: crate::proto::packets::SensorState::SensorIdle as i32,
        error_code: crate::proto::packets::ImuErrorCode::ImuNoError as i32,
    };

    println!("Packet encoded {:?}", packet);

    sender::send_ping(&socket, "127.0.0.1:9000", packet)
        .await
        .map_err(|e| e.to_string())?;

    println!("Packet sent");
    Ok(())
}

// Start streaming randomised IMU packets to 127.0.0.1:9000 at ~20 Hz.
/// Calling this again while a stream is already running stops the old one first.
#[tauri::command]
pub async fn start_dummy_imu_stream(
    udp: State<'_, UdpService>,
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    stop_dummy_imu_stream(handle.clone()).await?;
 
    let socket = udp.socket();
    let cancel = Arc::new(Mutex::new(false));
    let cancel_thread = cancel.clone();
 
    std::thread::spawn(move || {
        sender::stream_dummy_imu_blocking(socket, "127.0.0.1:9000".to_string(), cancel_thread);
    });
 
    *handle.0.lock().unwrap() = Some(cancel);
    println!("Dummy IMU stream started on dedicated OS thread");
    Ok(())
}
 
#[tauri::command]
pub async fn stop_dummy_imu_stream(
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    if let Some(cancel) = handle.0.lock().unwrap().take() {
        *cancel.lock().unwrap() = true;
        println!("Dummy IMU stream stop signalled");
    }
    Ok(())
}