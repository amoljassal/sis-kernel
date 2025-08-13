#!/usr/bin/env bash
set -euo pipefail
# Force fully-rendered diagnostics into a file so warnings don't drown errors.
RUSTFLAGS="-Zmacro-backtrace" \
cargo +nightly build --target x86_64-unknown-none \
  --features "apic,smp" \
  --message-format=json-diagnostic-rendered-ansi 2>build.err || true
echo
echo "---- Errors (if any) ----"
grep -n --color=always -E 'error\[[A-Z0-9]+\]' -n build.err || true
echo
echo "Open build.err for full context."