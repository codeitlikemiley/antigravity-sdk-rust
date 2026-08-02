#![allow(clippy::unwrap_used)]

fn main() {
    let descriptor_path =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("proto_descriptor.bin");

    let mut config = prost_build::Config::new();
    config
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&["proto/localharness.proto"], &["proto/"])
        .unwrap();

    let descriptor_set = std::fs::read(descriptor_path).unwrap();
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)
        .unwrap()
        // Tolerate fields this crate's schema does not know about. A newer
        // harness always adds fields before we regenerate, and pbjson's default
        // is to fail the whole message on the first unknown one — which drops
        // the entire event rather than the field. Note this does NOT cover
        // unknown *enum variants*: pbjson emits `unknown_variant` regardless,
        // so an enum rename still has to be caught by regenerating.
        .ignore_unknown_fields()
        .build(&[".antigravity.localharness"])
        .unwrap();

    // So ClientInfo.language_version reports something real instead of "unknown".
    println!(
        "cargo:rustc-env=RUSTC_VERSION={}",
        std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
    );
}
