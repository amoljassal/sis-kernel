#!/usr/bin/env bash
# Unified test runner for SIS kernel: BIOS/UEFI, macOS/Linux, serial logging, debug-exit mapping.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$HERE/.."
OUT="$ROOT/out"
mkdir -p "$OUT"

export TEST="${TEST:-}"            # e.g. USR_INIT, PFM_NX_EXEC, SMP_2, LAPIC_TIMER, …
export BOOT="${BOOT:-auto}"        # auto|bios|uefi
export SMP="${SMP:-1}"             # number of CPUs (SMP_2 sets this to 2 in CI)
export MEM="${MEM:-512M}"
export FEATURES="${FEATURES:-}"    # optional; auto-filled by _test_flags.sh
export TIMEOUT="${TIMEOUT:-30}"    # configurable timeout in seconds

# 1) Map TEST -> RUSTFLAGS/FEATURES (defaults)
source "$HERE/_test_flags.sh"
echo "[harness] TEST=$TEST FEATURES='$FEATURES' RUSTFLAGS='$RUSTFLAGS'"

# 2) Force build.rs to regenerate the image *every run*
export FORCE_BOOTIMG="$(date +%s)"
echo "[harness] FORCE_BOOTIMG=$FORCE_BOOTIMG (ensures fresh image)"

# Clean just the bootloader & previous image to avoid stale artifacts
cargo clean -p bootloader &>/dev/null || true
rm -f "$OUT/sis-bios.img" &>/dev/null || true

# Build kernel (debug, headless) and create bootable image using build.rs
export RUSTFLAGS
echo "[harness] cargo build with fresh image generation…"
cargo +nightly build -Z build-std=core,alloc --target x86_64-unknown-none --features "$FEATURES"

# Sanity: print the image we will boot and its hash
IMG="$OUT/sis-bios.img"
if [[ ! -f "$IMG" ]]; then
  echo "[harness] ERROR: expected image not found: $IMG" >&2
  exit 97
fi

echo "[harness] using fresh image: $IMG"
echo "[harness] image sha256: $(sha256sum "$IMG" | awk '{print $1}')"

# Bonus: ensure it's the *new* kernel by checking for old fatal string
if strings "$IMG" 2>/dev/null | grep -q "\[mem\]\[FATAL\] No physical memory mapping"; then
  echo "[harness] WARNING: image still contains old fatal string; rebuild likely stale" >&2
fi

KERNEL="$ROOT/target/x86_64-unknown-none/debug/sis_kernel"
BIOS_IMAGE="$IMG"

# 3) Force BIOS boot for new bootloader setup
BOOT="bios"
echo "[harness] BOOT=$BOOT (forced BIOS for bootloader 0.11.x)"

# Enhanced diagnostics
SERIAL_LOG="$OUT/qemu-serial.log"
rm -f "$SERIAL_LOG"

# Simple timer (ms)
_now_ms() { date +%s%3N 2>/dev/null || python3 - <<'PY'
import time; print(int(time.time()*1000))
PY
}
START_MS=$(_now_ms)

# 4) QEMU common flags - Q35 required for IOMMU/VFIO
COMMON=(-machine q35,kernel-irqchip=split -m 1024 -cpu qemu64 -smp 1 \
        -serial file:"$SERIAL_LOG" -display none -no-reboot -no-shutdown \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04)

# Add Intel IOMMU device for IOMMU feature testing
if [[ "$FEATURES" == *"iommu"* ]]; then
    COMMON+=(-device intel-iommu,intremap=on)
    echo "[harness] Intel IOMMU enabled for testing"
fi

# Add e1000e device for VFIO feature testing (MSI-capable) - force 00:02.0
if [[ "$FEATURES" == *"vfio"* ]]; then
    COMMON+=(-nic none -device e1000e,netdev=n1,mac=52:54:00:12:34:56,bus=pcie.0,addr=0x2 -netdev user,id=n1)
    echo "[harness] e1000e device pinned to 00:02.0"
fi

# 5) Boot mode specific flags
if [[ "$BOOT" == "uefi" ]]; then
  source "$HERE/_ovmf_paths.sh"
  if [[ -z "${OVMF_CODE}" || -z "${OVMF_VARS}" ]]; then
    echo "[harness] OVMF not found; set OVMF_CODE/OVMF_VARS or install OVMF. Falling back to BIOS."
    BOOT="bios"
  fi
fi

QEMU_BIN="${QEMU_BIN:-qemu-system-x86_64}"

echo "[harness] timeout set to ${TIMEOUT}s"

# Build QEMU command for BIOS boot with disk image (simple IDE interface)
echo "[harness] launching QEMU (BIOS)…"
if [[ -f "$BIOS_IMAGE" ]]; then
    echo "[harness] using BIOS disk image: $BIOS_IMAGE"
    QEMU_CMD=(timeout -k 2 "${TIMEOUT}s" "$QEMU_BIN" "${COMMON[@]}" -drive file="$BIOS_IMAGE",format=raw,if=ide,index=0,media=disk)
else
    echo "[harness] ERROR: BIOS disk image not found at $BIOS_IMAGE"
    echo "[harness] Build may have failed or build.rs didn't create the image"
    exit 1
fi

echo "[qemu.sh] QEMU command:"
printf ' %q' "${QEMU_CMD[@]}"; echo
echo "[qemu.sh] Launching..."

# Run QEMU and capture exit
set +e
"${QEMU_CMD[@]}"
RET=$?
set -e

END_MS=$(_now_ms)
ELAPSED=$(( END_MS - START_MS ))

# Basic classification helpers
banner_ok=0
golden_ok=0
grep -q "=== SIS KERNEL ENTRY ===" "$SERIAL_LOG" && banner_ok=1 || true
grep -q "\[PASS:" "$SERIAL_LOG" && golden_ok=1 || true

classification="unknown"
if [[ $banner_ok -eq 0 ]]; then
  classification="pre-entry"
elif [[ $golden_ok -eq 0 ]]; then
  classification="post-entry-pre-exit"
else
  classification="golden-mismatch"
fi

echo "[qemu.sh] timing: {\"test\":\"${TEST:-unknown}\",\"elapsed_ms\":$ELAPSED}" | tee -a "$SERIAL_LOG"

if [[ $RET -ne 0 ]]; then
  echo "[qemu.sh] QEMU exit code: $RET"
  echo "[qemu.sh] failure-classification: $classification"
  echo "[qemu.sh] tail(200) of serial log:"
  tail -n 200 "$SERIAL_LOG" || true
  exit $RET
fi