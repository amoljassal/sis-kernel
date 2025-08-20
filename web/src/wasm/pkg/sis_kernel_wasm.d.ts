/* tslint:disable */
/* eslint-disable */
/**
* @returns {string}
*/
export function get_version(): string;
/**
*/
export function set_panic_hook(): void;
/**
*/
export class SisKernelWasm {
  free(): void;
/**
*/
  constructor();
/**
* @returns {string}
*/
  get_version(): string;
/**
* @returns {boolean}
*/
  initialize(): boolean;
/**
* @param {string} nodes_json
* @param {string} connections_json
* @returns {WasmValidationResult}
*/
  validate_design(nodes_json: string, connections_json: string): WasmValidationResult;
/**
* @param {string} design_json
* @returns {number}
*/
  run_preflight_checks(design_json: string): number;
/**
* @param {string} nodes_json
* @param {string} connections_json
* @param {string} target
* @returns {string}
*/
  generate_hdl(nodes_json: string, connections_json: string, target: string): string;
/**
* @param {string} hdl_code
* @param {string} target
* @returns {Promise<any>}
*/
  synthesize_design(hdl_code: string, target: string): Promise<any>;
/**
* @returns {string}
*/
  get_hardware_status(): string;
/**
* @returns {string}
*/
  get_performance_metrics(): string;
}
/**
*/
export class WasmDesignConnection {
  free(): void;
/**
* @param {string} id
* @param {string} source_id
* @param {string} target_id
* @param {string} signal_name
*/
  constructor(id: string, source_id: string, target_id: string, signal_name: string);
/**
*/
  id: string;
/**
*/
  signal_name: string;
/**
*/
  source_id: string;
/**
*/
  target_id: string;
}
/**
*/
export class WasmDesignNode {
  free(): void;
/**
* @param {string} id
* @param {string} name
* @param {string} node_type
* @param {number} x
* @param {number} y
*/
  constructor(id: string, name: string, node_type: string, x: number, y: number);
/**
* @returns {string}
*/
  readonly id: string;
/**
* @returns {string}
*/
  readonly name: string;
/**
* @returns {string}
*/
  readonly node_type: string;
/**
* @returns {number}
*/
  readonly x: number;
/**
* @returns {number}
*/
  readonly y: number;
}
/**
*/
export class WasmValidationResult {
  free(): void;
/**
* @returns {number}
*/
  readonly duration_ms: number;
/**
* @returns {string}
*/
  readonly errors: string;
/**
* @returns {number}
*/
  readonly hazard_score: number;
/**
* @returns {boolean}
*/
  readonly success: boolean;
/**
* @returns {string}
*/
  readonly warnings: string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly get_version: () => number;
  readonly set_panic_hook: () => void;
  readonly __wbg_sisKernelWasm_free: (a: number) => void;
  readonly sisKernelWasm_new: () => number;
  readonly sisKernelWasm_get_version: (a: number, b: number) => void;
  readonly sisKernelWasm_initialize: (a: number) => number;
  readonly sisKernelWasm_validate_design: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly sisKernelWasm_run_preflight_checks: (a: number, b: number, c: number) => number;
  readonly sisKernelWasm_generate_hdl: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
  readonly sisKernelWasm_synthesize_design: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly sisKernelWasm_get_hardware_status: (a: number, b: number) => void;
  readonly sisKernelWasm_get_performance_metrics: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;