use std::sync::Mutex;
use tauri::State;
use serde::Deserialize;

pub struct RoverState {
    pub drive_manual_mode: Mutex<bool>,
    pub arm_manual_mode: Mutex<bool>,
    pub pickup_mode: Mutex<bool>,
}

#[derive(Deserialize)]
pub enum StateType {
    DriveManual,
    ArmManual,
    Pickup,
}

#[tauri::command]
pub async fn set_state(
    state_type: StateType,
    value: bool,
    state: State<'_, RoverState>,
) -> Result<(), String> {
    match state_type {
        StateType::DriveManual => {
            *state
                .drive_manual_mode
                .lock()
                .map_err(|_| "Lock failed")? = value;
        }
        StateType::ArmManual => {
            *state
                .arm_manual_mode
                .lock()
                .map_err(|_| "Lock failed")? = value;
        }
        StateType::Pickup => {
            *state
                .pickup_mode
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
        StateType::DriveManual => *state
            .drive_manual_mode
            .lock()
            .map_err(|_| "Lock failed")?,
        StateType::ArmManual => *state
            .arm_manual_mode
            .lock()
            .map_err(|_| "Lock failed")?,
        StateType::Pickup => *state
            .pickup_mode
            .lock()
            .map_err(|_| "Lock failed")?,
    };

    Ok(value)
}

// async fn sync_with_rover(state: State<'_, UdpService>, ) {

//     println!("Synced mode with rover: {}", );
//     let socket = state.socket();
//     let target = "127.0.0.1:9000";

//     // Build an envelope with the selected object ID (this is just an example, adjust as needed)
//     let envelope = PbEnvelope {
//         payload: Some(pb_envelope::Payload::ObjectSelection(
//             BasestationObjectSelection { object_id }
//         )),
//     };
    
//     sender::send_envelope(&socket, target, envelope).await.map_err(|e| {
//         println!("Failed to send object selection: {}", e);
//     }).ok();

//     println!("Synced with rover");
//     Ok(())
// }