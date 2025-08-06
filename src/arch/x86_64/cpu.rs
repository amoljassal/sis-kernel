//! CPU helper functions.
//!
//! This module provides wrappers around low-level CPU instructions
//! such as `hlt` and `pause`.  These functions are used by the
//! scheduler and idle loop to reduce power consumption.

use core::arch::asm;

/// Halt the CPU until the next interrupt.  This instruction puts
/// the processor into a low‑power state.
#[inline]
pub fn halt() {
    unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }
}

/// Hint to the CPU that the code is in a spin‑loop.  Modern CPUs
/// can use this to reduce power usage and improve performance in
/// hyper‑threaded environments.
#[inline]
pub fn pause() {
    unsafe { asm!("pause", options(nomem, nostack, preserves_flags)); }
}

/// Read the CPUID instruction with the given EAX input.  Returns
/// (EAX, EBX, ECX, EDX).  Unsafe because CPUID may not be available
/// on all CPUs (but on x86_64 it is).
#[inline]
pub fn cpuid(function: u32) -> (u32, u32, u32, u32) {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;
    unsafe {
        asm!(
            "cpuid",
            inlateout("eax") function => eax,
            lateout("ebx") ebx,
            lateout("ecx") ecx,
            lateout("edx") edx,
            options(nomem, nostack)
        );
    }
    (eax, ebx, ecx, edx)
}

/// Check if the CPU supports the IOMMU/AMD‑V features required for
/// VFIO passthrough.  This function returns a boolean and can be
/// extended to inspect additional feature bits.
pub fn check_iommu_support() -> bool {
    // On x86_64, the presence of an IOMMU can be inferred from
    // extended function CPUID leaf 0x8000_0001, bit 2 of ECX (SVM)
    // for AMD or bit 31 of ECX (VMX) for Intel.  We do not
    // distinguish vendor here.
    let (_, _, ecx, _) = cpuid(0x8000_0001);
    let svm = (ecx & (1 << 2)) != 0;
    let (_, _, ecx1, _) = cpuid(1);
    let vmx = (ecx1 & (1 << 5)) != 0;
    svm || vmx
}

/// Read the time stamp counter (TSC).  Returns the 64‑bit cycle
/// count.  Useful for micro‑benchmarking context switch latency or
/// other short events.  The TSC is not invariant on all CPUs.
#[inline]
pub fn rdtsc() -> u64 {
    let high: u32;
    let low: u32;
    unsafe {
        asm!("rdtsc", out("edx") high, out("eax") low, options(nomem, nostack, preserves_flags));
    }
    ((high as u64) << 32) | (low as u64)
}

/// Read a Model Specific Register (MSR).  Unsafe because MSR reads
/// may fault if the MSR is unavailable or privileged.
pub unsafe fn read_msr(msr: u32) -> u64 {
    let high: u32;
    let low: u32;
    asm!(
        "rdmsr",
        in("ecx") msr,
        out("edx") high,
        out("eax") low,
        options(nostack, preserves_flags)
    );
    ((high as u64) << 32) | (low as u64)
}