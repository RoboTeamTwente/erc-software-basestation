fn main() {
    tauri_build::build();

    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    std::env::set_var("PROTOC", protoc);

    prost_build::compile_protos(
        &["proto/message.proto"],
        &["proto"],
    )
    .unwrap();
}