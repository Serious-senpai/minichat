use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut protos = Vec::new();
    for entry in fs::read_dir("../../protobuf")?.flatten() {
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "proto" {
                protos.push(path.to_str().unwrap().to_string());
            }
        }
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&protos, &["../../protobuf"])?;

    Ok(())
}
