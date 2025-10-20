use std::fs;
use std::io::Write;

use miden_node_proto_build::rpc_api_descriptor;
use miden_note_transport_proto_build::mnt_api_descriptor;
use miette::IntoDiagnostic;

const STD_PROTO_OUT_DIR: &str = "src/rpc/generated/std";
const NO_STD_PROTO_OUT_DIR: &str = "src/rpc/generated/nostd";
const NOTE_TRANSPORT_STD_PROTO_OUT_DIR: &str = "src/note_transport/generated/std";
const NOTE_TRANSPORT_NO_STD_PROTO_OUT_DIR: &str = "src/note_transport/generated/nostd";

/// Defines whether the build script should generate files in `/src`.
/// The docs.rs build pipeline has a read-only filesystem, so we have to avoid writing to `src`,
/// otherwise the docs will fail to build there. Note that writing to `OUT_DIR` is fine.
const CODEGEN: bool = option_env!("CODEGEN").is_some();

fn main() -> miette::Result<()> {
    println!("cargo::rerun-if-env-changed=CODEGEN");
    if !CODEGEN {
        return Ok(());
    }

    compile_tonic_client_proto()?;
    compile_tonic_note_transport_proto()?;
    replace_no_std_types(NO_STD_PROTO_OUT_DIR.to_string() + "/rpc.rs");
    replace_no_std_types(NO_STD_PROTO_OUT_DIR.to_string() + "/rpc_store.rs");
    replace_no_std_types(NO_STD_PROTO_OUT_DIR.to_string() + "/block_producer.rs");
    replace_no_std_types(
        NOTE_TRANSPORT_NO_STD_PROTO_OUT_DIR.to_string() + "/miden_note_transport.rs",
    );

    Ok(())
}

// NOTE TRANSPORT CLIENT PROTO CODEGEN
// ===============================================================================================

/// Generates the Rust protobuf bindings for the Note Transport client.
fn compile_tonic_note_transport_proto() -> miette::Result<()> {
    let file_descriptors = mnt_api_descriptor();

    let mut prost_config = prost_build::Config::new();
    prost_config.skip_debug(["AccountId", "Digest"]);

    let mut web_tonic_prost_config = prost_build::Config::new();
    web_tonic_prost_config.skip_debug(["AccountId", "Digest"]);

    // Generate the header of the user facing server from its proto file
    tonic_build::configure()
        .build_transport(false)
        .build_server(false)
        .out_dir(NOTE_TRANSPORT_NO_STD_PROTO_OUT_DIR)
        .compile_fds_with_config(web_tonic_prost_config, file_descriptors.clone())
        .into_diagnostic()?;

    tonic_build::configure()
        .build_server(false)
        .out_dir(NOTE_TRANSPORT_STD_PROTO_OUT_DIR)
        .compile_fds_with_config(prost_config, file_descriptors)
        .into_diagnostic()?;

    Ok(())
}

// NODE RPC CLIENT PROTO CODEGEN
// ===============================================================================================

/// Generates the Rust protobuf bindings for the RPC client.
fn compile_tonic_client_proto() -> miette::Result<()> {
    let file_descriptors = rpc_api_descriptor();

    let mut prost_config = prost_build::Config::new();
    prost_config.skip_debug(["AccountId", "Digest"]);

    let mut web_tonic_prost_config = prost_build::Config::new();
    web_tonic_prost_config.skip_debug(["AccountId", "Digest"]);

    // Generate the header of the user facing server from its proto file
    tonic_build::configure()
        .build_transport(false)
        .build_server(false)
        .out_dir(NO_STD_PROTO_OUT_DIR)
        .compile_fds_with_config(web_tonic_prost_config, file_descriptors.clone())
        .into_diagnostic()?;

    tonic_build::configure()
        .build_server(false)
        .out_dir(STD_PROTO_OUT_DIR)
        .compile_fds_with_config(prost_config, file_descriptors)
        .into_diagnostic()?;

    Ok(())
}

/// This function replaces all `std::result` with `core::result` in the generated "rpc.rs" file
/// for the web tonic client. This is needed as `tonic_build` doesn't generate `no_std` compatible
/// files and we want to build wasm without `std`.
fn replace_no_std_types(path: String) {
    let file_str = fs::read_to_string(&path).unwrap();
    let new_file_str = file_str
        .replace("std::result", "core::result")
        .replace("std::marker", "core::marker");

    let mut f = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    f.write_all(new_file_str.as_bytes()).unwrap();
}
