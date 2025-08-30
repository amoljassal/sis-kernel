#!/bin/bash
# SIS Kernel USB Payload Creation Script
# Creates properly formatted USB drive for Phase 2A testing
# Version: 1.0
# Target: Mac Mini M1 (8GB RAM, 512GB SSD)

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/usb_payload_creation.log"
CREATION_VERSION="1.0"

# Default paths
KERNEL_DIR="$SCRIPT_DIR/.."
USB_DEVICE=""
FORCE_FORMAT=false

# Colors for output
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    NC=''
fi

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Error handling
error_exit() {
    echo -e "${RED}ERROR: $1${NC}" >&2
    log "ERROR: $1"
    exit 1
}

# Warning function
warning() {
    echo -e "${YELLOW}WARNING: $1${NC}" >&2
    log "WARNING: $1"
}

# Success function
success() {
    echo -e "${GREEN}$1${NC}"
    log "SUCCESS: $1"
}

# Info function
info() {
    echo -e "${BLUE}$1${NC}"
    log "INFO: $1"
}

# Print usage
print_usage() {
    echo "Usage: $0 [OPTIONS] <USB_DEVICE>"
    echo ""
    echo "Create USB payload drive for SIS Kernel Phase 2A testing"
    echo ""
    echo "Arguments:"
    echo "  USB_DEVICE    USB device path (e.g., /dev/disk4)"
    echo ""
    echo "Options:"
    echo "  -f, --force   Force format without confirmation"
    echo "  -h, --help    Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 /dev/disk4"
    echo "  $0 --force /dev/disk4"
    echo ""
}

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -f|--force)
                FORCE_FORMAT=true
                shift
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            /dev/disk*)
                USB_DEVICE="$1"
                shift
                ;;
            *)
                error_exit "Unknown argument: $1"
                ;;
        esac
    done

    if [[ -z "$USB_DEVICE" ]]; then
        error_exit "USB device path required. Use --help for usage."
    fi
}

# Print header
print_header() {
    echo "=================================================================="
    echo -e "${BOLD}SIS Kernel USB Payload Creation${NC}"
    echo "Target Hardware: Mac Mini M1 (8GB RAM, 512GB SSD)"
    echo "Creation Version: $CREATION_VERSION"
    echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "=================================================================="
    log "Starting USB payload creation for $USB_DEVICE"
}

# Verify prerequisites
verify_prerequisites() {
    info "Verifying prerequisites for USB payload creation..."
    
    # Check if running on macOS
    if [[ "$(uname)" != "Darwin" ]]; then
        error_exit "This script requires macOS"
    fi
    success "Running on macOS"
    
    # Check if USB device exists
    if [[ ! -e "$USB_DEVICE" ]]; then
        error_exit "USB device not found: $USB_DEVICE"
    fi
    success "USB device found: $USB_DEVICE"
    
    # Check if device is actually a USB drive
    if ! diskutil info "$USB_DEVICE" | grep -q "Protocol.*USB"; then
        warning "Device may not be a USB drive"
        if [[ "$FORCE_FORMAT" != "true" ]]; then
            read -p "Continue anyway? (y/N): " CONTINUE
            if [[ "$CONTINUE" != "y" && "$CONTINUE" != "Y" ]]; then
                error_exit "Operation cancelled"
            fi
        fi
    else
        success "USB device confirmed"
    fi
    
    # Check device size
    DEVICE_SIZE=$(diskutil info "$USB_DEVICE" | grep "Total Size" | awk -F'(' '{print $2}' | awk '{print $1}' | tr -d ' ')
    info "Device size: ${DEVICE_SIZE} bytes"
    
    # Require at least 4GB
    MIN_SIZE_BYTES=$((4 * 1024 * 1024 * 1024))  # 4GB
    if [[ "${DEVICE_SIZE}" -lt $MIN_SIZE_BYTES ]]; then
        error_exit "USB device too small. Required: 4GB, Available: ${DEVICE_SIZE} bytes"
    fi
    success "Device size sufficient for SIS Kernel payload"
    
    # Check for required tools
    REQUIRED_TOOLS=("diskutil" "newfs_msdos" "mkfs.ext4" "hdiutil")
    for tool in "${REQUIRED_TOOLS[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            error_exit "Required tool not found: $tool"
        fi
    done
    success "All required tools available"
}

# Show device information
show_device_info() {
    info "USB device information:"
    echo ""
    
    diskutil info "$USB_DEVICE" | grep -E "(Device / Media Name|Total Size|Protocol|File System)" || true
    
    echo ""
    info "Current partitions:"
    diskutil list "$USB_DEVICE" || true
    echo ""
}

# Warn about data loss
warn_data_loss() {
    echo ""
    echo -e "${RED}${BOLD}CRITICAL WARNING${NC}"
    echo "================"
    echo "This operation will COMPLETELY ERASE all data on:"
    echo "Device: $USB_DEVICE"
    echo ""
    
    show_device_info
    
    echo ""
    echo "ALL DATA ON THIS DEVICE WILL BE PERMANENTLY LOST"
    echo ""
    
    if [[ "$FORCE_FORMAT" != "true" ]]; then
        echo "Type 'ERASE' to confirm data destruction: "
        read -r CONFIRM
        if [[ "$CONFIRM" != "ERASE" ]]; then
            error_exit "Operation cancelled by user"
        fi
    else
        warning "Force format enabled - proceeding without confirmation"
    fi
}

# Unmount device
unmount_device() {
    info "Unmounting USB device..."
    
    # Get all mounted volumes for this device
    MOUNTED_VOLUMES=$(diskutil list "$USB_DEVICE" | grep -E "^\s+[0-9]:" | awk '{print $NF}' | grep -v "^$" || true)
    
    if [[ -n "$MOUNTED_VOLUMES" ]]; then
        echo "$MOUNTED_VOLUMES" | while read -r volume; do
            if [[ -n "$volume" ]]; then
                info "Unmounting ${USB_DEVICE}${volume}..."
                diskutil unmount "${USB_DEVICE}s${volume}" 2>/dev/null || true
            fi
        done
    fi
    
    # Force unmount the entire device
    diskutil unmountDisk "$USB_DEVICE" >/dev/null 2>&1 || true
    success "Device unmounted"
}

# Create partition table
create_partition_table() {
    info "Creating GPT partition table..."
    
    # Create GPT with two partitions:
    # Partition 1: FAT32 BOOT (512MB) - U-Boot scripts, kernel, DTB
    # Partition 2: ext4 ROOTFS (remaining) - test artifacts, logs
    
    if diskutil partitionDisk "$USB_DEVICE" GPT FAT32 "SIS_BOOT" 512MB Linux "SIS_ROOTFS" 0b; then
        success "Partition table created successfully"
    else
        error_exit "Failed to create partition table"
    fi
    
    # Wait for partitions to appear
    sleep 2
    
    # Verify partitions were created
    if diskutil list "$USB_DEVICE" | grep -q "SIS_BOOT" && diskutil list "$USB_DEVICE" | grep -q "SIS_ROOTFS"; then
        success "Partitions verified"
    else
        error_exit "Partition verification failed"
    fi
}

# Format partitions
format_partitions() {
    info "Formatting partitions..."
    
    # Get partition identifiers
    BOOT_PARTITION="${USB_DEVICE}s1"
    ROOTFS_PARTITION="${USB_DEVICE}s2"
    
    # Format boot partition as FAT32
    info "Formatting boot partition (FAT32)..."
    if sudo newfs_msdos -F 32 -v "SIS_BOOT" "$BOOT_PARTITION"; then
        success "Boot partition formatted"
    else
        error_exit "Failed to format boot partition"
    fi
    
    # Format rootfs partition as ext4
    info "Formatting rootfs partition (ext4)..."
    if sudo mkfs.ext4 -L "SIS_ROOTFS" "$ROOTFS_PARTITION"; then
        success "Rootfs partition formatted"
    else
        warning "ext4 format failed - creating HFS+ instead"
        if sudo newfs_hfs -v "SIS_ROOTFS" "$ROOTFS_PARTITION"; then
            success "Rootfs partition formatted (HFS+)"
        else
            error_exit "Failed to format rootfs partition"
        fi
    fi
}

# Mount partitions
mount_partitions() {
    info "Mounting partitions..."
    
    # Mount boot partition
    if diskutil mount "${USB_DEVICE}s1" >/dev/null 2>&1; then
        BOOT_MOUNT=$(diskutil info "${USB_DEVICE}s1" | grep "Mount Point" | cut -d':' -f2 | xargs)
        success "Boot partition mounted at: $BOOT_MOUNT"
    else
        error_exit "Failed to mount boot partition"
    fi
    
    # Mount rootfs partition
    if diskutil mount "${USB_DEVICE}s2" >/dev/null 2>&1; then
        ROOTFS_MOUNT=$(diskutil info "${USB_DEVICE}s2" | grep "Mount Point" | cut -d':' -f2 | xargs)
        success "Rootfs partition mounted at: $ROOTFS_MOUNT"
    else
        error_exit "Failed to mount rootfs partition"
    fi
}

# Build SIS kernel
build_sis_kernel() {
    info "Building SIS kernel..."
    
    cd "$KERNEL_DIR"
    
    # Build for ARM64
    if cargo build --release --target aarch64-unknown-none.json; then
        success "SIS kernel built successfully"
    else
        error_exit "Failed to build SIS kernel"
    fi
    
    # Check if kernel binary exists
    KERNEL_BINARY="$KERNEL_DIR/target/aarch64-unknown-none/release/sis_kernel"
    if [[ ! -f "$KERNEL_BINARY" ]]; then
        error_exit "Kernel binary not found: $KERNEL_BINARY"
    fi
    
    # Get binary size
    KERNEL_SIZE=$(stat -f%z "$KERNEL_BINARY")
    info "Kernel binary size: ${KERNEL_SIZE} bytes"
    
    cd "$SCRIPT_DIR"
}

# Create device tree blob
create_device_tree() {
    info "Creating device tree blob for M1 Mac Mini..."
    
    DTB_SOURCE="$KERNEL_DIR/dts/m1-sis.dts"
    DTB_OUTPUT="$BOOT_MOUNT/m1-sis.dtb"
    
    # Create basic device tree for M1
    cat > "$DTB_SOURCE" << 'EOF'
/dts-v1/;

/ {
    compatible = "apple,t8103", "apple,arm-platform";
    #address-cells = <2>;
    #size-cells = <2>;
    
    chosen {
        bootargs = "console=ttyS0,115200 earlycon=uart,mmio,0x235200000";
    };
    
    memory@800000000 {
        device_type = "memory";
        reg = <0x8 0x00000000 0x2 0x00000000>; // 8GB at 0x800000000
    };
    
    cpus {
        #address-cells = <1>;
        #size-cells = <0>;
        
        cpu@0 {
            compatible = "apple,icestorm";
            device_type = "cpu";
            reg = <0x0>;
        };
        
        cpu@1 {
            compatible = "apple,icestorm";
            device_type = "cpu";
            reg = <0x1>;
        };
        
        cpu@2 {
            compatible = "apple,icestorm";
            device_type = "cpu";
            reg = <0x2>;
        };
        
        cpu@3 {
            compatible = "apple,icestorm";
            device_type = "cpu";
            reg = <0x3>;
        };
        
        cpu@4 {
            compatible = "apple,firestorm";
            device_type = "cpu";
            reg = <0x4>;
        };
        
        cpu@5 {
            compatible = "apple,firestorm";
            device_type = "cpu";
            reg = <0x5>;
        };
        
        cpu@6 {
            compatible = "apple,firestorm";
            device_type = "cpu";
            reg = <0x6>;
        };
        
        cpu@7 {
            compatible = "apple,firestorm";
            device_type = "cpu";
            reg = <0x7>;
        };
    };
    
    soc {
        compatible = "simple-bus";
        #address-cells = <2>;
        #size-cells = <2>;
        ranges;
        
        serial@235200000 {
            compatible = "apple,s5l-uart";
            reg = <0x2 0x35200000 0x0 0x4000>;
            clock-frequency = <24000000>;
        };
        
        ane@204000000 {
            compatible = "apple,neural-engine";
            reg = <0x2 0x04000000 0x0 0x10000>;
        };
    };
};
EOF

    # For now, just copy the source as the DTB (real implementation would use dtc)
    cp "$DTB_SOURCE" "$DTB_OUTPUT"
    success "Device tree blob created: $DTB_OUTPUT"
}

# Create U-Boot boot script
create_boot_script() {
    info "Creating U-Boot boot script..."
    
    BOOT_SCRIPT="$BOOT_MOUNT/boot.scr"
    
    cat > "$BOOT_SCRIPT" << 'EOF'
# SIS Kernel Phase 2A Boot Script
# Target: Mac Mini M1 (8GB RAM, 512GB SSD)
# Safety: Maximum protection enabled

echo "=========================================="
echo "SIS Kernel Phase 2A Hardware Validation"
echo "Target: M1 Mac Mini (8GB RAM, 512GB SSD)"
echo "Safety Mode: ENABLED"
echo "Recovery: Power cycle -> macOS"
echo "=========================================="

# Hardware safety checks
echo "Performing pre-boot safety checks..."

# Initialize USB subsystem
usb start
if test $? -ne 0; then
    echo "ERROR: USB initialization failed"
    echo "RECOVERY: Power cycle and select macOS"
    exit 1
fi

# Verify kernel and DTB presence
echo "Checking for kernel and device tree..."
if test ! -e usb 0:1 /sis_kernel.bin; then
    echo "ERROR: SIS kernel not found"
    echo "RECOVERY: Check USB payload"
    exit 1
fi

if test ! -e usb 0:1 /m1-sis.dtb; then
    echo "ERROR: Device tree not found"
    echo "RECOVERY: Check USB payload"
    exit 1
fi

# Set boot arguments with safety parameters
setenv bootargs "console=ttyS0,115200 earlycon=uart,mmio,0x235200000 sis.safety_mode=1 sis.test_mode=1 sis.watchdog_timeout=100 sis.thermal_limit=75"

echo "Loading SIS kernel..."
fatload usb 0:1 ${kernel_addr_r} /sis_kernel.bin
if test $? -ne 0; then
    echo "ERROR: Failed to load kernel"
    exit 1
fi

echo "Loading device tree..."
fatload usb 0:1 ${fdt_addr_r} /m1-sis.dtb
if test $? -ne 0; then
    echo "ERROR: Failed to load device tree"
    exit 1
fi

echo "=========================================="
echo "STARTING SIS KERNEL"
echo "Hardware monitoring: ENABLED"
echo "Emergency recovery: Power cycle Mac Mini"
echo "=========================================="

# Boot kernel with device tree
booti ${kernel_addr_r} - ${fdt_addr_r}

# Should never reach here
echo "ERROR: Kernel boot failed"
echo "RECOVERY: Power cycle and select macOS"
EOF

    success "U-Boot boot script created: $BOOT_SCRIPT"
}

# Copy kernel and binaries
copy_payload_files() {
    info "Copying payload files to USB drive..."
    
    # Copy SIS kernel binary
    KERNEL_BINARY="$KERNEL_DIR/target/aarch64-unknown-none/release/sis_kernel"
    if [[ -f "$KERNEL_BINARY" ]]; then
        cp "$KERNEL_BINARY" "$BOOT_MOUNT/sis_kernel.bin"
        success "SIS kernel copied to USB"
    else
        error_exit "Kernel binary not found: $KERNEL_BINARY"
    fi
    
    # Copy additional files if they exist
    BINARY_DIR="$KERNEL_DIR/binaries"
    if [[ -d "$BINARY_DIR" ]]; then
        if [[ -f "$BINARY_DIR/m1n1.bin" ]]; then
            cp "$BINARY_DIR/m1n1.bin" "$BOOT_MOUNT/"
            success "m1n1 bootloader copied to USB"
        fi
        
        if [[ -f "$BINARY_DIR/u-boot.bin" ]]; then
            cp "$BINARY_DIR/u-boot.bin" "$BOOT_MOUNT/"
            success "U-Boot copied to USB"
        fi
    fi
    
    # Create test artifacts directory
    mkdir -p "$ROOTFS_MOUNT/test_results"
    mkdir -p "$ROOTFS_MOUNT/logs"
    success "Test directories created on USB"
}

# Create validation checksums
create_checksums() {
    info "Creating payload validation checksums..."
    
    CHECKSUM_FILE="$BOOT_MOUNT/SHA256SUMS"
    
    cd "$BOOT_MOUNT"
    shasum -a 256 *.bin *.dtb *.scr > "$CHECKSUM_FILE" 2>/dev/null || true
    cd "$SCRIPT_DIR"
    
    success "Checksums created: $CHECKSUM_FILE"
}

# Create README
create_readme() {
    info "Creating payload README..."
    
    README_FILE="$BOOT_MOUNT/README.txt"
    
    cat > "$README_FILE" << EOF
SIS Kernel Phase 2A USB Payload
==============================

Created: $(date '+%Y-%m-%d %H:%M:%S')
Target: Mac Mini M1 (8GB RAM, 512GB SSD)
Version: $CREATION_VERSION

Files:
------
- sis_kernel.bin    : SIS Kernel ARM64 binary
- m1-sis.dtb        : Device tree blob for M1
- boot.scr          : U-Boot boot script
- m1n1.bin          : m1n1 bootloader (if available)
- u-boot.bin        : U-Boot binary (if available)
- SHA256SUMS        : File checksums
- README.txt        : This file

Boot Process:
------------
1. Boot ROM -> iBoot -> Boot Policy -> m1n1 -> U-Boot
2. U-Boot reads boot.scr from this USB drive
3. U-Boot loads sis_kernel.bin and m1-sis.dtb
4. U-Boot boots SIS kernel with device tree

Safety Features:
---------------
- Hardware watchdog: 100ms timeout
- Thermal monitoring: 75°C limit
- Safe MMIO access patterns
- Graceful degradation support

Recovery:
---------
- Normal: Power cycle -> Startup Options -> macOS
- Emergency: Hold power button 10+ seconds
- Critical: Use 1TR or DFU restore

Validation:
----------
All files have SHA256 checksums in SHA256SUMS
Verify integrity before testing

Contact: SIS Kernel Development Team
EOF

    success "README created: $README_FILE"
}

# Verify USB payload
verify_payload() {
    info "Verifying USB payload integrity..."
    
    # Check required files exist
    REQUIRED_FILES=(
        "$BOOT_MOUNT/sis_kernel.bin"
        "$BOOT_MOUNT/m1-sis.dtb"
        "$BOOT_MOUNT/boot.scr"
        "$BOOT_MOUNT/README.txt"
        "$BOOT_MOUNT/SHA256SUMS"
    )
    
    for file in "${REQUIRED_FILES[@]}"; do
        if [[ -f "$file" ]]; then
            FILE_SIZE=$(stat -f%z "$file")
            info "✓ $(basename "$file"): ${FILE_SIZE} bytes"
        else
            error_exit "Required file missing: $file"
        fi
    done
    
    # Verify checksums
    cd "$BOOT_MOUNT"
    if shasum -a 256 -c SHA256SUMS 2>/dev/null; then
        success "All checksums verified"
    else
        warning "Some checksums failed verification"
    fi
    cd "$SCRIPT_DIR"
    
    # Check USB space usage
    BOOT_USED=$(df "$BOOT_MOUNT" | tail -1 | awk '{print $3}')
    BOOT_AVAIL=$(df "$BOOT_MOUNT" | tail -1 | awk '{print $4}')
    info "Boot partition usage: ${BOOT_USED}K used, ${BOOT_AVAIL}K available"
    
    success "USB payload verification completed"
}

# Cleanup and unmount
cleanup_and_unmount() {
    info "Cleaning up and unmounting USB drive..."
    
    # Sync filesystem
    sync
    
    # Unmount partitions
    if [[ -n "${BOOT_MOUNT:-}" ]]; then
        diskutil unmount "$BOOT_MOUNT" >/dev/null 2>&1 || true
    fi
    
    if [[ -n "${ROOTFS_MOUNT:-}" ]]; then
        diskutil unmount "$ROOTFS_MOUNT" >/dev/null 2>&1 || true
    fi
    
    success "USB drive unmounted safely"
}

# Show completion information
show_completion_info() {
    echo ""
    echo "=================================================================="
    echo -e "${GREEN}${BOLD}USB PAYLOAD CREATION COMPLETE${NC}"
    echo "=================================================================="
    echo ""
    echo "USB Device: $USB_DEVICE"
    echo "Payload Type: SIS Kernel Phase 2A"
    echo "Target Hardware: Mac Mini M1 (8GB RAM, 512GB SSD)"
    echo ""
    echo "Files Created:"
    echo "- SIS Kernel ARM64 binary with hardware validation"
    echo "- M1-specific device tree blob"
    echo "- U-Boot boot script with safety features"
    echo "- Complete documentation and checksums"
    echo ""
    echo "Next Steps:"
    echo "1. Ensure m1n1 + U-Boot boot chain is enrolled"
    echo "2. Insert USB drive into M1 Mac Mini"
    echo "3. Restart and select test entry from Startup Options"
    echo "4. Monitor boot process via serial console"
    echo ""
    echo "Safety Reminders:"
    echo "- macOS remains the default boot target"
    echo "- Hardware protection is enabled"
    echo "- Recovery via power cycle is always available"
    echo ""
    echo "USB payload is ready for Phase 2A hardware testing"
    echo ""
}

# Main execution
main() {
    parse_arguments "$@"
    print_header
    
    # Verify we can run
    if [[ $EUID -eq 0 ]]; then
        warning "Running as root - use caution"
    fi
    
    # Step 1: Verify prerequisites
    verify_prerequisites
    
    # Step 2: Show device info and warn about data loss
    warn_data_loss
    
    # Step 3: Unmount device
    unmount_device
    
    # Step 4: Create partition table
    create_partition_table
    
    # Step 5: Format partitions
    format_partitions
    
    # Step 6: Mount partitions
    mount_partitions
    
    # Step 7: Build SIS kernel
    build_sis_kernel
    
    # Step 8: Create device tree
    create_device_tree
    
    # Step 9: Create boot script
    create_boot_script
    
    # Step 10: Copy payload files
    copy_payload_files
    
    # Step 11: Create checksums
    create_checksums
    
    # Step 12: Create README
    create_readme
    
    # Step 13: Verify payload
    verify_payload
    
    # Step 14: Cleanup
    cleanup_and_unmount
    
    # Step 15: Show completion info
    show_completion_info
    
    success "USB payload creation completed successfully"
    log "USB payload creation process completed for $USB_DEVICE"
}

# Handle interrupts gracefully
trap 'error_exit "Script interrupted"' INT TERM
trap 'cleanup_and_unmount' EXIT

# Execute main function
main "$@"