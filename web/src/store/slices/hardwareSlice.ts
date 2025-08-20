import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import type { HardwareTarget, DeploymentJob } from '../../types/design'

export interface HardwareState {
  targets: HardwareTarget[]
  selectedTarget: string | null
  
  deployments: DeploymentJob[]
  activeDeployment: string | null
  
  // WASM bridge
  wasmModule: any | null
  isWasmLoaded: boolean
  
  // Real-time hardware monitoring
  monitoring: {
    enabled: boolean
    temperature: number
    powerConsumption: number
    utilization: number
  }
}

const initialState: HardwareState = {
  targets: [],
  selectedTarget: null,
  
  deployments: [],
  activeDeployment: null,
  
  wasmModule: null,
  isWasmLoaded: false,
  
  monitoring: {
    enabled: false,
    temperature: 0,
    powerConsumption: 0,
    utilization: 0,
  },
}

const hardwareSlice = createSlice({
  name: 'hardware',
  initialState,
  reducers: {
    setTargets: (state, action: PayloadAction<HardwareTarget[]>) => {
      state.targets = action.payload
    },
    
    selectTarget: (state, action: PayloadAction<string>) => {
      state.selectedTarget = action.payload
    },
    
    addDeployment: (state, action: PayloadAction<DeploymentJob>) => {
      state.deployments.unshift(action.payload)
      state.activeDeployment = action.payload.id
    },
    
    updateDeployment: (state, action: PayloadAction<{ id: string; updates: Partial<DeploymentJob> }>) => {
      const { id, updates } = action.payload
      const index = state.deployments.findIndex(job => job.id === id)
      if (index !== -1) {
        state.deployments[index] = { ...state.deployments[index], ...updates }
        
        // Clear active deployment if completed
        if (updates.status === 'completed' || updates.status === 'failed') {
          if (state.activeDeployment === id) {
            state.activeDeployment = null
          }
        }
      }
    },
    
    setWasmModule: (state, action: PayloadAction<any>) => {
      state.wasmModule = action.payload
      state.isWasmLoaded = true
    },
    
    setMonitoring: (state, action: PayloadAction<{ enabled: boolean }>) => {
      state.monitoring.enabled = action.payload.enabled
    },
    
    updateMonitoringData: (state, action: PayloadAction<{
      temperature?: number
      powerConsumption?: number
      utilization?: number
    }>) => {
      state.monitoring = { ...state.monitoring, ...action.payload }
    },
  },
})

export const {
  setTargets,
  selectTarget,
  addDeployment,
  updateDeployment,
  setWasmModule,
  setMonitoring,
  updateMonitoringData,
} = hardwareSlice.actions

export default hardwareSlice.reducer