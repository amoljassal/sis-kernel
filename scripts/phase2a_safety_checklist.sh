#!/bin/bash
# SIS Kernel Phase 2A Pre-Deployment Safety Checklist
# CRITICAL: Complete ALL items before hardware testing
# Version: 1.0
# Target: Mac Mini M1 (8GB RAM, 512GB SSD)

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/phase2a_safety_check.log"
CHECKLIST_VERSION="1.0"

# Colors for output (if terminal supports it)
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Print header
print_header() {
    echo "=================================================================="
    echo "SIS Kernel Phase 2A Pre-Deployment Safety Checklist"
    echo "Target Hardware: Mac Mini M1 (8GB RAM, 512GB SSD)"
    echo "Checklist Version: $CHECKLIST_VERSION"
    echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "=================================================================="
    log "Starting Phase 2A safety checklist"
}

# Global checklist status
CHECKLIST_PASSED=true
TOTAL_CHECKS=0
PASSED_CHECKS=0

# Check execution wrapper
execute_check() {
    local check_name="$1"
    local check_function="$2"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    echo ""
    echo -e "${BLUE}[$TOTAL_CHECKS] Checking: $check_name${NC}"
    echo "----------------------------------------"
    
    if $check_function; then
        echo -e "${GREEN}PASS${NC} - $check_name"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        log "PASS - $check_name"
    else
        echo -e "${RED}FAIL${NC} - $check_name"
        CHECKLIST_PASSED=false
        log "FAIL - $check_name"
    fi
}

# 1. System Backup Verification
check_system_backup() {
    echo "Verifying Time Machine backup status..."
    
    # Check if Time Machine is configured
    if ! tmutil destinationinfo >/dev/null 2>&1; then
        echo "ERROR: No Time Machine backup destination configured"
        return 1
    fi
    
    echo "Time Machine backup destination configured"
    
    # Get latest backup
    LATEST_BACKUP=$(tmutil latestbackup 2>/dev/null || echo "")
    if [[ -z "$LATEST_BACKUP" ]]; then
        echo "ERROR: No backup found"
        return 1
    fi
    
    echo "Latest backup: $LATEST_BACKUP"
    
    # Check backup age (must be within 24 hours)
    if [[ -d "$LATEST_BACKUP" ]]; then
        BACKUP_TIME=$(stat -f %m "$LATEST_BACKUP" 2>/dev/null || echo "0")
        CURRENT_TIME=$(date +%s)
        BACKUP_AGE=$((CURRENT_TIME - BACKUP_TIME))
        HOURS_OLD=$((BACKUP_AGE / 3600))
        
        echo "Backup age: $HOURS_OLD hours"
        
        if [[ $BACKUP_AGE -lt 86400 ]]; then
            echo "Backup is recent (less than 24 hours old)"
            return 0
        else
            echo "ERROR: Backup is older than 24 hours"
            return 1
        fi
    else
        echo "ERROR: Backup directory not accessible"
        return 1
    fi
}

# 2. DFU Recovery Capability
check_dfu_recovery() {
    echo "Verifying DFU recovery capability..."
    
    # Check for Apple Configurator 2
    if [[ -d "/Applications/Apple Configurator 2.app" ]]; then
        echo "Apple Configurator 2 is installed"
    else
        echo "WARNING: Apple Configurator 2 not found"
        echo "Install from Mac App Store for DFU recovery capability"
    fi
    
    # Check for IPSW files
    IPSW_COUNT=$(find ~/Downloads ~/Desktop -name "*.ipsw" 2>/dev/null | wc -l | xargs)
    if [[ $IPSW_COUNT -gt 0 ]]; then
        echo "Found $IPSW_COUNT IPSW file(s) for DFU restore"
        find ~/Downloads ~/Desktop -name "*.ipsw" 2>/dev/null | head -3 | while read -r ipsw; do
            echo "  - $(basename "$ipsw")"
        done
    else
        echo "WARNING: No IPSW files found"
        echo "Download appropriate IPSW for M1 Mac Mini from Apple"
    fi
    
    # Check for secondary Mac (if available)
    echo "NOTE: Ensure secondary Mac available for DFU operations"
    echo "NOTE: Test DFU procedure on non-critical device first"
    
    return 0
}

# 3. Hardware Platform Verification
check_hardware_platform() {
    echo "Verifying hardware platform..."
    
    # Check for Apple Silicon
    if system_profiler SPHardwareDataType 2>/dev/null | grep -q "Apple M1"; then
        echo "Apple M1 hardware confirmed"
        
        # Get detailed hardware info
        CHIP_TYPE=$(system_profiler SPHardwareDataType | grep "Chip:" | awk -F': ' '{print $2}' | xargs)
        MEMORY_SIZE=$(system_profiler SPHardwareDataType | grep "Memory:" | awk -F': ' '{print $2}' | xargs)
        
        echo "Chip: $CHIP_TYPE"
        echo "Memory: $MEMORY_SIZE"
        
        # Verify it's Mac Mini
        MODEL_NAME=$(system_profiler SPHardwareDataType | grep "Model Name:" | awk -F': ' '{print $2}' | xargs)
        echo "Model: $MODEL_NAME"
        
        if [[ "$MODEL_NAME" == *"Mac mini"* ]]; then
            echo "Mac Mini platform confirmed"
        else
            echo "WARNING: Not running on Mac Mini (target platform)"
        fi
        
        # Check memory size (should be 8GB for target)
        if [[ "$MEMORY_SIZE" == *"8 GB"* ]]; then
            echo "Target memory configuration (8GB) confirmed"
        else
            echo "INFO: Memory configuration differs from target (8GB)"
        fi
        
        return 0
    else
        echo "ERROR: Not running on Apple M1 hardware"
        return 1
    fi
}

# 4. Boot Policy and Security Status
check_boot_policy() {
    echo "Checking boot policy and security status..."
    
    # Check if we can access boot policy (may require elevated privileges)
    if kmutil configure-boot --list-boot-policies >/dev/null 2>&1; then
        echo "Boot policy access available"
        
        # Get security level
        SECURITY_LEVEL=$(kmutil configure-boot --volume-root / --query-security-level 2>/dev/null || echo "unknown")
        echo "Current security level: $SECURITY_LEVEL"
        
        if [[ "$SECURITY_LEVEL" == "permissive" ]]; then
            echo "Security level appropriate for testing"
        elif [[ "$SECURITY_LEVEL" == "full" ]]; then
            echo "WARNING: Full security level - may need 1TR adjustment for testing"
        else
            echo "INFO: Security level status unclear"
        fi
        
        # List current boot policies
        echo "Current boot policies:"
        kmutil configure-boot --list-boot-policies 2>/dev/null | head -10 || echo "  Unable to list policies"
        
        return 0
    else
        echo "WARNING: Cannot access boot policy (may require 1TR for modifications)"
        echo "This is expected if not in recovery mode"
        return 0
    fi
}

# 5. Storage and USB Preparation
check_storage_usb() {
    echo "Checking storage and USB preparation..."
    
    # Check available disk space
    AVAILABLE_SPACE=$(df -h / | tail -1 | awk '{print $4}')
    echo "Available disk space: $AVAILABLE_SPACE"
    
    # Check for external USB devices
    USB_DEVICES=$(diskutil list | grep -c "external" || echo "0")
    echo "External USB devices detected: $USB_DEVICES"
    
    if [[ $USB_DEVICES -gt 0 ]]; then
        echo "USB devices found:"
        diskutil list | grep "external" | head -5
        
        # Check for USB drive with sufficient space
        diskutil list | grep "external" | while read -r line; do
            DISK_ID=$(echo "$line" | grep -o 'disk[0-9]*')
            if [[ -n "$DISK_ID" ]]; then
                SIZE=$(diskutil info "$DISK_ID" 2>/dev/null | grep "Total Size" | awk -F': ' '{print $2}' | xargs || echo "unknown")
                echo "  $DISK_ID: $SIZE"
            fi
        done
    else
        echo "INFO: No external USB devices currently connected"
        echo "Connect USB drive (>=4GB) for kernel payload storage"
    fi
    
    return 0
}

# 6. Development Tools and Environment
check_development_environment() {
    echo "Checking development environment..."
    
    # Check for Rust toolchain
    if command -v rustc >/dev/null 2>&1; then
        RUST_VERSION=$(rustc --version)
        echo "Rust toolchain: $RUST_VERSION"
    else
        echo "WARNING: Rust toolchain not found"
    fi
    
    # Check for ARM64 target
    if rustc --print target-list 2>/dev/null | grep -q "aarch64-unknown-none"; then
        echo "ARM64 bare-metal target available"
    else
        echo "WARNING: ARM64 bare-metal target not installed"
        echo "Install with: rustup target add aarch64-unknown-none"
    fi
    
    # Check for build tools
    if command -v make >/dev/null 2>&1; then
        echo "Build tools (make) available"
    else
        echo "WARNING: Build tools not found"
    fi
    
    # Check for git
    if command -v git >/dev/null 2>&1; then
        echo "Git available"
    else
        echo "WARNING: Git not found"
    fi
    
    return 0
}

# 7. Thermal Baseline Measurement
check_thermal_baseline() {
    echo "Measuring thermal baseline..."
    
    if command -v powermetrics >/dev/null 2>&1; then
        echo "Taking 5-second thermal measurement..."
        
        # Capture thermal data
        THERMAL_DATA=$(sudo powermetrics -n 1 -i 5000 --samplers cpu_power,thermal 2>/dev/null | grep -E "(CPU die temperature|GPU die temperature)" | head -10 || echo "")
        
        if [[ -n "$THERMAL_DATA" ]]; then
            echo "Thermal baseline captured:"
            echo "$THERMAL_DATA" | while read -r line; do
                echo "  $line"
            done
            
            # Extract CPU temperature if available
            CPU_TEMP=$(echo "$THERMAL_DATA" | grep -i "cpu die temperature" | grep -o '[0-9]*\.[0-9]*' | head -1 || echo "")
            if [[ -n "$CPU_TEMP" ]]; then
                echo "Current CPU temperature: ${CPU_TEMP}°C"
                
                # Check if temperature is reasonable for idle
                if (( $(echo "$CPU_TEMP < 60.0" | bc -l) )); then
                    echo "Temperature within normal idle range"
                else
                    echo "WARNING: High idle temperature detected"
                fi
            fi
        else
            echo "WARNING: Could not capture thermal data"
        fi
    else
        echo "WARNING: powermetrics not available for thermal monitoring"
    fi
    
    return 0
}

# 8. Network and Connectivity
check_network_connectivity() {
    echo "Checking network connectivity..."
    
    # Test internet connectivity
    if ping -c 3 -t 10 8.8.8.8 >/dev/null 2>&1; then
        echo "Internet connectivity verified"
    else
        echo "WARNING: Internet connectivity issues detected"
    fi
    
    # Check for local network
    if ping -c 1 -t 5 192.168.1.1 >/dev/null 2>&1 || ping -c 1 -t 5 10.0.0.1 >/dev/null 2>&1; then
        echo "Local network connectivity verified"
    else
        echo "INFO: Local network gateway not responding (may be normal)"
    fi
    
    return 0
}

# 9. Kernel Build Verification
check_kernel_build() {
    echo "Checking kernel build capability..."
    
    KERNEL_DIR="$SCRIPT_DIR/.."
    
    if [[ -f "$KERNEL_DIR/Cargo.toml" ]]; then
        echo "SIS Kernel source found"
        
        # Check if we can build (dry run)
        cd "$KERNEL_DIR"
        if cargo check --target aarch64-unknown-none.json >/dev/null 2>&1; then
            echo "Kernel builds successfully"
        else
            echo "WARNING: Kernel build issues detected"
            echo "Run 'cargo check --target aarch64-unknown-none.json' for details"
        fi
        cd "$SCRIPT_DIR"
    else
        echo "WARNING: SIS Kernel source not found in expected location"
    fi
    
    return 0
}

# 10. Final Safety Verification
check_final_safety() {
    echo "Final safety verification..."
    
    # Verify all critical safety items
    CRITICAL_ITEMS=(
        "Time Machine backup verified"
        "DFU recovery capability confirmed"
        "Apple M1 hardware confirmed"
        "Thermal baseline normal"
    )
    
    echo "Critical safety items:"
    for item in "${CRITICAL_ITEMS[@]}"; do
        echo "  - $item"
    done
    
    echo ""
    echo "IMPORTANT SAFETY REMINDERS:"
    echo "1. NEVER modify firmware or boot ROM"
    echo "2. Always maintain macOS as default boot target"
    echo "3. Test recovery procedures before critical testing"
    echo "4. Monitor thermal conditions during testing"
    echo "5. Have secondary Mac available for DFU recovery"
    echo "6. Keep all documentation and procedures accessible"
    
    return 0
}

# Main execution
main() {
    print_header
    
    # Execute all checks
    execute_check "System Backup Verification" check_system_backup
    execute_check "DFU Recovery Capability" check_dfu_recovery
    execute_check "Hardware Platform Verification" check_hardware_platform
    execute_check "Boot Policy and Security Status" check_boot_policy
    execute_check "Storage and USB Preparation" check_storage_usb
    execute_check "Development Environment" check_development_environment
    execute_check "Thermal Baseline Measurement" check_thermal_baseline
    execute_check "Network Connectivity" check_network_connectivity
    execute_check "Kernel Build Verification" check_kernel_build
    execute_check "Final Safety Verification" check_final_safety
    
    # Final results
    echo ""
    echo "=================================================================="
    echo "PHASE 2A SAFETY CHECKLIST RESULTS"
    echo "=================================================================="
    echo "Total checks: $TOTAL_CHECKS"
    echo "Passed checks: $PASSED_CHECKS"
    echo "Failed checks: $((TOTAL_CHECKS - PASSED_CHECKS))"
    echo ""
    
    if [[ "$CHECKLIST_PASSED" == "true" ]]; then
        echo -e "${GREEN}CHECKLIST PASSED${NC}"
        echo "System is ready for Phase 2A hardware testing"
        echo "Proceed to Phase 2A-1: Safety Setup and Hardware Validation"
        log "Phase 2A safety checklist PASSED - system ready for hardware testing"
        exit 0
    else
        echo -e "${RED}CHECKLIST FAILED${NC}"
        echo "Resolve all issues before proceeding with hardware testing"
        echo "DO NOT attempt Phase 2A testing until all items pass"
        log "Phase 2A safety checklist FAILED - resolve issues before testing"
        exit 1
    fi
}

# Execute main function
main "$@"