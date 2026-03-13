use tokio::net::UdpSocket;
use prost::Message;

use crate::proto::packets::*;

pub async fn run_listener(socket: std::sync::Arc<UdpSocket>) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 4096];

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
                    println!("IMU info from {addr}: {:?}", msg);
                },
            }
        }

        // match envelope.r#type.try_into() {

        //     // ---------------- Sensor board ----------------

        //     Ok(PbMessageType::SensorBoardImuInformation) => {
        //         match ImuSensorInformation::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("IMU from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("IMU decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::SensorBoardPhSensorInformation) => {
        //         match PhSensorInformation::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("PH sensor from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("PH decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::SensorBoardGpsInformation) => {
        //         match GpsSensorInformation::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("GPS from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("GPS decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::SensorBoardDiagnostics) => {
        //         match SensorDiagnostics::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Sensor diagnostics from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("Sensor diagnostics decode error: {e}")
        //         }
        //     }

        //     // ---------------- Arm board ----------------

        //     Ok(PbMessageType::ArmBoardControlSignals) => {
        //         match ControlSignals::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Control signals from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("ControlSignals decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::ArmBoardDiagnostics) => {
        //         match ArmDiagnostics::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Arm diagnostics from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("ArmDiagnostics decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::ArmBoardMovementFeedback) => {
        //         match MovementFeedback::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Movement feedback from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("MovementFeedback decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::ArmBoardActualPositions) => {
        //         match ActualPositions::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Actual positions from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("ActualPositions decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::ArmBoardTargetMovement) => {
        //         match TargetMovement::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Target movement from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("TargetMovement decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::ArmBoardObstructions) => {
        //         match Obstructions::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Obstructions from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("Obstructions decode error: {e}")
        //         }
        //     }

        //     // ---------------- Driving board ----------------

        //     Ok(PbMessageType::DrivingBoardDiagnostics) => {
        //         match DrivingDiagnostics::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Driving diagnostics from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("DrivingDiagnostics decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::DrivingBoardMotorMessage) => {
        //         match DrivingBoardMotorMsg::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Driving motor message from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("DrivingBoardMotorMsg decode error: {e}")
        //         }
        //     }

        //     Ok(PbMessageType::DrivingBoardMotorPeriodicProgress) => {
        //         match MotorPeriodicProgress::decode(&envelope.data[..]) {
        //             Ok(msg) => println!("Driving motor periodic progress from {addr}: {:?}", msg),
        //             Err(e) => eprintln!("DrivingBoardMotorPeriodicProgress decode error: {e}")
        //         }
        //     }

        //     _ => {
        //         println!("Unhandled packet type from {addr}");
        //     }
        // }
    }
}