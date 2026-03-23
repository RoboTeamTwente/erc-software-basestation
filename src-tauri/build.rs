use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};

fn collect_protos(dir: &Path, protos: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_protos(&path, protos)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("proto") {
            // Accept proto only if it is inside a "components" subtree
            if path
                .ancestors()
                .any(|ancestor| ancestor.file_name().and_then(|n| n.to_str()) == Some("components"))
            {
                protos.push(path);
            }
        }
    }

    Ok(())
}

/// Copy a .proto file into `tmp_dir`, preserving its path relative to
/// `proto_root`, and inject `package packets;` as the second line.
/// Returns the path of the patched copy.
fn copy_and_patch_proto(
    original: &Path,
    proto_root: &Path,
    tmp_dir: &Path,
) -> std::io::Result<PathBuf> {
    let rel = original
        .strip_prefix(proto_root)
        .expect("proto file must be under proto_root");
 
    let dest = tmp_dir.join(rel);
 
    // Ensure parent directories exist inside the temp tree
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
 
    let source = fs::read_to_string(original)?;
 
    // Split into lines, inject `package packets;` after the first line
    let mut lines: Vec<&str> = source.lines().collect();
    let inject = "package packets;";
 
    // Only inject if not already present (idempotent, safe to re-run)
    if !lines.iter().any(|l| l.trim() == inject) {
        let insert_at = if lines.is_empty() { 0 } else { 1 };
        lines.insert(insert_at, inject);
    }
 
    // Re-join with newlines; preserve a trailing newline if the original had one
    let mut patched = lines.join("\n");
    if source.ends_with('\n') {
        patched.push('\n');
    }
 
    fs::write(&dest, patched)?;
    Ok(dest)
}


fn main() -> Result<()> {
    tauri_build::build();

    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    std::env::set_var("PROTOC", protoc);

    let proto_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proto");

    println!("Scanning from: {:?}", proto_root);
    if !proto_root.exists() {
        panic!("Proto root does not exist");
    }

    // Collect original .proto files
    let mut protos = Vec::new();
    collect_protos(&proto_root, &mut protos)?;

    println!("Found proto files:");
    for p in &protos {
        println!("{:?}", p);
    }

    if protos.is_empty() {
        panic!("No valid proto files found under components/");
    }


    // Create a temp directory that lives for the duration of this build script.
    // It is placed next to OUT_DIR so Cargo controls its lifetime
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    let tmp_proto_dir = out_dir.join("patched_protos");
    if tmp_proto_dir.exists() {
        fs::remove_dir_all(&tmp_proto_dir)?;
    }
    fs::create_dir_all(&tmp_proto_dir)?;
 
    // Produce patched copies
    let mut patched_protos = Vec::new();
    for original in &protos {
        let patched = copy_and_patch_proto(original, &proto_root, &tmp_proto_dir)?;
        println!("Patched -> {:?}", patched);
        patched_protos.push(patched);
    }

 
    // Derive serde::Serialize on every generated message struct and enum.
    // This means SensorBoardImuInfo (and all others) can be emitted directly
    // via tauri without any manual wrapper struct.
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize)]");
 
    config
        .compile_protos(
            &patched_protos,
            &[tmp_proto_dir.to_str().expect("Invalid tmp proto dir")],
        )
        .expect("Failed to compile proto files");
    Ok(())
}