use tauri::State;
use std::sync::{Arc, Mutex};
use crate::network::{service::UdpService, sender, dummy};
use crate::proto::packets::*;
use crate::network::dummy::DummyPacketType;
pub struct DummyStreamHandle(pub Mutex<Option<Arc<Mutex<bool>>>>);

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PingPacketType { Imu, Gps, Ph, ArmCtrl, DriveMotor }

const ALL_PACKET_TYPES: &[DummyPacketType] = &[
    DummyPacketType::ImuInfo,
    DummyPacketType::GpsInfo,
    DummyPacketType::PhInfo,
    DummyPacketType::ArmCtrl,
    DummyPacketType::ArmDiag,
    DummyPacketType::ArmFeedback,
    DummyPacketType::ArmPos,
    DummyPacketType::ArmTarget,
    DummyPacketType::ArmObstructions,
    DummyPacketType::DriveDiag,
    DummyPacketType::DriveMotor,
    DummyPacketType::DriveProgress,
    DummyPacketType::SensorDiag,
];

#[tauri::command]
pub async fn send_ping_cmd(
    state: State<'_, UdpService>,
    packet_type: Option<PingPacketType>,
) -> Result<(), String> {
    let socket = state.socket();
    let target = "127.0.0.1:9000";
    let packet_type = packet_type.unwrap_or(PingPacketType::Imu);

    let envelope = build_ping_envelope(packet_type);
    sender::send_envelope(&socket, target, envelope).await.map_err(|e| e.to_string())
}

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
        dummy::stream_dummy_imu_blocking(socket, "127.0.0.1:9000".to_string(), cancel_thread);
    });

    *handle.0.lock().unwrap() = Some(cancel);
    println!("Dummy IMU stream started");
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

// Moved out of the command body so it's easy to test independently
fn build_ping_envelope(packet_type: PingPacketType) -> PbEnvelope {
    match packet_type {
        PingPacketType::Imu => PbEnvelope {
            payload: Some(pb_envelope::Payload::ImuInfo(SensorBoardImuInfo {
                accel_x: 1.0, accel_y: 2.0, accel_z: 9.81,
                gyro_x: 0.1,  gyro_y: 0.2,  gyro_z: 0.5,
                mag_x: 10.0,  mag_y: 5.0,   mag_z: 42.0,
                is_calibrated: true,
                state: SensorState::SensorOperating as i32,
                error_code: ImuErrorCode::ImuNoError as i32,
            })),
        },
        PingPacketType::Gps => PbEnvelope {
            payload: Some(pb_envelope::Payload::GpsInfo(SensorBoardGpsInfo {
                latitude: 52.2297, longitude: 6.8978, altitude: 35.0,
                speed: 0.0, heading: 270.0, hdop: 1.2, vdop: 1.8,
                satellites: 9,
                fix_quality: GpsFixQuality::GpsFix as i32,
                state: SensorState::SensorOperating as i32,
                error_code: GpsErrorCode::GpsNoError as i32,
                utc_timestamp: 0,
            })),
        },
        PingPacketType::Ph => PbEnvelope {
            payload: Some(pb_envelope::Payload::PhInfo(SensorBoardPhInfo {
                ph_value: 7.2, voltage: 512.0, temperature: 21.5,
                state: SensorState::SensorOperating as i32,
                error_code: PhErrorCode::PhNoError as i32,
            })),
        },
        PingPacketType::ArmCtrl => PbEnvelope {
            payload: Some(pb_envelope::Payload::ArmCtrl(ArmBoardControlSignals {
                control_gripper_rotation: 0.0, control_gripper_pitch: 0.5,
                control_base: 1.2, control_jaw: 0.0,
                stepper_top_ena: true,    stepper_top_rev: false,
                stepper_bottom_ena: true, stepper_bottom_rev: false,
            })),
        },
        PingPacketType::DriveMotor => PbEnvelope {
            payload: Some(pb_envelope::Payload::DriveMotor(DrivingBoardMotorMessage {
                distance_to_go: 2.5, turning_radius: 0.0,
            })),
        },
    }
}


#[tauri::command]
pub async fn start_dummy_streams(
    udp: State<'_, UdpService>,
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    stop_dummy_streams(handle.clone()).await?;

    let cancel = Arc::new(Mutex::new(false));
    *handle.0.lock().unwrap() = Some(cancel.clone());

    for &packet_type in ALL_PACKET_TYPES {
        let socket = udp.socket();
        let cancel = cancel.clone();
        std::thread::spawn(move || {
            dummy::stream_dummy_blocking(socket, "127.0.0.1:9000".to_string(), packet_type, cancel);
        });
    }

    println!("All dummy streams started ({} threads)", ALL_PACKET_TYPES.len());
    Ok(())
}

#[tauri::command]
pub async fn stop_dummy_streams(
    handle: State<'_, DummyStreamHandle>,
) -> Result<(), String> {
    if let Some(cancel) = handle.0.lock().unwrap().take() {
        *cancel.lock().unwrap() = true;
        println!("All dummy streams stop signalled");
    }
    Ok(())
}