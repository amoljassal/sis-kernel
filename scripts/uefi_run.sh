#!/usr/bin/env bash
set -euo pipefail

# Build the UEFI boot app and run it under QEMU with edk2-aarch64 firmware.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
cd "$ROOT_DIR"
ESP_DIR="$SCRIPT_DIR/esp"
EFI_BOOT_DIR="$ESP_DIR/EFI/BOOT"
EFI_SIS_DIR="$ESP_DIR/EFI/SIS"

echo "[*] Building UEFI app (aarch64-unknown-uefi)..."
rustup target add aarch64-unknown-uefi >/dev/null 2>&1 || true
cargo build -p uefi-boot --release --target aarch64-unknown-uefi

UEFI_APP="$ROOT_DIR/target/aarch64-unknown-uefi/release/uefi-boot.efi"
if [[ ! -f "$UEFI_APP" ]]; then
  echo "[!] UEFI app not found: $UEFI_APP" >&2
  exit 1
fi

echo "[*] Preparing ESP at $ESP_DIR ..."
rm -rf "$ESP_DIR"
mkdir -p "$EFI_BOOT_DIR" "$EFI_SIS_DIR"
cp "$UEFI_APP" "$EFI_BOOT_DIR/BOOTAA64.EFI"

echo "[*] Building kernel (aarch64-unknown-none)..."
rustup target add aarch64-unknown-none >/dev/null 2>&1 || true
export RUSTFLAGS="-C link-arg=-T$ROOT_DIR/src/arch/aarch64/aarch64-qemu.ld"
if [[ "${BRINGUP:-}" != "" ]]; then
  echo "[*] Enabling bringup feature (STACK/VECTORS/MMU)"
  cargo +nightly build -p sis_kernel -Z build-std=core,alloc --target aarch64-unknown-none --features bringup
else
  cargo +nightly build -p sis_kernel -Z build-std=core,alloc --target aarch64-unknown-none
fi
KERNEL_ELF="$ROOT_DIR/target/aarch64-unknown-none/debug/sis_kernel"
if [[ ! -f "$KERNEL_ELF" ]]; then
  echo "[!] Kernel ELF not found: $KERNEL_ELF" >&2
  exit 1
fi
cp "$KERNEL_ELF" "$EFI_SIS_DIR/KERNEL.ELF"

echo "[*] ESP contents:"
ls -l "$EFI_BOOT_DIR" "$EFI_SIS_DIR"
if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$EFI_BOOT_DIR/BOOTAA64.EFI" "$EFI_SIS_DIR/KERNEL.ELF" | sed 's/^/  /'
fi

FIRMWARE="/opt/homebrew/share/qemu/edk2-aarch64-code.fd"
if [[ ! -f "$FIRMWARE" ]]; then
  echo "[!] EDK2 firmware not found at $FIRMWARE"
  echo "    Install with: brew install qemu (or edk2-aarch64)"
  exit 1
fi

echo "[*] Launching QEMU (UEFI) with GICv3, highmem, and VirtIO devices ..."
# Add debugging for VirtIO discovery if DEBUG env var is set
DEBUG_FLAGS=""
if [[ "${DEBUG:-}" != "" ]]; then
  DEBUG_FLAGS="-d int,mmio -D /tmp/qemu-debug.log"
  echo "[*] Debug mode enabled: logging to /tmp/qemu-debug.log"
fi

qemu-system-aarch64 \
  -M virt,gic-version=3,highmem=on,secure=off \
  -cpu cortex-a72 \
  -m 512M \
  -nographic \
  -serial file:/tmp/sis-kernel.log \
  -bios "$FIRMWARE" \
  -drive if=none,id=esp,format=raw,file=fat:rw:"$ESP_DIR" \
  -device virtio-blk-pci,drive=esp,id=boot-disk,disable-legacy=on \
  -device virtio-rng-pci,id=rng0,disable-legacy=on \
  -device virtio-net-pci,netdev=net0,id=net0,disable-legacy=on \
  -netdev user,id=net0 \
  -rtc base=utc \
  -no-reboot \
  -smp 2 \
  $DEBUG_FLAGS
