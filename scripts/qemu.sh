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

# 2) Build kernel (debug, headless) and create bootable image
export RUSTFLAGS
echo "[harness] cargo build…"
cargo +nightly build -Z build-std=core,alloc --target x86_64-unknown-none --features "$FEATURES"
KERNEL="$ROOT/target/x86_64-unknown-none/debug/sis_kernel"

# Find the most recent disk images
UEFI_IMAGE=$(find "$ROOT/target" -name "boot-uefi-sis_kernel.img" -printf '%T+ %p\n' | sort -r | head -1 | cut -d' ' -f2)
BIOS_IMAGE=$(find "$ROOT/target" -name "boot-bios-sis_kernel.img" -printf '%T+ %p\n' | sort -r | head -1 | cut -d' ' -f2)
echo "[harness] UEFI image: $UEFI_IMAGE"
echo "[harness] BIOS image: $BIOS_IMAGE"

# 3) Decide boot path
if [[ "$BOOT" == "auto" ]]; then
  # Prefer UEFI if OVMF available; otherwise BIOS
  source "$HERE/_ovmf_paths.sh"
  if [[ -n "${OVMF_CODE}" && -n "${OVMF_VARS}" ]]; then
    BOOT="uefi"
  else
    BOOT="bios"
  fi
fi
echo "[harness] BOOT=$BOOT"

# 4) QEMU common flags (headless, serial log, isa-debug-exit, KVM if available)
SERIAL_LOG="$OUT/qemu-serial.log"
rm -f "$SERIAL_LOG"
COMMON=(-nographic -serial file:"$SERIAL_LOG" -no-reboot -no-shutdown -m "$MEM" -smp "$SMP" \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 -display none)

# Add KVM acceleration if available
if [[ -r /dev/kvm && -w /dev/kvm ]]; then
    COMMON+=(-enable-kvm -cpu host)
    echo "[harness] KVM acceleration enabled"
else
    echo "[harness] KVM not available, using TCG"
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

if [[ "$BOOT" == "uefi" ]]; then
  echo "[harness] launching QEMU (UEFI)…"
  if [[ -f "$UEFI_IMAGE" ]]; then
    echo "[harness] using UEFI disk image: $UEFI_IMAGE"
    exec timeout -k 2 "${TIMEOUT}s" "$QEMU_BIN" "${COMMON[@]}" \
      -drive if=pflash,format=raw,unit=0,readonly=on,file="$OVMF_CODE" \
      -drive if=pflash,format=raw,unit=1,file="$OVMF_VARS" \
      -drive format=raw,file="$UEFI_IMAGE"
  else
    echo "[harness] UEFI image not found, falling back to direct kernel boot"
    exec timeout -k 2 "${TIMEOUT}s" "$QEMU_BIN" "${COMMON[@]}" \
      -drive if=pflash,format=raw,unit=0,readonly=on,file="$OVMF_CODE" \
      -drive if=pflash,format=raw,unit=1,file="$OVMF_VARS" \
      -kernel "$KERNEL"
  fi
else
  echo "[harness] launching QEMU (BIOS)…"
  if [[ -f "$BIOS_IMAGE" ]]; then
    echo "[harness] using BIOS disk image: $BIOS_IMAGE"
    exec timeout -k 2 "${TIMEOUT}s" "$QEMU_BIN" "${COMMON[@]}" -drive format=raw,file="$BIOS_IMAGE"
  else
    echo "[harness] BIOS image not found, trying direct kernel boot"
    exec timeout -k 2 "${TIMEOUT}s" "$QEMU_BIN" "${COMMON[@]}" -kernel "$KERNEL" -append ""
  fi
fi