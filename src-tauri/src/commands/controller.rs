use gilrs::{Axis, Button, Event, EventType, Gilrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::commands::rover_commands::RoverAddress;
use crate::commands::rover_states::RoverState;
use crate::network::sender;
use crate::network::service::UdpService;
use crate::proto::packets::{pb_envelope, BasestationManualBrake, BasestationManualDrive, PbEnvelope};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Minimum axis delta (and deadzone boundary) before a drive/arm packet is sent.
const AXIS_CHANGE_THRESHOLD: f32 = 0.05;

/// How long a momentary brake (right trigger) stays engaged before auto-release.
const MOMENTARY_BRAKE_DURATION: Duration = Duration::from_millis(500);

/// How often the last-known state is re-sent even without any changes.
/// Acts as a keepalive so the rover never silently loses commanded state.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);

// ─── Shared command state ─────────────────────────────────────────────────────

/// State used while pickup_mode is false (normal driving).
#[derive(Clone, Default)]
struct DriveState {
    axes: DriveAxes,
    brake: bool,
}

/// State used while pickup_mode is true (arm control).
/// All fields map to sint32 just like DriveAxes.
#[derive(Clone, Default)]
struct PickupState {
    // TODO: left stick X/Y, right stick X/Y, D-pad up/down, D-pad right/left
}

/// Top-level shared state, switched on pickup_mode.
#[derive(Clone, Default)]
struct CommandState {
    drive: DriveState,
    pickup: PickupState,
}

// ─── Drive axis tracker ───────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct DriveAxes {
    /// Left stick Y: positive = forward, negative = backward.
    forward_backward: f32,
    /// Left stick X: positive = right, negative = left.
    turn: f32,
}

impl DriveAxes {
    /// Applies the deadzone, then updates the stored value if the change
    /// exceeds [`AXIS_CHANGE_THRESHOLD`]. Returns `true` when a send is needed.
    fn update(&mut self, axis: Axis, raw: f32) -> bool {
        let value = apply_deadzone(raw);
        match axis {
            Axis::LeftStickY => {
                if (value - self.forward_backward).abs() >= AXIS_CHANGE_THRESHOLD {
                    self.forward_backward = value;
                    return true;
                }
            }
            Axis::LeftStickX => {
                if (value - self.turn).abs() >= AXIS_CHANGE_THRESHOLD {
                    self.turn = value;
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn to_proto(&self) -> BasestationManualDrive {
        BasestationManualDrive {
            forward_backward: scale_to_sint32(self.forward_backward),
            turn: scale_to_sint32(self.turn),
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Maps values within `(-AXIS_CHANGE_THRESHOLD, +AXIS_CHANGE_THRESHOLD)` to 0.
fn apply_deadzone(value: f32) -> f32 {
    if value.abs() < AXIS_CHANGE_THRESHOLD { 0.0 } else { value }
}

/// Scales `[-1.0, 1.0]` to the full `sint32` range.
/// Uses `f64` internally to avoid overflow from `f32` precision loss near `i32::MAX`.
fn scale_to_sint32(value: f32) -> i32 {
    (value.clamp(-1.0, 1.0) as f64 * i32::MAX as f64) as i32
}

/// Returns `true` when the rover is in pickup mode.
fn is_pickup_mode(app: &AppHandle) -> bool {
    *app.state::<RoverState>().pickup_mode.lock().unwrap()
}

// ─── UDP senders ──────────────────────────────────────────────────────────────

async fn send_drive(socket: Arc<tokio::net::UdpSocket>, target: String, drive: BasestationManualDrive) {
    let envelope = PbEnvelope {
        payload: Some(pb_envelope::Payload::ManualDrive(drive)),
    };
    if let Err(e) = sender::send_envelope(&socket, &target, envelope).await {
        eprintln!("[controller] Failed to send drive command: {e}");
    }
}

async fn send_brake(socket: Arc<tokio::net::UdpSocket>, target: String, engaged: bool) {
    let envelope = PbEnvelope {
        payload: Some(pb_envelope::Payload::ManualBrake(BasestationManualBrake {
            brake: engaged,
        })),
    };
    if let Err(e) = sender::send_envelope(&socket, &target, envelope).await {
        eprintln!("[controller] Failed to send brake command: {e}");
    }
}

// ─── Dispatch helpers (sync -> async bridge) ──────────────────────────────────

fn dispatch_drive(app: &AppHandle, drive: BasestationManualDrive) {
    let socket = app.state::<UdpService>().socket();
    let target = app.state::<RoverAddress>().ip.clone();
    println!("[controller] Drive -> fwd={} turn={}", drive.forward_backward, drive.turn);
    tauri::async_runtime::spawn(async move { send_drive(socket, target, drive).await });
}

fn dispatch_brake(app: &AppHandle, engaged: bool) {
    let socket = app.state::<UdpService>().socket();
    let target = app.state::<RoverAddress>().ip.clone();
    println!("[controller] Brake -> {}", if engaged { "ENGAGED" } else { "released" });
    tauri::async_runtime::spawn(async move { send_brake(socket, target, engaged).await });
}

// ─── Pickup mode handlers (stubs) ─────────────────────────────────────────────

/// Called on every axis/button event while pickup_mode is true.
/// TODO: map left stick X/Y, right stick X/Y, D-pad up/down, D-pad right/left
/// to their respective sint32 proto fields and dispatch.
fn handle_pickup_axis(_app: &AppHandle, _state: &mut PickupState, _axis: Axis, _raw: f32) {
    // TODO
}

/// TODO: map D-pad and any pickup-specific buttons.
fn handle_pickup_button(_app: &AppHandle, _state: &mut PickupState, _button: Button) {
    // TODO
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn start_controller_listener(app: AppHandle) {
    let shared = Arc::new(Mutex::new(CommandState::default()));

    // Heartbeat thread: every HEARTBEAT_INTERVAL, re-sends the relevant state
    // based on the current mode so the rover never silently loses commanded state.
    {
        let shared = Arc::clone(&shared);
        let app = app.clone();
        thread::spawn(move || loop {
            thread::sleep(HEARTBEAT_INTERVAL);

            if is_pickup_mode(&app) {
                // Pickup heartbeat: brake is always engaged during pickup (not toggleable).
                // TODO: also re-send arm state once handle_pickup_axis is implemented.
                dispatch_brake(&app, true);
            } else {
                // Drive heartbeat: re-send current drive axes and brake state.
                let state = shared.lock().unwrap().clone();
                dispatch_drive(&app, state.drive.axes.to_proto());
                dispatch_brake(&app, state.drive.brake);
            }
        });
    }

    // Event thread: processes gilrs events and dispatches commands.
    thread::spawn(move || {
        let mut gilrs = Gilrs::new().expect("Failed to initialise gilrs");

        for (_id, gamepad) in gilrs.gamepads() {
            println!("[controller] Found gamepad: {}", gamepad.name());
        }

        loop {
            while let Some(Event { id, event, .. }) = gilrs.next_event() {
                match event {
                    // Axis events are routed based on current mode.
                    EventType::AxisChanged(axis, value, _) => {
                        //println!("[controller] Axis {axis:?} = {value:.3} (pad {id})");

                        let mut state = shared.lock().unwrap();
                        if is_pickup_mode(&app) {
                            handle_pickup_axis(&app, &mut state.pickup, axis, value);
                        } else {
                            if state.drive.axes.update(axis, value) {
                                let proto = state.drive.axes.to_proto();
                                drop(state);
                                dispatch_drive(&app, proto);
                            }
                        }
                    }

                    // Left trigger: toggle latching brake (drive mode only).
                    // First press engages; second press disengages.
                    EventType::ButtonPressed(Button::LeftTrigger2, _) => {
                        if is_pickup_mode(&app) { continue; }

                        let mut state = shared.lock().unwrap();
                        state.drive.brake = !state.drive.brake;
                        let engaged = state.drive.brake;
                        drop(state);

                        println!(
                            "[controller] Left trigger -> brake {}",
                            if engaged { "ENGAGED (latched)" } else { "DISENGAGED" }
                        );
                        dispatch_brake(&app, engaged);
                    }
                    // Release is ignored: the latch is toggled on press only.
                    EventType::ButtonReleased(Button::LeftTrigger2, _) => {}

                    // Right trigger: momentary brake (drive mode only).
                    // Engages for MOMENTARY_BRAKE_DURATION then auto-releases.
                    EventType::ButtonPressed(Button::RightTrigger2, _) => {
                        if is_pickup_mode(&app) { continue; }

                        println!("[controller] Right trigger -> brake ENGAGED (momentary)");
                        shared.lock().unwrap().drive.brake = true;
                        dispatch_brake(&app, true);

                        let shared = Arc::clone(&shared);
                        let app = app.clone();
                        thread::spawn(move || {
                            thread::sleep(MOMENTARY_BRAKE_DURATION);
                            println!("[controller] Momentary brake released");
                            shared.lock().unwrap().drive.brake = false;
                            dispatch_brake(&app, false);
                        });
                    }

                    // All other buttons — Start/Select work in both modes.
                    // In pickup mode, additional inputs are also routed to the pickup handler.
                    EventType::ButtonPressed(button, _) => {
                        println!("[controller] Button pressed:  {button:?} (pad {id})");

                        handle_button_pressed(&app, button);

                        if is_pickup_mode(&app) {
                            let mut state = shared.lock().unwrap();
                            handle_pickup_button(&app, &mut state.pickup, button);
                        }
                    }
                    EventType::ButtonReleased(button, _) => {
                        println!("[controller] Button released: {button:?} (pad {id})");
                    }

                    EventType::Connected    => println!("[controller] Gamepad {id} connected"),
                    EventType::Disconnected => println!("[controller] Gamepad {id} disconnected"),

                    _ => {}
                }
            }

            thread::sleep(Duration::from_millis(8));
        }
    });
}

// ─── Drive mode button handler ────────────────────────────────────────────────

fn handle_button_pressed(app: &AppHandle, button: Button) {
    let state = app.state::<RoverState>();

    match button {
        Button::Start => {
            let mut pickup = state.pickup_mode.lock().unwrap();
            *pickup = !*pickup;
            println!("[controller] Pickup mode: {}", *pickup);
        }

        Button::Select => {
            let pickup = *state.pickup_mode.lock().unwrap();
            if pickup {
                let mut arm = state.arm_manual_mode.lock().unwrap();
                *arm = !*arm;
                println!("[controller] Arm manual mode: {}", *arm);
            } else {
                let mut drive = state.drive_manual_mode.lock().unwrap();
                *drive = !*drive;
                println!("[controller] Drive manual mode: {}", *drive);
            }
        }

        _ => {}
    }
}