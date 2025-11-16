use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from("proto");
    let chat_proto = proto_root.join("xai/api/v1/chat.proto");

    println!("cargo:rerun-if-changed={}", chat_proto.display());

    // Create src/generated directory if it doesn't exist
    std::fs::create_dir_all("src/generated")?;

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir("src/generated")
        .compile_protos(&[chat_proto], &[proto_root])?;

    Ok(())
}
