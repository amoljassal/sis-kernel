#!/bin/bash
# QEMU RISC-V Test Script for SIS Kernel
# Research-backed RISC-V development environment with AIA and device tree validation

set -e

# Configuration
KERNEL_TARGET="riscv64gc-unknown-none-elf"
KERNEL_BINARY="target/${KERNEL_TARGET}/debug/sis_kernel"
OPENSBI_PATH="/opt/homebrew/share/qemu/opensbi-riscv64-generic-fw_dynamic.bin"
QEMU_BINARY="/opt/homebrew/bin/qemu-system-riscv64"

# Build configuration
QEMU_MACHINE="virt"
QEMU_CPU="rv64"
QEMU_MEMORY="128M"
QEMU_SMP="4"

# Enable AIA (Advanced Interrupt Architecture) if supported
AIA_SUPPORT=${AIA:-"auto"}
if [ "$AIA_SUPPORT" = "on" ]; then
    QEMU_MACHINE="virt,aia=aplic-imsic"
    echo "=== RISC-V AIA Mode Enabled ==="
else
    echo "=== RISC-V Legacy PLIC Mode ==="
fi

# Debug and development options
DEBUG=${DEBUG:-0}
GDB=${GDB:-0}
BRINGUP=${BRINGUP:-0}

echo "=== SIS Kernel RISC-V Build & Test ==="
echo "Target: $KERNEL_TARGET"
echo "Machine: $QEMU_MACHINE"
echo "CPU: $QEMU_CPU"
echo "Memory: $QEMU_MEMORY"
echo "SMP: $QEMU_SMP cores"

# Check dependencies
if [ ! -f "$QEMU_BINARY" ]; then
    echo "ERROR: QEMU RISC-V not found at $QEMU_BINARY"
    echo "Install with: brew install qemu"
    exit 1
fi

# Build kernel for RISC-V
echo ""
echo "=== Building SIS Kernel for RISC-V ==="
if [ "$BRINGUP" = "1" ]; then
    echo "Building with bringup features..."
    cargo build --target "$KERNEL_TARGET" --features "riscv64,bringup"
else
    echo "Building standard kernel..."
    cargo build --target "$KERNEL_TARGET" --features "riscv64"
fi

if [ ! -f "$KERNEL_BINARY" ]; then
    echo "ERROR: Kernel binary not found at $KERNEL_BINARY"
    echo "Build may have failed"
    exit 1
fi

echo "Kernel binary size: $(ls -lh $KERNEL_BINARY | awk '{print $5}')"

# Prepare QEMU command
QEMU_CMD="$QEMU_BINARY"
QEMU_CMD="$QEMU_CMD -machine $QEMU_MACHINE"
QEMU_CMD="$QEMU_CMD -cpu $QEMU_CPU"
QEMU_CMD="$QEMU_CMD -m $QEMU_MEMORY"
QEMU_CMD="$QEMU_CMD -smp $QEMU_SMP"
QEMU_CMD="$QEMU_CMD -nographic"
QEMU_CMD="$QEMU_CMD -serial stdio"

# OpenSBI firmware (if available)
if [ -f "$OPENSBI_PATH" ]; then
    echo "Using OpenSBI firmware: $OPENSBI_PATH"
    QEMU_CMD="$QEMU_CMD -bios $OPENSBI_PATH"
else
    echo "OpenSBI not found, using QEMU built-in firmware"
    QEMU_CMD="$QEMU_CMD -bios default"
fi

# Kernel binary
QEMU_CMD="$QEMU_CMD -kernel $KERNEL_BINARY"

# Device tree options for validation
QEMU_CMD="$QEMU_CMD -machine dumpdtb=target/riscv64-virt.dtb"

# Debug options
if [ "$DEBUG" = "1" ]; then
    echo "Debug mode enabled"
    QEMU_CMD="$QEMU_CMD -d int,guest_errors,unimp"
    QEMU_CMD="$QEMU_CMD -D target/qemu-debug.log"
fi

# GDB support
if [ "$GDB" = "1" ]; then
    echo "GDB mode enabled - connect with: gdb-multiarch target/$KERNEL_TARGET/debug/sis_kernel"
    echo "Then: target remote :1234"
    QEMU_CMD="$QEMU_CMD -s -S"
fi

# Performance monitoring
if [ "$PERF" = "1" ]; then
    echo "Performance monitoring enabled"
    QEMU_CMD="$QEMU_CMD -icount shift=7,rr=record,rrfile=target/replay.bin"
fi

echo ""
echo "=== QEMU Command ==="
echo "$QEMU_CMD"
echo ""

# Run QEMU with timeout for automated testing
if [ "$TIMEOUT" != "" ]; then
    echo "=== Running with timeout: ${TIMEOUT}s ==="
    if command -v gtimeout >/dev/null 2>&1; then
        gtimeout "$TIMEOUT" $QEMU_CMD
    elif command -v timeout >/dev/null 2>&1; then
        timeout "$TIMEOUT" $QEMU_CMD
    else
        echo "WARNING: No timeout command found, running without timeout"
        $QEMU_CMD
    fi
else
    echo "=== Running SIS Kernel on RISC-V QEMU ==="
    echo "Press Ctrl+A, X to exit QEMU"
    echo ""
    $QEMU_CMD
fi

echo ""
echo "=== QEMU Session Ended ==="

# Post-run analysis
if [ -f "target/qemu-debug.log" ]; then
    echo ""
    echo "=== Debug Log Summary ==="
    echo "Log file: target/qemu-debug.log"
    echo "Size: $(ls -lh target/qemu-debug.log | awk '{print $5}')"
    
    # Show last few lines for quick debugging
    echo ""
    echo "=== Last 10 lines of debug log ==="
    tail -10 target/qemu-debug.log
fi

# Device tree analysis
if [ -f "target/riscv64-virt.dtb" ]; then
    echo ""
    echo "=== Device Tree Information ==="
    echo "DTB file: target/riscv64-virt.dtb"
    echo "Size: $(ls -lh target/riscv64-virt.dtb | awk '{print $5}')"
    
    # Convert to readable format if dtc is available
    if command -v dtc >/dev/null 2>&1; then
        echo "Converting DTB to readable format..."
        dtc -I dtb -O dts target/riscv64-virt.dtb > target/riscv64-virt.dts 2>/dev/null || true
        if [ -f "target/riscv64-virt.dts" ]; then
            echo "Device tree source: target/riscv64-virt.dts"
            echo "CPU count: $(grep -c 'device_type = "cpu"' target/riscv64-virt.dts || echo "unknown")"
            echo "Memory size: $(grep 'reg = <0x0 0x80000000' target/riscv64-virt.dts | head -1 || echo "unknown")"
        fi
    fi
fi