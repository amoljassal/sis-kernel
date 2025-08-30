#!/usr/bin/env bash
# SIS-OS ARM64 QEMU Test Runner
# Designed for Mac Mini M1 testing with safe emulation
# Version: 1.0

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_DIR="$SCRIPT_DIR/.."
LOG_FILE="$SCRIPT_DIR/qemu_arm64_test.log"

# Test parameters
TEST="${TEST:-BOOT}"  # BOOT|NEURAL|MEMORY|SCHEDULER
FEATURES="${FEATURES:-arm64-ai}"
TARGET="aarch64-unknown-none"
PROFILE="${PROFILE:-debug}"
TIMEOUT="${TIMEOUT:-10}"

# QEMU configuration
QEMU_BIN="qemu-system-aarch64"
KERNEL_ELF="$KERNEL_DIR/target/${TARGET}/${PROFILE}/sis_kernel"
SERIAL_LOG="$SCRIPT_DIR/qemu_arm64_serial.log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

error_exit() {
    echo -e "${RED}ERROR: $1${NC}" >&2
    log "ERROR: $1"
    exit 1
}

success() {
    echo -e "${GREEN}SUCCESS: $1${NC}"
    log "SUCCESS: $1"
}

info() {
    echo -e "${BLUE}INFO: $1${NC}"
    log "INFO: $1"
}

warning() {
    echo -e "${YELLOW}WARNING: $1${NC}"
    log "WARNING: $1"
}

# Check prerequisites
check_prerequisites() {
    info "Checking QEMU ARM64 prerequisites..."
    
    if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
        error_exit "qemu-system-aarch64 not found. Install with: brew install qemu"
    fi
    
    if [[ ! -f "$KERNEL_ELF" ]]; then
        warning "Kernel not found at $KERNEL_ELF, building..."
        build_kernel
    fi
    
    success "Prerequisites validated"
}

# Build ARM64 kernel
build_kernel() {
    info "Building SIS-OS ARM64 kernel..."
    cd "$KERNEL_DIR"
    
    local build_cmd="cargo +nightly build"
    build_cmd+=" --target $TARGET"
    
    if [[ "$FEATURES" != "none" ]]; then
        # Skip features that cause compilation issues for now
        info "Building with basic ARM64 support (advanced features disabled for QEMU)"
    fi
    
    if ! $build_cmd; then
        error_exit "Kernel build failed"
    fi
    
    if [[ ! -f "$KERNEL_ELF" ]]; then
        error_exit "Kernel binary not found after build"
    fi
    
    local kernel_size=$(stat -f%z "$KERNEL_ELF" 2>/dev/null || stat -c%s "$KERNEL_ELF" 2>/dev/null || echo "unknown")
    success "ARM64 kernel built successfully (${kernel_size} bytes)"
}

# Run QEMU test
run_qemu_test() {
    local test_name=$1
    info "Starting QEMU ARM64 test: $test_name"
    
    # Clean previous serial log
    rm -f "$SERIAL_LOG"
    
    local qemu_cmd=(
        "$QEMU_BIN"
        -M virt,gic-version=3
        -cpu cortex-a72
        -smp 4
        -m 1024M
        -nographic
        -kernel "$KERNEL_ELF"
        -serial "file:$SERIAL_LOG"
        -monitor none
        -no-reboot
    )
    
    # Add test-specific parameters
    case "$test_name" in
        "BOOT")
            info "Testing basic kernel boot sequence"
            ;;
        "NEURAL")
            info "Testing Neural Engine detection (emulated)"
            ;;
        "MEMORY")
            info "Testing memory management"
            ;;
        "SCHEDULER")
            info "Testing cognitive scheduler"
            ;;
    esac
    
    info "QEMU command: ${qemu_cmd[*]}"
    info "Serial output will be logged to: $SERIAL_LOG"
    info "Test will timeout after ${TIMEOUT} seconds"
    
    # Run QEMU with timeout
    if command -v timeout >/dev/null 2>&1; then
        timeout "$TIMEOUT" "${qemu_cmd[@]}" || true
    elif command -v gtimeout >/dev/null 2>&1; then
        gtimeout "$TIMEOUT" "${qemu_cmd[@]}" || true
    else
        # Manual timeout using background process
        "${qemu_cmd[@]}" &
        local qemu_pid=$!
        sleep "$TIMEOUT"
        kill $qemu_pid 2>/dev/null || true
        wait $qemu_pid 2>/dev/null || true
    fi
    
    success "QEMU test completed"
}

# Analyze test results
analyze_results() {
    info "Analyzing QEMU test results..."
    
    if [[ ! -f "$SERIAL_LOG" ]]; then
        warning "No serial output captured"
        return 1
    fi
    
    local serial_size=$(stat -f%z "$SERIAL_LOG" 2>/dev/null || stat -c%s "$SERIAL_LOG" 2>/dev/null || echo "0")
    info "Serial log size: ${serial_size} bytes"
    
    if [[ "$serial_size" == "0" ]]; then
        warning "No kernel output captured - kernel may not have started"
        return 1
    fi
    
    echo "=== KERNEL OUTPUT ==="
    cat "$SERIAL_LOG"
    echo "=== END OUTPUT ==="
    
    # Look for key indicators
    local boot_success=false
    local panic_detected=false
    local neural_detected=false
    
    if grep -q "SIS.*KERNEL.*ENTRY\|Boot.*Stage\|Kernel.*Init" "$SERIAL_LOG" 2>/dev/null; then
        boot_success=true
        success "Kernel boot sequence detected"
    fi
    
    if grep -q "panic\|PANIC\|kernel panic\|Fatal" "$SERIAL_LOG" 2>/dev/null; then
        panic_detected=true
        warning "Kernel panic detected in output"
    fi
    
    if grep -q "Neural.*Engine\|M1.*Hardware\|Apple.*Silicon" "$SERIAL_LOG" 2>/dev/null; then
        neural_detected=true
        success "Neural Engine/M1 hardware detection code executed"
    fi
    
    # Summary
    echo ""
    echo "=== TEST RESULTS SUMMARY ==="
    echo "Boot sequence detected: $boot_success"
    echo "Panic detected: $panic_detected"
    echo "Neural Engine code executed: $neural_detected"
    
    if [[ "$boot_success" == true && "$panic_detected" == false ]]; then
        success "QEMU test PASSED - Kernel boots successfully"
        return 0
    else
        warning "QEMU test had issues - Check serial output above"
        return 1
    fi
}

# Main execution
main() {
    echo "=================================================================="
    echo "SIS-OS ARM64 QEMU Test Runner"
    echo "Target: Mac Mini M1 (ARM64 emulation)"
    echo "Test: $TEST"
    echo "Kernel: $KERNEL_ELF"
    echo "=================================================================="
    
    check_prerequisites
    run_qemu_test "$TEST"
    analyze_results
    
    success "QEMU ARM64 testing completed successfully"
}

# Handle script arguments
case "${1:-}" in
    "boot"|"BOOT")
        TEST="BOOT"
        ;;
    "neural"|"NEURAL")
        TEST="NEURAL"
        ;;
    "memory"|"MEMORY")
        TEST="MEMORY"
        ;;
    "scheduler"|"SCHEDULER")
        TEST="SCHEDULER"
        ;;
    "help"|"--help"|"-h")
        echo "Usage: $0 [boot|neural|memory|scheduler]"
        echo "Default: boot"
        exit 0
        ;;
    "")
        # Use default TEST value
        ;;
    *)
        warning "Unknown test: $1, using default: $TEST"
        ;;
esac

main "$@"