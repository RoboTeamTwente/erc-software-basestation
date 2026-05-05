// src-tauri/src/map_commands.rs
//
// Tauri commands the Svelte frontend calls via invoke().

use std::path::PathBuf;
use tauri::Manager;

use crate::commands::map_processor::{process_map, MapMeta};

/// Render a 3D map file to a top-down PNG.
///
/// Called from JS:
///   invoke("render_map", { filename: "terrain.obj" })
///
/// The source file must already be in <appDataDir>/maps/.
/// The rendered PNG is written to <appDataDir>/maps/<stem>_preview.png.
/// Returns MapMeta so the frontend can do pixel→world transforms.
#[tauri::command]
pub async fn render_map(
    app: tauri::AppHandle,
    filename: String,
) -> Result<MapMeta, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve appDataDir: {e}"))?;

    let maps_dir = data_dir.join("maps");
    let src = maps_dir.join(&filename);

    if !src.exists() {
        return Err(format!("File not found: {}", src.display()));
    }

    // Build output path: same stem, suffix _preview, always PNG
    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;

    let out = maps_dir.join(format!("{stem}_preview.png"));

    // Run the heavy work on a blocking thread so we don't block the async runtime
    let src_clone = src.clone();
    let out_clone = out.clone();
    let meta = tauri::async_runtime::spawn_blocking(move || {
        process_map(&src_clone, &out_clone, 2048)
    })
    .await
    .map_err(|e| format!("Thread error: {e}"))??;

    Ok(meta)
}

/// Convert a pixel coordinate (from a click on the preview PNG) to
/// real-world (X, Y) in metres.
///
/// Called from JS:
///   invoke("pixel_to_world", { px, py, meta })
#[tauri::command]
pub fn pixel_to_world(px: f64, py: f64, meta: MapMeta) -> (f64, f64) {
    // px, py are display pixels from bottom-left of the displayed image.
    // X goes right, Y goes up. No rotation logic needed — frontend handles that.
    let world_x = px * meta.metres_per_pixel;
    let world_y = py * meta.metres_per_pixel;
    (world_x, world_y)

}