use tokio::net::UdpSocket;
use std::time::Instant;
use prost::Message;
use tauri::Emitter;
use crate::proto::packets::*;

pub async fn run_listener(socket: std::sync::Arc<UdpSocket>, app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 4096];
    let mut last_imu_emit = Instant::now();

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;

        let envelope = PbEnvelope::decode(&buf[..len])?;
        if let Some(payload) = envelope.payload {
            match payload {
                pb_envelope::Payload::PhInfo(msg) => {
                    println!("Sensor diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::ArmCtrl(msg) => {
                    println!("Arm diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::ArmDiag(msg) => {
                    println!("Arm diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::ArmFeedback(msg) => {
                    println!("Arm diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::ArmPos(msg) => {
                    println!("Arm diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::ArmTarget(msg) => {
                    println!("Arm diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::ArmObstructions(msg) => {
                    println!("Arm diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::DriveDiag(msg) => {
                    println!("Drive diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::DriveMotor(msg) => {
                    println!("Drive diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::DriveProgress(msg) => {
                    println!("Drive diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::SensorDiag(msg) => {
                    println!("Sensor diagnostics from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::GpsInfo(msg) => {
                    println!("Gps info from {addr}: {:?}", msg);
                },
                pb_envelope::Payload::ImuInfo(msg) => {
                    if last_imu_emit.elapsed().as_millis() >= 100 {
                        last_imu_emit = Instant::now();
                        if let Err(e) = app_handle.emit("imu-update", &msg) {
                            eprintln!("Failed to emit imu-update: {e}");
                        }
                    }
                },
            }
        }
    }
}