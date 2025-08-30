#!/bin/bash
# SIS Kernel Boot Policy Enrollment Script
# CRITICAL: Only run after complete backup and DFU preparation
# Version: 1.0
# Target: Mac Mini M1 (8GB RAM, 512GB SSD)

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/boot_policy_enrollment.log"
ENROLLMENT_VERSION="1.0"

# Default binary paths (can be overridden)
M1N1_BINARY="${M1N1_BINARY:-$SCRIPT_DIR/../binaries/m1n1.bin}"
UBOOT_BINARY="${UBOOT_BINARY:-$SCRIPT_DIR/../binaries/u-boot.bin}"
SIS_KERNEL_BINARY="${SIS_KERNEL_BINARY:-$SCRIPT_DIR/../target/aarch64-unknown-none/release/sis_kernel.bin}"

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

# Print header
print_header() {
    echo "=================================================================="
    echo -e "${BOLD}SIS Kernel Boot Policy Enrollment${NC}"
    echo "Target Hardware: Mac Mini M1 (8GB RAM, 512GB SSD)"
    echo "Enrollment Version: $ENROLLMENT_VERSION"
    echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "=================================================================="
    log "Starting boot policy enrollment process"
}

# Verify prerequisites
verify_prerequisites() {
    info "Verifying prerequisites for boot policy enrollment..."
    
    # Check if running on Apple Silicon
    if ! system_profiler SPHardwareDataType 2>/dev/null | grep -q "Apple M1"; then
        error_exit "Not running on Apple M1 hardware"
    fi
    success "Apple M1 hardware confirmed"
    
    # Check if in 1TR or have appropriate permissions
    if ! kmutil configure-boot --list-boot-policies >/dev/null 2>&1; then
        error_exit "Must run from 1TR with administrator privileges"
    fi
    success "Boot policy access confirmed"
    
    # Check security level
    SECURITY_LEVEL=$(kmutil configure-boot --volume-root / --query-security-level 2>/dev/null || echo "unknown")
    info "Current security level: $SECURITY_LEVEL"
    
    if [[ "$SECURITY_LEVEL" != "permissive" ]]; then
        warning "Security level is not permissive. Current: $SECURITY_LEVEL"
        echo "For fuOS enrollment, security level should be 'permissive'"
        echo "Set via 1TR > Utilities > Startup Security Utility"
        read -p "Continue anyway? (y/N): " CONTINUE
        if [[ "$CONTINUE" != "y" && "$CONTINUE" != "Y" ]]; then
            error_exit "Enrollment cancelled - adjust security level first"
        fi
    else
        success "Security level is permissive - ready for fuOS enrollment"
    fi
    
    # Verify required binaries exist
    if [[ ! -f "$M1N1_BINARY" ]]; then
        error_exit "m1n1 binary not found: $M1N1_BINARY"
    fi
    success "m1n1 binary found: $M1N1_BINARY"
    
    if [[ ! -f "$UBOOT_BINARY" ]]; then
        error_exit "U-Boot binary not found: $UBOOT_BINARY"
    fi
    success "U-Boot binary found: $UBOOT_BINARY"
    
    # SIS kernel binary is optional at this stage
    if [[ -f "$SIS_KERNEL_BINARY" ]]; then
        success "SIS kernel binary found: $SIS_KERNEL_BINARY"
    else
        warning "SIS kernel binary not found: $SIS_KERNEL_BINARY"
        warning "This is OK for initial boot chain setup"
    fi
}

# Calculate and display binary hashes
calculate_hashes() {
    info "Calculating binary hashes for verification..."
    
    M1N1_HASH=$(shasum -a 256 "$M1N1_BINARY" | cut -d' ' -f1)
    UBOOT_HASH=$(shasum -a 256 "$UBOOT_BINARY" | cut -d' ' -f1)
    
    echo ""
    echo "Binary Verification Hashes:"
    echo "=========================="
    echo "m1n1:    $M1N1_HASH"
    echo "U-Boot:  $UBOOT_HASH"
    
    if [[ -f "$SIS_KERNEL_BINARY" ]]; then
        SIS_KERNEL_HASH=$(shasum -a 256 "$SIS_KERNEL_BINARY" | cut -d' ' -f1)
        echo "SIS Kernel: $SIS_KERNEL_HASH"
    fi
    echo ""
    
    log "Binary hashes calculated - m1n1: $M1N1_HASH, U-Boot: $UBOOT_HASH"
}

# Show current boot policy
show_current_policy() {
    info "Current boot policy configuration:"
    echo ""
    
    if kmutil configure-boot --list-boot-policies 2>/dev/null | head -20; then
        echo ""
    else
        warning "Could not retrieve current boot policy"
    fi
    
    # Show current default boot volume
    STARTUP_DISK=$(bless --info / 2>/dev/null | grep "Boot Device" || echo "Unknown")
    info "Current startup disk: $STARTUP_DISK"
}

# Backup current boot policy
backup_boot_policy() {
    info "Backing up current boot policy..."
    
    BACKUP_FILE="$SCRIPT_DIR/boot_policy_backup_$(date +%Y%m%d_%H%M%S).txt"
    
    if kmutil configure-boot --list-boot-policies > "$BACKUP_FILE" 2>&1; then
        success "Boot policy backed up to: $BACKUP_FILE"
    else
        warning "Could not backup boot policy"
    fi
}

# Create combined m1n1 + U-Boot image
create_boot_image() {
    info "Creating combined boot image..."
    
    BOOT_IMAGE="$SCRIPT_DIR/../binaries/sis_boot_chain.bin"
    
    # For now, we'll use m1n1 as the primary boot image
    # U-Boot will be loaded by m1n1 from storage
    cp "$M1N1_BINARY" "$BOOT_IMAGE"
    
    success "Boot image created: $BOOT_IMAGE"
    
    # Calculate hash of boot image
    BOOT_IMAGE_HASH=$(shasum -a 256 "$BOOT_IMAGE" | cut -d' ' -f1)
    info "Boot image hash: $BOOT_IMAGE_HASH"
}

# Enroll boot policy
enroll_boot_policy() {
    info "Enrolling SIS Kernel boot policy..."
    
    BOOT_IMAGE="$SCRIPT_DIR/../binaries/sis_boot_chain.bin"
    
    if [[ ! -f "$BOOT_IMAGE" ]]; then
        error_exit "Boot image not found: $BOOT_IMAGE"
    fi
    
    echo ""
    echo -e "${YELLOW}CRITICAL WARNING${NC}"
    echo "================"
    echo "About to enroll custom boot policy for SIS Kernel testing"
    echo "This will modify the system's secure boot configuration"
    echo ""
    echo "Boot image: $BOOT_IMAGE"
    echo "Hash: $(shasum -a 256 "$BOOT_IMAGE" | cut -d' ' -f1)"
    echo ""
    echo "Recovery options if enrollment fails:"
    echo "1. Use 1TR to remove enrolled policy"
    echo "2. Use DFU restore with Apple Configurator 2"
    echo ""
    echo -e "${BOLD}macOS will remain the default boot target${NC}"
    echo ""
    
    read -p "Type 'YES' to proceed with enrollment: " CONFIRM
    if [[ "$CONFIRM" != "YES" ]]; then
        error_exit "Enrollment cancelled by user"
    fi
    
    # Perform enrollment
    info "Executing kmutil configure-boot..."
    
    if kmutil configure-boot --volume-root / --update-policy --raw --payload "$BOOT_IMAGE"; then
        success "Boot policy enrollment completed successfully"
        
        # Verify enrollment
        info "Verifying enrollment..."
        BOOT_IMAGE_HASH=$(shasum -a 256 "$BOOT_IMAGE" | cut -d' ' -f1)
        
        if kmutil configure-boot --list-boot-policies | grep -q "$BOOT_IMAGE_HASH"; then
            success "Enrollment verified - hash found in boot policy"
        else
            warning "Could not verify enrollment in boot policy list"
        fi
        
    else
        error_exit "Boot policy enrollment failed"
    fi
}

# Set macOS as default boot target
ensure_macos_default() {
    info "Ensuring macOS remains the default boot target..."
    
    # Get root volume
    ROOT_VOLUME=$(df / | tail -1 | awk '{print $1}')
    
    if bless --mount / --setBoot 2>/dev/null; then
        success "macOS confirmed as default boot target"
    else
        warning "Could not explicitly set macOS as default"
        warning "Manually set in System Preferences > Startup Disk"
    fi
}

# Create boot test script
create_boot_test_script() {
    info "Creating boot test helper script..."
    
    BOOT_TEST_SCRIPT="$SCRIPT_DIR/test_sis_boot.sh"
    
    cat > "$BOOT_TEST_SCRIPT" << 'EOF'
#!/bin/bash
# SIS Kernel Boot Test Helper
# Use this script to guide testing the enrolled boot policy

echo "SIS Kernel Boot Test Helper"
echo "=========================="
echo ""
echo "To test the SIS Kernel boot chain:"
echo ""
echo "1. Restart the Mac Mini"
echo "2. Hold the power button until 'Loading startup options' appears"
echo "3. You should see multiple boot options:"
echo "   - Macintosh HD (macOS) - DEFAULT"
echo "   - Your custom boot entry (SIS Kernel test chain)"
echo ""
echo "4. Select the SIS Kernel test entry to boot"
echo "5. Monitor the boot process via serial console if available"
echo ""
echo "RECOVERY:"
echo "- If boot fails, power cycle and select macOS"
echo "- If boot loop occurs, hold power button for 10+ seconds"
echo "- For emergencies, use 1TR to remove enrolled policy"
echo ""
echo "The enrolled boot policy can be removed with:"
echo "sudo kmutil configure-boot --volume-root / --update-policy --remove [hash]"
echo ""
EOF

    chmod +x "$BOOT_TEST_SCRIPT"
    success "Boot test helper created: $BOOT_TEST_SCRIPT"
}

# Show post-enrollment information
show_post_enrollment_info() {
    info "Boot Policy Enrollment Complete"
    echo ""
    echo "=================================================================="
    echo -e "${GREEN}ENROLLMENT SUCCESSFUL${NC}"
    echo "=================================================================="
    echo ""
    echo "What was enrolled:"
    echo "- Boot chain: m1n1 → U-Boot → SIS Kernel"
    echo "- Binary hash: $(shasum -a 256 "$SCRIPT_DIR/../binaries/sis_boot_chain.bin" | cut -d' ' -f1)"
    echo ""
    echo "Safety measures:"
    echo "- macOS remains the DEFAULT boot target"
    echo "- SIS test entry available in Startup Options"
    echo "- Manual selection required for testing"
    echo ""
    echo "Next steps:"
    echo "1. Prepare USB payload with SIS kernel and U-Boot configuration"
    echo "2. Test boot chain using Startup Options"
    echo "3. Monitor via serial console for debugging"
    echo ""
    echo "Recovery procedures:"
    echo "- Normal: Restart and select macOS"
    echo "- Emergency: Power cycle (hold power 10+ seconds)"
    echo "- Critical: Use 1TR to remove policy or DFU restore"
    echo ""
    echo "Boot policy can be removed with:"
    echo "sudo kmutil configure-boot --volume-root / --update-policy --remove $(shasum -a 256 "$SCRIPT_DIR/../binaries/sis_boot_chain.bin" | cut -d' ' -f1)"
    echo ""
}

# Main execution
main() {
    print_header
    
    # Verify we can run
    if [[ $EUID -ne 0 ]]; then
        error_exit "This script must be run as root (sudo)"
    fi
    
    # Step 1: Verify prerequisites
    verify_prerequisites
    
    # Step 2: Calculate and display hashes
    calculate_hashes
    
    # Step 3: Show current boot policy
    show_current_policy
    
    # Step 4: Backup current policy
    backup_boot_policy
    
    # Step 5: Create boot image
    create_boot_image
    
    # Step 6: Final confirmation
    echo ""
    echo -e "${BOLD}FINAL CONFIRMATION${NC}"
    echo "=================="
    echo "Ready to enroll SIS Kernel boot policy"
    echo "This will modify system boot configuration"
    echo ""
    read -p "Proceed with enrollment? (y/N): " FINAL_CONFIRM
    if [[ "$FINAL_CONFIRM" != "y" && "$FINAL_CONFIRM" != "Y" ]]; then
        info "Enrollment cancelled by user"
        exit 0
    fi
    
    # Step 7: Enroll boot policy
    enroll_boot_policy
    
    # Step 8: Ensure macOS remains default
    ensure_macos_default
    
    # Step 9: Create helper scripts
    create_boot_test_script
    
    # Step 10: Show completion info
    show_post_enrollment_info
    
    success "Boot policy enrollment completed successfully"
    log "Boot policy enrollment process completed"
}

# Handle interrupts gracefully
trap 'error_exit "Script interrupted"' INT TERM

# Execute main function
main "$@"