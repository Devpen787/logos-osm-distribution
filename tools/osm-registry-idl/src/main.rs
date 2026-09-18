use std::path::PathBuf;

fn main() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../methods/guest/src/bin/osm_registry.rs");

    let idl = spel_framework_core::idl_gen::generate_idl_from_file(&source)
        .unwrap_or_else(|err| panic!("failed to generate OSM registry IDL: {err}"));

    println!(
        "{}",
        serde_json::to_string_pretty(&idl).expect("serialize generated IDL")
    );
}
