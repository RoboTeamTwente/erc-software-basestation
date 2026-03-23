use tauri::State;

use crate::network::service::UdpService;
use crate::network::sender;
use crate::proto::packets::SensorBoardImuInfo;
use std::sync::{Arc, Mutex};
use crate::proto::packets::*;     

// Holds a cancel handle for the running dummy stream, if any.
// Stored as Tauri state so commands can stop a previous stream before starting a new one.
pub struct DummyStreamHandle(pub Mutex<Option<Arc<Mutex<bool>>>>);

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PingPacketType {
    Imu,
    Gps,
    Ph,
    ArmCtrl,
    DriveMotor,
}

#[tauri::command]
pub async fn send_ping_cmd(
    state: State<'_, UdpService>,
    packet_type: Option<PingPacketType>,   // defaults to Imu if omitted
) -> Result<(), String> {

    let socket = state.socket();
    let target  = "127.0.0.1:9000";
 
    let packet_type = packet_type.unwrap_or(PingPacketType::Imu);
    println!("[send_ping_cmd] Sending {:?} to {}", packet_type, target);
 
    let envelope = match packet_type {
        PingPacketType::Imu => {
            let msg = SensorBoardImuInfo {
                accel_x: 1.0,
                accel_y: 2.0,
                accel_z: 9.81,
                gyro_x: 0.1,
                gyro_y: 0.2,
                gyro_z: 0.5,
                mag_x: 10.0,
                mag_y: 5.0,
                mag_z: 42.0,
                is_calibrated: true,
                state: SensorState::SensorOperating as i32,
                error_code: ImuErrorCode::ImuNoError as i32,
            };
            PbEnvelope { payload: Some(pb_envelope::Payload::ImuInfo(msg)) }
        }
 
        PingPacketType::Gps => {
            let msg = SensorBoardGpsInfo {
                latitude: 52.2297,
                longitude: 6.8978,
                altitude: 35.0,
                speed: 0.0,
                heading: 270.0,
                hdop: 1.2,
                vdop: 1.8,
                satellites: 9,
                fix_quality: GpsFixQuality::GpsFix as i32,
                state: SensorState::SensorOperating as i32,
                error_code: GpsErrorCode::GpsNoError as i32,
                utc_timestamp: 0,
            };
            PbEnvelope { payload: Some(pb_envelope::Payload::GpsInfo(msg)) }
        }
 
        PingPacketType::Ph => {
            let msg = SensorBoardPhInfo {
                ph_value: 7.2,
                voltage: 512.0,
                temperature: 21.5,
                state: SensorState::SensorOperating as i32,
                error_code: PhErrorCode::PhNoError as i32,
            };
            PbEnvelope { payload: Some(pb_envelope::Payload::PhInfo(msg)) }
        }
 
        PingPacketType::ArmCtrl => {
            let msg = ArmBoardControlSignals {
                control_gripper_rotation: 0.0,
                control_gripper_pitch: 0.5,
                control_base: 1.2,
                control_jaw: 0.0,
                stepper_top_ena: true,
                stepper_top_rev: false,
                stepper_bottom_ena: true,
                stepper_bottom_rev: false,
            };
            PbEnvelope { payload: Some(pb_envelope::Payload::ArmCtrl(msg)) }
        }
 
        PingPacketType::DriveMotor => {
            let msg = DrivingBoardMotorMessage {
                distance_to_go: 2.5,
                turning_radius: 0.0,
            };
            PbEnvelope { payload: Some(pb_envelope::Payload::DriveMotor(msg)) }
        }
    };
 
    sender::send_envelope(&socket, target, envelope)
        .await
        .map_err(|e| e.to_string())
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