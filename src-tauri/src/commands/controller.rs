use serde::Deserialize;
use gilrs::{Gilrs, Event, EventType};
use tauri::AppHandle;
use std::thread;
use std::time::Duration;
use tauri::Manager;

use crate::commands::rover_states::RoverState;

#[derive(Debug, Deserialize)]
pub struct MovementCommand {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

#[tauri::command]
pub async fn pressed_key(_command: MovementCommand) -> Result<(), String> {
    Ok(())
}

pub fn start_controller_listener(app: AppHandle) {
    thread::spawn(move || {
        let mut gilrs = Gilrs::new().expect("Failed to initialize gilrs");

        // Print all detected gamepads at startup
        for (_id, gamepad) in gilrs.gamepads() {
            println!("[controller] Found: {}", gamepad.name());
        }

        loop {
            while let Some(Event { id, event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(button, _) => {
                        println!(
                            "[controller] Button pressed: {:?} (pad {})",
                            button, id
                        );

                        let state = app.state::<RoverState>();

                        match button {
                            gilrs::Button::Start => {
                                let mut pickup = state.pickup_mode.lock().unwrap();
                                *pickup = !*pickup;
                            }

                            gilrs::Button::Select => {
                                // Check pickup mode first
                                let pickup = *state.pickup_mode.lock().unwrap();

                                if pickup {
                                    let mut arm =
                                        state.arm_manual_mode.lock().unwrap();
                                    *arm = !*arm;
                                } else {
                                    let mut drive =
                                        state.drive_manual_mode.lock().unwrap();
                                    *drive = !*drive;
                                }
                            }

                            _ => {}
                        }
                    }

                    EventType::ButtonReleased(button, _) => {
                        println!(
                            "[controller] Button released: {:?} (pad {})",
                            button, id
                        );
                    }

                    EventType::AxisChanged(axis, value, _) => {
                        println!(
                            "[controller] Axis {:?} = {:.3} (pad {})",
                            axis, value, id
                        );
                    }

                    EventType::Connected => {
                        println!("[controller] Gamepad {} connected", id);
                    }

                    EventType::Disconnected => {
                        println!("[controller] Gamepad {} disconnected", id);
                    }

                    _ => {}
                }
            }

            thread::sleep(Duration::from_millis(8));
        }
    });
}