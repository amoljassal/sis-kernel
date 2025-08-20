#!/bin/bash

# Build script for SIS Kernel WebAssembly module
# This script builds the WASM module and copies it to the web project

set -e

echo "Building SIS Kernel WASM module..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "ERROR: wasm-pack is not installed. Please install it:"
    echo "   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "ERROR: Please run this script from the wasm directory"
    exit 1
fi

# Clean previous build
echo "Cleaning previous build..."
rm -rf pkg/

# Build the WASM package
echo "Building WASM package..."
wasm-pack build \
    --target web \
    --out-dir pkg \
    --release \
    --scope sis

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "SUCCESS: WASM build successful!"
    
    # Copy to web project
    WEB_WASM_DIR="../web/src/wasm/pkg"
    echo "Copying WASM package to web project..."
    
    # Create directory if it doesn't exist
    mkdir -p "$WEB_WASM_DIR"
    
    # Copy the built package
    cp -r pkg/* "$WEB_WASM_DIR/"
    
    echo "SUCCESS: WASM package copied to web project"
    echo "Package contents:"
    ls -la "$WEB_WASM_DIR"
    
    # Show package size
    WASM_SIZE=$(du -h "$WEB_WASM_DIR"/*.wasm | cut -f1)
    echo "WASM module size: $WASM_SIZE"
    
    echo ""
    echo "Build complete! The WASM module is ready for use."
    echo "To use in development:"
    echo "   cd ../web && npm run dev"
    
else
    echo "ERROR: WASM build failed!"
    exit 1
fi