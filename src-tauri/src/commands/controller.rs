use serde::Deserialize;

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