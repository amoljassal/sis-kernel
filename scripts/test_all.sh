#!/bin/bash
set -e

export PATH="$HOME/.cargo/bin:$PATH"
source ~/.cargo/env

echo "=== SIS Kernel Test Suite ==="

TESTS=("USR_INIT" "USR_SPAWN_TWO" "USR_ELF_EDGES" "USR_VFS_NEG")

for TEST_NAME in "${TESTS[@]}"; do
    echo ""
    echo "--- Running test: $TEST_NAME ---"
    
    # Clear previous output
    rm -f out/qemu-serial.log
    
    # Run test with longer timeout and capture exit code
    # Use gtimeout if available (GNU coreutils), otherwise timeout, with fallback
    if command -v gtimeout >/dev/null 2>&1; then
        gtimeout 15 bash -c "TEST=$TEST_NAME BOOT=bios ./scripts/qemu.sh" || TEST_EXIT=$?
    elif command -v timeout >/dev/null 2>&1; then
        timeout 15 bash -c "TEST=$TEST_NAME BOOT=bios ./scripts/qemu.sh" || TEST_EXIT=$?
    else
        # Manual timeout using background process
        bash -c "TEST=$TEST_NAME BOOT=bios ./scripts/qemu.sh" &
        TEST_PID=$!
        sleep 15
        kill $TEST_PID 2>/dev/null || true
        wait $TEST_PID 2>/dev/null || TEST_EXIT=$?
    fi
    
    echo "Test $TEST_NAME completed with exit code: ${TEST_EXIT:-0}"
    
    # Show serial output if exists
    if [ -f out/qemu-serial.log ]; then
        echo "Serial output:"
        tail -10 out/qemu-serial.log
    fi
    
    echo "----------------------------------------"
done

echo ""
echo "=== Test Summary ==="
echo "All tests completed. Check individual outputs above."