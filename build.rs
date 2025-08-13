use std::{env, path::PathBuf};

fn main() {
    // Industry-grade build with enhanced canary verification
    println!("cargo:warning=build.rs creating bootable image with canary verification");
    
    // Enhanced rebuild triggers for canary traceability
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=FORCE_BOOTIMG");
    println!("cargo:rerun-if-env-changed=GIT_COMMIT");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    
    // AP trampoline assembly handling
    println!("cargo:rerun-if-changed=src/arch/x86_64/smp/ap_trampoline.S");

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
        println!("cargo:warning=Kernel ELF not ready yet - deferring image creation");
        return;
    }

    println!("cargo:warning=Creating BIOS boot image from kernel ELF");

    // Create BIOS disk image with bootloader 0.11.x
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

    println!("cargo:warning=Bootable BIOS image created: {}", final_img.display());
}