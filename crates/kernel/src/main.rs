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

    // 64 KiB bootstrap stack (16-byte aligned)
    #[repr(C, align(16))]
    struct Stack([u8; 64 * 1024]);
    static mut BOOT_STACK: Stack = Stack([0; 64 * 1024]);

    // Level-1 translation table (4 KiB aligned)
    #[repr(C, align(4096))]
    struct Table512([u64; 512]);
    static mut L1_TABLE: Table512 = Table512([0; 512]);

    // Simple EL-aware VBAR install and UART helper from outer module
    extern "C" {
        static VECTORS: u8;
    }

    pub unsafe fn run() {
        // 1) Install stack
        let sp_top = BOOT_STACK.0.as_ptr().wrapping_add(BOOT_STACK.0.len()) as u64;
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
        super::uart_print(b"MMU: MAIR/TCR\n");
        // Memory attributes: AttrIdx0 = Device-nGnRE, AttrIdx1 = Normal WBWA
        let mair = (0x04u64) | (0xFFu64 << 8);
        asm!("msr MAIR_EL1, {x}", x = in(reg) mair, options(nostack, preserves_flags));

        // TCR: 4KB pages, Inner/Outer WBWA, Inner shareable,
        // 39-bit VA (T0SZ=25), 48-bit PA (IPS=5). Correct bit positions:
        // T0SZ[5:0], IRGN0[9:8], ORGN0[11:10], SH0[13:12], TG0[15:14], IPS[34:32]
        let t0sz: u64 = 64 - 39; // 25
        let tcr =
            (t0sz & 0x3Fu64) |
            (0b01u64 << 8)  | // IRGN0 = WBWA
            (0b01u64 << 10) | // ORGN0 = WBWA
            (0b11u64 << 12) | // SH0 = Inner Shareable
            (0b00u64 << 14) | // TG0 = 4KB
            (0b101u64 << 32);  // IPS = 48-bit PA
        asm!("msr TCR_EL1, {x}", x = in(reg) tcr, options(nostack, preserves_flags));
        asm!("isb", options(nostack, preserves_flags));

        // Build translation tables
        super::uart_print(b"MMU: TABLES\n");
        build_tables();

        // Set TTBR0 to L1 table
        let l1_pa = &L1_TABLE.0 as *const _ as u64;
        super::uart_print(b"MMU: TTBR0\n");
        asm!("msr TTBR0_EL1, {x}", x = in(reg) l1_pa, options(nostack, preserves_flags));
        asm!("dsb ish; isb", options(nostack, preserves_flags));

        // Enable MMU + caches
        super::uart_print(b"MMU: SCTLR\n");
        let mut sctlr: u64;
        asm!("mrs {x}, SCTLR_EL1", x = out(reg) sctlr);
        sctlr |= (1<<0) | (1<<2) | (1<<12); // M, C, I
        asm!("msr SCTLR_EL1, {x}", x = in(reg) sctlr);
        asm!("isb", options(nostack, preserves_flags));
    }

    unsafe fn build_tables() {
        // Clear tables
        for e in L1_TABLE.0.iter_mut() { *e = 0; }

        // Descriptor helpers
        const DESC_BLOCK: u64 = 1; // bits[1:0]=01 for block

        const SH_INNER: u64 = 0b11 << 8;
        const AF: u64 = 1 << 10;
        const ATTRIDX_NORMAL: u64 = 1 << 2; // AttrIndx=1
        const ATTRIDX_DEVICE: u64 = 0 << 2; // AttrIndx=0

        // L1[1] = 1GB block for 0x40000000..0x7FFFFFFF as Normal WBWA, InnerShareable
        let l1_idx_kernel = 0x4000_0000u64 >> 30; // 1
        L1_TABLE.0[l1_idx_kernel as usize] =
            ((0x4000_0000u64 >> 30) << 30) |
            DESC_BLOCK |
            AF | SH_INNER | ATTRIDX_NORMAL;

        // L1[0] = 1GB block for 0x00000000..0x3FFFFFFF as Device-nGnRE (covers UART 0x0900_0000)
        L1_TABLE.0[0] =
            (0x0000_0000u64) |
            DESC_BLOCK |
            AF | ATTRIDX_DEVICE; // Non-shareable default
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
