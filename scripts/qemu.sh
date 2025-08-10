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

# Enhanced diagnostics
SERIAL_LOG="$OUT/qemu-serial.log"
rm -f "$SERIAL_LOG"

# Simple timer (ms)
_now_ms() { date +%s%3N 2>/dev/null || python3 - <<'PY'
import time; print(int(time.time()*1000))
PY
}
START_MS=$(_now_ms)

# 4) QEMU common flags (headless, serial log, isa-debug-exit, KVM if available)
COMMON=(-nographic -serial file:"$SERIAL_LOG" -no-reboot -no-shutdown -m "$MEM" -smp "$SMP" \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 -display none)

# Add Intel IOMMU device for IOMMU feature testing
if [[ "$FEATURES" == *"iommu"* ]]; then
    COMMON+=(-device intel-iommu,intremap=on)
    echo "[harness] Intel IOMMU enabled for testing"
fi

# Add e1000 device for VFIO feature testing
if [[ "$FEATURES" == *"vfio"* ]]; then
    COMMON+=(-device e1000,netdev=net0 -netdev user,id=net0)
    echo "[harness] e1000 device enabled for VFIO testing"
fi

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

# Build QEMU command based on boot mode
if [[ "$BOOT" == "uefi" ]]; then
  echo "[harness] launching QEMU (UEFI)…"
  if [[ -f "$UEFI_IMAGE" ]]; then
    echo "[harness] using UEFI disk image: $UEFI_IMAGE"
    QEMU_CMD=(timeout -k 2 "${TIMEOUT}s" "$QEMU_BIN" "${COMMON[@]}" \
      -drive if=pflash,format=raw,unit=0,readonly=on,file="$OVMF_CODE" \
      -drive if=pflash,format=raw,unit=1,file="$OVMF_VARS" \
      -drive format=raw,file="$UEFI_IMAGE")
  else
    echo "[harness] UEFI image not found, falling back to direct kernel boot"
    QEMU_CMD=(timeout -k 2 "${TIMEOUT}s" "$QEMU_BIN" "${COMMON[@]}" \
      -drive if=pflash,format=raw,unit=0,readonly=on,file="$OVMF_CODE" \
      -drive if=pflash,format=raw,unit=1,file="$OVMF_VARS" \
      -kernel "$KERNEL")
  fi
else
  echo "[harness] launching QEMU (BIOS)…"
  if [[ -f "$BIOS_IMAGE" ]]; then
    echo "[harness] using BIOS disk image: $BIOS_IMAGE"
    QEMU_CMD=(timeout -k 2 "${TIMEOUT}s" "$QEMU_BIN" "${COMMON[@]}" -drive format=raw,file="$BIOS_IMAGE")
  else
    echo "[harness] BIOS image not found, trying direct kernel boot"
    QEMU_CMD=(timeout -k 2 "${TIMEOUT}s" "$QEMU_BIN" "${COMMON[@]}" -kernel "$KERNEL" -append "")
  fi
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