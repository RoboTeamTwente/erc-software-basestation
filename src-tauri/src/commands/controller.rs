use serde::Deserialize;
use gilrs::{Gilrs, Event, EventType};
use tauri::AppHandle;

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
    std::thread::spawn(move || {
        let mut gilrs = Gilrs::new().unwrap();

        // Print all detected gamepads at startup
        for (_id, gamepad) in gilrs.gamepads() {
            println!("[controller] Found: {} ", gamepad.name());
        }

        loop {
            while let Some(Event { id, event, time , .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(button, _) => {
                        println!("[controller] Button pressed:  {:?} (pad {})", button, id);
                    }
                    EventType::ButtonReleased(button, _) => {
                        println!("[controller] Button released: {:?} (pad {})", button, id);
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        println!("[controller] Axis {:?} = {:.3} (pad {})", axis, value, id);
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
            std::thread::sleep(std::time::Duration::from_millis(8));
        }
    });
}