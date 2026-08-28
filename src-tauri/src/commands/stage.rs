use tokio::net::UdpSocket;
use tauri::{AppHandle, Manager};    
use crate::UdpServiceHandle;
use crate::RoverAddress;
use crate::network::sender;


#[tauri::command]
pub async fn go_on_stage(app: AppHandle) -> Result<(), String> {

    let data = b"go_on_stage";

    send_stage_commands(app, data).await?;

    println!("Going on stage...");
    Ok(())
}

#[tauri::command]
pub async fn stop_going_on_stage(app: AppHandle) -> Result<(), String> {

    let data = b"stop";

    send_stage_commands(app, data).await?;

    println!("Stopping going on stage...");
    Ok(())
}

#[tauri::command]
pub async fn set_rover_profile(app: AppHandle, speed: i32, max_acceleration: u16, driving_time: u32) -> Result<(), String> {

    let data = format!("profile {} {} {}", speed, max_acceleration, driving_time).into_bytes().to_vec();

    send_stage_commands(app, &data).await?;

    println!("Set rover profile: speed={}, max_acceleration={}, driving_time={}", speed, max_acceleration, driving_time);
    Ok(())
}

pub async fn send_stage_commands(app: AppHandle, data: &[u8]) -> Result<(), String> {

    let socket = app.state::<UdpServiceHandle>().service.lock().await.socket();

    let target = app.state::<RoverAddress>().ip.lock().unwrap().clone();

    socket
        .send_to(data, target)
        .await
        .map_err(|e| format!("Failed to send 'Go on Stage' command: {}", e))?;

    println!("Going on stage...");
    Ok(())
}