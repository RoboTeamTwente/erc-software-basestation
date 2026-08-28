use crate::proto::packets::{pb_envelope::Payload, PbEnvelope, BasestationDetectedObject};
use prost::Message;
use std::time::Instant;
use std::sync::Arc;
use tauri::Emitter;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

const THROTTLE_MS: u128 = 100;

struct Throttle {
    last: Instant,
}

impl Throttle {
    fn new() -> Self {
        Self { last: Instant::now() }
    }

    fn ready(&mut self) -> bool {
        if self.last.elapsed().as_millis() >= THROTTLE_MS {
            self.last = Instant::now();
            true
        } else {
            false
        }
    }
}

struct Throttles {
    imu: Throttle,
    gps: Throttle,
    ph: Throttle,
    arm_ctrl: Throttle,
    arm_diag: Throttle,
    arm_feedback: Throttle,
    arm_pos: Throttle,
    arm_target: Throttle,
    arm_obstructions: Throttle,
    drive_diag: Throttle,
    drive_motor: Throttle,
    drive_progress: Throttle,
    sensor_diag: Throttle,
    control_mode: Throttle,
    mission_command: Throttle,
    detected_objects: Throttle,
    object_selection: Throttle,
    load_cell: Throttle,
    pressure: Throttle,
    rock_measure_request: Throttle,
    rock_measure_result: Throttle,
    rover_localization: Throttle,
}

impl Throttles {
    fn new() -> Self {
        Self {
            imu: Throttle::new(),
            gps: Throttle::new(),
            ph: Throttle::new(),
            arm_ctrl: Throttle::new(),
            arm_diag: Throttle::new(),
            arm_feedback: Throttle::new(),
            arm_pos: Throttle::new(),
            arm_target: Throttle::new(),
            arm_obstructions: Throttle::new(),
            drive_diag: Throttle::new(),
            drive_motor: Throttle::new(),
            drive_progress: Throttle::new(),
            sensor_diag: Throttle::new(),
            control_mode: Throttle::new(),
            mission_command: Throttle::new(),
            detected_objects: Throttle::new(),
            object_selection: Throttle::new(),
            load_cell: Throttle::new(),
            pressure: Throttle::new(),
            rock_measure_request: Throttle::new(),
            rock_measure_result: Throttle::new(),
            rover_localization: Throttle::new(),
        }
    }
}

pub async fn run_listener(
    socket: Arc<UdpSocket>,
    cancel: CancellationToken,
    app_handle: tauri::AppHandle,
) {
    println!("[listener] Started, bound to {:?}", socket.local_addr());

    let mut buf = vec![0u8; 4096];
    let mut t = Throttles::new();
    let mut det_buffer: Vec<BasestationDetectedObject> = Vec::new();
    let mut det_frame: u32 = 0;

    loop {
        let (len, _addr) = tokio::select! {
            _ = cancel.cancelled() => {
                println!("UDP listener shutting down");
                return;
            }
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("UDP recv error: {e}");
                        continue;
                    }
                }
            }
        };

        let envelope = match PbEnvelope::decode(&buf[..len]) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Decode error: {e}");
                continue;
            }
        };

        let Some(payload) = envelope.payload else {
            continue;
        };

        match payload {
            Payload::ImuInfo(msg) => {
                if t.imu.ready() { app_handle.emit("imu-update", &msg).ok(); }
            }
            Payload::GpsInfo(msg) => {
                if t.gps.ready() { app_handle.emit("gps-update", &msg).ok(); }
                println!("GPS update: lat {}, lon {}, alt {}", msg.latitude, msg.longitude, msg.altitude);
            }
            Payload::PhInfo(msg) => {
                if t.ph.ready() { app_handle.emit("ph-update", &msg).ok(); }
            }
            Payload::ArmCtrl(msg) => {
                if t.arm_ctrl.ready() { app_handle.emit("arm-ctrl-update", &msg).ok(); }
            }
            Payload::ArmDiag(msg) => {
                if t.arm_diag.ready() { app_handle.emit("arm-diag-update", &msg).ok(); }
            }
            Payload::ArmFeedback(msg) => {
                if t.arm_feedback.ready() { app_handle.emit("arm-feedback-update", &msg).ok(); }
            }
            Payload::ArmPos(msg) => {
                if t.arm_pos.ready() { app_handle.emit("arm-pos-update", &msg).ok(); }
            }
            Payload::ArmTarget(msg) => {
                if t.arm_target.ready() { app_handle.emit("arm-target-update", &msg).ok(); }
            }
            Payload::ArmObstructions(msg) => {
                if t.arm_obstructions.ready() { app_handle.emit("arm-obstructions-update", &msg).ok(); }
            }
            Payload::DriveDiag(msg) => {
                if t.drive_diag.ready() { app_handle.emit("drive-diag-update", &msg).ok(); }
                //println!("Drive diag update: front left {:?}, front right {:?}, middle left {:?}, middle right {:?}, back left {:?}, back right {:?}", msg.front_left_motor, msg.front_right_motor, msg.middle_left_motor, msg.middle_right_motor, msg.back_left_motor, msg.back_right_motor);
            }
            Payload::DriveMotor(msg) => {
                if t.drive_motor.ready() { app_handle.emit("drive-motor-update", &msg).ok(); }
                //println!("Drive motor update: left {}, right {}", msg.distance_to_go, msg.turning_radius);
            }
            Payload::DriveProgress(msg) => {
                if t.drive_progress.ready() { app_handle.emit("drive-progress-update", &msg).ok(); }
                //println!("Drive progress update: {} distance left", msg.distance_left);
            }
            Payload::SensorDiag(msg) => {
                if t.sensor_diag.ready() { app_handle.emit("sensor-diag-update", &msg).ok(); }
            }
            Payload::ControlMode(msg) => {
                if t.control_mode.ready() { app_handle.emit("control-mode-update", &msg).ok(); }
            }
            Payload::DetectedObject(msg) => {
                if msg.frame_id != det_frame {
                    if !det_buffer.is_empty() {
                        let handle = app_handle.clone();
                        let batch = det_buffer.clone();
                        tokio::spawn(async move {
                            handle.emit("detected-objects-update", &batch).ok();
                        });
                        det_buffer.clear();
                    }
                    det_frame = msg.frame_id;
                }

                det_buffer.push(msg.clone());

                if msg.index + 1 == msg.total_count {
                    let handle = app_handle.clone();
                    let batch = det_buffer.clone();
                    det_buffer.clear();
                    tokio::spawn(async move {
                        handle.emit("detected-objects-update", &batch).ok();
                    });
                }
            }
            Payload::ObjectSelection(msg) => {
                if t.object_selection.ready() { app_handle.emit("object-selection-update", &msg).ok(); }
            }
            Payload::LoadCellInfo(msg) => {
                if t.load_cell.ready() { app_handle.emit("load-cell-update", &msg).ok(); }
            }
            Payload::PressureInfo(msg) => {
                if t.pressure.ready() { app_handle.emit("pressure-update", &msg).ok(); }
            }
            Payload::RockMeasureRequest(msg) => {
                if t.rock_measure_request.ready() { app_handle.emit("rock-measure-request-update", &msg).ok(); }
            }
            Payload::RockMeasureResult(msg) => {
                if t.rock_measure_result.ready() { app_handle.emit("rock-measure-result-update", &msg).ok(); }
            }
            Payload::RoverLocalization(msg) => {
                if t.rover_localization.ready() { app_handle.emit("rover-localization-update", &msg).ok(); }
            }
            Payload::ManualDrive(msg) => {
                println!("Manual drive command received: {:?}", msg);
            }
            Payload::ManualBrake(msg) => {
                println!("Manual brake command received: {:?}", msg);
            }
            Payload::ManualArm(msg) => {
                println!("Manual arm command received: {:?}", msg);
            }
            other => {
                eprintln!("Unhandled payload: {:?}", other);
            }
        }
    }
}