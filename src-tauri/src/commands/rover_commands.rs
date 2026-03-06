
#[tauri::command]
pub async fn request_coordinates() -> Result<(i16, i16), i16> {
    println!("Requesting coordinates from rover...");
    // Simulate a delay for the request
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // Return dummy coordinates (latitude, longitude)
    Ok((37, -122))
}
