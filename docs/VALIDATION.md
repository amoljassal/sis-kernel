# SIS Kernel — Validation Harness

## Quick start

```bash
# Run a userland test (auto-picks BIOS or UEFI)
./scripts/qemu.sh TEST=USR_INIT

# Force UEFI if OVMF is installed (or set OVMF_CODE/OVMF_VARS)
BOOT=uefi ./scripts/qemu.sh TEST=USR_SPAWN_TWO
```

## Supported TEST values

- **Exceptions/IDT**: `DF`, `PF`, `GP`, `DIV0`
- **Ring3**: `RING3`, `RING3_RT`
- **Timers/APIC**: `LAPIC_TIMER`
- **SMP**: `SMP_2` (use `SMP=2`)
- **PFM v1/v2**: `PFM_NP_U_R`, `PFM_NP_U_W`, `PFM_US_VIOL`, `PFM_PROT_U_W`, `PFM_NX_EXEC`, `PFM_GUARD_UNDER`, `PFM_GUARD_OVER`
- **Userland**: `USR_INIT`, `USR_SPAWN_TWO`, `USR_ELF_EDGES`, `USR_VFS_NEG`

## Output

All runs write a serial log to `out/qemu-serial.log`. QEMU process exit code mirrors the kernel's debug-exit code.

## UEFI Notes

If UEFI fails to locate OVMF automatically:

```bash
export OVMF_CODE=/path/to/OVMF_CODE.fd
export OVMF_VARS=/path/to/OVMF_VARS.fd
BOOT=uefi ./scripts/qemu.sh TEST=USR_INIT
```