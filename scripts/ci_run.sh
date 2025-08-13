#!/usr/bin/env bash
set -euo pipefail

# CI runner shim so the workflow doesn't have to know harness details.
# Expects:
#   TEST      – e.g., USR_INIT / IPC_XCPU_PING / VFIO_MSI_SMOKE
#   FEATURES  – cargo features string (optional)
#   QEMU_SMP  – number of vCPUs (default 1)
#   CI        – when set, enables headless, shorter timeouts

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT_DIR}/out"
mkdir -p "${OUT_DIR}"

export QEMU_SMP="${QEMU_SMP:-1}"
export TEST="${TEST:-USR_INIT}"
export FEATURES="${FEATURES:-}"

# Fresh image to avoid stale-boot surprises in CI
export FORCE_BOOTIMG=1

# CI timeouts (shorter than dev)
if [[ "${CI:-}" == "true" ]]; then
  export TIMEOUT="${TIMEOUT:-45}"
  export HEADLESS=1
else
  export TIMEOUT="${TIMEOUT:-90}"
fi

echo "[ci] TEST=${TEST} FEATURES=${FEATURES} SMP=${QEMU_SMP}"

if [[ -n "${FEATURES}" ]]; then
  FEAT_ARG=(--features "${FEATURES}")
else
  FEAT_ARG=()
fi

# Build (kernel ELF + build.rs should emit BIOS image)
cargo build --target x86_64-unknown-none "${FEAT_ARG[@]}" -Z build-std=core,alloc

# Map TEST -> RUSTFLAGS and run QEMU via existing harness
export TEST
bash "${ROOT_DIR}/scripts/qemu.sh"
RC=$?

# Persist timing + exit diagnostics
if [[ -f "${OUT_DIR}/qemu-serial.log" ]]; then
  tail -n 200 "${OUT_DIR}/qemu-serial.log" || true
fi

exit ${RC}