#!/usr/bin/env rust-script
//! Industry-grade image creation bypass for build.rs bottleneck
//! Usage: ./create-image.rs <kernel-elf> <output-image>

use std::{env, path::PathBuf, process::Command};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <kernel-elf> <output-image>", args[0]);
        std::process::exit(1);
    }
    
    let kernel_elf = PathBuf::from(&args[1]);
    let output_image = PathBuf::from(&args[2]);
    
    if !kernel_elf.exists() {
        eprintln!("ERROR: Kernel ELF not found: {}", kernel_elf.display());
        std::process::exit(1);
    }
    
    println!("Creating boot image: {} -> {}", kernel_elf.display(), output_image.display());
    
    // Create output directory
    if let Some(parent) = output_image.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    
    // Use objcopy to create raw binary (simplest approach)
    let output = Command::new("objcopy")
        .args(&[
            "-O", "binary",
            kernel_elf.to_str().unwrap(),
            output_image.to_str().unwrap()
        ])
        .output();
        
    match output {
        Ok(result) if result.status.success() => {
            println!("Boot image created successfully");
        }
        Ok(result) => {
            eprintln!("objcopy failed: {}", String::from_utf8_lossy(&result.stderr));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run objcopy: {}", e);
            eprintln!("Falling back to file copy (for testing)");
            std::fs::copy(&kernel_elf, &output_image).unwrap();
        }
    }
}