use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from("proto");
    let chat_proto = proto_root.join("xai/api/v1/chat.proto");
    let models_proto = proto_root.join("xai/api/v1/models.proto");
    let embed_proto = proto_root.join("xai/api/v1/embed.proto");
    let tokenize_proto = proto_root.join("xai/api/v1/tokenize.proto");
    let auth_proto = proto_root.join("xai/api/v1/auth.proto");
    let sample_proto = proto_root.join("xai/api/v1/sample.proto");
    let image_proto = proto_root.join("xai/api/v1/image.proto");
    let documents_proto = proto_root.join("xai/api/v1/documents.proto");

    println!("cargo:rerun-if-changed={}", chat_proto.display());
    println!("cargo:rerun-if-changed={}", models_proto.display());
    println!("cargo:rerun-if-changed={}", embed_proto.display());
    println!("cargo:rerun-if-changed={}", tokenize_proto.display());
    println!("cargo:rerun-if-changed={}", auth_proto.display());
    println!("cargo:rerun-if-changed={}", sample_proto.display());
    println!("cargo:rerun-if-changed={}", image_proto.display());
    println!("cargo:rerun-if-changed={}", documents_proto.display());

    // Create src/generated directory if it doesn't exist
    std::fs::create_dir_all("src/generated")?;

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir("src/generated")
        .compile_protos(
            &[
                chat_proto,
                models_proto,
                embed_proto,
                tokenize_proto,
                auth_proto,
                sample_proto,
                image_proto,
                documents_proto,
            ],
            &[proto_root],
        )?;

    Ok(())
}
