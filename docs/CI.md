# SIS Kernel CI (Phase 6D Ready)

This repository's CI is split into two lanes:

## 1) Fast lane (blocking)
* **Lint (minimal)** – `clippy -D warnings` on a minimal feature set
* **Build (minimal)** – `--no-default-features`
* **Userland smoke** – boots `USR_INIT` under QEMU headless and verifies clean exit

These jobs **must** stay green to keep the repo badge clean.

## 2) Extended lane (non‑blocking)
* **SMP**: `IPC_XCPU_PING` with `SMP=2`, features: `apic smp selftests`
* **VFIO**: `VFIO_MSI_SMOKE`, features: `vfio qemu-intel-iommu-sim apic selftests`, Q35 + intel‑iommu

These run with `continue-on-error: true` to avoid breaking the badge while still surfacing regressions.
Artifacts (serial logs, boot images) are uploaded on every run for debugging.

## Local reproduction
```bash
# Minimal lint
cargo fmt --all -- --check
cargo clippy --features "ci-lint-min" -- -D warnings

# Userland smoke
TEST=USR_INIT FEATURES="userland selftests" QEMU_SMP=1 ./scripts/ci_run.sh

# Extended (non-blocking in CI)
TEST=IPC_XCPU_PING FEATURES="apic smp selftests" QEMU_SMP=2 ./scripts/ci_run.sh
TEST=VFIO_MSI_SMOKE FEATURES="vfio qemu-intel-iommu-sim apic selftests" QEMU_SMP=1 ./scripts/ci_run.sh
```

## Notes
* Toolchain pinned to `nightly-2024-12-15`.
* BIOS-only bootloader 0.11.x path assumed; build.rs must emit a fresh BIOS image when `FORCE_BOOTIMG=1`.
* CI sets `CARGO_NET_OFFLINE=true` to keep runs deterministic and fast.