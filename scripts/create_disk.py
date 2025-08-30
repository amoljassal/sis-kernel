#!/usr/bin/env python3
"""
Simple helper to create a bootable disk image using the bootloader crate.
This is a temporary solution until we set up proper bootloader integration.
"""

import subprocess
import sys
import os
from pathlib import Path

def create_bootable_image():
    root = Path(__file__).parent
    kernel_path = root / "target" / "x86_64-unknown-none" / "debug" / "sis_kernel"
    out_dir = root / "out"
    out_dir.mkdir(exist_ok=True)
    
    if not kernel_path.exists():
        print(f"Kernel not found at {kernel_path}")
        print("Run 'cargo build' first")
        return False
    
    # For now, just copy the kernel as a placeholder
    # TODO: Use proper bootloader crate integration
    bootable_image = out_dir / "bios.img"
    
    print(f"Creating bootable image at {bootable_image}")
    print("Note: This is a placeholder - proper bootloader integration needed")
    
    # This is just a stub - we need proper bootloader integration
    return False

if __name__ == "__main__":
    success = create_bootable_image()
    sys.exit(0 if success else 1)