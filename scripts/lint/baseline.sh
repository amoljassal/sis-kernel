#!/usr/bin/env bash
set -euo pipefail

# ---- CONFIGS ---------------------------------------------------------------
TARGET="x86_64-unknown-none"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/out/lint-baseline"
RAW="$OUT/raw"
REPORT="$OUT/report"
TS="$(date -u +"%Y%m%dT%H%M%SZ")"

# Build configurations (name|feature string)
# Order: firewall → dev → selftests → userland → full
declare -a JOBS=(
  "firewall|firewall"
  "dev|apic smp scheduler"
  "selftests|apic smp scheduler selftests"
  "userland|userland"
  "full|apic smp scheduler selftests vfio iommu userland"
)

# Clippy config(s)
declare -a CLIPPY=(
  "clippy-dev|apic smp scheduler"
)

# ---- PREP -----------------------------------------------------------------
mkdir -p "$RAW" "$REPORT"

echo "[lint-baseline] start: $TS"
rustc --version || true
cargo --version || true

# Make rustc print all warnings even if build fails other crates
export RUSTFLAGS="${RUSTFLAGS:-} -W warnings"

# Speed & stability knobs (safe even if missing)
export CARGO_NET_OFFLINE=${CARGO_NET_OFFLINE:-true}
export CARGO_TERM_COLOR=never

# ---- FORMAT CHECK ----------------------------------------------------------
echo "[lint-baseline] fmt check"
FMT_LOG="$RAW/fmt.log"
if cargo fmt -- --check >"$FMT_LOG" 2>&1; then
  echo "[ok] cargo fmt --check"
else
  echo "[warn] cargo fmt --check failed (see $FMT_LOG)"
fi

# ---- BUILD MATRIX ----------------------------------------------------------
for row in "${JOBS[@]}"; do
  name="${row%%|*}"
  feats="${row#*|}"
  log="$RAW/build-${name}.log"
  echo "[lint-baseline] build: $name  (features: $feats)"
  # Capture both stdout/stderr so we keep all warnings
  if cargo build --target "$TARGET" --features "$feats" >"$log" 2>&1; then
    echo "[ok] build $name"
  else
    echo "[fail] build $name (see $log)"
    # We still keep going to collect whatever warnings we can.
  fi
done

# ---- CLIPPY (strict) -------------------------------------------------------
for row in "${CLIPPY[@]}"; do
  name="${row%%|*}"
  feats="${row#*|}"
  log="$RAW/${name}.log"
  echo "[lint-baseline] clippy: $name  (features: $feats)"
  # -D warnings: fail intentionally; we still capture output
  if cargo clippy --target "$TARGET" --features "$feats" -- -D warnings >"$log" 2>&1; then
    echo "[ok] clippy $name"
  else
    echo "[fail] clippy $name (strict) — expected if there are warnings (see $log)"
  fi
done

# ---- PARSE & SUMMARIZE -----------------------------------------------------
echo "[lint-baseline] parsing"
python3 "$ROOT/scripts/lint/parse_warnings.py" \
  --input "$RAW" \
  --output-json "$REPORT/warnings.json" \
  --output-md "$REPORT/summary.md" \
  --tag "$TS"

echo
echo "[lint-baseline] DONE"
echo "Artifacts:"
echo "  - $REPORT/summary.md"
echo "  - $REPORT/warnings.json"
echo "  - $RAW/*.log"