
#[tauri::command]
pub async fn request_coordinates() -> Result<(i16, i16), i16> {
    println!("Requesting coordinates from rover...");
    // Simulate a delay for the request
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // Return dummy coordinates (latitude, longitude)
    Ok((37, -122))
}

#[tauri::command]
pub async fn request_weight() -> Result<i16, i16> {
    println!("Requesting rock weight from rover...");
    // Simulate a delay for the request
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // Return dummy weight
    Ok(222)
}

#[tauri::command]
pub async fn request_measurement(camera1: String, x1: f64, y1: f64, camera2: String, x2: f64, y2: f64) -> Result<i16, i16> {
    println!("Requesting measurement from rover...");
    // Simulate a delay for the request
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    println!("Received pixel data: camera1={}, x1={}, y1={}, camera2={}, x2={}, y2={}", camera1, x1, y1, camera2, x2, y2);

    // Return dummy measurement
    Ok(17)
}

#[tauri::command]
pub async fn send_pixel(camera: String, x: f64, y: f64) -> Result<(), ()> {
    println!("Received pixel from frontend: camera={}, x={}, y={}", camera, x, y);
    // Here you would send the pixel information to the rover
    Ok(())
}
