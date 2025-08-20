// Design types for the SIS AI-Lab interface

export type SafetyMode = 'beginner' | 'advanced' | 'pro'

export type NodeType = 
  | 'cpu'
  | 'dsp'
  | 'mcu'
  | 'memory'
  | 'cache'
  | 'rom'
  | 'io'
  | 'uart'
  | 'spi'
  | 'i2c'
  | 'logic'
  | 'mux'
  | 'alu'
  | 'ai_accelerator'
  | 'tensor_core'
  | 'interface'
  | 'pcie'
  | 'usb'
  | 'custom'

export interface Position {
  x: number
  y: number
}

export interface Port {
  id: string
  name: string
  type: 'input' | 'output' | 'bidirectional'
  dataType: 'digital' | 'analog' | 'power' | 'clock' | 'data'
  bitWidth?: number
  position: Position
}

export interface DesignNode {
  id: string
  name: string
  type: NodeType
  position: Position
  ports: Port[]
  properties: Record<string, any>
  description?: string
  
  // Visual properties
  color?: string
  size?: { width: number; height: number }
  
  // Hardware properties
  powerConsumption?: number // mW
  clockFrequency?: number   // MHz
  gateCount?: number
  
  // Safety properties
  hazardLevel: 'safe' | 'warning' | 'danger'
  verificationStatus: 'unverified' | 'verified' | 'failed'
}

export interface DesignConnection {
  id: string
  sourceId: string
  sourcePort: string
  targetId: string
  targetPort: string
  
  // Signal properties
  signalName?: string
  bitWidth: number
  signalType: 'combinational' | 'sequential' | 'clock' | 'reset'
  
  // Timing properties
  delay?: number // ns
  skew?: number  // ps
  
  // Visual properties
  color?: string
  style?: 'solid' | 'dashed' | 'dotted'
  
  // Safety properties
  isCritical: boolean
  verificationStatus: 'unverified' | 'verified' | 'failed'
}

export interface DesignConstraints {
  timing: {
    clockPeriod: number // ns
    setupTime: number   // ps
    holdTime: number    // ps
  }
  
  power: {
    maxPower: number    // mW
    voltage: number     // V
    current: number     // mA
  }
  
  area: {
    maxArea: number     // mm²
    aspectRatio: number
  }
  
  thermal: {
    maxTemp: number     // °C
    ambientTemp: number // °C
  }
}

export interface SimulationResult {
  success: boolean
  duration: number // ms
  warnings: string[]
  errors: string[]
  
  // Performance metrics
  timing: {
    criticalPath: number // ns
    clockFreq: number    // MHz
    slack: number        // ps
  }
  
  power: {
    totalPower: number   // mW
    staticPower: number  // mW
    dynamicPower: number // mW
  }
  
  area: {
    totalArea: number    // mm²
    utilization: number  // %
    gateCount: number
  }
  
  // Waveform data for visualization
  waveforms?: {
    signals: string[]
    timebase: number[]
    data: number[][]
  }
}

export interface ValidationReport {
  timestamp: number
  designId: string
  
  // Safety assessment
  hazardScore: number // 0-100
  safetyMode: SafetyMode
  blockers: SafetyBlocker[]
  warnings: SafetyWarning[]
  
  // Technical checks
  syntaxCheck: CheckResult
  timingCheck: CheckResult
  powerCheck: CheckResult
  thermalCheck: CheckResult
  
  // Overall status
  canDeploy: boolean
  requiredApprovals: string[]
}

export interface SafetyBlocker {
  severity: 'critical' | 'major' | 'minor'
  category: 'timing' | 'power' | 'thermal' | 'logic' | 'safety'
  message: string
  node?: string
  connection?: string
  suggestedFix?: string
}

export interface SafetyWarning {
  category: 'performance' | 'optimization' | 'best_practice'
  message: string
  node?: string
  connection?: string
  recommendation?: string
}

export interface CheckResult {
  passed: boolean
  score: number // 0-100
  details: string
  metrics?: Record<string, number>
}

// Collaboration types
export interface CollaborationCursor {
  userId: string
  userName: string
  position: Position
  color: string
  lastUpdate: number
}

export interface DesignComment {
  id: string
  userId: string
  userName: string
  position: Position
  content: string
  timestamp: number
  resolved: boolean
  replies: DesignComment[]
}

export interface DesignVersion {
  id: string
  version: string
  timestamp: number
  userId: string
  userName: string
  message: string
  nodes: DesignNode[]
  connections: DesignConnection[]
}

// Hardware deployment types
export interface HardwareTarget {
  id: string
  name: string
  type: 'fpga' | 'asic' | 'simulation'
  vendor: 'xilinx' | 'intel' | 'microsemi' | 'custom'
  model: string
  resources: {
    luts: number
    flipFlops: number
    bram: number
    dsp: number
  }
  available: boolean
  cost?: number // USD per hour for cloud targets
}

export interface DeploymentJob {
  id: string
  designId: string
  targetId: string
  status: 'queued' | 'running' | 'completed' | 'failed'
  progress: number // 0-100
  startTime?: number
  endTime?: number
  result?: {
    success: boolean
    bitstreamUrl?: string
    reportUrl?: string
    errors?: string[]
  }
}

// Marketplace types
export interface IPBlock {
  id: string
  name: string
  description: string
  category: string
  tags: string[]
  author: string
  version: string
  price: number
  rating: number
  downloads: number
  verified: boolean
  preview?: {
    image: string
    nodes: DesignNode[]
    connections: DesignConnection[]
  }
}