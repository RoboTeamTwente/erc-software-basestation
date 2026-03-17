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

fn main() -> Result<()> {
    tauri_build::build();

    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    std::env::set_var("PROTOC", protoc);

    let proto_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proto");

    let mut protos = Vec::new();

    println!("Scanning from: {:?}", proto_root);

    if !proto_root.exists() {
        panic!("Proto root does not exist");
    }

    collect_protos(&proto_root, &mut protos)?;

    println!("Found proto files:");
    for p in &protos {
        println!("{:?}", p);
    }

    if protos.is_empty() {
        panic!("No valid proto files found under components/");
    }

    let mut config = prost_build::Config::new();
 
    // Derive serde::Serialize on every generated message struct and enum.
    // This means SensorBoardImuInfo (and all others) can be emitted directly
    // via tauri without any manual wrapper struct.
    config.type_attribute(".", "#[derive(serde::Serialize)]");
 
    config
        .compile_protos(
            &protos,
            &[proto_root.to_str().expect("Invalid proto root")],
        )
        .expect("Failed to compile proto files");
    Ok(())
}