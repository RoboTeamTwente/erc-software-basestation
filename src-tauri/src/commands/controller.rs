
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MovementCommand {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

#[tauri::command]
pub async fn pressed_key(command: MovementCommand) -> Result<(), String> {
    println!("Received command: {:?}", command);

    // TODO:
    // Send this to:
    // - Serial
    // - TCP socket
    // - BLE
    // - CAN bus
    // - GPIO
    // Whatever your robot uses

    Ok(())
}
