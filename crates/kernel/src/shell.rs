//! Simple interactive shell for SIS kernel
//!
//! Provides basic command-line interface functionality with built-in commands.
//! Demonstrates userspace-like interaction through the syscall interface.

use crate::syscall::{SyscallError, SyscallNumber};
use core::arch::asm;
use alloc::format;

/// Maximum command line length
const MAX_CMD_LEN: usize = 256;

/// Shell command buffer
static mut CMD_BUFFER: [u8; MAX_CMD_LEN] = [0; MAX_CMD_LEN];

/// Shell prompt
const SHELL_PROMPT: &[u8] = b"sis> ";

/// Simple shell implementation
pub struct Shell {
    running: bool,
}

impl Shell {
    /// Create new shell instance
    pub fn new() -> Self {
        Shell { running: true }
    }

    /// Main shell loop
    pub fn run(&mut self) {
        unsafe {
            crate::uart_print(b"\n=== SIS Kernel Shell ===\n");
            crate::uart_print(b"Type 'help' for available commands\n\n");
        }

        while self.running {
            self.print_prompt();

            // Read real user input from UART
            let cmd_len = self.read_command_input();

            if cmd_len > 0 {
                self.process_command(cmd_len);
            }

            if !self.running {
                break;
            }
        }

        unsafe {
            crate::uart_print(b"Shell terminated\n");
        }
    }

    /// Print shell prompt
    fn print_prompt(&self) {
        unsafe {
            crate::uart_print(SHELL_PROMPT);
        }
    }

    /// Read command input from UART with line editing
    fn read_command_input(&mut self) -> usize {
        unsafe {
            let buffer_ptr = &raw mut CMD_BUFFER;
            let len = crate::uart::read_line(&mut *buffer_ptr);

            // Null terminate the command
            if len < MAX_CMD_LEN {
                (*buffer_ptr)[len] = 0;
            }

            len
        }
    }

    /// Process a command
    fn process_command(&mut self, cmd_len: usize) {
        if cmd_len == 0 {
            return;
        }

        // Runtime verification hook for shell command processing
        #[cfg(target_arch = "riscv64")]
        {
            use crate::arch::riscv64::verification::CriticalOperation;
            crate::verify_lightweight!(CriticalOperation::ShellCommand, "shell_command_process");
        }

        unsafe {
            let cmd_str = core::str::from_utf8_unchecked(&CMD_BUFFER[..cmd_len]);
            let parts: heapless::Vec<&str, 8> = cmd_str.split_whitespace().collect();

            if parts.is_empty() {
                return;
            }

            match parts[0] {
                "help" => self.cmd_help(),
                "echo" => self.cmd_echo(&parts[1..]),
                "info" => self.cmd_info(),
                "test" => self.cmd_test(),
                "perf" => self.cmd_perf(),
                "bench" => self.cmd_bench(),
                "stress" => self.cmd_stress(),
                "overhead" => self.cmd_overhead(),
                "graphdemo" => self.cmd_graph_demo(),
                "detdemo" => self.cmd_deterministic_demo(),
                "aidemo" => self.cmd_ai_scheduler_demo(),
                "cbsdemo" => self.cmd_cbs_budget_demo(),
                "mldemo" => self.cmd_ml_demo(),
                "infdemo" => self.cmd_inference_demo(),
                "npudemo" => self.cmd_npu_demo(),
                "npudriver" => self.cmd_npu_driver_demo(),
                "graphctl" => self.cmd_graphctl(&parts[1..]),
                "ctlhex" => self.cmd_ctlhex(&parts[1..]),
                "pmu" => self.cmd_pmu_demo(),
                "mem" => self.cmd_mem(),
                "regs" => self.cmd_regs(),
                "dtb" => self.cmd_dtb(),
                "vector" => self.cmd_vector(),
                "board" => self.cmd_board(),
                "verify" => self.cmd_verify(),
                "perf_test" => self.cmd_perf_test(),
                "ai_bench" => self.cmd_ai_bench(),
                "clear" => self.cmd_clear(),
                "exit" => self.cmd_exit(),
                _ => self.cmd_unknown(parts[0]),
            }
        }
    }

    /// Help command
    fn cmd_help(&self) {
        unsafe {
            crate::uart_print(b"Available commands:\n");
            crate::uart_print(b"  help     - Show this help message\n");
            crate::uart_print(b"  echo     - Echo text to output\n");
            crate::uart_print(b"  info     - Show kernel information\n");
            crate::uart_print(b"  test     - Run syscall tests\n");
            crate::uart_print(b"  perf     - Show performance metrics report\n");
            crate::uart_print(b"  bench    - Run syscall performance benchmarks\n");
            crate::uart_print(b"  stress   - Run syscall stress tests\n");
            crate::uart_print(b"  overhead - Measure syscall overhead\n");
            crate::uart_print(b"  graphdemo- Run graph demo (feature: graph-demo)\n");
            crate::uart_print(b"  detdemo  - Run deterministic scheduler demo (feature: deterministic)\n");
            crate::uart_print(b"  aidemo   - Run AI-enhanced scheduler demo with real-time inference\n");
            crate::uart_print(b"  cbsdemo  - Run CBS+EDF budget management demo for AI inference\n");
            crate::uart_print(b"  mldemo   - Run Phase 3 TinyML demo (AI inference)\n");
            crate::uart_print(b"  infdemo  - Run deterministic inference demo (cycle-accurate)\n");
            crate::uart_print(b"  npudemo  - Run NPU device emulation demo (MMIO/IRQ)\n");
            crate::uart_print(b"  npudriver- Run NPU driver demo with interrupt handling\n");
            crate::uart_print(b"  graphctl - Control graph: create | add-channel <cap> | add-operator <op_id> [--in N|none] [--out N|none] [--prio P] [--stage acquire|clean|explore|model|explain] | start <steps> | stats\n");
            crate::uart_print(b"  ctlhex   - Inject control frame as hex (Create/Add/Start)\n");
            crate::uart_print(b"  pmu      - Run PMU demo (cycles/inst/l1d_refill)\n");
            crate::uart_print(b"  mem      - Show memory information\n");
            crate::uart_print(b"  regs     - Show system registers\n");
            crate::uart_print(b"  dtb      - Show device tree information\n");
            crate::uart_print(b"  vector   - Show vector extension information\n");
            crate::uart_print(b"  board    - Show board-specific information\n");
            crate::uart_print(b"  verify   - Run comprehensive verification tests (property-based, metamorphic)\n");
            crate::uart_print(b"  perf_test- Run RISC-V performance optimization tests\n");
            crate::uart_print(b"  ai_bench - Run AI/ML benchmarks (AI mode only)\n");
            crate::uart_print(b"  clear    - Clear screen\n");
            crate::uart_print(b"  exit     - Exit shell\n");
        }
    }

    /// Echo command
    fn cmd_echo(&self, args: &[&str]) {
        unsafe {
            if args.is_empty() {
                crate::uart_print(b"\n");
            } else {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        crate::uart_print(b" ");
                    }
                    crate::uart_print(arg.as_bytes());
                }
                crate::uart_print(b"\n");
            }
        }
    }

    /// Info command
    fn cmd_info(&self) {
        unsafe {
            crate::uart_print(b"SIS Kernel Information:\n");
            crate::uart_print(b"  Architecture: ARM64 (AArch64)\n");
            crate::uart_print(b"  Boot Method: UEFI\n");
            crate::uart_print(b"  MMU Status: Enabled\n");
            crate::uart_print(b"  Syscalls: 13 POSIX-compatible\n");
            crate::uart_print(b"  Exception Level: EL1\n");

            // Get current PID via syscall
            match self.syscall_getpid() {
                Ok(pid) => {
                    crate::uart_print(b"  Current PID: ");
                    self.print_number(pid as u64);
                    crate::uart_print(b"\n");
                }
                Err(_) => {
                    crate::uart_print(b"  Current PID: Error\n");
                }
            }
        }
    }

    /// Test command
    fn cmd_test(&self) {
        unsafe {
            crate::uart_print(b"Running syscall tests...\n");
        }
        crate::userspace_test::run_syscall_tests();
    }

    /// Performance metrics report command
    fn cmd_perf(&self) {
        crate::syscall::print_syscall_performance_report();
    }

    /// Performance benchmarks command
    fn cmd_bench(&self) {
        crate::userspace_test::run_syscall_performance_tests();
    }

    /// Stress test command
    fn cmd_stress(&self) {
        crate::userspace_test::run_syscall_stress_test();
    }

    /// Syscall overhead measurement command
    fn cmd_overhead(&self) {
        crate::userspace_test::measure_syscall_overhead();
    }

    /// Graph demo command
    fn cmd_graph_demo(&self) {
        unsafe { crate::uart_print(b"[GRAPH] Running demo (64 items)\n"); }
        let mut demo = crate::graph::GraphDemo::new(64);
        demo.run();
        unsafe { crate::uart_print(b"[GRAPH] Demo complete\n"); }
    }

    /// Phase 2 deterministic scheduler demo command
    fn cmd_deterministic_demo(&self) {
        #[cfg(feature = "deterministic")]
        {
            unsafe { crate::uart_print(b"[DETERMINISTIC] Running Phase 2 comprehensive demo\n"); }
            crate::graph::deterministic_demo();
            unsafe { crate::uart_print(b"[DETERMINISTIC] Demo complete\n"); }
        }
        #[cfg(not(feature = "deterministic"))]
        {
            unsafe { crate::uart_print(b"[DETERMINISTIC] Requires 'deterministic' feature\n"); }
        }
    }

    fn cmd_ml_demo(&self) {
        unsafe { crate::uart_print(b"[ML] Running Phase 3 TinyML demonstration\n"); }
        crate::ml::ml_demo();
        unsafe { crate::uart_print(b"[ML] Phase 3 demonstration complete\n"); }
    }

    fn cmd_inference_demo(&self) {
        unsafe { crate::uart_print(b"[INFERENCE] Running deterministic inference demonstration\n"); }
        crate::inference::deterministic_inference_demo();
        unsafe { crate::uart_print(b"[INFERENCE] Deterministic inference demonstration complete\n"); }
    }

    fn cmd_npu_demo(&self) {
        unsafe { crate::uart_print(b"[NPU] Running NPU device emulation demonstration\n"); }
        crate::npu::npu_demo();
        unsafe { crate::uart_print(b"[NPU] NPU device emulation demonstration complete\n"); }
    }

    /// NPU driver demo command (MMIO interface and interrupt handling)
    fn cmd_npu_driver_demo(&self) {
        unsafe { crate::uart_print(b"[NPU DRIVER] Running NPU driver demonstration with interrupt handling\n"); }
        npu_driver_demo();
        unsafe { crate::uart_print(b"[NPU DRIVER] NPU driver demonstration complete\n"); }
    }

    /// AI-enhanced scheduler demo command
    fn cmd_ai_scheduler_demo(&self) {
        #[cfg(feature = "deterministic")]
        {
            unsafe { crate::uart_print(b"[AI SCHEDULER] Running AI-enhanced deterministic scheduler demonstration\n"); }
            crate::deterministic::ai_scheduler_demo();
            unsafe { crate::uart_print(b"[AI SCHEDULER] AI-enhanced scheduler demonstration complete\n"); }
        }
        #[cfg(not(feature = "deterministic"))]
        {
            unsafe { crate::uart_print(b"[AI SCHEDULER] AI scheduler demo requires 'deterministic' feature\n"); }
        }
    }

    /// CBS budget management demo command
    fn cmd_cbs_budget_demo(&self) {
        #[cfg(feature = "deterministic")]
        {
            unsafe { crate::uart_print(b"[CBS BUDGET] Running CBS+EDF AI inference budget management demonstration\n"); }
            crate::deterministic::cbs_ai_budget_demo();
            unsafe { crate::uart_print(b"[CBS BUDGET] CBS+EDF budget management demonstration complete\n"); }
        }
        #[cfg(not(feature = "deterministic"))]
        {
            unsafe { crate::uart_print(b"[CBS BUDGET] CBS budget demo requires 'deterministic' feature\n"); }
        }
    }

    /// PMU demo command (cycles, instructions, L1D refills)
    fn cmd_pmu_demo(&self) {
        #[cfg(feature = "perf-verbose")]
        {
            unsafe { crate::uart_print(b"[PMU] Demo: setup events and run busy loop\n"); }
            unsafe { crate::pmu::aarch64::setup_events(); }
            let s0 = unsafe { crate::pmu::aarch64::read_snapshot() };

            // Busy loop: arithmetic + memory touches
            let mut acc: u64 = 0;
            let mut buf: [u64; 128] = [0; 128];
            for i in 0..8192 {
                acc = acc.wrapping_mul(6364136223846793005).wrapping_add(1);
                let idx = (i & 127) as usize;
                buf[idx] = buf[idx].wrapping_add(acc ^ (i as u64));
            }
            unsafe { core::ptr::read_volatile(&acc); }

            let s1 = unsafe { crate::pmu::aarch64::read_snapshot() };
            let d_cycles = s1.cycles.saturating_sub(s0.cycles);
            let d_inst = s1.inst.saturating_sub(s0.inst);
            let d_l1d = s1.l1d_refill.saturating_sub(s0.l1d_refill);
            unsafe {
                crate::uart_print(b"METRIC pmu_cycles="); self.print_number(d_cycles);
                crate::uart_print(b"\nMETRIC pmu_inst="); self.print_number(d_inst);
                crate::uart_print(b"\nMETRIC pmu_l1d_refill="); self.print_number(d_l1d);
                crate::uart_print(b"\n");
            }
            if d_inst == 0 {
                unsafe { crate::uart_print(b"[PMU] Note: instructions counter may be unsupported in this QEMU build\n"); }
            }
        }
        #[cfg(not(feature = "perf-verbose"))]
        unsafe {
            crate::uart_print(b"[PMU] perf-verbose feature not enabled\n");
        }
    }

    /// Inject a control-plane frame as hex (V0 framing: 'C', ver, cmd, flags, len, payload)
    fn cmd_ctlhex(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: ctlhex <hex>\n"); }
            return;
        }
        let s = args[0].trim();
        let bytes = s.as_bytes();
        let mut buf = [0u8; 256];
        let mut bi = 0usize;
        let mut i = 0usize;
        while i + 1 < bytes.len() && bi < buf.len() {
            let hn = match bytes[i] {
                b'0'..=b'9' => bytes[i] - b'0',
                b'a'..=b'f' => bytes[i] - b'a' + 10,
                b'A'..=b'F' => bytes[i] - b'A' + 10,
                _ => 0xFF,
            };
            let ln = match bytes[i + 1] {
                b'0'..=b'9' => bytes[i + 1] - b'0',
                b'a'..=b'f' => bytes[i + 1] - b'a' + 10,
                b'A'..=b'F' => bytes[i + 1] - b'A' + 10,
                _ => 0xFF,
            };
            if hn > 15 || ln > 15 {
                unsafe { crate::uart_print(b"[CTL] invalid hex\n"); }
                return;
            }
            buf[bi] = ((hn as u8) << 4) | (ln as u8);
            bi += 1;
            i += 2;
        }
        match crate::control::handle_frame(&buf[..bi]) {
            Ok(()) => unsafe { crate::uart_print(b"[CTL] ok\n"); }
            Err(_) => unsafe { crate::uart_print(b"[CTL] error\n"); }
        }
    }

    /// Graph control convenience command
    fn cmd_graphctl(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: graphctl <create|add-channel|add-operator|start> ...\n"); }
            return;
        }

        // Helper to send a framed control message
        fn send_frame(cmd: u8, payload: &[u8]) -> bool {
            let mut buf = [0u8; 64];
            let total = 8 + payload.len();
            if total > buf.len() { unsafe { crate::uart_print(b"[CTL] payload too large\n"); } return false; }
            buf[0] = 0x43; // 'C'
            buf[1] = 0;    // ver
            buf[2] = cmd;  // cmd
            buf[3] = 0;    // flags
            let len = payload.len() as u32;
            let le = len.to_le_bytes();
            buf[4] = le[0]; buf[5] = le[1]; buf[6] = le[2]; buf[7] = le[3];
            for i in 0..payload.len() { buf[8 + i] = payload[i]; }
            match crate::control::handle_frame(&buf[..total]) {
                Ok(()) => unsafe { crate::uart_print(b"[CTL] ok\n"); true },
                Err(_) => unsafe { crate::uart_print(b"[CTL] error\n"); false },
            }
        }

        match args[0] {
            "create" => {
                let _ = send_frame(0x01, &[]);
            }
            "add-channel" => {
                if args.len() < 2 { unsafe { crate::uart_print(b"Usage: graphctl add-channel <capacity>\n"); } return; }
                if let Ok(cap) = args[1].parse::<u32>() {
                    if cap == 0 || cap > 65535 { unsafe { crate::uart_print(b"[CTL] capacity must be 1..65535\n"); } return; }
                    let cap_le = (cap as u16).to_le_bytes();
                    let payload = [cap_le[0], cap_le[1]];
                    let _ = send_frame(0x02, &payload);
                } else {
                    unsafe { crate::uart_print(b"[CTL] invalid capacity\n"); }
                }
            }
            "add-operator" => {
                if args.len() < 2 { unsafe { crate::uart_print(b"Usage: graphctl add-operator <op_id> [--in N|none] [--out N|none] [--prio P] [--stage acquire|clean|explore|model|explain]\n"); } return; }
                let op_id = match args[1].parse::<u32>() { Ok(v) => v, Err(_) => { unsafe { crate::uart_print(b"[CTL] invalid op_id\n"); } return; } };
                let mut in_ch: Option<u16> = None;
                let mut out_ch: Option<u16> = None;
                let mut prio: u8 = 10;
                let mut stage: u8 = 0; // acquire

                let mut i = 2;
                while i < args.len() {
                    match args[i] {
                        "--in" => {
                            i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --in requires a value\n"); } return; }
                            let v = args[i];
                            if v.eq_ignore_ascii_case("none") { in_ch = None; } else if let Ok(n) = v.parse::<u32>() { if n <= 0xFFFF { in_ch = Some(n as u16); } else { unsafe { crate::uart_print(b"[CTL] --in out of range\n"); } return; } } else { unsafe { crate::uart_print(b"[CTL] invalid --in\n"); } return; }
                        }
                        "--out" => {
                            i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --out requires a value\n"); } return; }
                            let v = args[i];
                            if v.eq_ignore_ascii_case("none") { out_ch = None; } else if let Ok(n) = v.parse::<u32>() { if n <= 0xFFFF { out_ch = Some(n as u16); } else { unsafe { crate::uart_print(b"[CTL] --out out of range\n"); } return; } } else { unsafe { crate::uart_print(b"[CTL] invalid --out\n"); } return; }
                        }
                        "--prio" | "--priority" => {
                            i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --prio requires a value\n"); } return; }
                            match args[i].parse::<u32>() { Ok(n) if n <= 255 => prio = n as u8, _ => { unsafe { crate::uart_print(b"[CTL] invalid --prio\n"); } return; } }
                        }
                        "--stage" => {
                            i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --stage requires a value\n"); } return; }
                            stage = match args[i] {
                                "acquire" => 0,
                                "clean" => 1,
                                "explore" => 2,
                                "model" => 3,
                                "explain" => 4,
                                _ => { unsafe { crate::uart_print(b"[CTL] invalid stage (use acquire|clean|explore|model|explain)\n"); } return; }
                            };
                        }
                        _ => { unsafe { crate::uart_print(b"[CTL] unknown option\n"); } return; }
                    }
                    i += 1;
                }

                let op_le = op_id.to_le_bytes();
                let in_le = match in_ch { Some(v) => v.to_le_bytes(), None => 0xFFFFu16.to_le_bytes() };
                let out_le = match out_ch { Some(v) => v.to_le_bytes(), None => 0xFFFFu16.to_le_bytes() };
                let payload = [op_le[0],op_le[1],op_le[2],op_le[3], in_le[0],in_le[1], out_le[0],out_le[1], prio, stage];
                let _ = send_frame(0x03, &payload);
            }
            "start" => {
                if args.len() < 2 { unsafe { crate::uart_print(b"Usage: graphctl start <steps>\n"); } return; }
                if let Ok(steps) = args[1].parse::<u32>() {
                    let le = steps.to_le_bytes();
                    let payload = [le[0], le[1], le[2], le[3]];
                    let _ = send_frame(0x04, &payload);
                } else {
                    unsafe { crate::uart_print(b"[CTL] invalid steps\n"); }
                }
            }
            "stats" => {
                // Print a concise summary and METRICs for graph structure
                if let Some((ops, chans)) = crate::control::current_graph_counts() {
                    unsafe {
                        crate::uart_print(b"GRAPH: counts ops="); self.print_number(ops as u64); crate::uart_print(b" channels="); self.print_number(chans as u64); crate::uart_print(b"\n");
                        crate::uart_print(b"METRIC graph_stats_ops="); self.print_number(ops as u64); crate::uart_print(b"\n");
                        crate::uart_print(b"METRIC graph_stats_channels="); self.print_number(chans as u64); crate::uart_print(b"\n");
                    }
                } else {
                    unsafe { crate::uart_print(b"GRAPH: no active graph\n"); }
                }
            }
            _ => unsafe { crate::uart_print(b"Usage: graphctl <create|add-channel|add-operator|start> ...\n"); }
        }
    }


    /// AI benchmark command
    fn cmd_ai_bench(&self) {
        #[cfg(feature = "arm64-ai")]
        {
            unsafe {
                crate::uart_print(b"Running AI/ML benchmarks...\n");
            }
            crate::ai_benchmark::run_ai_benchmarks();
        }
        
        #[cfg(not(feature = "arm64-ai"))]
        {
            unsafe {
                crate::uart_print(b"AI benchmarks are only available when AI features are enabled.\n");
                crate::uart_print(b"Run with AI=1 environment variable to enable AI features.\n");
            }
        }
    }
    
    /// Exit command
    fn cmd_exit(&mut self) {
        unsafe {
            crate::uart_print(b"Goodbye!\n");
        }
        self.running = false;
    }

    /// Memory information command
    fn cmd_mem(&self) {
        unsafe {
            crate::uart_print(b"Memory Information:\n");
            crate::uart_print(b"  Kernel loaded at: 0x40080000\n");
            crate::uart_print(b"  MMU Status: Enabled (39-bit VA)\n");
            crate::uart_print(b"  Page Size: 4KB\n");
            crate::uart_print(b"  Address Space Layout:\n");
            crate::uart_print(b"    0x00000000-0x3FFFFFFF: Device Memory\n");
            crate::uart_print(b"    0x40000000-0x7FFFFFFF: Normal Memory\n");
            crate::uart_print(b"    UART Base: 0x09000000\n");
        }
    }

    /// System registers command  
    fn cmd_regs(&self) {
        use core::arch::asm;

        unsafe {
            #[cfg(target_arch = "aarch64")]
            {
                crate::uart_print(b"ARM64 System Registers:\n");

                let mut reg_val: u64;

                // Current Exception Level
                asm!("mrs {}, CurrentEL", out(reg) reg_val);
                crate::uart_print(b"  CurrentEL: ");
                self.print_hex(reg_val);
                crate::uart_print(b" (EL");
                self.print_number((reg_val >> 2) & 0x3);
                crate::uart_print(b")\n");

                // Main ID Register
                asm!("mrs {}, MIDR_EL1", out(reg) reg_val);
                crate::uart_print(b"  MIDR_EL1:  ");
                self.print_hex(reg_val);
                crate::uart_print(b"\n");

                // System Control Register
                asm!("mrs {}, SCTLR_EL1", out(reg) reg_val);
                crate::uart_print(b"  SCTLR_EL1: ");
                self.print_hex(reg_val);
                crate::uart_print(b" (MMU=");
                self.print_number(reg_val & 1);
                crate::uart_print(b")\n");

                // Translation Control Register
                asm!("mrs {}, TCR_EL1", out(reg) reg_val);
                crate::uart_print(b"  TCR_EL1:   ");
                self.print_hex(reg_val);
            }

            #[cfg(target_arch = "riscv64")]
            {
                crate::uart_print(b"RISC-V System Registers:\n");

                let mut reg_val: u64;

                // Machine Status Register
                asm!("csrr {}, sstatus", out(reg) reg_val);
                crate::uart_print(b"  sstatus:   ");
                self.print_hex(reg_val);
                crate::uart_print(b"\n");

                // Supervisor Trap Vector
                asm!("csrr {}, stvec", out(reg) reg_val);
                crate::uart_print(b"  stvec:     ");
                self.print_hex(reg_val);
                crate::uart_print(b"\n");

                // Supervisor Address Translation and Protection
                asm!("csrr {}, satp", out(reg) reg_val);
                crate::uart_print(b"  satp:      ");
                self.print_hex(reg_val);
                crate::uart_print(b" (MMU=");
                self.print_number((reg_val >> 60) & 0xF);
                crate::uart_print(b")\n");

                // Hart ID (if available)
                asm!("csrr {}, mhartid", out(reg) reg_val);
                crate::uart_print(b"  mhartid:   ");
                self.print_hex(reg_val);
            }
            crate::uart_print(b"\n");
        }
    }

    /// Device tree information command  
    fn cmd_dtb(&self) {
        #[cfg(target_arch = "riscv64")]
        {
            crate::arch::riscv64::dtb::print_dtb_info();
        }
        
        #[cfg(not(target_arch = "riscv64"))]
        {
            unsafe {
                crate::uart_print(b"Device tree parsing only supported on RISC-V\n");
            }
        }
    }

    /// Vector extension information command  
    fn cmd_vector(&self) {
        #[cfg(target_arch = "riscv64")]
        {
            crate::arch::riscv64::vector::print_vector_info();
        }
        
        #[cfg(not(target_arch = "riscv64"))]
        {
            unsafe {
                crate::uart_print(b"Vector extension only supported on RISC-V\n");
            }
        }
    }

    /// Board information command
    fn cmd_board(&self) {
        #[cfg(target_arch = "riscv64")]
        {
            crate::arch::riscv64::boards::vikram3201::print_board_info();
        }
        
        #[cfg(not(target_arch = "riscv64"))]
        {
            unsafe {
                crate::uart_print(b"Board-specific information only supported on RISC-V\n");
            }
        }
    }

    /// Formal verification status command
    fn cmd_verify(&self) {
        #[cfg(target_arch = "riscv64")]
        {
            crate::arch::riscv64::verification::print_verification_status();
            
            unsafe {
                crate::uart_print(b"\nRunning basic verification check...\n");
            }
            
            if let Some(verifier) = crate::arch::riscv64::verification::get_verifier() {
                match verifier.check_invariants() {
                    Ok(_) => unsafe {
                        crate::uart_print(b"[OK] Basic invariants satisfied\n");
                    },
                    Err(_) => unsafe {
                        crate::uart_print(b"[ERR] Basic invariant violations detected\n");
                    },
                }
            }

            // Run comprehensive property-based testing
            unsafe {
                crate::uart_print(b"\nRunning property-based testing suite...\n");
            }
            let invariant_tests_passed = crate::arch::riscv64::verification::run_comprehensive_invariant_tests();
            
            // Run metamorphic testing
            let metamorphic_tests_passed = crate::arch::riscv64::verification::run_metamorphic_tests();
            
            // Run advanced invariant checking
            let advanced_tests_passed = crate::arch::riscv64::verification::run_advanced_invariant_checking();

            // Display runtime verification hook statistics
            crate::arch::riscv64::verification::print_verification_hook_stats();

            // Summary
            unsafe {
                crate::uart_print(b"\n=== Verification Summary ===\n");
                crate::uart_print(b"Invariant Tests: ");
                if invariant_tests_passed {
                    crate::uart_print(b"[PASS]\n");
                } else {
                    crate::uart_print(b"[FAIL]\n");
                }

                crate::uart_print(b"Metamorphic Tests: ");
                if metamorphic_tests_passed {
                    crate::uart_print(b"[PASS]\n");
                } else {
                    crate::uart_print(b"[FAIL]\n");
                }

                crate::uart_print(b"Advanced Tests: ");
                if advanced_tests_passed {
                    crate::uart_print(b"[PASS]\n");
                } else {
                    crate::uart_print(b"[FAIL]\n");
                }

                if invariant_tests_passed && metamorphic_tests_passed && advanced_tests_passed {
                    crate::uart_print(b"\n[OVERALL] All verification tests passed!\n");
                } else {
                    crate::uart_print(b"\n[OVERALL] Some verification tests failed.\n");
                }
            }
        }
        
        #[cfg(not(target_arch = "riscv64"))]
        {
            unsafe {
                crate::uart_print(b"Formal verification only supported on RISC-V\n");
            }
        }
    }

    /// Performance optimization test command
    fn cmd_perf_test(&self) {
        #[cfg(target_arch = "riscv64")]
        {
            unsafe {
                crate::uart_print(b"\n=== RISC-V Performance Optimization Tests ===\n");
                
                // Test 1: Cache-optimized memory operations
                crate::uart_print(b"\n1. Testing cache-optimized memory operations:\n");
                self.test_memory_operations();
                
                // Test 2: RISC-V instruction optimizations
                crate::uart_print(b"\n2. Testing RISC-V instruction optimizations:\n");
                self.test_instruction_optimizations();
                
                // Test 3: Cache-friendly algorithms
                crate::uart_print(b"\n3. Testing cache-friendly algorithms:\n");
                self.test_cache_algorithms();
                
                // Test 4: Performance profiling
                crate::uart_print(b"\n4. Performance profiling demonstration:\n");
                self.test_performance_profiling();
                
                crate::uart_print(b"\n[PERF] All performance optimization tests completed!\n");
            }
        }
        
        #[cfg(not(target_arch = "riscv64"))]
        {
            unsafe {
                crate::uart_print(b"Performance optimization tests only supported on RISC-V\n");
            }
        }
    }

    /// Test memory operations
    #[cfg(target_arch = "riscv64")]
    fn test_memory_operations(&self) {
        use crate::arch::riscv64::performance::memory_ops::*;
        
        const TEST_SIZE: usize = 1024;
        let mut source = [0u8; TEST_SIZE];
        let mut dest = [0u8; TEST_SIZE];
        let mut buffer = [0u8; TEST_SIZE];
        
        // Initialize test data
        for i in 0..TEST_SIZE {
            source[i] = (i % 256) as u8;
        }
        
        unsafe {
            // Test optimized memcpy
            let counter = crate::arch::riscv64::performance::PerformanceCounter::start("optimized_memcpy");
            optimized_memcpy(dest.as_mut_ptr(), source.as_ptr(), TEST_SIZE);
            let result = counter.stop();
            result.print();
            
            // Test optimized memset
            let counter = crate::arch::riscv64::performance::PerformanceCounter::start("optimized_memset");
            optimized_memset(buffer.as_mut_ptr(), 0xAA, TEST_SIZE);
            let result = counter.stop();
            result.print();
            
            // Test optimized memcmp
            let counter = crate::arch::riscv64::performance::PerformanceCounter::start("optimized_memcmp");
            let cmp_result = optimized_memcmp(source.as_ptr(), dest.as_ptr(), TEST_SIZE);
            let result = counter.stop();
            result.print();
            
            crate::uart_print(b"  Memory comparison result: ");
            if cmp_result == 0 {
                crate::uart_print(b"EQUAL (correct)\n");
            } else {
                crate::uart_print(b"NOT EQUAL (unexpected)\n");
            }
        }
    }

    /// Test instruction optimizations
    #[cfg(target_arch = "riscv64")]
    fn test_instruction_optimizations(&self) {
        use crate::arch::riscv64::performance::instruction_opt::*;
        
        unsafe {
            // Test fast square root
            let test_values = [16u32, 64, 100, 256, 1024];
            crate::uart_print(b"  Fast square root tests:\n");
            for &value in &test_values {
                let counter = crate::arch::riscv64::performance::PerformanceCounter::start("fast_sqrt");
                let sqrt_result = fast_sqrt_u32(value);
                let result = counter.stop();
                
                crate::uart_print(b"    sqrt(");
                self.print_number(value as u64);
                crate::uart_print(b") = ");
                self.print_number(sqrt_result as u64);
                crate::uart_print(b" (");
                print_u64_simple(result.cycles);
                crate::uart_print(b" cycles)\n");
            }
            
            // Test population count
            let test_values = [0x0Fu64, 0xF0F0, 0xFFFF, 0xAAAAAAAA, 0xFFFFFFFFFFFFFFFF];
            crate::uart_print(b"  Population count tests:\n");
            for &value in &test_values {
                let counter = crate::arch::riscv64::performance::PerformanceCounter::start("popcount");
                let pop_result = popcount_u64(value);
                let result = counter.stop();
                
                crate::uart_print(b"    popcount(0x");
                self.print_hex_simple(value);
                crate::uart_print(b") = ");
                self.print_number(pop_result as u64);
                crate::uart_print(b" (");
                print_u64_simple(result.cycles);
                crate::uart_print(b" cycles)\n");
            }
        }
    }

    /// Test cache-friendly algorithms
    #[cfg(target_arch = "riscv64")]
    fn test_cache_algorithms(&self) {
        const ARRAY_SIZE: usize = 256;
        let mut test_array = [0u32; ARRAY_SIZE];
        
        // Initialize with reverse-sorted data
        for i in 0..ARRAY_SIZE {
            test_array[i] = (ARRAY_SIZE - i) as u32;
        }
        
        unsafe {
            crate::uart_print(b"  Cache-friendly sorting test:\n");
            let counter = crate::arch::riscv64::performance::PerformanceCounter::start("cache_friendly_sort");
            
            crate::arch::riscv64::performance::algorithms::cache_friendly_sort(
                test_array.as_mut_ptr(),
                ARRAY_SIZE,
                |a, b| {
                    let val_a = *a;
                    let val_b = *b;
                    if val_a < val_b { -1 } else if val_a > val_b { 1 } else { 0 }
                }
            );
            
            let result = counter.stop();
            result.print();
            
            // Verify sorting worked
            let mut is_sorted = true;
            for i in 1..ARRAY_SIZE {
                if test_array[i-1] > test_array[i] {
                    is_sorted = false;
                    break;
                }
            }
            
            crate::uart_print(b"    Array sorting result: ");
            if is_sorted {
                crate::uart_print(b"SORTED CORRECTLY\n");
            } else {
                crate::uart_print(b"SORTING FAILED\n");
            }
        }
    }

    /// Test performance profiling
    #[cfg(target_arch = "riscv64")]
    fn test_performance_profiling(&self) {
        unsafe {
            crate::uart_print(b"  Testing performance measurement macros:\n");
            
            // Use the with_performance_measurement macro
            let _result = crate::with_performance_measurement!("dummy_computation", {
                let mut sum = 0u64;
                for i in 0..1000 {
                    sum += i * i;
                }
                sum
            });
        }
    }

    /// Simple hex printing helper
    #[cfg(target_arch = "riscv64")]
    fn print_hex_simple(&self, mut num: u64) {
        if num == 0 {
            unsafe { crate::uart_print(b"0"); }
            return;
        }

        let mut digits = [0u8; 16];
        let mut i = 0;

        while num > 0 && i < 8 {  // Print only first 8 hex digits
            let digit = (num % 16) as u8;
            digits[i] = if digit < 10 { b'0' + digit } else { b'A' + digit - 10 };
            num /= 16;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            unsafe { crate::uart_print(&[digits[i]]); }
        }
    }

/// Simple u64 printing function for performance tests  
#[cfg(target_arch = "riscv64")]
fn print_u64_simple(mut num: u64) {
    if num == 0 {
        unsafe { crate::uart_print(b"0"); }
        return;
    }

    let mut digits = [0u8; 20];
    let mut i = 0;

    while num > 0 {
        digits[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        unsafe { crate::uart_print(&[digits[i]]); }
    }

}

// reserved: control-plane injection helpers (to be added later as needed)

    /// Clear screen command
    fn cmd_clear(&self) {
        unsafe {
            // ANSI escape sequence to clear screen
            crate::uart_print(b"\x1b[2J\x1b[H");
        }
    }

    /// Unknown command handler
    fn cmd_unknown(&self, cmd: &str) {
        unsafe {
            crate::uart_print(b"Unknown command: ");
            crate::uart_print(cmd.as_bytes());
            crate::uart_print(b"\nType 'help' for available commands\n");
        }
    }

    /// Print a number (simple implementation)
    fn print_number(&self, mut num: u64) {
        if num == 0 {
            unsafe {
                crate::uart_print(b"0");
            }
            return;
        }

        let mut digits = [0u8; 20];
        let mut i = 0;

        while num > 0 {
            digits[i] = b'0' + (num % 10) as u8;
            num /= 10;
            i += 1;
        }

        // Print digits in reverse order
        while i > 0 {
            i -= 1;
            unsafe {
                crate::uart_print(&[digits[i]]);
            }
        }
    }

    /// Print a hexadecimal number
    fn print_hex(&self, mut num: u64) {
        unsafe {
            crate::uart_print(b"0x");
        }

        if num == 0 {
            unsafe {
                crate::uart_print(b"0");
            }
            return;
        }

        let mut digits = [0u8; 16]; // 64-bit number has max 16 hex digits
        let mut i = 0;

        while num > 0 {
            let digit = (num % 16) as u8;
            digits[i] = if digit < 10 {
                b'0' + digit
            } else {
                b'A' + digit - 10
            };
            num /= 16;
            i += 1;
        }

        // Print digits in reverse order
        while i > 0 {
            i -= 1;
            unsafe {
                crate::uart_print(&[digits[i]]);
            }
        }
    }

    /// Get PID syscall wrapper
    fn syscall_getpid(&self) -> Result<u32, SyscallError> {
        let mut result: i64;
        unsafe {
            #[cfg(target_arch = "aarch64")]
            asm!(
                "mov x8, {syscall_num}",
                "svc #0",
                "mov {result}, x0",
                syscall_num = in(reg) SyscallNumber::GetPid as u64,
                result = out(reg) result,
                out("x8") _,
                out("x0") _,
            );

            #[cfg(target_arch = "x86_64")]
            asm!(
                "mov rax, {syscall_num}",
                "int 0x80",
                "mov {result}, rax",
                syscall_num = in(reg) SyscallNumber::GetPid as u64,
                result = out(reg) result,
                out("rax") _,
            );

            #[cfg(target_arch = "riscv64")]
            asm!(
                "mv a7, {syscall_num}",
                "ecall",
                "mv {result}, a0",
                syscall_num = in(reg) SyscallNumber::GetPid as u64,
                result = out(reg) result,
                out("a7") _,
                out("a0") _,
            );
        }

        if result < 0 {
            Err(match result {
                -38 => SyscallError::ENOSYS,
                _ => SyscallError::EINVAL,
            })
        } else {
            Ok(result as u32)
        }
    }
}

/// Initialize and run the shell
pub fn run_shell() {
    let mut shell = Shell::new();
    shell.run();
}

/// NPU driver demonstration function
pub fn npu_driver_demo() {
    use crate::npu_driver::{initialize_npu_driver, submit_ai_inference, get_npu_stats, NPU_DRIVER};
    use crate::ml::{VerifiedMLModel, ModelMetadata, ModelId, ArenaPtr, DataType};
    use crate::npu::NpuPriority;
    
    unsafe {
        crate::uart_print(b"[NPU DRIVER] Initializing NPU driver...\n");
    }
    
    // Initialize NPU driver
    match initialize_npu_driver() {
        Ok(()) => {
            unsafe { crate::uart_print(b"[NPU DRIVER] NPU driver initialized successfully\n"); }
        }
        Err(e) => {
            unsafe { 
                crate::uart_print(b"[NPU DRIVER] Failed to initialize NPU driver: ");
                crate::uart_print(format!("{:?}", e).as_bytes());
                crate::uart_print(b"\n");
                return;
            }
        }
    }
    
    // Create a test model
    let test_metadata = ModelMetadata {
        input_shape: [4, 1, 1, 1],
        output_shape: [4, 1, 1, 1], 
        input_dtype: DataType::Float32,
        output_dtype: DataType::Float32,
        arena_size_required: 1024 * 1024,
        wcet_cycles: 100000,
        operator_count: 10,
        tensor_count: 5,
    };
    
    let test_model = VerifiedMLModel {
        id: ModelId(1),
        data_ptr: ArenaPtr { ptr: core::ptr::null_mut(), size: 0, generation: 0 },
        metadata: test_metadata,
        security_index: 0,
    };
    
    unsafe { crate::uart_print(b"[NPU DRIVER] Submitting test inference job...\n"); }
    
    // Submit a test inference job
    let test_input = [1.0f32, 2.0, 3.0, 4.0];
    match submit_ai_inference(&test_model, &test_input, 4, NpuPriority::High) {
        Ok(job_id) => {
            unsafe { 
                crate::uart_print(b"[NPU DRIVER] Submitted inference job with ID: ");
                print_number_simple(job_id as u64);
                crate::uart_print(b"\n");
            }
            
            // Simulate interrupt handling by polling
            unsafe { crate::uart_print(b"[NPU DRIVER] Simulating interrupt handling...\n"); }
            
            for i in 0..5 {
                NPU_DRIVER.handle_interrupt();
                
                // Brief delay simulation
                for _ in 0..1000 {
                    core::hint::spin_loop();
                }
                
                unsafe {
                    crate::uart_print(b"[NPU DRIVER] Interrupt handling cycle ");
                    print_number_simple(i + 1);
                    crate::uart_print(b"\n");
                }
            }
            
            // Get statistics
            let stats = get_npu_stats();
            unsafe {
                crate::uart_print(b"[NPU DRIVER] NPU Statistics:\n");
                crate::uart_print(b"  Jobs submitted: ");
                print_number_simple(stats.total_jobs_submitted);
                crate::uart_print(b"\n  Jobs completed: ");
                print_number_simple(stats.total_jobs_completed);
                crate::uart_print(b"\n  Jobs failed: ");
                print_number_simple(stats.total_jobs_failed);
                crate::uart_print(b"\n  Pending jobs: ");
                print_number_simple(stats.current_pending_jobs as u64);
                crate::uart_print(b"\n  Peak queue depth: ");
                print_number_simple(stats.peak_queue_depth as u64);
                crate::uart_print(b"\n  Average completion time: ");
                print_number_simple(stats.average_completion_time_cycles);
                crate::uart_print(b" cycles\n");
            }
        }
        Err(e) => {
            unsafe {
                crate::uart_print(b"[NPU DRIVER] Failed to submit inference job: ");
                crate::uart_print(format!("{:?}", e).as_bytes());
                crate::uart_print(b"\n");
            }
        }
    }
}

/// Simple number printing helper for demo
fn print_number_simple(mut num: u64) {
    if num == 0 {
        unsafe { crate::uart_print(b"0"); }
        return;
    }

    let mut digits = [0u8; 20];
    let mut i = 0;

    while num > 0 {
        digits[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        unsafe { crate::uart_print(&[digits[i]]); }
    }
}
