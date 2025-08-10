// build.rs - Simple build script for bootloader 0.10.x
// No custom build needed - bootimage handles everything
fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");
}