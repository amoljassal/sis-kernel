#!/usr/bin/env bash
# Simple QEMU run script for the SIS kernel.  This script assumes
# that you have built the kernel using `cargo build --target
# x86_64-unknown-none --release` and that the resulting binary is
# available at `target/x86_64-unknown-none/release/sis_kernel`.  It
# runs the kernel with 512 MiB of RAM and redirects the serial
# console to your terminal.  Adjust the `-kernel` argument if you
# use the bootloader crate to produce a UEFI image.

qemu-system-x86_64 \
  -machine q35 \
  -m 512M \
  -nographic \
  -serial mon:stdio \
  -kernel target/x86_64-unknown-none/debug/sis_kernel \
  -smp 2 \
  -no-reboot \
  -d int,cpu_reset \
  # Uncomment the lines below to pass through emulated GPUs via VFIO
  # -device vfio-pci,host=0000:01:00.0 \
  # -device vfio-pci,host=0000:02:00.0