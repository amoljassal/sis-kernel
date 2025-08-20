import React, { useCallback, useRef, useEffect } from 'react'
import ReactFlow, {
  MiniMap,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Node,
  NodeTypes,
  EdgeTypes,
  ReactFlowProvider,
  Panel,
} from 'reactflow'
import 'reactflow/dist/style.css'
import { useSelector, useDispatch } from 'react-redux'
import type { RootState } from '../../../store/store'
import type { Port } from '../../../types/design'
import { addNode, addConnection, selectNode, clearSelection } from '../../../store/slices/designerSlice'
import { useDesignValidation } from '../../../hooks/useWasm'
import HardwareNode from '../Nodes/HardwareNode'
import WasmStatus from '../../WasmStatus/WasmStatus'
import CollaborationCursors from '../Collaboration/CollaborationCursors'

// Custom node types for hardware components
const nodeTypes: NodeTypes = {
  hardware: HardwareNode,
}

// Custom edge types (for future signal visualization)
const edgeTypes: EdgeTypes = {}

interface DesignCanvasProps {
  className?: string
}

const DesignCanvas: React.FC<DesignCanvasProps> = ({ className = '' }) => {
  const dispatch = useDispatch()
  const reactFlowWrapper = useRef<HTMLDivElement>(null)
  const [reactFlowInstance, setReactFlowInstance] = React.useState<any>(null)
  
  // Redux state
  const { nodes, connections, selectedNodes, isSimulating, hazardScore } = useSelector(
    (state: RootState) => state.designer
  )
  const { safetyMode } = useSelector((state: RootState) => state.settings)
  
  // WASM validation hook
  const { validateDesign, isValidating } = useDesignValidation()
  
  // Convert Redux state to ReactFlow format
  const [flowNodes, setFlowNodes, onNodesChange] = useNodesState(
    nodes.map(node => ({
      id: node.id,
      type: 'hardware',
      position: { x: node.position.x, y: node.position.y },
      data: { 
        node,
        isSelected: selectedNodes.includes(node.id),
        hazardLevel: node.hazardLevel,
      },
      selected: selectedNodes.includes(node.id),
    }))
  )
  
  const [flowEdges, setFlowEdges, onEdgesChange] = useEdgesState(
    connections.map(conn => ({
      id: conn.id,
      source: conn.sourceId,
      target: conn.targetId,
      sourceHandle: conn.sourcePort,
      targetHandle: conn.targetPort,
      type: 'smoothstep',
      animated: conn.signalType === 'clock',
      style: {
        stroke: conn.isCritical ? '#ef4444' : '#64748b',
        strokeWidth: conn.isCritical ? 3 : 2,
      },
      data: { connection: conn },
    }))
  )
  
  // Sync Redux state with ReactFlow
  useEffect(() => {
    setFlowNodes(nodes.map(node => ({
      id: node.id,
      type: 'hardware',
      position: { x: node.position.x, y: node.position.y },
      data: { 
        node,
        isSelected: selectedNodes.includes(node.id),
        hazardLevel: node.hazardLevel,
      },
      selected: selectedNodes.includes(node.id),
    })))
  }, [nodes, selectedNodes, setFlowNodes])
  
  useEffect(() => {
    setFlowEdges(connections.map(conn => ({
      id: conn.id,
      source: conn.sourceId,
      target: conn.targetId,
      sourceHandle: conn.sourcePort,
      targetHandle: conn.targetPort,
      type: 'smoothstep',
      animated: conn.signalType === 'clock',
      style: {
        stroke: conn.isCritical ? '#ef4444' : '#64748b',
        strokeWidth: conn.isCritical ? 3 : 2,
      },
      data: { connection: conn },
    })))
  }, [connections, setFlowEdges])
  
  // Handle new connections
  const onConnect = useCallback(
    (params: Connection) => {
      if (!params.source || !params.target) return
      
      const connectionId = `${params.source}-${params.target}-${Date.now()}`
      const newConnection = {
        id: connectionId,
        sourceId: params.source,
        sourcePort: params.sourceHandle || 'output',
        targetId: params.target,
        targetPort: params.targetHandle || 'input',
        signalName: `sig_${connectionId.slice(-4)}`,
        bitWidth: 1,
        signalType: 'combinational' as const,
        isCritical: false,
        verificationStatus: 'unverified' as const,
      }
      
      dispatch(addConnection(newConnection))
      setFlowEdges((eds) => addEdge(params, eds))
    },
    [dispatch, setFlowEdges]
  )
  
  // Handle node selection
  const onNodeClick = useCallback(
    (event: React.MouseEvent, node: Node) => {
      event.stopPropagation()
      dispatch(selectNode(node.id))
    },
    [dispatch]
  )
  
  // Handle canvas click (clear selection)
  const onPaneClick = useCallback(() => {
    dispatch(clearSelection())
  }, [dispatch])
  
  // Handle drag over for component drop
  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])
  
  // Handle component drop from palette
  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault()
      
      const reactFlowBounds = reactFlowWrapper.current?.getBoundingClientRect()
      const type = event.dataTransfer.getData('application/reactflow')
      
      if (typeof type === 'undefined' || !type || !reactFlowInstance) {
        return
      }
      
      const position = reactFlowInstance.project({
        x: event.clientX - (reactFlowBounds?.left || 0),
        y: event.clientY - (reactFlowBounds?.top || 0),
      })
      
      // Create new hardware node
      const nodeId = `${type}_${Date.now()}`
      const newNode = {
        id: nodeId,
        name: `${type.toUpperCase()}_${nodeId.slice(-4)}`,
        type: type as any,
        position,
        ports: getDefaultPorts(type),
        properties: getDefaultProperties(type),
        hazardLevel: 'safe' as const,
        verificationStatus: 'unverified' as const,
      }
      
      dispatch(addNode(newNode))
      
      // Trigger validation after adding node
      if (nodes.length > 0) {
        validateDesign([...nodes, newNode], connections)
      }
    },
    [reactFlowInstance, dispatch, nodes, connections, validateDesign]
  )
  
  // Auto-validate design when nodes/connections change
  useEffect(() => {
    if (nodes.length > 0 && !isValidating) {
      const timeoutId = setTimeout(() => {
        validateDesign(nodes, connections)
      }, 500) // Debounce validation
      
      return () => clearTimeout(timeoutId)
    }
  }, [nodes, connections, validateDesign, isValidating])
  
  return (
    <div className={`h-full ${className}`} ref={reactFlowWrapper}>
      <ReactFlowProvider>
        <CollaborationCursors canvasRef={reactFlowWrapper} />
        <ReactFlow
          nodes={flowNodes}
          edges={flowEdges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={onNodeClick}
          onPaneClick={onPaneClick}
          onDragOver={onDragOver}
          onDrop={onDrop}
          onInit={setReactFlowInstance}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          fitView
          snapToGrid
          snapGrid={[20, 20]}
          defaultEdgeOptions={{
            type: 'smoothstep',
            animated: false,
          }}
          className="bg-sis-gray-900"
          minZoom={0.2}
          maxZoom={2}
        >
          <Controls 
            position="top-left"
            className="bg-sis-gray-800 border border-sis-gray-700"
          />
          
          <MiniMap 
            position="bottom-right"
            className="bg-sis-gray-800 border border-sis-gray-700"
            nodeColor={(node) => {
              const hazardLevel = node.data?.hazardLevel || 'safe'
              switch (hazardLevel) {
                case 'danger': return '#ef4444'
                case 'warning': return '#f59e0b'
                case 'safe': return '#10b981'
                default: return '#64748b'
              }
            }}
          />
          
          <Background 
            color="#334155" 
            gap={20} 
            size={1}
          />
          
          {/* Status Panel */}
          <Panel position="top-right" className="bg-sis-gray-800 border border-sis-gray-700 rounded p-4">
            <div className="space-y-2">
              <div className="text-sm font-medium text-white">Design Status</div>
              
              <div className="flex items-center space-x-2">
                <span className="text-xs text-sis-gray-400">Safety:</span>
                <span className={`text-xs font-medium ${
                  safetyMode === 'beginner' ? 'text-green-400' :
                  safetyMode === 'advanced' ? 'text-yellow-400' :
                  safetyMode === 'pro' ? 'text-red-400' :
                  'text-sis-gray-400'
                }`}>
                  {safetyMode.toUpperCase()}
                </span>
              </div>
              
              <div className="flex items-center space-x-2">
                <span className="text-xs text-sis-gray-400">Risk:</span>
                <span className={`text-xs font-medium ${
                  hazardScore <= 25 ? 'text-green-400' :
                  hazardScore <= 50 ? 'text-yellow-400' :
                  hazardScore <= 75 ? 'text-orange-400' :
                  'text-red-400'
                }`}>
                  {hazardScore}/100
                </span>
              </div>
              
              <div className="flex items-center space-x-2">
                <span className="text-xs text-sis-gray-400">Nodes:</span>
                <span className="text-xs text-white">{nodes.length}</span>
              </div>
              
              <div className="flex items-center space-x-2">
                <span className="text-xs text-sis-gray-400">Connections:</span>
                <span className="text-xs text-white">{connections.length}</span>
              </div>
              
              <WasmStatus className="mt-2" />
              
              {isValidating && (
                <div className="flex items-center space-x-2 text-sis-blue-400">
                  <div className="w-2 h-2 bg-sis-blue-400 rounded-full animate-pulse" />
                  <span className="text-xs">Validating...</span>
                </div>
              )}
              
              {isSimulating && (
                <div className="flex items-center space-x-2 text-green-400">
                  <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
                  <span className="text-xs">Simulating...</span>
                </div>
              )}
            </div>
          </Panel>
        </ReactFlow>
      </ReactFlowProvider>
    </div>
  )
}

// Helper functions for component defaults
function getDefaultPorts(nodeType: string): Port[] {
  switch (nodeType) {
    case 'cpu':
      return [
        { id: 'clk', name: 'Clock', type: 'input' as const, dataType: 'clock' as const, position: { x: 0, y: 20 } },
        { id: 'rst', name: 'Reset', type: 'input' as const, dataType: 'digital' as const, position: { x: 0, y: 40 } },
        { id: 'data_out', name: 'Data Out', type: 'output' as const, dataType: 'data' as const, bitWidth: 32, position: { x: 100, y: 20 } },
      ]
    case 'memory':
      return [
        { id: 'clk', name: 'Clock', type: 'input' as const, dataType: 'clock' as const, position: { x: 0, y: 20 } },
        { id: 'addr', name: 'Address', type: 'input' as const, dataType: 'data' as const, bitWidth: 16, position: { x: 0, y: 40 } },
        { id: 'data_in', name: 'Data In', type: 'input' as const, dataType: 'data' as const, bitWidth: 8, position: { x: 0, y: 60 } },
        { id: 'data_out', name: 'Data Out', type: 'output' as const, dataType: 'data' as const, bitWidth: 8, position: { x: 100, y: 40 } },
      ]
    case 'io':
      return [
        { id: 'gpio', name: 'GPIO', type: 'bidirectional' as const, dataType: 'digital' as const, bitWidth: 8, position: { x: 50, y: 30 } },
      ]
    default:
      return [
        { id: 'in', name: 'Input', type: 'input' as const, dataType: 'digital' as const, position: { x: 0, y: 20 } },
        { id: 'out', name: 'Output', type: 'output' as const, dataType: 'digital' as const, position: { x: 100, y: 20 } },
      ]
  }
}

function getDefaultProperties(nodeType: string) {
  switch (nodeType) {
    case 'cpu':
      return {
        architecture: 'RISC-V',
        bitWidth: 32,
        clockFrequency: 100, // MHz
        powerConsumption: 250, // mW
      }
    case 'memory':
      return {
        type: 'SRAM',
        size: 1024, // bytes
        accessTime: 10, // ns
        powerConsumption: 50, // mW
      }
    case 'io':
      return {
        type: 'GPIO',
        pins: 8,
        voltage: 3.3, // V
        current: 10, // mA
      }
    default:
      return {
        type: 'Generic',
        delay: 1, // ns
      }
  }
}

export default DesignCanvas