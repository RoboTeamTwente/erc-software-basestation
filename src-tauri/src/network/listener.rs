use crate::proto::packets::{pb_envelope::Payload, PbEnvelope};
use prost::Message;
use std::time::Instant;
use tauri::Emitter;
use tokio::net::UdpSocket;

const THROTTLE_MS: u128 = 100;

struct Throttle {
    last: Instant,
}

impl Throttle {
    fn new() -> Self {
        Self {
            last: Instant::now(),
        }
    }

    // Returns true and resets the timer if enough time has passed
    fn ready(&mut self) -> bool {
        if self.last.elapsed().as_millis() >= THROTTLE_MS {
            self.last = Instant::now();
            true
        } else {
            false
        }
    }
}

// One throttle per payload variant, indexed for clarity
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
    object_selection: Throttle
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
            object_selection: Throttle::new()
        }
    }
}

pub async fn run_listener(
    socket: std::sync::Arc<UdpSocket>,
    app_handle: tauri::AppHandle,
) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 4096];
    let mut t = Throttles::new();

    loop {
        let (len, _addr) = socket.recv_from(&mut buf).await?;

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
                if t.imu.ready() {
                    app_handle.emit("imu-update", &msg);
                }
            }
            Payload::GpsInfo(msg) => {
                if t.gps.ready() {
                    app_handle.emit( "gps-update", &msg);
                }
            }
            Payload::PhInfo(msg) => {
                if t.ph.ready() {
                    app_handle.emit( "ph-update", &msg);
                }
            }
            Payload::ArmCtrl(msg) => {
                if t.arm_ctrl.ready() {
                    app_handle.emit( "arm-ctrl-update", &msg);
                }
            }
            Payload::ArmDiag(msg) => {
                if t.arm_diag.ready() {
                    app_handle.emit( "arm-diag-update", &msg);
                }
            }
            Payload::ArmFeedback(msg) => {
                if t.arm_feedback.ready() {
                    app_handle.emit( "arm-feedback-update", &msg);
                }
            }
            Payload::ArmPos(msg) => {
                if t.arm_pos.ready() {
                    app_handle.emit( "arm-pos-update", &msg);
                }
            }
            Payload::ArmTarget(msg) => {
                if t.arm_target.ready() {
                    app_handle.emit( "arm-target-update", &msg);
                }
            }
            Payload::ArmObstructions(msg) => {
                if t.arm_obstructions.ready() {
                    app_handle.emit( "arm-obstructions-update", &msg);
                }
            }
            Payload::DriveDiag(msg) => {
                if t.drive_diag.ready() {
                    app_handle.emit( "drive-diag-update", &msg);
                }
            }
            Payload::DriveMotor(msg) => {
                if t.drive_motor.ready() {
                    app_handle.emit( "drive-motor-update", &msg);
                }
            }
            Payload::DriveProgress(msg) => {
                if t.drive_progress.ready() {
                    app_handle.emit( "drive-progress-update", &msg);
                }
            }
            Payload::SensorDiag(msg) => {
                if t.sensor_diag.ready() {
                    app_handle.emit( "sensor-diag-update", &msg);
                }
            }
            Payload::ControlMode(msg) => {
                if t.control_mode.ready() {
                    app_handle.emit( "control-mode-update", &msg);
                }
            }
            Payload::MissionCommand(msg) => {
                if t.mission_command.ready() {
                    app_handle.emit( "mission-command-update", &msg);
                }
            }
            Payload::DetectedObjects(msg) => {
                if t.detected_objects.ready() {
                    app_handle.emit( "detected-objects-update", &msg);
                }
            }
            Payload::ObjectSelection(msg) => {
                if t.object_selection.ready() {
                    app_handle.emit( "object-selection-update", &msg);
                }
            }
        }
    }
}
