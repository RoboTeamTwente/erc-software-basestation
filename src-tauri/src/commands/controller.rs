use std::sync::Mutex;
use tauri::State;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MovementCommand {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

pub struct ControlModeState {
    pub manual_mode: Mutex<bool>,
}
pub struct PickupMode {
    pub pickup_mode: Mutex<bool>,
}

#[tauri::command]
pub async fn pressed_key(command: MovementCommand) -> Result<(), String> {
    println!("Received command: {:?}", command);

    Ok(())
}

#[tauri::command]
pub async fn control_mode_to_backend(manual_mode: bool, state: State<'_, ControlModeState>) -> Result<(), String> {
    let mut mode = state
        .manual_mode
        .lock()
        .map_err(|_| "Failed to lock state")?;

    *mode = manual_mode;

    Ok(())
}

#[tauri::command]
pub async fn control_mode_from_backend(state: State<'_, ControlModeState>) -> Result<bool, String> {
    let mode = state
        .manual_mode
        .lock()
        .map_err(|_| "Failed to lock state")?;

    Ok(*mode)
}

#[tauri::command]
pub async fn pickup_mode_to_backend(pickup_mode: bool, state: State<'_, PickupMode>) -> Result<(), String> {
    let mut mode = state
        .pickup_mode
        .lock()
        .map_err(|_| "Failed to lock state")?;

    *mode = pickup_mode;

    Ok(())
}

#[tauri::command]
pub async fn pickup_mode_from_backend(state: State<'_, PickupMode>) -> Result<bool, String> {
    let mode = state
        .pickup_mode
        .lock()
        .map_err(|_| "Failed to lock state")?;

    Ok(*mode)
}