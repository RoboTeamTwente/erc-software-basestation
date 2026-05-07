use std::fs;
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
pub struct RoverConfig {
    pub ip: String,
    pub local_port: u16,
}

impl Default for RoverConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1:9000".into(),
            local_port: 9000,
        }
    }
}

fn config_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path().app_data_dir()
        .expect("No app data dir")
        .join("rover_config.json")
}

pub fn load_config(app: &tauri::AppHandle) -> RoverConfig {
    let path = config_path(app);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(app: &tauri::AppHandle, config: &RoverConfig) -> Result<(), String> {
    let path = config_path(app);
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}