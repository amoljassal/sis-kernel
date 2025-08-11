use std::{env, path::PathBuf};

fn main() {
    // Ensure we rebuild image whenever the kernel binary changes
    println!("cargo:rerun-if-changed=build.rs");

    // Where cargo puts the compiled kernel ELF
    let target_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()));
    let profile = env::var("PROFILE").unwrap(); // debug/release
    let target = "x86_64-unknown-none";

    // Kernel ELF path (produced by cargo build)
    let kernel_elf = target_dir
        .join(target)
        .join(&profile)
        .join("sis_kernel");

    // Output bootable image path
    let out_img = target_dir.join("boot-bios.img");

    // Make sure the kernel ELF exists before we call the builder
    if !kernel_elf.exists() {
        // In the first build run, the ELF may not be present yet; just return.
        // The second cargo invocation (or `cargo build` re-run) will create the image.
        return;
    }

    // Create BIOS disk image with correct 0.11.x config
    let mut config = bootloader::BootConfig::default();
    config.serial_logging = true;
    
    // Configure frame buffer
    config.frame_buffer.minimum_framebuffer_height = Some(480);
    config.frame_buffer.minimum_framebuffer_width = Some(640);
    
    let mut bios = bootloader::BiosBoot::new(&kernel_elf);
    bios.set_boot_config(&config);
    bios.create_disk_image(&out_img)
        .expect("failed to create BIOS disk image");

    // Ensure the `target/boot-bios.img` exists for qemu.sh
    println!("cargo:warning=Bootable BIOS image written to {}", out_img.display());
}