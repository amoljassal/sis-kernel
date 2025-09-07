#![no_std]
#![no_main]

#[cfg(target_arch = "aarch64")]
#[link_section = ".text._start"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe { uart_print(b"KERNEL(U)\n"); }

    #[cfg(all(target_arch = "aarch64", feature = "bringup"))]
    unsafe {
        bringup::run();
    }

    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

#[inline(always)]
unsafe fn uart_print(msg: &[u8]) {
    const UART0_DR: *mut u32 = 0x0900_0000 as *mut u32;
    for &b in msg {
        core::ptr::write_volatile(UART0_DR, b as u32);
    }
}

#[cfg(all(target_arch = "aarch64", feature = "bringup"))]
mod bringup {
    use core::arch::asm;

    // 64 KiB bootstrap stack
    #[repr(align(16))]
    static mut BOOT_STACK: [u8; 64 * 1024] = [0; 64 * 1024];

    // Level-1 and Level-2 translation tables (4 KiB aligned)
    #[repr(align(4096))]
    static mut L1_TABLE: [u64; 512] = [0; 512];
    #[repr(align(4096))]
    static mut L2_TABLE_0: [u64; 512] = [0; 512];

    // Simple EL-aware VBAR install and UART helper from outer module
    extern "C" {
        static VECTORS: u8;
    }

    pub unsafe fn run() {
        // 1) Install stack
        let sp_top = BOOT_STACK.as_ptr().add(BOOT_STACK.len()) as u64;
        asm!("mov sp, {sp}", sp = in(reg) sp_top, options(nostack, preserves_flags));
        super::uart_print(b"STACK OK\n");

        // 2) Install exception vectors
        install_vectors();
        super::uart_print(b"VECTORS OK\n");

        // 3) Enable MMU (EL1 only). If not EL1, skip with message.
        let current_el: u64; asm!("mrs {el}, CurrentEL", el = out(reg) current_el);
        let el = (current_el >> 2) & 0x3;
        if el != 1 {
            super::uart_print(b"MMU SKIP (EL!=1)\n");
            return;
        }
        enable_mmu_el1();
        super::uart_print(b"MMU ON\n");
    }

    unsafe fn install_vectors() {
        let base = &VECTORS as *const u8 as u64;
        // Try EL1 first, else EL2
        let current_el: u64; asm!("mrs {el}, CurrentEL", el = out(reg) current_el);
        match (current_el >> 2) & 0x3 { 
            1 => asm!("msr VBAR_EL1, {v}", v = in(reg) base, options(nostack, preserves_flags)),
            2 => asm!("msr VBAR_EL2, {v}", v = in(reg) base, options(nostack, preserves_flags)),
            _ => {}
        }
        asm!("isb", options(nostack, preserves_flags));
    }

    unsafe fn enable_mmu_el1() {
        // Memory attributes: AttrIdx0 = Device-nGnRE, AttrIdx1 = Normal WBWA
        let mair = (0x04u64) | (0xFFu64 << 8);
        asm!("msr MAIR_EL1, {x}", x = in(reg) mair, options(nostack, preserves_flags));

        // TCR: 4KB pages, Inner/Outer WBWA, Inner shareable, 39-bit VA (T0SZ=9), 48-bit PA (IPS=5)
        let t0sz: u64 = 48 - 39; // 9
        let tcr =
            (0b00u64 << 14) | // TG1 (unused)
            (0b00u64 << 30) | // TG0 = 4KB
            (0b11u64 << 12) | // SH0 = Inner Shareable
            (0b01u64 << 10) | // ORGN0 = WBWA
            (0b01u64 << 8)  | // IRGN0 = WBWA
            (0b101u64 << 32) | // IPS = 48-bit PA
            (t0sz & 0x3Fu64);
        asm!("msr TCR_EL1, {x}", x = in(reg) tcr, options(nostack, preserves_flags));

        // Build translation tables
        build_tables();

        // Set TTBR0 to L1 table
        let l1_pa = &L1_TABLE as *const _ as u64;
        asm!("msr TTBR0_EL1, {x}", x = in(reg) l1_pa, options(nostack, preserves_flags));
        asm!("dsb ish; isb", options(nostack, preserves_flags));

        // Enable MMU + caches
        let mut sctlr: u64;
        asm!("mrs {x}, SCTLR_EL1", x = out(reg) sctlr);
        sctlr |= (1<<0) | (1<<2) | (1<<12); // M, C, I
        asm!("msr SCTLR_EL1, {x}", x = in(reg) sctlr);
        asm!("isb", options(nostack, preserves_flags));
    }

    unsafe fn build_tables() {
        // Clear tables
        for e in L1_TABLE.iter_mut() { *e = 0; }
        for e in L2_TABLE_0.iter_mut() { *e = 0; }

        // Descriptor helpers
        const DESC_VALID: u64 = 1; // bit0
        const DESC_TABLE: u64 = 3; // bits[1:0]=11 for next-level table

        const SH_INNER: u64 = 0b11 << 8;
        const AF: u64 = 1 << 10;
        const ATTRIDX_NORMAL: u64 = 1 << 2; // AttrIndx=1
        const ATTRIDX_DEVICE: u64 = 0 << 2; // AttrIndx=0

        // L1[1] = 1GB block for 0x40000000..0x7FFFFFFF as Normal WBWA, InnerShareable
        let l1_idx_kernel = 0x4000_0000u64 >> 30; // 1
        L1_TABLE[l1_idx_kernel as usize] =
            ((0x4000_0000u64 >> 30) << 30) | // output address aligned
            (0b01) | // block descriptor
            AF | SH_INNER | ATTRIDX_NORMAL;

        // L1[0] = pointer to L2_TABLE_0 for low 1GB
        let l2_pa = &L2_TABLE_0 as *const _ as u64;
        L1_TABLE[0] = (l2_pa & !0xFFFu64) | DESC_TABLE;

        // Map UART at 0x0900_0000 as a 2MB device block in L2
        let uart_addr: u64 = 0x0900_0000;
        let l2_index = (uart_addr >> 21) & 0x1FF; // 2MB blocks
        L2_TABLE_0[l2_index as usize] =
            ((uart_addr >> 21) << 21) | // output address aligned
            (0b01) | // block descriptor (for level 2)
            AF | ATTRIDX_DEVICE; // Non-shareable is fine for device
    }

    core::arch::global_asm!(
        r#"
        .balign 2048
        .global VECTORS
    VECTORS:
        // EL1t
        b .
        b .
        b .
        b .
        // EL1h
        b .
        b .
        b .
        b .
        // EL0_64
        b .
        b .
        b .
        b .
        // EL0_32 (unused)
        b .
        b .
        b .
        b .
        "#
    );
}
