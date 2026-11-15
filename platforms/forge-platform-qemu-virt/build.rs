mod build_config;

use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rustc-link-arg=-T{}/linker.ld", root.display());
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=build_config.rs");
    println!("cargo:rerun-if-env-changed=FORGE_CONFIG");
    let workspace = root.join("../..");
    let selected = env::var_os("FORGE_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("forge.cfg"));
    // PathBuf::join keeps an absolute selected path. Relative paths use the
    // workspace root, independent of the directory where Cargo was started.
    let path = workspace.join(selected);
    println!("cargo:rerun-if-changed={}", path.display());
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read Forge config {}: {error}", path.display()));
    let config = build_config::BuildConfig::parse(&text)
        .unwrap_or_else(|error| panic!("invalid Forge config {}: {error}", path.display()));
    let generated = format!(
        "pub const BOOT_FRAME_SIZE: usize = {};\n",
        config.boot_frame_size
    );
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("forge_config.rs");
    fs::write(output, generated).expect("cannot write generated Forge configuration");
}
