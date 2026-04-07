use std::{
    net::ToSocketAddrs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use prost::Message;
use tokio::{
    net::UdpSocket,
    sync::mpsc,
    time::{Instant, sleep},
};

use crate::proto::packets::*;


// CONFIG
#[derive(Clone)]
pub struct SimulatorConfig {
    pub jitter_ms: u64,
    pub packet_loss: f32, // 0.0 → 1.0
    pub record_path: Option<PathBuf>,
    pub replay_path: Option<PathBuf>,
}

// Deterministic rng for dummy data
struct Lcg {
    state: u32,
}

impl Lcg {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state as f32) / (u32::MAX as f32)
    }
}

// Streaming tables

pub type GeneratorFn = fn(f32) -> pb_envelope::Payload;

pub struct StreamSpec {
    pub interval: Duration,
    pub generator: GeneratorFn,
}

pub fn stream_table() -> Vec<StreamSpec> {
    vec![
        StreamSpec { interval: Duration::from_millis(20),  generator: gen_imu },
        StreamSpec { interval: Duration::from_millis(200), generator: gen_gps },
        StreamSpec { interval: Duration::from_millis(500), generator: gen_ph },
        StreamSpec { interval: Duration::from_millis(50),  generator: gen_arm_ctrl },
        StreamSpec { interval: Duration::from_millis(500), generator: gen_arm_diag },
        StreamSpec { interval: Duration::from_millis(100), generator: gen_arm_feedback },
        StreamSpec { interval: Duration::from_millis(50),  generator: gen_arm_pos },
        StreamSpec { interval: Duration::from_millis(200), generator: gen_arm_target },
        StreamSpec { interval: Duration::from_millis(300), generator: gen_arm_obstructions },
        StreamSpec { interval: Duration::from_millis(500), generator: gen_drive_diag },
        StreamSpec { interval: Duration::from_millis(50),  generator: gen_drive_motor },
        StreamSpec { interval: Duration::from_millis(100), generator: gen_drive_progress },
        StreamSpec { interval: Duration::from_millis(500), generator: gen_sensor_diag },
    ]
}

// ─────────────────────────────────────────────────────────────
// ALL GENERATORS (complete coverage)
// ─────────────────────────────────────────────────────────────

fn gen_imu(t: f32) -> pb_envelope::Payload {
    let heading = (t * 0.008).sin() * 0.35;

    pb_envelope::Payload::ImuInfo(SensorBoardImuInfo {
        accel_x: (t * 0.10).sin() * 0.05,
        accel_y: (t * 0.13).cos() * 0.03,
        accel_z: 9.81 + (t * 0.3).sin() * 0.08,
        gyro_x: (t * 0.25).sin() * 0.8,
        gyro_y: (t * 0.18).cos() * 0.5,
        gyro_z: (t * 0.008).cos() * 0.35 * 0.008_f32.to_degrees(),
        mag_x: 20.0 * heading.cos(),
        mag_y: 20.0 * heading.sin(),
        mag_z: 42.0,
        is_calibrated: true,
        state: SensorState::SensorOperating as i32,
        error_code: ImuErrorCode::ImuNoError as i32,
    })
}

fn gen_gps(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::GpsInfo(SensorBoardGpsInfo {
        latitude: 52.2297 + (t * 0.001).sin() as f64 * 0.0005,
        longitude: 6.8978 + (t * 0.0013).cos() as f64 * 0.0005,
        altitude: 35.0,
        speed: 1.0,
        heading: ((t * 0.05).sin() * 180.0 + 180.0) % 360.0,
        hdop: 1.2,
        vdop: 1.8,
        satellites: 9,
        fix_quality: GpsFixQuality::GpsFix as i32,
        state: SensorState::SensorOperating as i32,
        error_code: GpsErrorCode::GpsNoError as i32,
        utc_timestamp: 0,
    })
}

fn gen_arm_diag(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::ArmDiag(ArmBoardDiagnostics {
        state: arm_board_diagnostics::State::Operating as i32,
        gripper_rotation_motor: Some(dummy_motor(0, t, 0.3)),
        gripper_pitch_motor: Some(dummy_motor(1, t, 0.2)),
        base_motor: Some(dummy_motor(2, t, 0.15)),
        top_motor: Some(dummy_motor(3, t, 0.18)),
        bottom_motor: Some(dummy_motor(4, t, 0.2)),
        jaw_motor: Some(dummy_motor(5, t, 0.4)),
    })
}

fn gen_ph(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::PhInfo(SensorBoardPhInfo {
        ph_value: 7.0 + (t * 0.05).sin() * 0.4,
        voltage: 512.0 + (t * 0.1).cos() * 10.0,
        temperature: 21.5 + (t * 0.02).sin() * 0.5,
        state: SensorState::SensorOperating as i32,
        error_code: PhErrorCode::PhNoError as i32,
    })
}

fn gen_arm_ctrl(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::ArmCtrl(ArmBoardControlSignals {
        control_gripper_rotation: (t * 0.3).sin() * 1.5,
        control_gripper_pitch: (t * 0.2).cos() * 1.0,
        control_base: (t * 0.15).sin() * 2.0,
        control_jaw: (t * 0.4).cos().abs() * 0.8,
        stepper_top_ena: (t as u32 % 4) < 3,
        stepper_top_rev: (t * 0.1).sin() > 0.0,
        stepper_bottom_ena: (t as u32 % 5) < 4,
        stepper_bottom_rev: (t * 0.1).cos() > 0.0,
    })
}

fn gen_arm_feedback(t: f32) -> pb_envelope::Payload {
    let codes = [
        arm_board_movement_feedback::ArmError::AllOk,
        arm_board_movement_feedback::ArmError::AllOk,
        arm_board_movement_feedback::ArmError::Obstruction,
    ];

    let idx = ((t * 0.5) as usize) % codes.len();

    pb_envelope::Payload::ArmFeedback(ArmBoardMovementFeedback {
        arm_error: codes[idx] as i32,
    })
}

fn gen_arm_pos(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::ArmPos(ArmBoardActualPositions {
        jaw_open: (t * 0.3).sin() > 0.0,
        jaw_actual_position: ((t * 0.3).sin() * 45.0).abs(),
        base_actual_position: (t * 0.15).sin() * 90.0,
        stepper_top_actual_position: (t * 0.18).sin() * 60.0,
        stepper_bottom_actual_position: (t * 0.20).cos() * 60.0,
        gripper_rotation_actual_position: (t * 0.30).sin() * 180.0,
        gripper_pitch_actual_position: (t * 0.22).cos() * 90.0,
    })
}

fn gen_arm_target(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::ArmTarget(ArmBoardTargetMovement {
        target_x: (t * 0.1).cos() * 0.3,
        target_y: 0.2 + (t * 0.07).sin() * 0.05,
        target_z: 0.4 + (t * 0.1).sin() * 0.15,
        rotation_angle: (t * 0.15).sin() * 45.0,
        open_jaw: (t as u32 % 6) < 4,
    })
}

fn gen_arm_obstructions(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::ArmObstructions(ArmBoardObstructions {
        obstructions_list: (t * 0.2).sin().abs(),
    })
}

fn gen_drive_diag(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::DriveDiag(DrivingBoardDiagnostics {
        state: driving_board_diagnostics::State::Operating as i32,

        front_left_motor: Some(dummy_motor(0, t, 0.31)),
        middle_left_motor: Some(dummy_motor(1, t, 0.31)),
        back_left_motor: Some(dummy_motor(2, t, 0.31)),

        front_right_motor: Some(dummy_motor(3, t, 0.31)),
        middle_right_motor: Some(dummy_motor(4, t, 0.31)),
        back_right_motor: Some(dummy_motor(5, t, 0.31)),

        steering_front_left_motor: Some(dummy_motor(6, t, 0.18)),
        steering_back_left_motor: Some(dummy_motor(7, t, 0.18)),
        steering_front_right_motor: Some(dummy_motor(8, t, 0.18)),
        steering_back_right_motor: Some(dummy_motor(9, t, 0.18)),
    })
}

fn gen_drive_motor(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::DriveMotor(DrivingBoardMotorMessage {
        distance_to_go: (t * 0.05).sin() * 5.0 + 5.0,
        turning_radius: (t * 0.08).sin() * 2.0,
    })
}

fn gen_drive_progress(t: f32) -> pb_envelope::Payload {
    pb_envelope::Payload::DriveProgress(DrivingBoardMotorPeriodicProgress {
        distance_left: (10.0 - (t % 10.0)).max(0.0),
    })
}

fn gen_sensor_diag(t: f32) -> pb_envelope::Payload {
    let heading = (t * 0.008).sin() * 0.35;

    pb_envelope::Payload::SensorDiag(SensorBoardDiagnostics {
        state: sensor_board_diagnostics::State::Operating as i32,

        ph_sensor: Some(SensorBoardPhInfo {
            ph_value: 7.0,
            voltage: 512.0,
            temperature: 22.0,
            state: SensorState::SensorOperating as i32,
            error_code: PhErrorCode::PhNoError as i32,
        }),

        imu_sensor: Some(SensorBoardImuInfo {
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 9.81,
            gyro_x: 0.0,
            gyro_y: 0.0,
            gyro_z: 0.0,
            mag_x: 20.0 * heading.cos(),
            mag_y: 20.0 * heading.sin(),
            mag_z: 42.0,
            is_calibrated: true,
            state: SensorState::SensorOperating as i32,
            error_code: ImuErrorCode::ImuNoError as i32,
        }),

        gps_sensor_1: Some(SensorBoardGpsInfo {
            latitude: 52.2297,
            longitude: 6.8978,
            altitude: 35.0,
            speed: 1.0,
            heading: 180.0,
            hdop: 1.2,
            vdop: 1.8,
            satellites: 9,
            fix_quality: GpsFixQuality::GpsFix as i32,
            state: SensorState::SensorOperating as i32,
            error_code: GpsErrorCode::GpsNoError as i32,
            utc_timestamp: 0,
        }),

        board_temperature: 42.0 + (t * 0.03).sin() * 2.0,
        board_voltage: 3.3 + (t * 0.05).cos() * 0.02,
    })
}

// HELPER FUNCTIONS

fn dummy_motor(id: i32, t: f32, freq: f32) -> MotorInformation {
    let phase = id as f32 * 0.5;

    MotorInformation {
        state: motor_information::State::Operating as i32,
        motor_id: id,
        rpm: ((t * freq + phase).sin() * 30.0 + 60.0).abs(),
        voltage: 12.0 + (t * 0.1).sin() * 0.3,
        encoder_angle: (t * freq * 360.0) % 360.0,
    }
}


// ASYNG SIMULATOR

pub async fn run_simulator(
    socket: Arc<UdpSocket>,
    addr: String,
    cancel: Arc<Mutex<bool>>,
    config: SimulatorConfig,
) -> anyhow::Result<()> {
    let socket_addr = addr.to_socket_addrs()?.next().unwrap();

    // Replay mode
    if let Some(path) = config.replay_path.clone() {
        return replay_loop(socket, socket_addr, cancel, path).await;
    }

    let streams = stream_table();
    let mut last_times: Vec<Instant> = streams.iter().map(|_| Instant::now()).collect();

    let mut t = 0.0f32;
    let mut last_global = Instant::now();
    let mut rng = Lcg::new(0xdeadbeef);

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1024);

    // Sender task (network simulation)
    let socket_clone = socket.clone();
    let cfg = config.clone();
    tokio::spawn(async move {
        while let Some(buf) = rx.recv().await {
            // Packet loss
            if rng.next_f32() < cfg.packet_loss {
                continue;
            }

            // Jitter
            let jitter = (rng.next_f32() * cfg.jitter_ms as f32) as u64;
            sleep(Duration::from_millis(jitter)).await;

            let _ = socket_clone.send_to(&buf, socket_addr).await;
        }
    });

    // Recorder
    let mut recorder = config.record_path.map(|p| std::fs::File::create(p).unwrap());

    loop {
        if *cancel.lock().unwrap() {
            break;
        }

        let now = Instant::now();
        let dt = now.duration_since(last_global).as_secs_f32();
        last_global = now;
        t += dt;

        for (i, spec) in streams.iter().enumerate() {
            if now.duration_since(last_times[i]) >= spec.interval {
                last_times[i] = now;

                let payload = (spec.generator)(t);

                let envelope = PbEnvelope {
                    payload: Some(payload),
                };

                let mut buf = Vec::new();
                if envelope.encode(&mut buf).is_ok() {

                    // Record
                    if let Some(file) = recorder.as_mut() {
                        let len = (buf.len() as u32).to_le_bytes();
                        let _ = file.write_all(&len);
                        let _ = file.write_all(&buf);
                    }

                    let _ = tx.send(buf).await;
                }
            }
        }

        sleep(Duration::from_millis(1)).await;
    }

    Ok(())
}

// REPLAY MODE
use std::io::{Read, Write};

async fn replay_loop(
    socket: Arc<UdpSocket>,
    addr: std::net::SocketAddr,
    cancel: Arc<Mutex<bool>>,
    path: PathBuf,
) -> anyhow::Result<()> {
    let mut file = std::fs::File::open(path)?;

    loop {
        if *cancel.lock().unwrap() {
            break;
        }

        let mut len_buf = [0u8; 4];
        if file.read_exact(&mut len_buf).is_err() {
            break;
        }

        let len = u32::from_le_bytes(len_buf);
        let mut buf = vec![0u8; len as usize];

        file.read_exact(&mut buf)?;

        socket.send_to(&buf, addr).await?;

        // crude pacing (can be improved with timestamps if needed)
        sleep(Duration::from_millis(20)).await;
    }

    Ok(())
}
