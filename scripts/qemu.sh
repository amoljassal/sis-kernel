#!/usr/bin/env bash
set -euo pipefail

# ====== PARAMS ======
TEST="${TEST:-DF}"            # DF|PF|GP|DIV0|SYSCALL|TIMER|RING3|RING3_RT|LAPIC_TIMER|SMP_2|PFM_NP_U_R|PFM_NP_U_W|PFM_US_VIOL|PFM_PROT_U_W|PFM_NX_EXEC|PFM_GUARD_UNDER|PFM_GUARD_OVER|AS_PER_TASK_ISOLATION|IPC_PING
FEATURES="idt-selftest"
TARGET="x86_64-unknown-none"
PROFILE="${PROFILE:-dev}"
SERIAL_LOG="qemu-serial.log"
QEMU_BIN="${QEMU_BIN:-qemu-system-x86_64}"
USE_UEFI="${USE_UEFI:-0}"     # default to legacy -kernel unless you have an EFI image path
OVMF_CODE="${OVMF_CODE:-/opt/homebrew/share/edk2/ovmf/OVMF_CODE.fd}"
KERNEL_ELF="target/${TARGET}/debug/sis_kernel"

# ====== BUILD ======
echo "[*] Building kernel for TEST=${TEST}"
if [[ "${TEST}" == "LAPIC_TIMER" ]]; then
  FEATURES="idt-selftest,apic"
elif [[ "${TEST}" == "SMP_2" ]]; then
  FEATURES="idt-selftest,apic,smp"
elif [[ "${TEST}" == "PFM_NP_U_R" || "${TEST}" == "PFM_NP_U_W" || "${TEST}" == "PFM_US_VIOL" || "${TEST}" == "PFM_PROT_U_W" || "${TEST}" == "PFM_NX_EXEC" || "${TEST}" == "PFM_GUARD_UNDER" || "${TEST}" == "PFM_GUARD_OVER" ]]; then
  FEATURES="idt-selftest,pf-matrix"
elif [[ "${TEST}" == "AS_PER_TASK_ISOLATION" ]]; then
  FEATURES="idt-selftest,per-task-mm"
elif [[ "${TEST}" == "IPC_PING" ]]; then
  FEATURES="idt-selftest,ipc"
elif [[ "${TEST}" == "VFIO_MSI_SMOKE" ]]; then
  FEATURES="idt-selftest,vfio,apic,iommu"
fi
RUSTFLAGS="--cfg selftest_${TEST}" cargo +nightly build -Z build-std=core,alloc --features "${FEATURES}" --target "${TARGET}"

# ====== RUN QEMU ======
rm -f "${SERIAL_LOG}"
echo "[*] Launching QEMU (${TEST}) ..."

SMP_FLAGS=""
if [[ "${TEST}" == "SMP_2" ]]; then
  SMP_FLAGS="-smp 2"
fi

COMMON="-machine q35,accel=tcg ${SMP_FLAGS} -m 512M -device isa-debug-exit,iobase=0xf4,iosize=0x04 -device e1000,netdev=n0 -netdev user,id=n0 -serial file:${SERIAL_LOG} -display none -no-reboot -no-shutdown"

if [[ "${USE_UEFI}" == "1" ]]; then
  ${QEMU_BIN} ${COMMON} -bios "${OVMF_CODE}" || true
else
  ${QEMU_BIN} ${COMMON} -kernel "${KERNEL_ELF}" || true
fi

# ====== VERIFY EXIT & LOGS ======
status=$?
echo "[*] QEMU exit status: ${status}"

# Some QEMU variants return (code<<1)|1; consider both success forms (0 or 1) when code=0.
if [[ "${status}" != "0" && "${status}" != "1" ]]; then
  echo "[!] Non-zero exit status (not success form): ${status}"
  cat "${SERIAL_LOG}" || true
  exit 2
fi

if [[ ! -s "${SERIAL_LOG}" ]]; then
  echo "[!] Serial log empty"
  exit 3
fi

grep -q '\[init\] IDT loaded' "${SERIAL_LOG}" || { echo "[!] Missing 'IDT loaded'"; exit 4; }
grep -q '\[selftest\] starting IDT self-tests...' "${SERIAL_LOG}" || { echo "[!] Missing self-test start"; exit 5; }

case "${TEST}" in
  DF)
    grep -q '\[selftest\] about to trigger DF' "${SERIAL_LOG}" || { echo "[!] Missing DF banner"; exit 6; }
    grep -q '\[df\] double fault — halting' "${SERIAL_LOG}" || { echo "[!] Missing DF handler log"; exit 7; }
    ;;
  PF)
    grep -q '\[selftest\] about to trigger PF' "${SERIAL_LOG}" || { echo "[!] Missing PF banner"; exit 8; }
    grep -q '\[exc\] page-fault — halting' "${SERIAL_LOG}" || { echo "[!] Missing PF handler log"; exit 9; }
    ;;
  GP)
    grep -q '\[selftest\] about to trigger GP' "${SERIAL_LOG}" || { echo "[!] Missing GP banner"; exit 10; }
    grep -q '\[exc\] general-protection-fault — halting' "${SERIAL_LOG}" || { echo "[!] Missing GP handler log"; exit 11; }
    ;;
  DIV0)
    grep -q '\[selftest\] about to trigger DIV0' "${SERIAL_LOG}" || { echo "[!] Missing DIV0 banner"; exit 12; }
    grep -q '\[exc\] divide-by-zero — halting' "${SERIAL_LOG}" || { echo "[!] Missing DIV0 handler log"; exit 13; }
    ;;
  SYSCALL)
    grep -q '\[selftest\] about to trigger SYSCALL ping' "${SERIAL_LOG}" || { echo "[!] Missing SYSCALL banner"; exit 14; }
    grep -q '\[syscall\] ping' "${SERIAL_LOG}" || { echo "[!] Missing syscall log"; exit 15; }
    ;;
  TIMER)
    grep -q '\[selftest\] about to wait for TIMER ticks' "${SERIAL_LOG}" || { echo "[!] Missing TIMER banner"; exit 16; }
    grep -q '\[tick\] n=10' "${SERIAL_LOG}" || { echo "[!] Missing tick milestone"; exit 17; }
    ;;
  RING3)
    grep -q '\[selftest\] about to trigger RING3 syscall trampoline' "${SERIAL_LOG}" || { echo "[!] Missing RING3 banner"; exit 18; }
    grep -q '\[syscall\] Ring-0 syscall' "${SERIAL_LOG}" || { echo "[!] Missing Ring-0 syscall detection"; exit 19; }
    ;;
  RING3_RT)
    grep -q '\[selftest\] starting RING3 round-trip...' "${SERIAL_LOG}" || { echo "[!] Missing RING3_RT start"; exit 20; }
    grep -q '\[selftest\] jumping to user mode...' "${SERIAL_LOG}" || { echo "[!] Missing jump banner"; exit 21; }
    grep -q '\[syscall\] Ring-3 syscall detected!' "${SERIAL_LOG}" || { echo "[!] Missing R3 syscall detect"; exit 22; }
    grep -q '\[user\] hello from ring-3!' "${SERIAL_LOG}" || { echo "[!] Missing user hello"; exit 23; }
    grep -q '\[user\] syscall returned successfully!' "${SERIAL_LOG}" || { echo "[!] Missing user return banner"; exit 24; }
    grep -q '\[syscall\] Ring-3 exit request' "${SERIAL_LOG}" || { echo "[!] Missing exit request"; exit 25; }
    ;;
  LAPIC_TIMER)
    grep -q '\[init\] LAPIC enabled' "${SERIAL_LOG}" || { echo "[!] Missing LAPIC init"; exit 26; }
    grep -q '\[init\] LAPIC timer periodic' "${SERIAL_LOG}" || { echo "[!] Missing LAPIC timer init"; exit 27; }
    grep -q '\[lapic-tick\] n=10' "${SERIAL_LOG}" || { echo "[!] Missing LAPIC tick milestone"; exit 28; }
    ;;
  SMP_2)
    grep -q '\[init\] LAPIC enabled' "${SERIAL_LOG}" || { echo "[!] Missing LAPIC init"; exit 29; }
    grep -q '\[smp\] bsp apic=' "${SERIAL_LOG}" || { echo "[!] Missing BSP line"; exit 30; }
    grep -q '\[smp\] cpu=' "${SERIAL_LOG}" || { echo "[!] Missing AP online"; exit 31; }
    grep -q '\[lapic-tick\] cpu=' "${SERIAL_LOG}" || { echo "[!] Missing per-CPU tick"; exit 32; }
    ;;
  PFM_NP_U_R)
    grep -q '\[selftest\] starting Page-Fault Matrix tests' "${SERIAL_LOG}" || { echo "[!] Missing PFM start"; exit 33; }
    grep -q '\[pfm\] Test NP_U_R: Ring-3 read of unmapped user page' "${SERIAL_LOG}" || { echo "[!] Missing NP_U_R banner"; exit 34; }
    grep -q '\[pfm\] NP_U_R: Correct error_code=4 (P=0,W/R=0,U/S=1)' "${SERIAL_LOG}" || { echo "[!] Missing NP_U_R validation"; exit 35; }
    ;;
  PFM_NP_U_W)
    grep -q '\[selftest\] starting Page-Fault Matrix tests' "${SERIAL_LOG}" || { echo "[!] Missing PFM start"; exit 36; }
    grep -q '\[pfm\] Test NP_U_W: Ring-3 write to unmapped user page' "${SERIAL_LOG}" || { echo "[!] Missing NP_U_W banner"; exit 37; }
    grep -q '\[pfm\] NP_U_W: Correct error_code=6 (P=0,W/R=1,U/S=1)' "${SERIAL_LOG}" || { echo "[!] Missing NP_U_W validation"; exit 38; }
    ;;
  PFM_US_VIOL)
    grep -q '\[selftest\] starting Page-Fault Matrix tests' "${SERIAL_LOG}" || { echo "[!] Missing PFM start"; exit 39; }
    grep -q '\[pfm\] Test US_VIOL: Ring-3 read of supervisor page' "${SERIAL_LOG}" || { echo "[!] Missing US_VIOL banner"; exit 40; }
    grep -q '\[pfm\] US_VIOL: Correct error_code=5 (P=1,W/R=0,U/S=1)' "${SERIAL_LOG}" || { echo "[!] Missing US_VIOL validation"; exit 41; }
    ;;
  PFM_PROT_U_W)
    grep -q '\[selftest\] starting Page-Fault Matrix tests' "${SERIAL_LOG}" || { echo "[!] Missing PFM start"; exit 42; }
    grep -q '\[pfm\] Test PROT_U_W: Ring-3 write to present RO user page' "${SERIAL_LOG}" || { echo "[!] Missing PROT_U_W banner"; exit 43; }
    grep -q '\[pfm\] PROT_U_W: Correct error_code=7 (P=1,W/R=1,U/S=1,ID=0)' "${SERIAL_LOG}" || { echo "[!] Missing PROT_U_W validation"; exit 44; }
    ;;
  PFM_NX_EXEC)
    grep -q '\[selftest\] starting Page-Fault Matrix tests' "${SERIAL_LOG}" || { echo "[!] Missing PFM start"; exit 45; }
    grep -q '\[pfm\] Test NX_EXEC: Ring-3 execute from present NX user page' "${SERIAL_LOG}" || { echo "[!] Missing NX_EXEC banner"; exit 46; }
    grep -q '\[pfm\] NX_EXEC: Correct error_code=17 (P=1,W/R=0,U/S=1,ID=1)' "${SERIAL_LOG}" || { echo "[!] Missing NX_EXEC validation"; exit 47; }
    ;;
  PFM_GUARD_UNDER)
    grep -q '\[selftest\] starting Page-Fault Matrix tests' "${SERIAL_LOG}" || { echo "[!] Missing PFM start"; exit 48; }
    grep -q '\[pfm\] Test GUARD_UNDER: Ring-3 write to unmapped page below stack' "${SERIAL_LOG}" || { echo "[!] Missing GUARD_UNDER banner"; exit 49; }
    grep -q '\[pfm\] GUARD_UNDER: Correct error_code=6 (P=0,W/R=1,U/S=1,ID=0)' "${SERIAL_LOG}" || { echo "[!] Missing GUARD_UNDER validation"; exit 50; }
    ;;
  PFM_GUARD_OVER)
    grep -q '\[selftest\] starting Page-Fault Matrix tests' "${SERIAL_LOG}" || { echo "[!] Missing PFM start"; exit 51; }
    grep -q '\[pfm\] Test GUARD_OVER: Ring-3 write to unmapped page above stack' "${SERIAL_LOG}" || { echo "[!] Missing GUARD_OVER banner"; exit 52; }
    grep -q '\[pfm\] GUARD_OVER: Correct error_code=6 (P=0,W/R=1,U/S=1,ID=0)' "${SERIAL_LOG}" || { echo "[!] Missing GUARD_OVER validation"; exit 53; }
    ;;
  AS_PER_TASK_ISOLATION)
    grep -q '\[selftest\] starting per-task address space isolation test' "${SERIAL_LOG}" || { echo "[!] Missing AS isolation start"; exit 54; }
    grep -q '\[as\] create two address spaces' "${SERIAL_LOG}" || { echo "[!] Missing AS creation"; exit 55; }
    grep -q '\[as\] write A=0xaa at 0x0000000040000000' "${SERIAL_LOG}" || { echo "[!] Missing AS write A"; exit 56; }
    grep -q '\[as\] write B=0xbb at 0x0000000040000000' "${SERIAL_LOG}" || { echo "[!] Missing AS write B"; exit 57; }
    grep -q '\[as\] verify A ok' "${SERIAL_LOG}" || { echo "[!] Missing AS verify A"; exit 58; }
    grep -q '\[as\] verify B ok' "${SERIAL_LOG}" || { echo "[!] Missing AS verify B"; exit 59; }
    grep -q '\[as\] isolation PASS' "${SERIAL_LOG}" || { echo "[!] Missing AS isolation pass"; exit 60; }
    ;;
  IPC_PING)
    grep -q '\[selftest\] starting IPC ping test' "${SERIAL_LOG}" || { echo "[!] Missing IPC ping start"; exit 61; }
    grep -q '\[ipc\] selftest start' "${SERIAL_LOG}" || { echo "[!] Missing IPC selftest start"; exit 62; }
    grep -q '\[ipc\] create channel ok' "${SERIAL_LOG}" || { echo "[!] Missing IPC channel creation"; exit 63; }
    grep -q '\[userA\] send ping' "${SERIAL_LOG}" || { echo "[!] Missing userA send ping"; exit 64; }
    grep -q '\[userB\] recv ping' "${SERIAL_LOG}" || { echo "[!] Missing userB recv ping"; exit 65; }
    ;;
  VFIO_MSI_SMOKE)
    grep -q '\[init\] IDT loaded' "${SERIAL_LOG}" || { echo "[!] Missing IDT loaded"; exit 66; }
    grep -q '\[VFIO\] Initializing VFIO-lite subsystem' "${SERIAL_LOG}" || { echo "[!] Missing VFIO init"; exit 67; }
    grep -q '\[VFIO\] MSI armed' "${SERIAL_LOG}" || { echo "[!] Missing MSI armed"; exit 68; }
    grep -q '\[msi-trigger\] e1000 BAR0' "${SERIAL_LOG}" || { echo "[!] Missing e1000 trigger"; exit 69; }
    grep -q '\[vfio-irq\] vector 0x5E fired' "${SERIAL_LOG}" || { echo "[!] Missing MSI interrupt delivery"; exit 70; }
    grep -q '\[vfio-irq\] Selftest exit: first MSI delivered successfully' "${SERIAL_LOG}" || { echo "[!] Missing selftest success"; exit 71; }
    ;;
  *)
    echo "[!] Unknown TEST=${TEST}"
    exit 72
    ;;
esac

echo "[OK] ${TEST} test passed."