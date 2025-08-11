use std::{env, path::PathBuf};

fn main() {
    // Ensure we rebuild image whenever the kernel binary changes
    println!("cargo:rerun-if-changed=build.rs");
    
    // Force build.rs to regenerate the image *every run*
    println!("cargo:rerun-if-env-changed=FORCE_BOOTIMG");

    // Where cargo puts the compiled kernel ELF
    let target_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()));
    let profile = env::var("PROFILE").unwrap(); // debug/release
    let target = "x86_64-unknown-none";

    // Kernel ELF path (produced by cargo build)
    let kernel_elf = target_dir
        .join(target)
        .join(&profile)
        .join("sis_kernel");

    // Output bootable image paths
    let out_img = target_dir.join("boot-bios.img");
    let final_img = PathBuf::from("out/sis-bios.img");

    // Make sure the kernel ELF exists before we call the builder
    if !kernel_elf.exists() {
        // In the first build run, the ELF may not be present yet; just return.
        // The second cargo invocation (or `cargo build` re-run) will create the image.
        return;
    }

    // Create BIOS disk image with bootloader 0.11.x
    // Try to enable physical memory mapping if the API supports it
    let mut config = bootloader::BootConfig::default();
    config.serial_logging = true;
    config.frame_buffer_logging = false; // avoid VESA path
    
    let mut bios = bootloader::BiosBoot::new(&kernel_elf);
    bios.set_boot_config(&config);
    bios.create_disk_image(&out_img)
        .expect("failed to create BIOS disk image");

    // Copy to final location and ensure parent directory exists
    std::fs::create_dir_all(final_img.parent().unwrap()).unwrap();
    std::fs::copy(&out_img, &final_img)
        .expect("failed to copy BIOS image to final location");

    // Ensure the images exist for qemu.sh
    println!("cargo:warning=Bootable BIOS image written to {}", out_img.display());
    println!("cargo:warning=Final BIOS image copied to {}", final_img.display());
}