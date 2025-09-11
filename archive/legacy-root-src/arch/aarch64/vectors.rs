//! ARM64 Exception Vector Table
//!
//! Minimal exception vectors for early boot

use core::arch::global_asm;

// Define exception vector table in assembly
global_asm!(
    r#"
.section .text.vectors
.balign 0x800
.global __exception_vectors
__exception_vectors:

// Current EL with SP0
.balign 0x80
curr_el_sp0_sync:
    wfe
    b curr_el_sp0_sync

.balign 0x80
curr_el_sp0_irq:
    wfe
    b curr_el_sp0_irq

.balign 0x80
curr_el_sp0_fiq:
    wfe
    b curr_el_sp0_fiq

.balign 0x80
curr_el_sp0_serr:
    wfe
    b curr_el_sp0_serr

// Current EL with SPx
.balign 0x80
curr_el_spx_sync:
    wfe
    b curr_el_spx_sync

.balign 0x80
curr_el_spx_irq:
    wfe
    b curr_el_spx_irq

.balign 0x80
curr_el_spx_fiq:
    wfe
    b curr_el_spx_fiq

.balign 0x80
curr_el_spx_serr:
    wfe
    b curr_el_spx_serr

// Lower EL using AArch64
.balign 0x80
lower_el_aarch64_sync:
    wfe
    b lower_el_aarch64_sync

.balign 0x80
lower_el_aarch64_irq:
    wfe
    b lower_el_aarch64_irq

.balign 0x80
lower_el_aarch64_fiq:
    wfe
    b lower_el_aarch64_fiq

.balign 0x80
lower_el_aarch64_serr:
    wfe
    b lower_el_aarch64_serr

// Lower EL using AArch32
.balign 0x80
lower_el_aarch32_sync:
    wfe
    b lower_el_aarch32_sync

.balign 0x80
lower_el_aarch32_irq:
    wfe
    b lower_el_aarch32_irq

.balign 0x80
lower_el_aarch32_fiq:
    wfe
    b lower_el_aarch32_fiq

.balign 0x80
lower_el_aarch32_serr:
    wfe
    b lower_el_aarch32_serr

"#
);