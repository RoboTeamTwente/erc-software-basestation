use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::net::ToSocketAddrs;
use prost::Message;
use tokio::net::UdpSocket;
use crate::proto::packets::*;

pub fn stream_dummy_imu_blocking(
    socket: Arc<UdpSocket>,
    addr: String,
    cancel: Arc<Mutex<bool>>,
) {
    let socket_addr = addr
        .to_socket_addrs()
        .expect("Invalid address")
        .next()
        .expect("Could not resolve address");

    let mut t: f32 = 0.0;
    let dt: f32 = 0.05;
    let mut heading: f32 = 0.0;
    let mut n = [0f32; 8];
    let mut seed: u32 = 0xdeadbeef;

    let mut raw_noise = |s: &mut u32| -> f32 {
        *s = s.wrapping_mul(1664525).wrapping_add(1013904223);
        (*s as f32 / u32::MAX as f32) * 2.0 - 1.0
    };
    let alpha: f32 = 0.05;

    loop {
        std::thread::sleep(Duration::from_millis(100));

        if *cancel.lock().unwrap() {
            println!("Dummy IMU stream exiting");
            break;
        }

        t += dt;
        for v in n.iter_mut() {
            *v += alpha * (raw_noise(&mut seed) - *v);
        }

        heading = (t * 0.008).sin() * 0.35;
        let yaw_rate_degs = (t * 0.008).cos() * 0.35 * 0.008_f32.to_degrees() * 0.05;
        let heave = (t * 0.3).sin() * 0.08;

        let msg = SensorBoardImuInfo {
            accel_x: (t * 0.10).sin() * 0.05 + n[0] * 0.03,
            accel_y: n[1] * 0.02,
            accel_z: 9.81 + heave + n[2] * 0.04,
            gyro_x:  (t * 0.25).sin() * 0.8 + n[3] * 0.2,
            gyro_y:  (t * 0.18).cos() * 0.5 + n[4] * 0.2,
            gyro_z:  yaw_rate_degs          + n[5] * 0.1,
            mag_x:   20.0 * heading.cos()   + n[6] * 0.3,
            mag_y:   20.0 * heading.sin()   + n[6] * 0.3,
            mag_z:   42.0                   + n[7] * 0.2,
            is_calibrated: true,
            state: SensorState::SensorOperating as i32,
            error_code: ImuErrorCode::ImuNoError as i32,
        };

        let envelope = PbEnvelope {
            payload: Some(pb_envelope::Payload::ImuInfo(msg)),
        };

        let mut buf = Vec::new();
        if envelope.encode(&mut buf).is_err() {
            eprintln!("Failed to encode dummy IMU packet");
            continue;
        }

        if let Err(e) = socket.try_send_to(&buf, socket_addr) {
            if e.kind() != std::io::ErrorKind::WouldBlock {
                eprintln!("Failed to send dummy IMU packet: {e}");
            }
        }
    }
}


pub fn build_dummy_envelope(packet_type: &DummyPacketType, t: f32) -> PbEnvelope {
    let payload = match packet_type {

        DummyPacketType::ImuInfo => {
            let heading = (t * 0.008).sin() * 0.35;
            pb_envelope::Payload::ImuInfo(SensorBoardImuInfo {
                accel_x:      (t * 0.10).sin() * 0.05,
                accel_y:      (t * 0.13).cos() * 0.03,
                accel_z:      9.81 + (t * 0.3).sin() * 0.08,
                gyro_x:       (t * 0.25).sin() * 0.8,
                gyro_y:       (t * 0.18).cos() * 0.5,
                gyro_z:       (t * 0.008).cos() * 0.35 * 0.008_f32.to_degrees() * 0.05,
                mag_x:        20.0 * heading.cos(),
                mag_y:        20.0 * heading.sin(),
                mag_z:        42.0,
                is_calibrated: true,
                state:        SensorState::SensorOperating as i32,
                error_code:   ImuErrorCode::ImuNoError as i32,
            })
        }

        DummyPacketType::GpsInfo => {
            // Slow drift around Enschede
            pb_envelope::Payload::GpsInfo(SensorBoardGpsInfo {
                latitude:      52.2297 + (t * 0.001).sin() as f64 * 0.0005,
                longitude:     6.8978  + (t * 0.0013).cos() as f64 * 0.0005,
                altitude:      35.0 + (t * 0.05).sin() * 0.5,
                speed:         (t * 0.2).sin().abs() * 1.5,
                heading:       ((t * 0.05).sin() * 180.0 + 180.0) % 360.0,
                hdop:          1.2 + (t * 0.1).sin().abs() * 0.3,
                vdop:          1.8 + (t * 0.07).cos().abs() * 0.2,
                satellites:    9,
                fix_quality:   GpsFixQuality::GpsFix as i32,
                state:         SensorState::SensorOperating as i32,
                error_code:    GpsErrorCode::GpsNoError as i32,
                utc_timestamp: 0,
            })
        }

        DummyPacketType::PhInfo => {
            pb_envelope::Payload::PhInfo(SensorBoardPhInfo {
                ph_value:    7.0 + (t * 0.05).sin() * 0.4,
                voltage:     512.0 + (t * 0.1).cos() * 10.0,
                temperature: 21.5 + (t * 0.02).sin() * 0.5,
                state:       SensorState::SensorOperating as i32,
                error_code:  PhErrorCode::PhNoError as i32,
            })
        }

        DummyPacketType::ArmCtrl => {
            pb_envelope::Payload::ArmCtrl(ArmBoardControlSignals {
                control_gripper_rotation: (t * 0.3).sin() * 1.5,
                control_gripper_pitch:    (t * 0.2).cos() * 1.0,
                control_base:             (t * 0.15).sin() * 2.0,
                control_jaw:              (t * 0.4).cos().abs() * 0.8,
                stepper_top_ena:          (t as u32 % 4) < 3,
                stepper_top_rev:          (t * 0.1).sin() > 0.0,
                stepper_bottom_ena:       (t as u32 % 5) < 4,
                stepper_bottom_rev:       (t * 0.1).cos() > 0.0,
            })
        }

        DummyPacketType::ArmDiag => {
            pb_envelope::Payload::ArmDiag(ArmBoardDiagnostics {
                state: arm_board_diagnostics::State::Operating as i32,
                gripper_rotation_motor: Some(dummy_motor(0, t, 0.30)),
                gripper_pitch_motor:    Some(dummy_motor(1, t, 0.22)),
                base_motor:             Some(dummy_motor(2, t, 0.15)),
                top_motor:              Some(dummy_motor(3, t, 0.18)),
                bottom_motor:           Some(dummy_motor(4, t, 0.20)),
                jaw_motor:              Some(dummy_motor(5, t, 0.40)),
            })
        }

        DummyPacketType::ArmFeedback => {
            // Cycle through error codes to exercise the frontend
            let codes = [
                arm_board_movement_feedback::ArmError::AllOk,
                arm_board_movement_feedback::ArmError::AllOk,
                arm_board_movement_feedback::ArmError::AllOk,
                arm_board_movement_feedback::ArmError::Obstruction,
            ];
            let code = codes[(t as usize / 3) % codes.len()];
            pb_envelope::Payload::ArmFeedback(ArmBoardMovementFeedback {
                arm_error: code as i32,
            })
        }

        DummyPacketType::ArmPos => {
            pb_envelope::Payload::ArmPos(ArmBoardActualPositions {
                jaw_open:                       (t * 0.3).sin() > 0.0,
                jaw_actual_position:            ((t * 0.3).sin() * 45.0).abs(),
                base_actual_position:           (t * 0.15).sin() * 90.0,
                stepper_top_actual_position:    (t * 0.18).sin() * 60.0,
                stepper_bottom_actual_position: (t * 0.20).cos() * 60.0,
                gripper_rotation_actual_position: (t * 0.30).sin() * 180.0,
                gripper_pitch_actual_position:  (t * 0.22).cos() * 90.0,
            })
        }

        DummyPacketType::ArmTarget => {
            // Trace a slow ellipse in the XZ plane
            pb_envelope::Payload::ArmTarget(ArmBoardTargetMovement {
                target_x:       (t * 0.1).cos() * 0.3,
                target_y:       0.2 + (t * 0.07).sin() * 0.05,
                target_z:       0.4 + (t * 0.1).sin() * 0.15,
                rotation_angle: (t * 0.15).sin() * 45.0,
                open_jaw:       (t as u32 % 6) < 4,
            })
        }

        DummyPacketType::ArmObstructions => {
            pb_envelope::Payload::ArmObstructions(ArmBoardObstructions {
                obstructions_list: (t * 0.2).sin().abs(),
            })
        }

        DummyPacketType::DriveDiag => {
            pb_envelope::Payload::DriveDiag(DrivingBoardDiagnostics {
                state: driving_board_diagnostics::State::Operating as i32,
                front_left_motor:          Some(dummy_motor(0,  t, 0.31)),
                middle_left_motor:         Some(dummy_motor(1,  t, 0.31)),
                back_left_motor:           Some(dummy_motor(2,  t, 0.31)),
                front_right_motor:         Some(dummy_motor(3,  t, 0.31)),
                middle_right_motor:        Some(dummy_motor(4,  t, 0.31)),
                back_right_motor:          Some(dummy_motor(5,  t, 0.31)),
                steering_front_left_motor: Some(dummy_motor(6,  t, 0.18)),
                steering_back_left_motor:  Some(dummy_motor(7,  t, 0.18)),
                steering_front_right_motor:Some(dummy_motor(8,  t, 0.18)),
                steering_back_right_motor: Some(dummy_motor(9,  t, 0.18)),
            })
        }

        DummyPacketType::DriveMotor => {
            pb_envelope::Payload::DriveMotor(DrivingBoardMotorMessage {
                distance_to_go:  ((t * 0.05).sin() * 5.0 + 5.0),
                turning_radius:  (t * 0.08).sin() * 2.0,
            })
        }

        DummyPacketType::DriveProgress => {
            pb_envelope::Payload::DriveProgress(DrivingBoardMotorPeriodicProgress {
                distance_left: (10.0 - (t % 10.0)).max(0.0),
            })
        }

        DummyPacketType::SensorDiag => {
            let heading = (t * 0.008).sin() * 0.35;
            pb_envelope::Payload::SensorDiag(SensorBoardDiagnostics {
                state: sensor_board_diagnostics::State::Operating as i32,
                ph_sensor: Some(SensorBoardPhInfo {
                    ph_value:    7.0 + (t * 0.05).sin() * 0.4,
                    voltage:     512.0 + (t * 0.1).cos() * 10.0,
                    temperature: 21.5 + (t * 0.02).sin() * 0.5,
                    state:       SensorState::SensorOperating as i32,
                    error_code:  PhErrorCode::PhNoError as i32,
                }),
                imu_sensor: Some(SensorBoardImuInfo {
                    accel_x:      (t * 0.10).sin() * 0.05,
                    accel_y:      (t * 0.13).cos() * 0.03,
                    accel_z:      9.81 + (t * 0.3).sin() * 0.08,
                    gyro_x:       (t * 0.25).sin() * 0.8,
                    gyro_y:       (t * 0.18).cos() * 0.5,
                    gyro_z:       0.0,
                    mag_x:        20.0 * heading.cos(),
                    mag_y:        20.0 * heading.sin(),
                    mag_z:        42.0,
                    is_calibrated: true,
                    state:        SensorState::SensorOperating as i32,
                    error_code:   ImuErrorCode::ImuNoError as i32,
                }),
                gps_sensor_1: Some(SensorBoardGpsInfo {
                    latitude:      52.2297 + (t * 0.001).sin() as f64 * 0.0005,
                    longitude:     6.8978  + (t * 0.0013).cos() as f64 * 0.0005,
                    altitude:      35.0 + (t * 0.05).sin() * 0.5,
                    speed:         (t * 0.2).sin().abs() * 1.5,
                    heading:       ((t * 0.05).sin() * 180.0 + 180.0) % 360.0,
                    hdop:          1.2,
                    vdop:          1.8,
                    satellites:    9,
                    fix_quality:   GpsFixQuality::GpsFix as i32,
                    state:         SensorState::SensorOperating as i32,
                    error_code:    GpsErrorCode::GpsNoError as i32,
                    utc_timestamp: 0,
                }),
                board_temperature: 42.0 + (t * 0.03).sin() * 2.0,
                board_voltage:     3.28 + (t * 0.05).cos() * 0.02,
            })
        }
    };

    PbEnvelope { payload: Some(payload) }
}

// Reusable helper — generates a plausible motor reading for a given motor_id and phase offset
fn dummy_motor(motor_id: i32, t: f32, freq: f32) -> MotorInformation {
    let phase = motor_id as f32 * 0.5;
    MotorInformation {
        state:         motor_information::State::Operating as i32,
        motor_id,
        rpm:           ((t * freq + phase).sin() * 30.0 + 60.0).abs(),
        voltage:       12.0 + (t * 0.1 + phase).sin() * 0.3,
        encoder_angle: (t * freq * 360.0 + phase * 57.3) % 360.0,
    }
}

// ── Streaming ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DummyPacketType {
    ImuInfo,
    GpsInfo,
    PhInfo,
    ArmCtrl,
    ArmDiag,
    ArmFeedback,
    ArmPos,
    ArmTarget,
    ArmObstructions,
    DriveDiag,
    DriveMotor,
    DriveProgress,
    SensorDiag,
}

pub fn stream_dummy_blocking(
    socket: Arc<UdpSocket>,
    addr: String,
    packet_type: DummyPacketType,
    cancel: Arc<Mutex<bool>>,
) {
    let socket_addr = addr
        .to_socket_addrs()
        .expect("Invalid address")
        .next()
        .expect("Could not resolve address");

    let mut t: f32 = 0.0;
    let dt: f32 = 0.1;

    loop {
        std::thread::sleep(Duration::from_millis(100));

        if *cancel.lock().unwrap() {
            println!("Dummy stream ({:?}) exiting", packet_type);
            break;
        }

        let envelope = build_dummy_envelope(&packet_type, t);
        t += dt;

        let mut buf = Vec::new();
        if envelope.encode(&mut buf).is_err() {
            eprintln!("Failed to encode dummy packet");
            continue;
        }

        if let Err(e) = socket.try_send_to(&buf, socket_addr) {
            if e.kind() != std::io::ErrorKind::WouldBlock {
                eprintln!("Failed to send dummy packet: {e}");
            }
        }
    }
}