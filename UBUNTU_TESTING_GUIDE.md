# SIS-OS MacBook Pro Mid-2012 Testing Guide

**Target Hardware**: MacBook Pro mid-2012 (Intel i5, 16GB RAM, 500GB SSD)  
**Operating System**: Ubuntu Latest  
**Purpose**: Safe x86_64 hardware testing environment

## Prerequisites Setup

### 1. Install Rust Toolchain
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install nightly and x86_64 target
rustup install nightly
rustup default nightly
rustup target add x86_64-unknown-none
```

### 2. Install Development Tools
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install build essentials
sudo apt install -y \
  build-essential \
  git \
  qemu-system-x86 \
  qemu-utils \
  ovmf \
  gdb \
  hexdump

# Install Rust tools
cargo install bootimage
```

### 3. Clone SIS-OS Project
```bash
git clone https://github.com/amoljassal/sis-kernel.git
cd sis-kernel
```

## Build Process

### 1. Test Basic Compilation
```bash
# Check basic compilation
cargo check --target x86_64-unknown-none

# Expected: Some warnings, should compile successfully
```

### 2. Build x86_64 Kernel
```bash
# Build with core features
cargo build --target x86_64-unknown-none --features "idt-selftest,apic"

# Build bootable image
cargo bootimage --target x86_64-unknown-none
```

### 3. QEMU Testing (Safe)
```bash
# Test in QEMU first
./scripts/qemu.sh

# or use existing x86_64 QEMU script
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-sis_kernel.bin \
  -serial stdio \
  -display none
```

## Hardware Testing (After QEMU Success)

### 1. Create Bootable USB
```bash
# Find USB device (replace /dev/sdX with your USB device)
sudo fdisk -l

# Create bootable USB (DANGER: This will erase USB drive)
sudo dd if=target/x86_64-unknown-none/debug/bootimage-sis_kernel.bin of=/dev/sdX bs=1M status=progress
```

### 2. Boot from USB
```bash
# 1. Insert USB drive
# 2. Reboot MacBook Pro
# 3. Hold Option/Alt key during boot
# 4. Select USB drive
# 5. SIS-OS should boot
```

## Safety Protocols

### Ubuntu Backup (Optional)
```bash
# Create system snapshot (if using LVM)
sudo lvcreate -L1G -s -n ubuntu-backup /dev/vg0/root

# Or create full backup to external drive
sudo rsync -aAXv / --exclude={/dev/*,/proc/*,/sys/*,/tmp/*,/run/*,/mnt/*,/media/*,/lost+found} /path/to/backup/
```

### Recovery Plan
1. **USB Testing Only**: No risk to main system
2. **GRUB Recovery**: Ubuntu GRUB should remain intact
3. **Live USB Recovery**: Keep Ubuntu live USB handy
4. **Complete Reinstall**: Worst case - fresh Ubuntu installation

## Expected Test Results

### Positive Indicators
- Kernel loads and displays boot messages
- Serial output shows initialization sequences  
- Hardware detection (CPU, memory, interrupts)
- Clean shutdown or controlled panic

### Warning Signs
- Immediate reboot/crash
- No serial output
- Hardware compatibility issues
- Thermal problems (unlikely)

## Troubleshooting

### Build Issues
```bash
# Clear build cache
cargo clean

# Update toolchain
rustup update nightly

# Check target availability
rustup target list | grep x86_64-unknown-none
```

### Boot Issues
```bash
# Check bootimage creation
ls -la target/x86_64-unknown-none/debug/bootimage-*

# Verify USB creation
sudo fdisk -l /dev/sdX
```

### QEMU Debugging
```bash
# Run with GDB
qemu-system-x86_64 -s -S -drive format=raw,file=bootimage.bin

# In another terminal
gdb target/x86_64-unknown-none/debug/sis_kernel
(gdb) target remote localhost:1234
```

## Project Status

- **Phase 5 Complete**: Production hardening finished
- **Zero Compilation Errors**: ARM64 build successful
- **Professional Structure**: Enterprise-ready organization
- **Hardware Ready**: Comprehensive hardware detection implemented

## Contact Information

- **Repository**: https://github.com/amoljassal/sis-kernel
- **Issues**: Report via GitHub Issues
- **Documentation**: See docs/ directory

---

**Note**: This is a sophisticated AI-native kernel with extensive hardware integration. Take your time with testing and don't hesitate to start with QEMU validation before hardware deployment.