use tauri::State;

use crate::network::service::UdpService;
use crate::network::sender;
use crate::proto::packets::SensorBoardImuInfo;

#[tauri::command]
pub async fn send_ping_cmd(
    state: State<'_, UdpService>,
) -> Result<(), String> {
    println!("Called ping command");

    let socket = state.socket();

    println!("Socket gotten");

    let packet = SensorBoardImuInfo {
        accel_x: 0.0,
        accel_y: 0.0,
        accel_z: 0.0,
        gyro_x: 0.0,
        gyro_y: 0.0,
        gyro_z: 0.0,
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