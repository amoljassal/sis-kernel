This src/ tree is legacy and not part of the active Cargo workspace.

Active kernel: crates/kernel
Active UEFI loader: crates/uefi-boot
Active test runner: crates/testing

Notes:
- Files here were retained for reference only and are not built by the workspace.
- New development should target the crates/ layout.
- To boot under QEMU UEFI: use scripts/uefi_run.sh.
- To run the test runner: cargo run -p sis-testing --release

