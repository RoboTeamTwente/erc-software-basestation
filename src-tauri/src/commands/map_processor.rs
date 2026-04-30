// src-tauri/src/map_processor.rs
//
// Converts 3D map files (.obj, .las, .laz, .e57) into a top-down orthographic
// PNG image coloured by height (Z), and exposes coordinate transforms so the
// frontend can convert a 2D pixel click back to real-world (X, Y) metres.

use image::{ImageBuffer, Rgb, RgbImage};
use std::path::Path;

// ── Public types shared with Tauri commands ──────────────────────────────────

/// Metadata sent to the frontend so it can do pixel → world transforms.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MapMeta {
    /// Width of the generated PNG in pixels
    pub img_width: u32,
    /// Height of the generated PNG in pixels
    pub img_height: u32,
    /// Real-world X of the left edge of the image (metres)
    pub world_x_min: f64,
    /// Real-world Y of the bottom edge of the image (metres)
    pub world_y_min: f64,
    /// How many metres one pixel represents (same in X and Y)
    pub metres_per_pixel: f64,
    /// Source format detected
    pub format: String,
}

/// A raw point in world space, including Z for height colouring.
struct Point3D {
    x: f64,
    y: f64,
    z: f64,
}

// ── Entry point ──────────────────────────────────────────────────────────────

/// Load any supported map file, render a top-down height-coloured PNG,
/// return its path + meta.
///
/// `src_path`  – full path to the source file (.obj / .las / .laz)
/// `out_path`  – where to write the rendered PNG
/// `img_size`  – desired longest edge in pixels (image is scaled to fit)
pub fn process_map(
    src_path: &Path,
    out_path: &Path,
    img_size: u32,
) -> Result<MapMeta, String> {
    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (points, format) = match ext.as_str() {
        "obj"        => (load_obj(src_path)?, "obj".to_string()),
        "las" | "laz" => (load_las(src_path)?, ext.clone()),
        other => {
            return Err(format!(
                "Unsupported format: .{other}. Supported: .obj, .las, .laz"
            ))
        }
    };

    if points.is_empty() {
        return Err("File parsed but contained no points.".to_string());
    }

    render_heightmap(&points, out_path, img_size, format)
}

// ── OBJ loader ───────────────────────────────────────────────────────────────

fn load_obj(path: &Path) -> Result<Vec<Point3D>, String> {
    let (models, _materials) =
        tobj::load_obj(path, &tobj::LoadOptions::default())
            .map_err(|e| format!("OBJ load error: {e}"))?;

    let mut points = Vec::new();
    for model in &models {
        let positions = &model.mesh.positions;
        // tobj stores positions as flat [x0,y0,z0, x1,y1,z1, ...]
        // OBJ convention: Y is up.  Ground plane is X/Z; height is Y.
        for chunk in positions.chunks(3) {
            if let [x, y_up, z_fwd] = chunk {
                points.push(Point3D {
                    x: *x as f64,
                    y: *z_fwd as f64, // world Y  = OBJ Z (forward)
                    z: *y_up as f64,  // height   = OBJ Y (up)
                });
            }
        }
    }
    Ok(points)
}

// ── LAS / LAZ loader ─────────────────────────────────────────────────────────

fn load_las(path: &Path) -> Result<Vec<Point3D>, String> {
    let mut reader =
        las::Reader::from_path(path).map_err(|e| format!("LAS read error: {e}"))?;

    let mut points = Vec::new();
    for wrapped in reader.points() {
        let p = wrapped.map_err(|e| format!("LAS point error: {e}"))?;
        points.push(Point3D { x: p.x, y: p.y, z: p.z });
    }
    Ok(points)
}

// ── Colour map ───────────────────────────────────────────────────────────────

/// Turbo-inspired perceptually-uniform heightmap palette.
/// `t` in [0.0, 1.0] → RGB (u8, u8, u8).
/// Low = deep blue/teal, mid = green/yellow, high = orange/red.
fn height_color(t: f64) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);

    // Five control points: deep-blue → cyan → green → yellow → red
    const STOPS: [[f64; 3]; 5] = [
        [0.10, 0.10, 0.60], // 0.0  – deep blue
        [0.05, 0.60, 0.80], // 0.25 – cyan
        [0.15, 0.75, 0.25], // 0.5  – green
        [0.95, 0.85, 0.05], // 0.75 – yellow
        [0.85, 0.10, 0.05], // 1.0  – red
    ];

    let scaled = t * (STOPS.len() - 1) as f64;
    let lo = scaled.floor() as usize;
    let hi = (lo + 1).min(STOPS.len() - 1);
    let frac = scaled - lo as f64;

    let r = STOPS[lo][0] + frac * (STOPS[hi][0] - STOPS[lo][0]);
    let g = STOPS[lo][1] + frac * (STOPS[hi][1] - STOPS[lo][1]);
    let b = STOPS[lo][2] + frac * (STOPS[hi][2] - STOPS[lo][2]);

    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
}

// ── Renderer ─────────────────────────────────────────────────────────────────

fn render_heightmap(
    points: &[Point3D],
    out_path: &Path,
    img_size: u32,
    format: String,
) -> Result<MapMeta, String> {
    // 1. Compute bounding box (X, Y, Z)
    let (mut x_min, mut x_max) = (f64::MAX, f64::MIN);
    let (mut y_min, mut y_max) = (f64::MAX, f64::MIN);
    let (mut z_min, mut z_max) = (f64::MAX, f64::MIN);

    for p in points {
        if p.x < x_min { x_min = p.x; }
        if p.x > x_max { x_max = p.x; }
        if p.y < y_min { y_min = p.y; }
        if p.y > y_max { y_max = p.y; }
        if p.z < z_min { z_min = p.z; }
        if p.z > z_max { z_max = p.z; }
    }

    let world_w = x_max - x_min;
    let world_h = y_max - y_min;
    let z_range = z_max - z_min;

    if world_w == 0.0 || world_h == 0.0 {
        return Err("All points are collinear — cannot build a 2D map.".to_string());
    }

    // 2. Compute pixel dimensions preserving aspect ratio
    let aspect = world_w / world_h;
    let (img_w, img_h) = if aspect >= 1.0 {
        (img_size, (img_size as f64 / aspect).round() as u32)
    } else {
        ((img_size as f64 * aspect).round() as u32, img_size)
    };

    let metres_per_pixel = world_w / img_w as f64;

    // 3. For each pixel, keep the MAX Z seen (closest to sky = most visible).
    //    We store z values as f64; unset pixels stay at f64::MIN.
    let pixel_count = (img_w * img_h) as usize;
    let mut z_buf: Vec<f64> = vec![f64::MIN; pixel_count];
    // Also track hit count so we can differentiate "no data" from z=0.
    let mut hit: Vec<bool> = vec![false; pixel_count];

    for p in points {
        let px = ((p.x - x_min) / world_w * (img_w - 1) as f64).round() as u32;
        // Flip Y so that world-north is image-top
        let py = img_h - 1 - ((p.y - y_min) / world_h * (img_h - 1) as f64).round() as u32;

        let idx = (py * img_w + px) as usize;
        if idx < pixel_count && p.z > z_buf[idx] {
            z_buf[idx] = p.z;
            hit[idx] = true;
        }
    }

    // 4. Fill unvisited pixels by nearest-neighbour in a quick two-pass scan
    //    (horizontal then vertical) so the image has no black holes.
    //    This is a simple inpainting approximation — good enough for sparse LiDAR.
    fill_gaps(&mut z_buf, &mut hit, img_w, img_h);

    // 5. Paint height → colour
    let mut img: RgbImage = ImageBuffer::new(img_w, img_h);

    for (idx, &z) in z_buf.iter().enumerate() {
        let x = (idx as u32) % img_w;
        let y = (idx as u32) / img_w;

        let color = if !hit[idx] {
            // Truly empty pixel (shouldn't happen after fill, but just in case)
            [20u8, 20, 30]
        } else if z_range < 1e-9 {
            // Flat terrain — use mid-green
            height_color(0.5)
        } else {
            let t = (z - z_min) / z_range;
            height_color(t)
        };

        img.put_pixel(x, y, Rgb(color));
    }

    // 6. Save
    img.save(out_path)
        .map_err(|e| format!("Failed to save PNG: {e}"))?;

    Ok(MapMeta {
        img_width: img_w,
        img_height: img_h,
        world_x_min: x_min,
        world_y_min: y_min,
        metres_per_pixel,
        format,
    })
}

// ── Gap filling ───────────────────────────────────────────────────────────────

/// Simple scanline inpainting: propagate the last known Z left→right, then
/// right→left, then top→bottom, then bottom→top.  Four passes give reasonable
/// fill for sparse point clouds without any expensive neighbour search.
fn fill_gaps(z_buf: &mut Vec<f64>, hit: &mut Vec<bool>, w: u32, h: u32) {
    // Horizontal passes
    for row in 0..h {
        // Left → right
        let mut last_z = f64::MIN;
        for col in 0..w {
            let idx = (row * w + col) as usize;
            if hit[idx] {
                last_z = z_buf[idx];
            } else if last_z != f64::MIN {
                z_buf[idx] = last_z;
                hit[idx] = true;
            }
        }
        // Right → left
        let mut last_z = f64::MIN;
        for col in (0..w).rev() {
            let idx = (row * w + col) as usize;
            if hit[idx] {
                last_z = z_buf[idx];
            } else if last_z != f64::MIN {
                z_buf[idx] = last_z;
                hit[idx] = true;
            }
        }
    }
    // Vertical passes
    for col in 0..w {
        // Top → bottom
        let mut last_z = f64::MIN;
        for row in 0..h {
            let idx = (row * w + col) as usize;
            if hit[idx] {
                last_z = z_buf[idx];
            } else if last_z != f64::MIN {
                z_buf[idx] = last_z;
                hit[idx] = true;
            }
        }
        // Bottom → top
        let mut last_z = f64::MIN;
        for row in (0..h).rev() {
            let idx = (row * w + col) as usize;
            if hit[idx] {
                last_z = z_buf[idx];
            } else if last_z != f64::MIN {
                z_buf[idx] = last_z;
                hit[idx] = true;
            }
        }
    }
}