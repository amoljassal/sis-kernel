//! Boot Error Recovery and Diagnostics
//!
//! ChatGPT's structured error recovery implementation

use crate::boot::{BootCode, BootStage};
use crate::kernel::serial;
use core::panic::PanicInfo;

/// Boot error recovery strategies
#[derive(Debug, Clone, Copy)]
pub enum RecoveryStrategy {
    Retry,           // Retry the operation
    Skip,            // Skip and continue
    Fallback,        // Use fallback mechanism
    CpuOnly,         // Continue without Neural Engine
    SafeMode,        // Boot in safe mode
    Abort,           // Cannot recover
}

/// Boot diagnostic information
#[derive(Debug)]
pub struct BootDiagnostic {
    pub stage: BootStage,
    pub error_code: BootCode,
    pub recovery_strategy: RecoveryStrategy,
    pub details: alloc::string::String,
}

/// Determine recovery strategy based on error
pub fn determine_recovery_strategy(stage: BootStage, code: BootCode) -> RecoveryStrategy {
    match (stage, code) {
        // Neural Engine failures can be recovered by CPU-only mode
        (BootStage::S25_NeuralProbe, BootCode::NeuralEngineTimeout) |
        (BootStage::S25_NeuralProbe, BootCode::NeuralEngineNotFound) |
        (BootStage::S45_NeuralOnline, _) => RecoveryStrategy::CpuOnly,
        
        // Memory init failures are critical
        (BootStage::S15_MemoryInit, _) => RecoveryStrategy::Abort,
        
        // Hardware detection can be retried
        (BootStage::S20_HwDetect, _) => RecoveryStrategy::Retry,
        
        // Capability init can fallback to defaults
        (BootStage::S30_CapabilityInit, _) => RecoveryStrategy::Fallback,
        
        // Scheduler failures might allow safe mode
        (BootStage::S40_SchedulerInit, _) => RecoveryStrategy::SafeMode,
        
        // Early serial failures can be skipped (no output)
        (BootStage::S05_SerialEarly, _) => RecoveryStrategy::Skip,
        
        // Default: abort
        _ => RecoveryStrategy::Abort,
    }
}

/// Execute recovery strategy
pub fn execute_recovery(strategy: RecoveryStrategy, stage: BootStage) -> Result<(), BootCode> {
    match strategy {
        RecoveryStrategy::Retry => {
            serial::write_str("[RECOVERY] Retrying stage ");
            serial::write_str(stage_name(stage));
            serial::write_str("\n");
            // Return Ok to signal retry should be attempted
            Ok(())
        }
        
        RecoveryStrategy::Skip => {
            serial::write_str("[RECOVERY] Skipping stage ");
            serial::write_str(stage_name(stage));
            serial::write_str("\n");
            Ok(())
        }
        
        RecoveryStrategy::Fallback => {
            serial::write_str("[RECOVERY] Using fallback for ");
            serial::write_str(stage_name(stage));
            serial::write_str("\n");
            apply_fallback(stage)
        }
        
        RecoveryStrategy::CpuOnly => {
            serial::write_str("[RECOVERY] Continuing in CPU-only mode\n");
            serial::write_str("[HW] ne=absent reason=recovery\n");
            // Mark Neural Engine as unavailable
            unsafe {
                CPU_ONLY_MODE = true;
            }
            Ok(())
        }
        
        RecoveryStrategy::SafeMode => {
            serial::write_str("[RECOVERY] Booting in safe mode\n");
            unsafe {
                SAFE_MODE = true;
            }
            Ok(())
        }
        
        RecoveryStrategy::Abort => {
            serial::write_str("[RECOVERY] Cannot recover from error\n");
            Err(BootCode::UnrecoverableError)
        }
    }
}

/// Apply fallback configuration
fn apply_fallback(stage: BootStage) -> Result<(), BootCode> {
    match stage {
        BootStage::S30_CapabilityInit => {
            // Use default capabilities
            serial::write_str("[RECOVERY] Using default hardware capabilities\n");
            Ok(())
        }
        _ => Err(BootCode::RecoveryFailed),
    }
}

/// Generate diagnostic report
pub fn generate_diagnostic_report(stage: BootStage, code: BootCode) -> BootDiagnostic {
    let details = match code {
        BootCode::SerialInitFailed => "UART initialization failed",
        BootCode::MemoryInitFailed => "Memory subsystem initialization failed",
        BootCode::NeuralEngineTimeout => "Neural Engine probe timeout",
        BootCode::NeuralEngineNotFound => "Neural Engine not detected",
        BootCode::SchedulerInitFailed => "Scheduler initialization failed",
        BootCode::UnrecoverableError => "Unrecoverable system error",
        BootCode::RecoveryFailed => "Recovery attempt failed",
        _ => "Unknown error",
    };
    
    BootDiagnostic {
        stage,
        error_code: code,
        recovery_strategy: determine_recovery_strategy(stage, code),
        details: alloc::string::String::from(details),
    }
}

/// Print diagnostic information
pub fn print_diagnostics(diagnostic: &BootDiagnostic) {
    serial::write_str("\n=== BOOT DIAGNOSTICS ===\n");
    serial::write_str("Stage: ");
    serial::write_str(stage_name(diagnostic.stage));
    serial::write_str("\nError: ");
    serial::write_str(&diagnostic.details);
    serial::write_str("\nCode: 0x");
    write_hex(diagnostic.error_code.as_u16());
    serial::write_str("\nRecovery: ");
    serial::write_str(recovery_strategy_name(diagnostic.recovery_strategy));
    serial::write_str("\n========================\n");
}

/// Recovery strategy name for logging
fn recovery_strategy_name(strategy: RecoveryStrategy) -> &'static str {
    match strategy {
        RecoveryStrategy::Retry => "Retry",
        RecoveryStrategy::Skip => "Skip",
        RecoveryStrategy::Fallback => "Fallback",
        RecoveryStrategy::CpuOnly => "CPU-Only",
        RecoveryStrategy::SafeMode => "Safe-Mode",
        RecoveryStrategy::Abort => "Abort",
    }
}

/// Stage name helper
fn stage_name(stage: BootStage) -> &'static str {
    match stage {
        BootStage::S00_Reset => "S00_Reset",
        BootStage::S05_SerialEarly => "S05_SerialEarly",
        BootStage::S10_ArchEarly => "S10_ArchEarly",
        BootStage::S15_MemoryInit => "S15_MemoryInit",
        BootStage::S20_HwDetect => "S20_HwDetect",
        BootStage::S25_NeuralProbe => "S25_NeuralProbe",
        BootStage::S30_CapabilityInit => "S30_CapabilityInit",
        BootStage::S35_KernelInit => "S35_KernelInit",
        BootStage::S40_SchedulerInit => "S40_SchedulerInit",
        BootStage::S45_NeuralOnline => "S45_NeuralOnline",
        BootStage::S50_BootComplete => "S50_BootComplete",
    }
}

/// Write hex value
fn write_hex(val: u16) {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    serial::write_byte(HEX_CHARS[((val >> 12) & 0xF) as usize]);
    serial::write_byte(HEX_CHARS[((val >> 8) & 0xF) as usize]);
    serial::write_byte(HEX_CHARS[((val >> 4) & 0xF) as usize]);
    serial::write_byte(HEX_CHARS[(val & 0xF) as usize]);
}

/// Global recovery state
static mut CPU_ONLY_MODE: bool = false;
static mut SAFE_MODE: bool = false;

/// Check if running in CPU-only mode
pub fn is_cpu_only_mode() -> bool {
    unsafe { CPU_ONLY_MODE }
}

/// Check if running in safe mode
pub fn is_safe_mode() -> bool {
    unsafe { SAFE_MODE }
}

/// Panic handler for boot failures  
pub fn boot_panic(info: &PanicInfo) -> ! {
    serial::write_str("\n[PANIC] Boot panic occurred!\n");
    
    if let Some(location) = info.location() {
        serial::write_str("Location: ");
        serial::write_str(location.file());
        serial::write_str(":");
        write_decimal(location.line());
        serial::write_str("\n");
    }
    
    if let Some(msg) = info.payload().downcast_ref::<&str>() {
        serial::write_str("Message: ");
        serial::write_str(msg);
        serial::write_str("\n");
    }
    
    serial::write_str("[BOOT] FAILURE code=0xDEAD\n");
    
    // Halt the system
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe { core::arch::asm!("hlt") }
        #[cfg(target_arch = "aarch64")]
        unsafe { core::arch::asm!("wfe") }
    }
}

/// Write decimal number
fn write_decimal(mut n: u32) {
    if n == 0 {
        serial::write_byte(b'0');
        return;
    }
    
    let mut buffer = [0u8; 10];
    let mut pos = 0;
    
    while n > 0 {
        buffer[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        pos += 1;
    }
    
    while pos > 0 {
        pos -= 1;
        serial::write_byte(buffer[pos]);
    }
}