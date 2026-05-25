use std::sync::Mutex;
use tauri::State;
use serde::Deserialize;

pub struct RoverState {
    pub manual_mode: Mutex<bool>,
    pub pickup_mode: Mutex<bool>,
    pub braked: Mutex<bool>,
}

#[derive(Deserialize)]
pub enum StateType {
    Manual,
    Pickup,
    Braked,
}

#[tauri::command]
pub async fn set_state(
    state_type: StateType,
    value: bool,
    state: State<'_, RoverState>,
) -> Result<(), String> {
    match state_type {
        StateType::Manual => {
            *state
                .manual_mode
                .lock()
                .map_err(|_| "Lock failed")? = value;
        }
        StateType::Pickup => {
            *state
                .pickup_mode
                .lock()
                .map_err(|_| "Lock failed")? = value;
        }
        StateType::Braked => {
            *state
                .braked
                .lock()
                .map_err(|_| "Lock failed")? = value;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_state(
    state_type: StateType,
    state: State<'_, RoverState>,
) -> Result<bool, String> {
    let value = match state_type {
        StateType::Manual => *state
            .manual_mode
            .lock()
            .map_err(|_| "Lock failed")?,
        StateType::Pickup => *state
            .pickup_mode
            .lock()
            .map_err(|_| "Lock failed")?,
        StateType::Braked => *state
            .braked
            .lock()
            .map_err(|_| "Lock failed")?,
    };

    Ok(value)
}
