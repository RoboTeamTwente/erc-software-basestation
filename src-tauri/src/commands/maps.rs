use::std::sync::Mutex;

pub struct MapState {
    pub displayed_map_name: Mutex<String>,
}

#[tauri::command]
pub async fn selected_map_to_backend(state: tauri::State<'_, MapState>, file_name: String,) -> Result<(), String> {
    let mut name = state.displayed_map_name.lock().unwrap();
    *name = file_name;
    println!("Selected map: {}", &name);
    Ok(())
}

#[tauri::command]
pub async fn selected_map_from_backend(state: tauri::State<'_, MapState>,) -> Result<String, String> {
    let name = state.displayed_map_name.lock().unwrap();
    Ok(name.clone())
}