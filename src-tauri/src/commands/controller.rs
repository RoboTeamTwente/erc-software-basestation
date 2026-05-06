use gilrs::{Axis, Button, Event, EventType, Gilrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::RoverAddress;
use crate::commands::rover_states::RoverState;
use crate::network::sender;
use crate::network::service::UdpService;
use crate::proto::packets::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Minimum axis delta (and deadzone boundary) before a drive/arm packet is sent.
const AXIS_CHANGE_THRESHOLD: f32 = 0.05;

/// How long a momentary brake (right trigger) stays engaged before auto-release.
const MOMENTARY_BRAKE_DURATION: Duration = Duration::from_millis(500);

/// How often the last-known state is re-sent even without any changes.
/// Acts as a keepalive so the rover never silently loses commanded state.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);

/// Time in seconds for a ramped axis to travel from 0.0 to ±1.0 when held.
const RAMP_DURATION_SECS: f32 = 1.0;

/// Tick rate for the ramp thread — how often the ramp value is incremented (~60 Hz).
const RAMP_TICK_MS: u64 = 16;

// ─── Ramp axis ────────────────────────────────────────────────────────────────

/// Tracks a boolean-driven axis that ramps from 0.0 toward ±1.0 while held,
/// and snaps back to 0.0 on release. Thread-safe via Arc<Mutex<RampAxis>>.
#[derive(Default)]
struct RampAxis {
    /// Current normalised value in [-1.0, 1.0].
    value: f32,
    /// +1.0 while the positive button is held, -1.0 for negative, 0.0 when released.
    direction: f32,
}

impl RampAxis {
    /// Called on button press. `direction` must be +1.0 or -1.0.
    fn press(&mut self, direction: f32) {
        self.direction = direction;
    }

    /// Called on button release — immediately zeros both value and direction.
    fn release(&mut self) {
        self.direction = 0.0;
        self.value = 0.0;
    }

    /// Advances the ramp by one tick. Returns the new value when it changed.
    fn tick(&mut self) -> Option<f32> {
        if self.direction == 0.0 {
            return None;
        }
        let step = self.direction * (RAMP_TICK_MS as f32 / 1000.0) / RAMP_DURATION_SECS;
        let new_value = (self.value + step).clamp(-1.0, 1.0);
        if (new_value - self.value).abs() > f32::EPSILON {
            self.value = new_value;
            Some(new_value)
        } else {
            None
        }
    }
}

// ─── Shared command state ─────────────────────────────────────────────────────

/// State used while pickup_mode is false (normal driving).
#[derive(Clone, Default)]
struct DriveState {
    axes: DriveAxes,
    brake: bool,
}

/// State used while pickup_mode is true (arm control).
/// Analog axes are deadzoned and threshold-filtered.
/// Z and speed are driven by boolean buttons ramped over time via [`RampAxis`].
struct PickupState {
    /// Right stick X -> end effector velocity X (positive = right, negative = left).
    x: f32,
    /// Right stick Y -> end effector velocity Y (positive = forward, negative = backward).
    y: f32,
    /// D-pad up/down -> end effector velocity Z, ramped. Positive = up, negative = down.
    z: f32,
    /// Left stick X  -> rotation (positive = clockwise, negative = counterclockwise).
    rotate: f32,
    /// Left stick Y  -> flick (positive = forward, negative = backward).
    flick: f32,
    /// LeftTrigger (close, -1.0) / RightTrigger (open, +1.0) -> gripper speed, ramped.
    speed: f32,
    /// Ramp state for Z (DPadUp = positive, DPadDown = negative).
    z_ramp: Arc<Mutex<RampAxis>>,
    /// Ramp state for gripper speed (LeftTrigger = negative, RightTrigger = positive).
    speed_ramp: Arc<Mutex<RampAxis>>,
}

impl Default for PickupState {
    fn default() -> Self {
        Self {
            x: 0.0, y: 0.0, z: 0.0,
            rotate: 0.0, flick: 0.0, speed: 0.0,
            z_ramp: Arc::new(Mutex::new(RampAxis::default())),
            speed_ramp: Arc::new(Mutex::new(RampAxis::default())),
        }
    }
}

impl Clone for PickupState {
    fn clone(&self) -> Self {
        Self {
            x: self.x, y: self.y, z: self.z,
            rotate: self.rotate, flick: self.flick, speed: self.speed,
            // Arc clones share the same ramp instances so the ramp threads
            // always write to the same Mutex the event loop reads.
            z_ramp: Arc::clone(&self.z_ramp),
            speed_ramp: Arc::clone(&self.speed_ramp),
        }
    }
}

impl PickupState {
    /// Updates an analog field if the deadzoned delta exceeds [`AXIS_CHANGE_THRESHOLD`].
    /// Z and speed are excluded — they are driven by ramped boolean inputs, not axes.
    fn update_axis(&mut self, axis: Axis, raw: f32) -> bool {
        let value = apply_deadzone(raw);
        let (current, new_val) = match axis {
            Axis::RightStickX => (&mut self.x,      value),
            Axis::RightStickY => (&mut self.y,      value),
            Axis::LeftStickX  => (&mut self.rotate, value),
            Axis::LeftStickY  => (&mut self.flick,  value),
            _ => return false,
        };
        if (new_val - *current).abs() >= AXIS_CHANGE_THRESHOLD {
            *current = new_val;
            true
        } else {
            false
        }
    }

    fn to_proto(&self) -> BasestationManualArmMovement {
        BasestationManualArmMovement {
            x:      scale_to_sint32(self.x),
            y:      scale_to_sint32(self.y),
            z:      scale_to_sint32(self.z),
            rotate: scale_to_sint32(self.rotate),
            flick:  scale_to_sint32(self.flick),
            speed:  scale_to_sint32(self.speed),
        }
    }
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

fn is_manual_mode(app: &AppHandle) -> bool {
    //println!("[controller] Checking manual mode: {}", *app.state::<RoverState>().manual_mode.lock().unwrap());
    *app.state::<RoverState>().manual_mode.lock().unwrap()
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

async fn send_arm(socket: Arc<tokio::net::UdpSocket>, target: String, arm: BasestationManualArmMovement) {
    let envelope = PbEnvelope {
        payload: Some(pb_envelope::Payload::ManualArm(arm)),
    };
    if let Err(e) = sender::send_envelope(&socket, &target, envelope).await {
        eprintln!("[controller] Failed to send arm command: {e}");
    }
}

// ─── Dispatch helpers (sync -> async bridge) ──────────────────────────────────

fn dispatch_drive(app: &AppHandle, drive: BasestationManualDrive) {
    if is_pickup_mode(app) || !is_manual_mode(app) {
        return;
    }

    let socket = app.state::<UdpService>().socket();
    let target = app.state::<RoverAddress>().ip.lock().unwrap().clone();

    tauri::async_runtime::spawn(async move {
        send_drive(socket, target, drive).await
    });
}

fn dispatch_brake(app: &AppHandle, engaged: bool) {
    // Brake rules depend on mode
    if !is_manual_mode(app){
        return;
    }

    let rov_state = app.state::<RoverState>();
    let mut braked = rov_state.braked.lock().unwrap();
    *braked = engaged;

    let socket = app.state::<UdpService>().socket();
    let target = app.state::<RoverAddress>().ip.lock().unwrap().clone();

    tauri::async_runtime::spawn(async move {
        send_brake(socket, target, engaged).await
    });
}

fn dispatch_arm(app: &AppHandle, arm: BasestationManualArmMovement) {
    if !is_pickup_mode(app) || !is_manual_mode(app){
        return;
    }

    let socket = app.state::<UdpService>().socket();
    let target = app.state::<RoverAddress>().ip.lock().unwrap().clone();

    tauri::async_runtime::spawn(async move {
        send_arm(socket, target, arm).await
    });
}

// ─── Pickup mode handlers ─────────────────────────────────────────────────────

/// Routes axis events to the arm state and dispatches when a meaningful change occurs.
///
/// Mapping:
///   Right stick X/Y  -> end effector X/Y velocity
///   D-pad Y          -> end effector Z velocity (up/down)
///   Left stick X     -> rotate
///   Left stick Y     -> flick
///   Speed is set via trigger buttons in the event loop (LeftTrigger2 / RightTrigger2).
fn handle_pickup_axis(app: &AppHandle, state: &mut PickupState, axis: Axis, raw: f32) {
    if state.update_axis(axis, raw) {
        dispatch_arm(app, state.to_proto());
    }
}

/// Routes button events that are specific to pickup/arm mode.
/// DPad up/down (Z ramp) are handled at the call site where `shared` is available.
/// Start and Select are handled globally before this is called.
fn handle_pickup_button(_app: &AppHandle, _state: &mut PickupState, _button: Button) {
    // Additional pickup button mappings go here.
}

/// Spawns a thread that ticks `ramp` at [`RAMP_TICK_MS`] and calls `on_tick(value)`
/// each time the value changes, until the ramp direction becomes 0.0 (released).
fn spawn_ramp_thread<F>(ramp: Arc<Mutex<RampAxis>>, on_tick: F)
where
    F: Fn(f32) + Send + 'static,
{
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(RAMP_TICK_MS));
        let mut r = ramp.lock().unwrap();
        if r.direction == 0.0 {
            break;
        }
        if let Some(value) = r.tick() {
            drop(r); // release lock before calling on_tick
            on_tick(value);
        }
    });
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
                // Also re-sends the current arm state so the rover doesn't lose velocity commands.
                let state = shared.lock().unwrap().clone();
                dispatch_brake(&app, true);
                dispatch_arm(&app, state.pickup.to_proto());
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
                    // Dispatching is suppressed when the relevant manual mode is inactive.
                    EventType::AxisChanged(axis, value, _) => {
                        //println!("[controller] Axis {axis:?} = {value:.3} (pad {id})");
 
                        let mut state = shared.lock().unwrap();
                        if is_manual_mode(&app) {
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
                    }
 
                    // Left trigger (LeftTrigger / trigger 1):
                    //   Drive mode  — toggle latching brake.
                    //   Pickup mode — ramp gripper speed toward -1.0 (close) while held.
                    EventType::ButtonPressed(Button::LeftTrigger, _) => {
                        if is_pickup_mode(&app) {
                            //println!("[controller] Left trigger -> gripper ramping CLOSE");
                            let speed_ramp = Arc::clone(&shared.lock().unwrap().pickup.speed_ramp);
                            speed_ramp.lock().unwrap().press(-1.0);
                            spawn_ramp_thread(Arc::clone(&speed_ramp), {
                                let shared = Arc::clone(&shared);
                                let app = app.clone();
                                move |v| {
                                    shared.lock().unwrap().pickup.speed = v;
                                    dispatch_arm(&app, shared.lock().unwrap().pickup.to_proto());
                                }
                            });
                        } else {
                            let mut state = shared.lock().unwrap();
                            state.drive.brake = !state.drive.brake;
                            let engaged = state.drive.brake;
                            drop(state);
                            // println!(
                            //     "[controller] Left trigger -> brake {}",
                            //     if engaged { "ENGAGED (latched)" } else { "DISENGAGED" }
                            // );
                            dispatch_brake(&app, engaged);
                        }
                    }
                    // Release: stop ramping and zero gripper speed (pickup) or ignore (drive latch).
                    EventType::ButtonReleased(Button::LeftTrigger, _) => {
                        //println!("[controller] Left trigger released -> gripper speed 0");
                        let state = shared.lock().unwrap();
                        state.pickup.speed_ramp.lock().unwrap().release();
                        drop(state);
                        let mut state = shared.lock().unwrap();
                        state.pickup.speed = 0.0;
                        let proto = state.pickup.to_proto();
                        drop(state);
                        dispatch_arm(&app, proto);
                        // Drive mode: release is intentionally ignored (latch stays).
                    }
 
                    // Right trigger (RightTrigger / trigger 1):
                    //   Drive mode  — momentary brake.
                    //   Pickup mode — ramp gripper speed toward +1.0 (open) while held.
                    EventType::ButtonPressed(Button::RightTrigger, _) => {
                        if is_pickup_mode(&app) {
                            //println!("[controller] Right trigger -> gripper ramping OPEN");
                            let speed_ramp = Arc::clone(&shared.lock().unwrap().pickup.speed_ramp);
                            speed_ramp.lock().unwrap().press(1.0);
                            spawn_ramp_thread(Arc::clone(&speed_ramp), {
                                let shared = Arc::clone(&shared);
                                let app = app.clone();
                                move |v| {
                                    shared.lock().unwrap().pickup.speed = v;
                                    dispatch_arm(&app, shared.lock().unwrap().pickup.to_proto());
                                }
                            });
                        }
                    }
                    // Release: stop ramping and zero gripper speed (pickup) or ignore (drive).
                    EventType::ButtonReleased(Button::RightTrigger, _) => {
                        //println!("[controller] Left trigger released -> gripper speed 0");
                        let state = shared.lock().unwrap();
                        state.pickup.speed_ramp.lock().unwrap().release();
                        drop(state);
                        let mut state = shared.lock().unwrap();
                        state.pickup.speed = 0.0;
                        let proto = state.pickup.to_proto();
                        drop(state);
                        dispatch_arm(&app, proto);
                    }

                    // Right trigger 2 (drive mode only — kept for temporary brake).
                    EventType::ButtonPressed(Button::RightTrigger2, _) => {
                        if !is_pickup_mode(&app) {
                            //println!("[controller] Right trigger -> brake ENGAGED (momentary)");
                            shared.lock().unwrap().drive.brake = true;

                            dispatch_brake(&app, true);
 
                            let shared = Arc::clone(&shared);
                            let app = app.clone();
                            // thread::spawn(move || {
                            //     thread::sleep(MOMENTARY_BRAKE_DURATION);
                            //     //println!("[controller] Momentary brake released");
                            //     shared.lock().unwrap().drive.brake = false;
                            //     dispatch_brake(&app, false);
                            // });
                        }
                    }
                    EventType::ButtonReleased(Button::RightTrigger2, _) => {
                        if !is_pickup_mode(&app) {
                            // Drive mode: release brake .
                            shared.lock().unwrap().drive.brake = false;

                            dispatch_brake(&app, false);
                        }
                    }
 
                    // Left trigger 2 (drive mode only — kept for brake toggle).
                    EventType::ButtonPressed(Button::LeftTrigger2, _) => {
                        if !is_pickup_mode(&app) {
                            let mut state = shared.lock().unwrap();
                            state.drive.brake = !state.drive.brake;
                            let engaged = state.drive.brake;

                            drop(state);
                            // println!(
                            //     "[controller] Left trigger2 -> brake {}",
                            //     if engaged { "ENGAGED (latched)" } else { "DISENGAGED" }
                            // );
                            dispatch_brake(&app, engaged);
                        }
                    }
                    EventType::ButtonReleased(Button::LeftTrigger2, _) => {}
 
                    // All other buttons — Start/Select work in both modes.
                    // DPad up/down control Z ramp in pickup mode.
                    EventType::ButtonPressed(button, _) => {
                        //println!("[controller] Button pressed:  {button:?} (pad {id})");
 
                        handle_button_pressed(&app, button);
 
                        if is_pickup_mode(&app) {
                            match button {
                                // D-pad up: ramp Z toward +1.0 (end effector up).
                                Button::DPadUp => {
                                    //println!("[controller] DPad up -> Z ramping UP");
                                    let z_ramp = Arc::clone(&shared.lock().unwrap().pickup.z_ramp);
                                    z_ramp.lock().unwrap().press(1.0);
                                    spawn_ramp_thread(Arc::clone(&z_ramp), {
                                        let shared = Arc::clone(&shared);
                                        let app = app.clone();
                                        move |v| {
                                            shared.lock().unwrap().pickup.z = v;
                                            dispatch_arm(&app, shared.lock().unwrap().pickup.to_proto());
                                        }
                                    });
                                }
                                // D-pad down: ramp Z toward -1.0 (end effector down).
                                Button::DPadDown => {
                                    //println!("[controller] DPad down -> Z ramping DOWN");
                                    let z_ramp = Arc::clone(&shared.lock().unwrap().pickup.z_ramp);
                                    z_ramp.lock().unwrap().press(-1.0);
                                    spawn_ramp_thread(Arc::clone(&z_ramp), {
                                        let shared = Arc::clone(&shared);
                                        let app = app.clone();
                                        move |v| {
                                            shared.lock().unwrap().pickup.z = v;
                                            dispatch_arm(&app, shared.lock().unwrap().pickup.to_proto());
                                        }
                                    });
                                }
                                _ => {
                                    let mut state = shared.lock().unwrap();
                                    handle_pickup_button(&app, &mut state.pickup, button);
                                }
                            }
                        }
                    }
                    EventType::ButtonReleased(button, _) => {
                        //println!("[controller] Button released: {button:?} (pad {id})");
 
                        if is_pickup_mode(&app) {
                            match button {
                                // D-pad release: stop Z ramp and zero Z immediately.
                                Button::DPadUp | Button::DPadDown => {
                                    //println!("[controller] DPad released -> Z = 0");
                                    let mut state = shared.lock().unwrap();
                                    state.pickup.z_ramp.lock().unwrap().release();
                                    state.pickup.z = 0.0;
                                    let proto = state.pickup.to_proto();
                                    drop(state);
                                    dispatch_arm(&app, proto);
                                }
                                _ => {}
                            }
                        }
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
            let mut manual = state.manual_mode.lock().unwrap();
            *manual = !*manual;
            println!("[controller] Manual mode: {}", *manual);
        }

        _ => {}
    }
}