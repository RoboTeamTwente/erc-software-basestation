use std::{
    net::ToSocketAddrs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

use once_cell::sync::Lazy;
use rand::RngExt;
use prost::Message;
use tokio::{
    net::UdpSocket,
    sync::{mpsc},
    time::{Instant, sleep},
};
use std::io::{Write};

use crate::proto::packets::*;

struct SimObject {
    id: u32,
    obj_type: DetectedObjectType,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    vx: f32,
    vy: f32,
    confidence: f32,
}

struct SimState {
    pool: [SimObject; 12],       // All 12 objects, always alive
    visible: Vec<usize>,         // Indices into pool that are currently "detected"
    current_index: usize,        // Which visible object we're emitting next
    frame_id: u32,
}

static STATE: Lazy<Mutex<SimState>> = Lazy::new(|| {
    Mutex::new(SimState {
        pool: std::array::from_fn(|i| make_object(i as u32)),
        visible: Vec::new(),
        current_index: 0,
        frame_id: 0,
    })
});

// CONFIG
#[derive(Clone)]
pub struct SimulatorConfig {
    pub jitter_ms: u64,
    pub packet_loss: f32, // 0.0 → 1.0
    pub record_path: Option<PathBuf>,
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
        StreamSpec { interval: Duration::from_millis(50), generator: gen_detected_objects },
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

        load_cell: Some(SensorBoardLoadCellInfo {
            sensor_index: 1,
            force_newtons: 2.5 + (t * 0.1).sin() * 0.5,
            mass_grams: 250.0 + (t * 0.1).cos() * 50.0,
            raw_counts: 1024,

            // Calibration status and parameters
            is_calibrated: true,
            scale_newtons_per_count: 6.0,
            tare_offset_counts: 7,

            state: SensorState::SensorOperating as i32,
            error_code: LoadCellErrorCode::LoadCellNoError as i32,
        }),

        pressure_sensor: Some(SensorBoardPressureInfo {
            sensor_index: 1, // 0-based index of the pressure sensor

            // Pressure data
            pressure_kpa: 2.0, // Pressure in kilopascals
            temperature_c: 3.0, // Temperature in Celsius (if available)
            voltage: 4.0, // Sensor output voltage (if available)

            // Calibration status
            is_calibrated: true,

            state: SensorState::SensorOperating as i32,
            error_code: PressureErrorCode::PressureNoError as i32,
        }),

        board_temperature: 42.0 + (t * 0.03).sin() * 2.0,
        board_voltage: 3.3 + (t * 0.05).cos() * 0.02,
    })
}

fn gen_detected_objects(t: f32) -> pb_envelope::Payload {
    let mut state = STATE.lock().unwrap();
    let mut rng = rand::rng();

    // Debug
    let start = std::time::Instant::now();
    // let mut state = STATE.lock().unwrap();
    let lock_time = start.elapsed();
    // Debug end

    // Start a new frame once we've emitted all visible objects
    if state.current_index >= state.visible.len() {
        state.frame_id += 1;
        state.current_index = 0;

        // Pick how many objects are visible this frame: 1–6
        let count = rng.random_range(1..=6);

        // Shuffle indices 0..12 and take `count` of them
        let mut indices: Vec<usize> = (0..12).collect();
        // Fisher-Yates partial shuffle (only need `count` elements)
        for i in 0..count {
            let j = rng.random_range(i..12);
            indices.swap(i, j);
        }
        state.visible = indices[..count].to_vec();
        state.visible.sort(); // optional: emit in ID order for tidiness

        // Collect indices to release the borrow on state.visible
        let visible_indices: Vec<usize> = state.visible.clone();

        // Update physics for all visible objects
        for vi in visible_indices {
            let obj = &mut state.pool[vi];
            obj.x = (obj.x + obj.vx).clamp(0.0, 1.0);
            obj.y = (obj.y + obj.vy).clamp(0.0, 1.0);
            // Bounce off edges
            if obj.x <= 0.0 || obj.x >= 1.0 { obj.vx = -obj.vx; }
            if obj.y <= 0.0 || obj.y >= 1.0 { obj.vy = -obj.vy; }
            let wobble = (t * 1.37 + obj.id as f32).sin() * 0.01;
            obj.confidence = (obj.confidence + wobble).clamp(0.5, 0.99);
        }
    }

    // Debug
    let total_time = start.elapsed();
    if total_time.as_millis() > 50 {
        println!("[detected] slow gen: lock={:?} total={:?}", lock_time, total_time);
    }
    //Debug end

    let total = state.visible.len() as u32;

    if total == 0 {
        return pb_envelope::Payload::DetectedObject(BasestationDetectedObject {
            frame_id: state.frame_id,
            total_count: 0,
            index: 0,
            id: 0,
            r#type: DetectedObjectType::ObjectUnknown as i32,
            bbox: None,
            confidence: 0.0,
        });
    }

    let idx = state.current_index;
    let pool_idx = state.visible[idx];
    let obj = &state.pool[pool_idx];

    let payload = pb_envelope::Payload::DetectedObject(BasestationDetectedObject {
        frame_id: state.frame_id,
        total_count: total,
        index: idx as u32,
        id: obj.id,
        r#type: obj.obj_type as i32,
        bbox: Some(BoundingBox {
            // bbox is normalized 0–1, cast to u32 loses everything — 
            // multiply by image dims if your receiver expects pixel coords,
            // or change the proto to float. Using *1000 as a fixed-point here:
            x: (obj.x * 1000.0) as u32,
            y: (obj.y * 1000.0) as u32,
            width: (obj.w * 1000.0) as u32,
            height: (obj.h * 1000.0) as u32,
        }),
        confidence: obj.confidence,
    });

    state.current_index += 1;
    payload
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

fn make_object(id: u32) -> SimObject {
    // Spread initial positions across the normalized space
    let x = (id as f32 * 0.083) % 1.0;          // ~evenly spread 0..1
    let y = (id as f32 * 0.137 + 0.05) % 1.0;

    // Small random-feeling velocities based on ID
    let vx = ((id * 7 + 1) % 9) as f32 * 0.001 - 0.004;
    let vy = ((id * 13 + 3) % 9) as f32 * 0.001 - 0.004;

    // Cycle through the meaningful object types (skip UNKNOWN = 0)
    let types = [
        DetectedObjectType::ObjectMainSwitch,
        DetectedObjectType::ObjectButton,
        DetectedObjectType::ObjectSwitch,
        DetectedObjectType::ObjectRotarySwitch,
        DetectedObjectType::ObjectSocket,
        DetectedObjectType::ObjectElectromagnet,
        DetectedObjectType::ObjectPlate,
        DetectedObjectType::ObjectCable,
    ];
    let obj_type = types[id as usize % types.len()];

    SimObject {
        id,
        obj_type,
        x,
        y,
        w: 0.05 + (id % 5) as f32 * 0.01,   // widths ~0.05–0.09
        h: 0.05 + (id % 7) as f32 * 0.01,   // heights ~0.05–0.11
        vx,
        vy,
        confidence: 0.70 + (id % 12) as f32 * 0.02,
    }
}

// ASYNG SIMULATOR

pub async fn run_simulator(
    socket: Arc<UdpSocket>,
    addr: String,
    token: CancellationToken,
    config: SimulatorConfig,
    streams: Vec<StreamSpec>,  
) -> anyhow::Result<()> {
    let socket_addr = addr.to_socket_addrs()?.next().unwrap();
    let mut last_times: Vec<Instant> = streams.iter().map(|_| Instant::now()).collect();
    let mut t = 0.0f32;
    let mut last_global = Instant::now();
    let mut rng = Lcg::new(0xdeadbeef);
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(4096);

    // Sender task
    let socket_clone = socket.clone();
    let cfg = config.clone();
    tokio::spawn(async move {
        while let Some(buf) = rx.recv().await {
            if rng.next_f32() < cfg.packet_loss { continue; }
            let jitter = (rng.next_f32() * cfg.jitter_ms as f32) as u64;
            sleep(Duration::from_millis(jitter)).await;
            let _ = socket_clone.send_to(&buf, socket_addr).await;
        }
    });

    let mut recorder = config.record_path.map(|p| std::fs::File::create(p).unwrap());

    loop {
        tokio::select! {
            biased;
            _ = token.cancelled() => {
                break;  // instant, no polling needed
            }
            _ = sleep(Duration::from_millis(1)) => {
                let now = Instant::now();
                let dt = now.duration_since(last_global).as_secs_f32();
                last_global = now;
                t += dt;

                for (i, spec) in streams.iter().enumerate() {
                    if now.duration_since(last_times[i]) >= spec.interval {
                        last_times[i] = now;
                        let payload = (spec.generator)(t);
                        let envelope = PbEnvelope { payload: Some(payload) };
                        let mut buf = Vec::new();
                        if envelope.encode(&mut buf).is_ok() {
                            if let Some(file) = recorder.as_mut() {
                                let _ = file.write_all(&(buf.len() as u32).to_le_bytes());
                                let _ = file.write_all(&buf);
                            }
                            if tx.send(buf).await.is_err() {
                                break; // receiver dropped, simulator shutting down
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}


pub fn spawn_simulator(
    token: CancellationToken,
    target_addr: String,
    config: SimulatorConfig,
    streams: Vec<StreamSpec>, 
) {
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                // Own socket, not shared with the listener
                let socket = Arc::new(
                    UdpSocket::bind("0.0.0.0:0").await.unwrap()
                );
                run_simulator(socket, target_addr, token, config, streams).await.ok();
            });
    });
}

pub fn detection_only_stream() -> Vec<StreamSpec> {
    vec![
        StreamSpec {
            interval: Duration::from_millis(200),
            generator: gen_detected_objects,
        },
    ]
}