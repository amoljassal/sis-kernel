# Path Migration Note

The repository now uses a crates/ workspace layout:

- Kernel: `crates/kernel`
- UEFI loader: `crates/uefi-boot`
- Test runner: `crates/testing`

Historical documentation may refer to files under `src/...` at the repository root. Replace these with the corresponding paths under `crates/kernel/src/...` when following instructions.

Examples:
- `src/arch/aarch64/aarch64-qemu.ld` → `crates/kernel/src/arch/aarch64/aarch64-qemu.ld`
- `src/arch/riscv64/mmu.rs` → `crates/kernel/src/arch/riscv64/mmu.rs`
- `src/main.rs` → `crates/kernel/src/main.rs`

Build notes:
- Pass the linker script via `RUSTFLAGS="-C link-arg=-Tcrates/kernel/src/arch/aarch64/aarch64-qemu.ld"` when targeting AArch64.
- Host tools (like the test runner) should be built without global `build-std` or `no_std` overrides.

