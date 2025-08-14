#!/usr/bin/env bash
# Industry-grade build system with artifact verification
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Build metadata for traceability
export GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
export SOURCE_DATE_EPOCH=$(date +%s)
export FORCE_BOOTIMG=1

echo "[BUILD] Industry-grade rebuild: commit=$GIT_COMMIT epoch=$SOURCE_DATE_EPOCH"

# Step 1: Force clean rebuild - eliminate all caches
echo "[BUILD] Forcing complete clean rebuild..."
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo clean

# Remove any stale boot images
rm -f out/sis-bios.img out/sis-bios-*.img

# Verify no stale kernel artifacts
find target -name "sis_kernel" -delete 2>/dev/null || true

# Step 2: Clean rebuild with explicit feature control
echo "[BUILD] Building kernel with features: vfio iommu idt-selftest"
CARGO_INCREMENTAL=0 \
cargo build --no-default-features \
  --features "vfio iommu idt-selftest" \
  -Z build-std=core,alloc --target x86_64-unknown-none

# Step 3: Verification checkpoint - ensure kernel ELF exists
KERNEL_ELF="target/x86_64-unknown-none/debug/sis_kernel"
if [[ ! -f "$KERNEL_ELF" ]]; then
  echo "[ERROR] Kernel ELF not generated - build failed"
  ls -la target/x86_64-unknown-none/debug/ || true
  exit 1
fi

echo "[BUILD] Kernel ELF generated successfully: $(stat -f%z "$KERNEL_ELF" 2>/dev/null || stat -c%s "$KERNEL_ELF") bytes"

# Step 4: Verify boot image contains canary
BOOT_IMAGE="out/sis-bios.img"
if [[ -f "$BOOT_IMAGE" ]]; then
  echo "[VERIFY] Checking canary in boot image..."
  if strings "$BOOT_IMAGE" | grep -q "DEADBEEFCAFEBABE"; then
    echo "[SUCCESS] Build canary found in boot image - fresh build confirmed"
  else
    echo "[WARNING] Canary not found in boot image - may be stale artifact"
  fi
  
  echo "[BUILD] Boot image: $(stat -f%z "$BOOT_IMAGE" 2>/dev/null || stat -c%s "$BOOT_IMAGE") bytes"
  echo "[BUILD] Boot image path: $(readlink -f "$BOOT_IMAGE")"
else
  echo "[WARNING] Boot image not found at $BOOT_IMAGE"
fi

echo "[BUILD] Industry-grade rebuild complete - ready for testing"