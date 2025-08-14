#!/usr/bin/env bash
set -euo pipefail
: "${TEST:?TEST not set}"
: "${QEMU_SMP:=1}"
: "${TIMEOUT:=180}"    # seconds; CI-fast tests should finish well within

echo "[ci] run-extended: TEST=$TEST QEMU_SMP=$QEMU_SMP TIMEOUT=$TIMEOUT"

# Ensure flags are prepared
source scripts/_test_flags.sh

# Build with matrix features
echo "[ci] building..."
cargo build --target x86_64-unknown-none --features "${features:-${FEATURES:-}}" 1>out/build-stdout.txt 2>out/build-stderr.txt || {
  echo "::error::build failed"
  tail -n 200 out/build-stderr.txt || true
  exit 2
}

# Fresh boot image each run (protect against staleness)  
mkdir -p out

# Run QEMU headless with a hard timeout; capture logs and exit code semantics
echo "[ci] launching qemu..."
set +e
QEMU_ACCEL=tcg QEMU_SMP="$QEMU_SMP" timeout "${TIMEOUT}s" ./scripts/qemu.sh > out/qemu-serial.log 2>&1
rc=$?
set -e

echo "[ci] qemu exit rc=$rc"
tail -n 200 out/qemu-serial.log || true

# Classify
if [[ $rc -eq 0 ]]; then
  echo "[ci] PASS (qemu returned 0)"
  exit 0
elif [[ $rc -eq 124 ]]; then
  echo "::error::Timeout (124) — test likely did not call qemu::exit_ok()."
  exit 124
else
  echo "::error::QEMU returned rc=$rc"
  exit $rc
fi