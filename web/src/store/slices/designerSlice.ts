import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import type { DesignNode, DesignConnection, SafetyMode } from '../../types/design'

export interface DesignerState {
  // Current design
  designId: string | null
  designName: string
  nodes: DesignNode[]
  connections: DesignConnection[]
  
  // UI state
  selectedNodes: string[]
  isSimulating: boolean
  simulationProgress: number
  
  // Safety state
  safetyMode: SafetyMode
  hazardScore: number
  safetyWarnings: string[]
  
  // Editor state
  codeContent: string
  isCodeEditorOpen: boolean
  
  // History for undo/redo
  history: {
    past: Array<{ nodes: DesignNode[]; connections: DesignConnection[] }>
    present: { nodes: DesignNode[]; connections: DesignConnection[] }
    future: Array<{ nodes: DesignNode[]; connections: DesignConnection[] }>
  }
}

const initialState: DesignerState = {
  designId: null,
  designName: 'Untitled Design',
  nodes: [],
  connections: [],
  
  selectedNodes: [],
  isSimulating: false,
  simulationProgress: 0,
  
  safetyMode: 'beginner',
  hazardScore: 0,
  safetyWarnings: [],
  
  codeContent: '',
  isCodeEditorOpen: false,
  
  history: {
    past: [],
    present: { nodes: [], connections: [] },
    future: [],
  },
}

const designerSlice = createSlice({
  name: 'designer',
  initialState,
  reducers: {
    // Design management
    newDesign: (state) => {
      state.designId = null
      state.designName = 'Untitled Design'
      state.nodes = []
      state.connections = []
      state.selectedNodes = []
      state.codeContent = ''
      state.hazardScore = 0
      state.safetyWarnings = []
    },
    
    loadDesign: (state, action: PayloadAction<{
      id: string
      name: string
      nodes: DesignNode[]
      connections: DesignConnection[]
    }>) => {
      const { id, name, nodes, connections } = action.payload
      state.designId = id
      state.designName = name
      state.nodes = nodes
      state.connections = connections
      state.selectedNodes = []
    },
    
    setDesignName: (state, action: PayloadAction<string>) => {
      state.designName = action.payload
    },
    
    // Node operations
    addNode: (state, action: PayloadAction<DesignNode>) => {
      // Save to history
      state.history.past.push({
        nodes: [...state.nodes],
        connections: [...state.connections],
      })
      state.history.future = []
      
      state.nodes.push(action.payload)
    },
    
    updateNode: (state, action: PayloadAction<{ id: string; updates: Partial<DesignNode> }>) => {
      const { id, updates } = action.payload
      const nodeIndex = state.nodes.findIndex(node => node.id === id)
      if (nodeIndex !== -1) {
        // Save to history
        state.history.past.push({
          nodes: [...state.nodes],
          connections: [...state.connections],
        })
        state.history.future = []
        
        state.nodes[nodeIndex] = { ...state.nodes[nodeIndex], ...updates }
      }
    },
    
    deleteNode: (state, action: PayloadAction<string>) => {
      const nodeId = action.payload
      
      // Save to history
      state.history.past.push({
        nodes: [...state.nodes],
        connections: [...state.connections],
      })
      state.history.future = []
      
      // Remove node and its connections
      state.nodes = state.nodes.filter(node => node.id !== nodeId)
      state.connections = state.connections.filter(
        conn => conn.sourceId !== nodeId && conn.targetId !== nodeId
      )
      state.selectedNodes = state.selectedNodes.filter(id => id !== nodeId)
    },
    
    // Connection operations
    addConnection: (state, action: PayloadAction<DesignConnection>) => {
      // Save to history
      state.history.past.push({
        nodes: [...state.nodes],
        connections: [...state.connections],
      })
      state.history.future = []
      
      state.connections.push(action.payload)
    },
    
    deleteConnection: (state, action: PayloadAction<string>) => {
      const connectionId = action.payload
      
      // Save to history
      state.history.past.push({
        nodes: [...state.nodes],
        connections: [...state.connections],
      })
      state.history.future = []
      
      state.connections = state.connections.filter(conn => conn.id !== connectionId)
    },
    
    // Selection
    selectNode: (state, action: PayloadAction<string>) => {
      const nodeId = action.payload
      if (!state.selectedNodes.includes(nodeId)) {
        state.selectedNodes.push(nodeId)
      }
    },
    
    deselectNode: (state, action: PayloadAction<string>) => {
      const nodeId = action.payload
      state.selectedNodes = state.selectedNodes.filter(id => id !== nodeId)
    },
    
    selectNodes: (state, action: PayloadAction<string[]>) => {
      state.selectedNodes = action.payload
    },
    
    clearSelection: (state) => {
      state.selectedNodes = []
    },
    
    // Simulation
    startSimulation: (state) => {
      state.isSimulating = true
      state.simulationProgress = 0
    },
    
    updateSimulationProgress: (state, action: PayloadAction<number>) => {
      state.simulationProgress = action.payload
    },
    
    stopSimulation: (state) => {
      state.isSimulating = false
      state.simulationProgress = 0
    },
    
    // Safety
    setSafetyMode: (state, action: PayloadAction<SafetyMode>) => {
      state.safetyMode = action.payload
    },
    
    updateHazardScore: (state, action: PayloadAction<number>) => {
      state.hazardScore = action.payload
    },
    
    setSafetyWarnings: (state, action: PayloadAction<string[]>) => {
      state.safetyWarnings = action.payload
    },
    
    // Code editor
    setCodeContent: (state, action: PayloadAction<string>) => {
      state.codeContent = action.payload
    },
    
    toggleCodeEditor: (state) => {
      state.isCodeEditorOpen = !state.isCodeEditorOpen
    },
    
    // History operations
    undo: (state) => {
      if (state.history.past.length > 0) {
        const previous = state.history.past.pop()!
        state.history.future.unshift({
          nodes: [...state.nodes],
          connections: [...state.connections],
        })
        state.nodes = previous.nodes
        state.connections = previous.connections
      }
    },
    
    redo: (state) => {
      if (state.history.future.length > 0) {
        const next = state.history.future.shift()!
        state.history.past.push({
          nodes: [...state.nodes],
          connections: [...state.connections],
        })
        state.nodes = next.nodes
        state.connections = next.connections
      }
    },
  },
})

export const {
  newDesign,
  loadDesign,
  setDesignName,
  addNode,
  updateNode,
  deleteNode,
  addConnection,
  deleteConnection,
  selectNode,
  deselectNode,
  selectNodes,
  clearSelection,
  startSimulation,
  updateSimulationProgress,
  stopSimulation,
  setSafetyMode,
  updateHazardScore,
  setSafetyWarnings,
  setCodeContent,
  toggleCodeEditor,
  undo,
  redo,
} = designerSlice.actions

export default designerSlice.reducer