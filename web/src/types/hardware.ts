// Hardware integration types for FPGA connections and cloud services
export type FPGAVendor = 'xilinx' | 'intel' | 'lattice' | 'microsemi' | 'open-source';
export type CloudProvider = 'aws-f1' | 'azure-np' | 'local' | 'custom';
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error' | 'busy';
export type DeploymentState = 'idle' | 'synthesizing' | 'placing' | 'routing' | 'bitstream' | 'programming' | 'deployed' | 'failed';

export interface FPGADevice {
  id: string;
  name: string;
  vendor: FPGAVendor;
  family: string;
  part_number: string;
  speed_grade: string;
  package: string;
  connection_type: 'usb' | 'jtag' | 'ethernet' | 'cloud';
  status: ConnectionStatus;
  capabilities: {
    logic_cells: number;
    block_ram_kb: number;
    dsp_blocks: number;
    io_pins: number;
    max_frequency_mhz: number;
  };
  utilization?: {
    logic_percent: number;
    memory_percent: number;
    dsp_percent: number;
    io_percent: number;
  };
  temperature_c?: number;
  power_consumption_w?: number;
}

export interface CloudFPGAInstance {
  id: string;
  provider: CloudProvider;
  instance_type: string;
  region: string;
  cost_per_hour: number;
  device: FPGADevice;
  auto_terminate_minutes?: number;
  spot_instance: boolean;
}

export interface DeploymentJob {
  id: string;
  design_id: string;
  target_device: FPGADevice | CloudFPGAInstance;
  state: DeploymentState;
  progress_percent: number;
  started_at: Date;
  completed_at?: Date;
  error_message?: string;
  bitstream_url?: string;
  synthesis_report?: {
    timing_met: boolean;
    max_frequency_achieved_mhz: number;
    resource_utilization: FPGADevice['utilization'];
    critical_warnings: string[];
    errors: string[];
  };
  deployment_config: {
    clock_frequency_mhz: number;
    optimization_target: 'speed' | 'area' | 'power';
    enable_debug_cores: boolean;
    two_person_approval_required: boolean;
  };
}

export interface HardwareMonitor {
  device_id: string;
  timestamp: Date;
  metrics: {
    temperature_c: number;
    power_w: number;
    voltage_v: number;
    current_a: number;
    utilization_percent: number;
    error_count: number;
    uptime_seconds: number;
  };
  alarms: {
    thermal_warning: boolean;
    power_budget_exceeded: boolean;
    timing_violations: boolean;
    configuration_error: boolean;
  };
}

export interface SafetyCheck {
  id: string;
  name: string;
  category: 'thermal' | 'power' | 'timing' | 'erc' | 'drc';
  criticality: 'info' | 'warning' | 'error' | 'critical';
  status: 'pending' | 'running' | 'passed' | 'failed';
  message: string;
  details?: string;
  auto_fix_available: boolean;
}

export interface EmergencyStop {
  trigger_id: string;
  device_ids: string[];
  action: 'pause' | 'stop' | 'power_off' | 'reset';
  reason: string;
  triggered_at: Date;
  acknowledged: boolean;
  acknowledged_by?: string;
}