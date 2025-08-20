# SIS Kernel WebAssembly Integration

This directory contains the WebAssembly bindings and TypeScript interface for the SIS Kernel.

## Architecture

```
┌─────────────────────────────────────────┐
│              React Frontend             │
│  (Design UI, Validation, Collaboration) │
└─────────────────┬───────────────────────┘
                  │ TypeScript Interface
┌─────────────────▼───────────────────────┐
│           WebAssembly Bridge            │
│    (sisKernel.ts, useWasm hooks)        │
└─────────────────┬───────────────────────┘
                  │ wasm-bindgen
┌─────────────────▼───────────────────────┐
│            SIS Kernel WASM              │
│  (Rust kernel compiled to WebAssembly) │
└─────────────────────────────────────────┘
```

## Building the WASM Module

### Prerequisites

1. Install wasm-pack:
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

2. Install Rust with WebAssembly target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Build Process

1. From the project root:
   ```bash
   cd wasm
   ./build.sh
   ```

2. This will:
   - Build the Rust kernel with WASM target
   - Generate TypeScript bindings
   - Copy the built package to `web/src/wasm/pkg/`

### Development Workflow

1. Make changes to kernel code
2. Rebuild WASM module: `cd wasm && ./build.sh`
3. Restart the web dev server: `cd web && npm run dev`

## API Overview

The WASM bridge exposes these main functions:

### Core Functions
- `initialize()` - Initialize kernel subsystems
- `get_version()` - Get kernel version info

### Design Validation
- `validate_design(nodes, connections)` - Full design validation
- `run_preflight_checks(design)` - Quick safety checks

### HDL Generation
- `generate_hdl(nodes, connections, target)` - Generate Verilog/VHDL
- `synthesize_design(hdl, target)` - Synthesize for hardware

### Hardware Management
- `get_hardware_status()` - Check FPGA availability
- `get_performance_metrics()` - Runtime performance data

## TypeScript Integration

### Using the SIS Kernel API

```typescript
import { getSisKernel } from './wasm/sisKernel'

const kernel = await getSisKernel()
const report = await kernel.validateDesign(nodes, connections)
```

### React Hooks

```typescript
import { useDesignValidation } from './hooks/useWasm'

const { validateDesign, isValidating } = useDesignValidation()
const report = await validateDesign(nodes, connections)
```

## Performance Considerations

- WASM module is lazy-loaded when first needed
- Large designs may require streaming/chunking
- Use Web Workers for heavy computations
- Cache validation results when possible

## Security Notes

- All user input is validated before passing to WASM
- WASM runs in sandboxed environment
- No direct file system access from WASM
- Network requests handled by JavaScript layer

## Troubleshooting

### Common Issues

1. **Build fails with "wasm-pack not found"**
   - Install wasm-pack: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`

2. **TypeScript errors about missing types**
   - Rebuild WASM: `cd wasm && ./build.sh`
   - Check that `pkg/` directory was copied to `web/src/wasm/pkg/`

3. **Runtime errors loading WASM**
   - Check browser console for detailed error messages
   - Ensure WASM file is served with correct MIME type
   - Verify file size limits (some servers restrict large files)

### Debug Mode

Enable WASM debug logging:
```typescript
// In browser console
localStorage.setItem('wasm-debug', 'true')
```

## File Structure

```
web/src/wasm/
├── README.md              # This file
├── sisKernel.ts          # TypeScript API wrapper
├── pkg/                  # Generated WASM package (after build)
│   ├── sis_kernel_wasm.js
│   ├── sis_kernel_wasm_bg.wasm
│   └── sis_kernel_wasm.d.ts
└── hooks/
    └── useWasm.ts        # React hooks for WASM integration
```