SIS Kernel – AI Memory Master Summary
(Machine-readable full technical recall document)

1. Project Identity & Purpose
Name: SIS (Sovereign Interface System) Kernel

Type: Bare-metal Rust microkernel for x86_64, UEFI bootloader

Primary Concept: Dual-role architecture

Philosophy Parent Task → abstract reasoning, decision-making, policy interpretation

Technical Parent Task → system-level operations, resource control

Child Tasks → dynamic, inheriting from parent roles

Key Differentiator: Secure, deterministic governance kernel with optional AI-driven logic modules

2. Target Architecture & Build Setup
Target: x86_64-unknown-none

Toolchain: Rust nightly, with core + alloc only (#![no_std])

Bootloader: bootloader + bootloader_api v0.11.3 (UEFI enabled)

Build Flags:

toml
Copy
Edit
[unstable]
build-std = ["core", "alloc"]
build-std-features = ["compiler-builtins-mem"]
Rust Flags: -C code-model=kernel, -C relocation-model=static

Allocator: linked_list_allocator v0.10.5 (use_spin feature)

x86_64 crate: v0.14.9 with abi_x86_interrupt feature

Build Time: ~2.3s, 0 errors, ~50 warnings (expected for bare-metal)

3. Core Kernel Components
Location: src/arch/x86_64/ and src/kernel/

3.1 Boot Process
UEFI bootloader → bootloader_api initializes:

Memory map

Frame allocator

GDT + TSS + IST

IDT

Transfers control to kernel main in main.rs

3.2 Memory Management
Heap allocation via linked_list_allocator

Paging & frame allocation

Custom BootInfoFrameAllocator bypassing static lifetime issues

Safe heap init with:

rust
Copy
Edit
ALLOCATOR.lock().init(heap_start.as_u64() as *mut u8, HEAP_SIZE)
Static mutable region for memory map:

rust
Copy
Edit
static mut MEMORY_REGIONS: Option<&[MemoryRegion]> = None;
3.3 Interrupt Handling (IDT)
Location: idt.rs

Handlers for:

Divide by zero

GP fault

Page fault

Pending: Double fault handler (requires compatible &mut InterruptStackFrame ABI)

Timer ISR integration needed

Registered via init_idt()

3.4 GDT & TSS
Location: gdt.rs

Spin-based lazy static initialization

IST setup for critical faults (planned use in double fault)

Task State Segment configured

3.5 Scheduler
Location: scheduler.rs

Round-robin

Parent/child priority rules

Context switching using raw pointers to bypass borrow checker

Affinity support (planned)

3.6 Syscalls
Location: syscall.rs

Basic syscall infrastructure (int 0x80 vector)

Placeholder for expansion

3.7 Device & I/O
Serial output via UART (serial.rs)

PCI enumeration (pci.rs)

VFIO framework skeleton (vfio.rs)

PIT timer driver (pit.rs) — integration with scheduler pending

4. Known Issues & Compatibility Gaps
Double Fault Handler → ABI mismatch; must use &mut InterruptStackFrame

Timer ISR → needs integration into IDT with correct signature

x86_64 crate API constraint → affects handler signatures across 0.14.9–0.15.1

Crypto libs (ecdsa, sha2) temporarily removed; need no_std-compatible alternatives

PIC handling → currently embedded in IDT setup; no pic.rs separation

Warnings → unused code, static mut, deprecated x86_64 calls

5. User Space / Web App Parallel Layer
Web app version & kernel-mock version share same core logic

Kernel-mock acts as bridge layer → API emulates kernel calls

Allows parallel UI/UX development before kernel maturity

Core design in user space matches Philosophy/Technical task separation

Integration goal: Replace mock bridge with real syscall IPC once kernel stable

6. Product Vision
SIS Kernel is not a generic desktop OS; it’s a governance/control microkernel for:

Defense systems

Banking & finance policy enforcement

Secure IoT hubs

Cloud command arbitration

AI governance appliances

End product = appliance or embedded controller that:

Loads policies (philosophy layer)

Executes them with technical precision

Attests results

Interfaces via web or dedicated terminal

7. Roadmap
Patch double fault handler with ABI-correct signature

Integrate timer ISR with scheduler

Add crypto in no_std

Implement affinity features

Harden VFIO for GPU passthrough

Merge user space logic into syscall interface

End-to-end prototype with web UI