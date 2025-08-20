//! SIS Kernel WebAssembly Bridge
//! 
//! This module provides WebAssembly bindings for the SIS Kernel, enabling
//! the React frontend to directly invoke kernel functions for design validation,
//! hardware synthesis, and safety checks.

use wasm_bindgen::prelude::*;
use js_sys::{Array, Object, Promise};
use web_sys::console;
use serde::{Deserialize, Serialize};

// Import specific kernel modules we want to expose
use sis_kernel::kernel::ai::safety_framework::{SafetyFramework, PreflightReport};
use sis_kernel::kernel::ai::validation_framework::ValidationFramework;
use sis_kernel::kernel::ai::dcon::DesignContract;
use sis_kernel::kernel::ai::design_graph::DesignGraph;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Set up panic hook for better error messages in console
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    console::log_1(&"SIS Kernel WASM module initialized".into());
}

// Utility function to log to browser console
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

//=============================================================================
// Data structures for JavaScript interop
//=============================================================================

#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmDesignNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
impl WasmDesignNode {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, name: String, node_type: String, x: f64, y: f64) -> WasmDesignNode {
        WasmDesignNode { id, name, node_type, x, y }
    }
    
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String { self.id.clone() }
    
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String { self.name.clone() }
    
    #[wasm_bindgen(getter)]
    pub fn node_type(&self) -> String { self.node_type.clone() }
    
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> f64 { self.x }
    
    #[wasm_bindgen(getter)]
    pub fn y(&self) -> f64 { self.y }
}

#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmDesignConnection {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub signal_name: String,
}

#[wasm_bindgen]
impl WasmDesignConnection {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, source_id: String, target_id: String, signal_name: String) -> WasmDesignConnection {
        WasmDesignConnection { id, source_id, target_id, signal_name }
    }
}

#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmValidationResult {
    pub success: bool,
    pub hazard_score: u8,
    pub errors: String, // JSON string of errors array
    pub warnings: String, // JSON string of warnings array
    pub duration_ms: u32,
}

#[wasm_bindgen]
impl WasmValidationResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool { self.success }
    
    #[wasm_bindgen(getter)]
    pub fn hazard_score(&self) -> u8 { self.hazard_score }
    
    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> String { self.errors.clone() }
    
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> String { self.warnings.clone() }
    
    #[wasm_bindgen(getter)]
    pub fn duration_ms(&self) -> u32 { self.duration_ms }
}

//=============================================================================
// Main SIS Kernel WebAssembly API
//=============================================================================

#[wasm_bindgen]
pub struct SisKernelWasm {
    safety_framework: SafetyFramework,
    validation_framework: ValidationFramework,
}

#[wasm_bindgen]
impl SisKernelWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SisKernelWasm {
        console_log!("Initializing SIS Kernel WASM instance");
        
        SisKernelWasm {
            safety_framework: SafetyFramework::new(),
            validation_framework: ValidationFramework::new(),
        }
    }
    
    /// Get kernel version and build info
    #[wasm_bindgen]
    pub fn get_version(&self) -> String {
        format!("SIS Kernel WASM v0.1.0 - Build {}", env!("CARGO_PKG_VERSION"))
    }
    
    /// Initialize kernel subsystems
    #[wasm_bindgen]
    pub fn initialize(&mut self) -> bool {
        console_log!("Initializing kernel subsystems...");
        
        // Initialize safety framework
        match self.safety_framework.initialize() {
            Ok(_) => console_log!("Safety framework initialized successfully"),
            Err(e) => {
                console_log!("Failed to initialize safety framework: {:?}", e);
                return false;
            }
        }
        
        // Initialize validation framework
        match self.validation_framework.initialize() {
            Ok(_) => console_log!("Validation framework initialized successfully"),
            Err(e) => {
                console_log!("Failed to initialize validation framework: {:?}", e);
                return false;
            }
        }
        
        console_log!("SIS Kernel fully initialized");
        true
    }
    
    /// Validate a design and return safety assessment
    #[wasm_bindgen]
    pub fn validate_design(&self, nodes_json: &str, connections_json: &str) -> WasmValidationResult {
        console_log!("Starting design validation...");
        let start_time = js_sys::Date::now();
        
        // Parse the JSON input (in a real implementation)
        // For now, we'll create a mock validation
        let node_count = match serde_json::from_str::<Vec<WasmDesignNode>>(nodes_json) {
            Ok(nodes) => nodes.len(),
            Err(_) => 0,
        };
        
        let conn_count = match serde_json::from_str::<Vec<WasmDesignConnection>>(connections_json) {
            Ok(connections) => connections.len(),
            Err(_) => 0,
        };
        
        console_log!("Validating design with {} nodes and {} connections", node_count, conn_count);
        
        // Mock validation logic - in reality this would call the actual kernel
        let hazard_score = if node_count == 0 {
            0
        } else if node_count > 10 {
            45 // Higher complexity = higher risk
        } else {
            15
        };
        
        let success = hazard_score < 50;
        let errors = if success {
            "[]".to_string()
        } else {
            r#"["Design complexity exceeds safety threshold"]"#.to_string()
        };
        
        let warnings = if node_count > 5 {
            r#"["Consider breaking design into smaller modules"]"#.to_string()
        } else {
            "[]".to_string()
        };
        
        let duration_ms = (js_sys::Date::now() - start_time) as u32;
        
        console_log!("Validation completed in {}ms, hazard score: {}", duration_ms, hazard_score);
        
        WasmValidationResult {
            success,
            hazard_score,
            errors,
            warnings,
            duration_ms,
        }
    }
    
    /// Run preflight safety checks
    #[wasm_bindgen]
    pub fn run_preflight_checks(&self, design_json: &str) -> u8 {
        console_log!("Running preflight safety checks...");
        
        // Mock preflight check - would integrate with actual safety framework
        let design_data: Result<serde_json::Value, _> = serde_json::from_str(design_json);
        
        match design_data {
            Ok(data) => {
                let node_count = data["nodes"].as_array().map_or(0, |arr| arr.len());
                // Simple heuristic for demo
                if node_count == 0 { 0 }
                else if node_count > 20 { 75 }
                else if node_count > 10 { 35 }
                else { 10 }
            }
            Err(_) => 100 // Invalid design = maximum hazard
        }
    }
    
    /// Generate hardware description language code
    #[wasm_bindgen]
    pub fn generate_hdl(&self, nodes_json: &str, connections_json: &str, target: &str) -> String {
        console_log!("Generating {} HDL for design...", target);
        
        // Mock HDL generation
        let nodes: Result<Vec<WasmDesignNode>, _> = serde_json::from_str(nodes_json);
        let connections: Result<Vec<WasmDesignConnection>, _> = serde_json::from_str(connections_json);
        
        match (nodes, connections) {
            (Ok(nodes), Ok(connections)) => {
                match target {
                    "verilog" => generate_verilog_hdl(&nodes, &connections),
                    "vhdl" => generate_vhdl_hdl(&nodes, &connections),
                    "systemverilog" => generate_systemverilog_hdl(&nodes, &connections),
                    _ => "// Unsupported HDL target".to_string()
                }
            }
            _ => "// Error: Invalid design data".to_string()
        }
    }
    
    /// Synthesize design for specific hardware target
    #[wasm_bindgen]
    pub fn synthesize_design(&self, hdl_code: &str, target: &str) -> Promise {
        console_log!("Starting synthesis for target: {}", target);
        
        // Return a Promise for async synthesis
        let promise = Promise::new(&mut |resolve, reject| {
            // Mock synthesis process
            let success = !hdl_code.contains("Error");
            
            if success {
                let result = js_sys::Object::new();
                js_sys::Reflect::set(&result, &"success".into(), &true.into()).unwrap();
                js_sys::Reflect::set(&result, &"utilization".into(), &85.5.into()).unwrap();
                js_sys::Reflect::set(&result, &"timing".into(), &"Met".into()).unwrap();
                js_sys::Reflect::set(&result, &"warnings".into(), &2.into()).unwrap();
                resolve.call1(&JsValue::UNDEFINED, &result).unwrap();
            } else {
                reject.call1(&JsValue::UNDEFINED, &"Synthesis failed".into()).unwrap();
            }
        });
        
        promise
    }
    
    /// Check hardware availability and status
    #[wasm_bindgen]
    pub fn get_hardware_status(&self) -> String {
        console_log!("Checking hardware availability...");
        
        // Mock hardware status
        let status = serde_json::json!({
            "available_boards": [
                {
                    "id": "xilinx_u250_1",
                    "type": "Xilinx Alveo U250",
                    "status": "available",
                    "utilization": 0
                },
                {
                    "id": "intel_arria10_1", 
                    "type": "Intel Arria 10",
                    "status": "busy",
                    "utilization": 75
                }
            ],
            "cloud_fpgas": {
                "aws_f1": {
                    "available": true,
                    "cost_per_hour": 1.65,
                    "regions": ["us-west-2", "us-east-1"]
                }
            },
            "simulation_available": true
        });
        
        status.to_string()
    }
}

//=============================================================================
// HDL Generation Helper Functions
//=============================================================================

fn generate_verilog_hdl(nodes: &[WasmDesignNode], connections: &[WasmDesignConnection]) -> String {
    let mut hdl = String::new();
    
    hdl.push_str("// Generated by SIS AI-Lab\n");
    hdl.push_str("// Verilog HDL\n\n");
    hdl.push_str("module sis_design(\n");
    hdl.push_str("    input clk,\n");
    hdl.push_str("    input rst_n\n");
    hdl.push_str(");\n\n");
    
    // Generate module instances for each node
    for node in nodes {
        hdl.push_str(&format!("    // Node: {} ({})\n", node.name, node.node_type));
        hdl.push_str(&format!("    {} {}(\n", node.node_type, node.id));
        hdl.push_str("        .clk(clk),\n");
        hdl.push_str("        .rst_n(rst_n)\n");
        hdl.push_str("    );\n\n");
    }
    
    // Generate wire connections
    for conn in connections {
        hdl.push_str(&format!("    wire {};\n", conn.signal_name));
    }
    
    hdl.push_str("endmodule\n");
    hdl
}

fn generate_vhdl_hdl(nodes: &[WasmDesignNode], connections: &[WasmDesignConnection]) -> String {
    let mut hdl = String::new();
    
    hdl.push_str("-- Generated by SIS AI-Lab\n");
    hdl.push_str("-- VHDL HDL\n\n");
    hdl.push_str("library IEEE;\n");
    hdl.push_str("use IEEE.std_logic_1164.all;\n\n");
    hdl.push_str("entity sis_design is\n");
    hdl.push_str("    port (\n");
    hdl.push_str("        clk : in std_logic;\n");
    hdl.push_str("        rst_n : in std_logic\n");
    hdl.push_str("    );\n");
    hdl.push_str("end sis_design;\n\n");
    hdl.push_str("architecture behavioral of sis_design is\n");
    
    // Generate signal declarations
    for conn in connections {
        hdl.push_str(&format!("    signal {} : std_logic;\n", conn.signal_name));
    }
    
    hdl.push_str("begin\n");
    
    // Generate component instances
    for node in nodes {
        hdl.push_str(&format!("    -- Node: {} ({})\n", node.name, node.node_type));
    }
    
    hdl.push_str("end behavioral;\n");
    hdl
}

fn generate_systemverilog_hdl(nodes: &[WasmDesignNode], connections: &[WasmDesignConnection]) -> String {
    let mut hdl = String::new();
    
    hdl.push_str("// Generated by SIS AI-Lab\n");
    hdl.push_str("// SystemVerilog HDL\n\n");
    hdl.push_str("module sis_design(\n");
    hdl.push_str("    input logic clk,\n");
    hdl.push_str("    input logic rst_n\n");
    hdl.push_str(");\n\n");
    
    // Generate interfaces and modports
    hdl.push_str("    // Design interfaces\n");
    for conn in connections {
        hdl.push_str(&format!("    logic {};\n", conn.signal_name));
    }
    
    // Generate module instances with interfaces
    for node in nodes {
        hdl.push_str(&format!("    // Node: {} ({})\n", node.name, node.node_type));
        hdl.push_str(&format!("    {} {} (\n", node.node_type, node.id));
        hdl.push_str("        .clk,\n");
        hdl.push_str("        .rst_n\n");
        hdl.push_str("    );\n\n");
    }
    
    hdl.push_str("endmodule : sis_design\n");
    hdl
}

//=============================================================================
// Utility functions for JavaScript interop
//=============================================================================

/// Convert Rust Vec to JavaScript Array
#[wasm_bindgen]
pub fn rust_vec_to_js_array(data: &str) -> Array {
    let array = Array::new();
    if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(data) {
        for item in items {
            array.push(&JsValue::from_str(&item.to_string()));
        }
    }
    array
}

/// Performance monitoring
#[wasm_bindgen]
pub fn get_performance_metrics() -> String {
    let metrics = serde_json::json!({
        "memory_usage": "WASM memory usage not available",
        "compilation_time": 0,
        "last_validation_time": 0,
        "cache_hit_rate": 0.85
    });
    
    metrics.to_string()
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}