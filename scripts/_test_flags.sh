#!/usr/bin/env bash
set -euo pipefail

# Map TEST=… to RUSTFLAGS cfgs and default FEATURES if not provided.
# You can extend this table as new tests arrive.

case "${TEST:-}" in
  # ---- IDT / exceptions ----
  DF|PF|GP|DIV0)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT=""
    ;;

  # ---- Ring3 syscall trampoline / round-trip ----
  RING3|RING3_RT)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT="idt-selftest"
    ;;

  # ---- LAPIC/APIC timer ----
  LAPIC_TIMER)
    export RUSTFLAGS="--cfg selftest_LAPIC_TIMER"
    FEATURES_DEFAULT="apic"
    ;;

  # ---- SMP 2 cores ----
  SMP_2)
    export RUSTFLAGS="--cfg selftest_SMP_2"
    FEATURES_DEFAULT="apic smp"
    ;;

  # ---- Page Fault Matrix v1/v2 ----
  PFM_NP_U_R|PFM_NP_U_W|PFM_US_VIOL|PFM_PROT_U_W|PFM_NX_EXEC|PFM_GUARD_UNDER|PFM_GUARD_OVER)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT="pf-matrix"
    ;;

  # ---- Userland Phase 4 / 4.1 ----
  USR_INIT|USR_SPAWN_TWO|USR_ELF_EDGES|USR_VFS_NEG)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT="userland"
    ;;

  # ---- IPC / Scheduler checks ----
  IPC_PING|IPC_BLOCK_WAKE|IPC_PING_PONG_2TASK|SCHED_RR_FAIR|TIMER_SLEEP)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT="ipc scheduler"
    ;;

  # ---- IOMMU Phase 5A ----
  IOMMU_PROBE|IOMMU_DENY_DEFAULT)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT="iommu qemu-intel-iommu-sim"
    ;;

  # ---- VFIO Phase 5B ----
  VFIO_BIND_E1000|VFIO_CFG_READ|VFIO_MAP_BAR|VFIO_IRQ_SETUP)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT="vfio"
    ;;

  # ---- VFIO Phase 5C-A: Domain + DMA ----
  VFIO_DOMAIN_CREATE|VFIO_DMA_STAGING)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT="vfio qemu-intel-iommu-sim"
    ;;

  # ---- VFIO Phase 5C-B: MSI + ISR ----
  VFIO_MSI_SMOKE)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT="vfio apic idt-selftest"
    ;;
  VFIO_MSI_SOAK)
    export RUSTFLAGS="--cfg selftest_${TEST}"
    FEATURES_DEFAULT="vfio apic idt-selftest"
    ;;

  *)
    echo "error: unknown or missing TEST=… ($TEST)"; exit 2;;
esac

# If caller didn't pass FEATURES, use our defaults
if [[ -z "${FEATURES:-}" ]]; then
  FEATURES="${FEATURES_DEFAULT}"
fi
export FEATURES