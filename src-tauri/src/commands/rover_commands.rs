use tauri::State;
use crate::network::sender;
use crate::network::service::UdpService;
use crate::proto::packets::*;
use crate::RoverAddress;


#[tauri::command]
pub async fn request_coordinates(state: State<'_, UdpService>, rover_addr: State<'_, RoverAddress>,) -> Result<(i16, i16), i16> {
    let _socket = state.socket();
    let _target = rover_addr.ip.as_str(); 
    println!("Requesting coordinates from rover...");
    // Simulate a delay for the request
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // Return dummy coordinates (latitude, longitude)
    Ok((37, -122))
}

#[tauri::command]
pub async fn request_weight(state: State<'_, UdpService>, rover_addr: State<'_, RoverAddress>,) -> Result<i16, i16> {
    let _socket = state.socket();
    let _target = rover_addr.ip.as_str(); 
    println!("Requesting rock weight from rover...");
    // Simulate a delay for the request
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // Return dummy weight
    Ok(222)
}

#[tauri::command]
pub async fn request_measurement(state: State<'_, UdpService>, rover_addr: State<'_, RoverAddress>, camera1: String, x1: f64, y1: f64, camera2: String, x2: f64, y2: f64) -> Result<i16, i16> {
    println!("Requested rock meassurement between: x1={}, y1={}, x2={}, y2={}", x1, y1, x2, y2);
    let socket = state.socket();
    let target = rover_addr.ip.as_str(); 

    let x1 = (x1 * 1000.0) as u32;
    let y1 = (y1 * 1000.0) as u32;
    let x2 = (x2 * 1000.0) as u32;
    let y2 = (y2 * 1000.0) as u32;

    // Build an envelope with the selected object ID (this is just an example, adjust as needed)
    let envelope = PbEnvelope {
        payload: Some(pb_envelope::Payload::RockMeasureRequest(
            BasestationRockMeasureRequest { x1, y1, x2, y2 }
        )),
    };
    
    sender::send_envelope(&socket, target, envelope).await.map_err(|e| {
        println!("Failed to send object selection: {}", e);
    }).ok();

    println!("Received pixel data: camera1={}, x1={}, y1={}, camera2={}, x2={}, y2={}", camera1, x1, y1, camera2, x2, y2);

    // Return dummy measurement
    Ok(24)
}

#[tauri::command]
pub async fn send_pixel(state: State<'_, UdpService>, rover_addr: State<'_, RoverAddress>, camera: String, x: f64, y: f64) -> Result<(), ()> {
    let _socket = state.socket();
    let _target = rover_addr.ip.as_str(); 

    println!("Received pixel from frontend: camera={}, x={}, y={}", camera, x, y);
    // Here you would send the pixel information to the rover
    Ok(())
}

#[tauri::command]
pub async fn select_object(state: State<'_, UdpService>, rover_addr: State<'_, RoverAddress>, object_id: u32) -> Result<(), ()> {
    println!("Object selected with ID: {}", object_id);
    let socket = state.socket();
    let target = rover_addr.ip.as_str(); 

    // Build an envelope with the selected object ID (this is just an example, adjust as needed)
    let envelope = PbEnvelope {
        payload: Some(pb_envelope::Payload::ObjectSelection(
            BasestationObjectSelection { object_id }
        )),
    };
    
    sender::send_envelope(&socket, target, envelope).await.map_err(|e| {
        println!("Failed to send object selection: {}", e);
    }).ok();

    Ok(())
}
