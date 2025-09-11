#!/usr/bin/env bash
set -euo pipefail

# Multi-core ARM64 QEMU test script for SIS kernel
# Tests SMP functionality with multiple CPU cores

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
cd "$ROOT_DIR"

# Configuration
NUM_CPUS="${NUM_CPUS:-4}"
MEMORY="${MEMORY:-512M}"
DEBUG="${DEBUG:-}"
GDB="${GDB:-}"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║           SIS Kernel ARM64 Multi-Core Boot Test           ║"
echo "╠════════════════════════════════════════════════════════════╣"
echo "║ CPUs: $NUM_CPUS cores                                              ║"
echo "║ Memory: $MEMORY                                            ║"
echo "║ GICv3: Enabled                                             ║"
echo "║ PMU: Performance monitoring enabled                        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo

# Build the kernel with SMP features
echo "[*] Building kernel with SMP support..."
export RUSTFLAGS="-C link-arg=-T$ROOT_DIR/crates/kernel/src/arch/aarch64/aarch64-qemu.ld"
cargo +nightly build -p sis_kernel \
    -Z build-std=core,alloc \
    --target aarch64-unknown-none \
    --features bringup,smp \
    2>&1 | tail -20

KERNEL_ELF="$ROOT_DIR/target/aarch64-unknown-none/debug/sis_kernel"
if [[ ! -f "$KERNEL_ELF" ]]; then
    echo "[!] Kernel ELF not found: $KERNEL_ELF" >&2
    exit 1
fi

# Build UEFI bootloader
echo "[*] Building UEFI bootloader..."
cargo build -p uefi-boot --release --target aarch64-unknown-uefi 2>&1 | tail -5

UEFI_APP="$ROOT_DIR/target/aarch64-unknown-uefi/release/uefi-boot.efi"
if [[ ! -f "$UEFI_APP" ]]; then
    echo "[!] UEFI app not found: $UEFI_APP" >&2
    exit 1
fi

# Prepare ESP
ESP_DIR="$SCRIPT_DIR/esp"
rm -rf "$ESP_DIR"
mkdir -p "$ESP_DIR/EFI/BOOT" "$ESP_DIR/EFI/SIS"
cp "$UEFI_APP" "$ESP_DIR/EFI/BOOT/BOOTAA64.EFI"
cp "$KERNEL_ELF" "$ESP_DIR/EFI/SIS/KERNEL.ELF"

# Check for EDK2 firmware
FIRMWARE="/opt/homebrew/share/qemu/edk2-aarch64-code.fd"
if [[ ! -f "$FIRMWARE" ]]; then
    # Try alternative locations
    if [[ -f "/usr/share/qemu/edk2-aarch64-code.fd" ]]; then
        FIRMWARE="/usr/share/qemu/edk2-aarch64-code.fd"
    else
        echo "[!] EDK2 firmware not found. Install with:"
        echo "    brew install qemu"
        echo "    or apt-get install qemu-efi-aarch64"
        exit 1
    fi
fi

# Build QEMU command
QEMU_CMD=(
    qemu-system-aarch64
    -M virt,gic-version=3,highmem=on
    -cpu cortex-a72
    -smp "$NUM_CPUS"
    -m "$MEMORY"
    -nographic
    -serial mon:stdio
    -bios "$FIRMWARE"
    -drive if=none,id=esp,format=raw,file=fat:rw:"$ESP_DIR"
    -device virtio-blk-pci,drive=esp
    -device virtio-rng-pci
    -device virtio-serial-pci,id=serial0
    -no-reboot
)

# Add debug options if requested
if [[ "$DEBUG" != "" ]]; then
    QEMU_CMD+=(
        -d int,cpu_reset,guest_errors
        -D /tmp/qemu-smp-debug.log
    )
    echo "[*] Debug mode enabled: logging to /tmp/qemu-smp-debug.log"
fi

# Add GDB server if requested
if [[ "$GDB" != "" ]]; then
    QEMU_CMD+=(-s -S)
    echo "[*] GDB server enabled on port 1234"
    echo "[*] Connect with: gdb-multiarch $KERNEL_ELF"
    echo "[*]              (gdb) target remote :1234"
fi

# Add performance monitoring (try to enable KVM if available)
# Note: KVM is not available on macOS, so this will be ignored
# QEMU_CMD+=(-enable-kvm)

echo "[*] Launching QEMU with $NUM_CPUS CPU cores..."
echo "[*] Command: ${QEMU_CMD[@]}"
echo
echo "════════════════════════════════════════════════════════════"
echo

# Run QEMU
"${QEMU_CMD[@]}" || true

echo
echo "════════════════════════════════════════════════════════════"
echo "[*] QEMU terminated"

# Show debug log if it exists
if [[ -f /tmp/qemu-smp-debug.log ]]; then
    echo "[*] Debug log tail:"
    tail -20 /tmp/qemu-smp-debug.log
fi
