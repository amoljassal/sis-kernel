#![no_std]
#![no_main]

// System call interface module
pub mod syscall;
// Userspace test module
pub mod userspace_test;
// Interactive shell module
pub mod shell;

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

#[macro_export]
macro_rules! kprint {
    ($($t:tt)*) => {{
        #[allow(unused_unsafe)]
        unsafe { crate::uart_print(format_args!($($t)*).to_string().as_bytes()); }
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($t:tt)*) => { $crate::kprint!("{}\n", format_args!($($t)*)) };
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

        // 4) Initialize GICv3 + timer and enable interrupts (skip GIC for now)
        super::uart_print(b"GIC: SKIPPED\n");
        // gicv3_init_ppi27(); // Skip problematic GIC init for QEMU
        // timer_init_1hz();   // Skip timer without GIC
        // enable_irq();       // Skip interrupt enable
        
        // 5) Test syscall functionality
        super::uart_print(b"SYSCALL TESTS\n");
        crate::userspace_test::run_syscall_tests();
        
        // 6) Launch interactive shell
        super::uart_print(b"LAUNCHING SHELL\n");
        crate::shell::run_shell();
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
        b irq_el1h
        b .
        b .
        // EL0_64 (userspace)
        b sync_el0_64
        b .
        b .
        b .
        // EL0_32 (unused)
        b .
        b .
        b .
        b .

    irq_el1h:
        bl irq_handler
        eret

    sync_el0_64:
        // Save all registers for system call
        sub sp, sp, #(34 * 8)        // Allocate SyscallFrame
        
        // Save general purpose registers x0-x30
        stp x0, x1, [sp, #(0 * 8)]
        stp x2, x3, [sp, #(2 * 8)]
        stp x4, x5, [sp, #(4 * 8)]
        stp x6, x7, [sp, #(6 * 8)]
        stp x8, x9, [sp, #(8 * 8)]
        stp x10, x11, [sp, #(10 * 8)]
        stp x12, x13, [sp, #(12 * 8)]
        stp x14, x15, [sp, #(14 * 8)]
        stp x16, x17, [sp, #(16 * 8)]
        stp x18, x19, [sp, #(18 * 8)]
        stp x20, x21, [sp, #(20 * 8)]
        stp x22, x23, [sp, #(22 * 8)]
        stp x24, x25, [sp, #(24 * 8)]
        stp x26, x27, [sp, #(26 * 8)]
        stp x28, x29, [sp, #(28 * 8)]
        str x30, [sp, #(30 * 8)]
        
        // Save EL0 stack pointer
        mrs x0, sp_el0
        str x0, [sp, #(31 * 8)]
        
        // Save exception info
        mrs x0, elr_el1
        mrs x1, spsr_el1
        stp x0, x1, [sp, #(32 * 8)]
        
        // Call system call handler
        mov x0, sp
        bl syscall_handler
        
        // Restore all registers
        ldp x0, x1, [sp, #(32 * 8)]
        msr elr_el1, x0
        msr spsr_el1, x1
        
        ldr x0, [sp, #(31 * 8)]
        msr sp_el0, x0
        
        // Restore GPRs
        ldp x0, x1, [sp, #(0 * 8)]
        ldp x2, x3, [sp, #(2 * 8)]
        ldp x4, x5, [sp, #(4 * 8)]
        ldp x6, x7, [sp, #(6 * 8)]
        ldp x8, x9, [sp, #(8 * 8)]
        ldp x10, x11, [sp, #(10 * 8)]
        ldp x12, x13, [sp, #(12 * 8)]
        ldp x14, x15, [sp, #(14 * 8)]
        ldp x16, x17, [sp, #(16 * 8)]
        ldp x18, x19, [sp, #(18 * 8)]
        ldp x20, x21, [sp, #(20 * 8)]
        ldp x22, x23, [sp, #(22 * 8)]
        ldp x24, x25, [sp, #(24 * 8)]
        ldp x26, x27, [sp, #(26 * 8)]
        ldp x28, x29, [sp, #(28 * 8)]
        ldr x30, [sp, #(30 * 8)]
        
        add sp, sp, #(34 * 8)        // Restore stack
        eret
        "#
    );

    #[no_mangle]
    extern "C" fn irq_handler() {
        unsafe {
            let mut irq: u64;
            asm!("mrs {x}, icc_iar1_el1", x = out(reg) irq);
            super::uart_print(b"tick\n");
            // reload timer for ~1s
            let mut frq: u64;
            asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
            asm!("msr cntv_tval_el0, {x}", x = in(reg) frq);
            // signal end of interrupt
            asm!("msr icc_eoir1_el1, {x}", x = in(reg) irq);
            asm!("msr icc_dir_el1, {x}", x = in(reg) irq);
        }
    }

    unsafe fn enable_irq() {
        // Unmask IRQs in PSTATE
        asm!("msr daifclr, #2", options(nostack, preserves_flags));
    }

    unsafe fn timer_init_1hz() {
        let mut frq: u64;
        asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        // Set initial interval ~1s
        asm!("msr cntv_tval_el0, {x}", x = in(reg) frq);
        // Enable virtual timer, unmask
        let ctl: u64 = 1; // ENABLE=1, IMASK=0
        asm!("msr cntv_ctl_el0, {x}", x = in(reg) ctl);
    }

    unsafe fn gicv3_init_ppi27() {
        super::uart_print(b"GIC: INIT\n");
        // Wake redistributor for CPU0 and enable PPI ID27 (virtual timer)
        const GICR_BASE: u64 = 0x080A_0000;
        const GICR_WAKER: u64 = 0x0014;
        const GICR_IGROUPR0: u64 = 0x0080;
        const GICR_ISENABLER0: u64 = 0x0100;
        const GICR_IPRIORITYR: u64 = 0x0400;

        let waker = (GICR_BASE + GICR_WAKER) as *mut u32;
        // Clear ProcessorSleep bit [1]
        super::uart_print(b"GIC: WAKER READ\n");
        let mut w: u32 = core::ptr::read_volatile(waker);
        super::uart_print(b"GIC: WAKER WRITE\n");
        w &= !(1 << 1);
        core::ptr::write_volatile(waker, w);
        // Wait for ChildrenAsleep bit [2] to clear with timeout
        super::uart_print(b"GIC: WAIT LOOP\n");
        let mut timeout = 1000000;
        loop {
            let v = core::ptr::read_volatile(waker);
            if (v & (1 << 2)) == 0 { 
                super::uart_print(b"GIC: WAKER OK\n");
                break; 
            }
            timeout -= 1;
            if timeout == 0 {
                super::uart_print(b"GIC: WAKER TIMEOUT\n");
                break;
            }
        }

        // Group 1 (non-secure)
        super::uart_print(b"GIC: GROUP\n");
        let igroupr0 = (GICR_BASE + GICR_IGROUPR0) as *mut u32;
        let mut grp = core::ptr::read_volatile(igroupr0);
        grp |= 1 << 27;
        core::ptr::write_volatile(igroupr0, grp);

        // Priority for all PPIs
        super::uart_print(b"GIC: PRIORITY\n");
        let iprio = (GICR_BASE + GICR_IPRIORITYR) as *mut u32;
        for i in 0..8 { // 32 PPIs priorities (4 per u32)
            core::ptr::write_volatile(iprio.add(i), 0x80808080);
        }

        // Enable PPI 27
        super::uart_print(b"GIC: ENABLE\n");
        let isenabler0 = (GICR_BASE + GICR_ISENABLER0) as *mut u32;
        core::ptr::write_volatile(isenabler0, 1 << 27);

        // CPU interface via system registers
        super::uart_print(b"GIC: CPU IF\n");
        let pmr: u64 = 0xFF; // unmask all priorities
        asm!("msr icc_pmr_el1, {x}", x = in(reg) pmr);
        let grp1: u64 = 1;
        asm!("msr icc_igrpen1_el1, {x}", x = in(reg) grp1);
        asm!("isb", options(nostack, preserves_flags));
        super::uart_print(b"GIC: DONE\n");
    }
}
