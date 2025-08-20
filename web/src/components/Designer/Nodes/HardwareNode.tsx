import React from 'react'
import { Handle, Position } from 'reactflow'
import type { NodeProps } from 'reactflow'
import type { DesignNode } from '../../../types/design'

interface HardwareNodeData {
  node: DesignNode
  isSelected: boolean
  hazardLevel: 'safe' | 'warning' | 'danger'
}

const HardwareNode: React.FC<NodeProps<HardwareNodeData>> = ({ data, selected }) => {
  const { node, hazardLevel } = data
  
  // Get node styling based on type and hazard level
  const getNodeStyling = () => {
    const baseClasses = "px-4 py-3 rounded-lg border-2 transition-all duration-200 min-w-32 bg-sis-gray-800"
    
    // Hazard level styling
    const hazardClasses = {
      safe: "border-green-500 shadow-lg shadow-green-500/20",
      warning: "border-yellow-500 shadow-lg shadow-yellow-500/20",
      danger: "border-red-500 shadow-lg shadow-red-500/20"
    }
    
    // Selection styling
    const selectionClasses = selected 
      ? "ring-2 ring-sis-blue-400 scale-105" 
      : "hover:scale-102 hover:shadow-lg"
    
    return `${baseClasses} ${hazardClasses[hazardLevel]} ${selectionClasses}`
  }
  
  // Get icon for node type
  const getNodeIcon = () => {
    switch (node.type) {
      case 'cpu': return '🧠'
      case 'memory': return '💾'
      case 'io': return '🔌'
      case 'logic': return '⚡'
      case 'ai_accelerator': return '🤖'
      case 'interface': return '🔗'
      case 'custom': return '⚙️'
      default: return '📦'
    }
  }
  
  // Get node type display name
  const getNodeTypeName = () => {
    switch (node.type) {
      case 'cpu': return 'CPU'
      case 'memory': return 'Memory'
      case 'io': return 'I/O'
      case 'logic': return 'Logic'
      case 'ai_accelerator': return 'AI Accelerator'
      case 'interface': return 'Interface'
      case 'custom': return 'Custom'
      default: return 'Component'
    }
  }
  
  return (
    <div className={getNodeStyling()}>
      {/* Node Header */}
      <div className="flex items-center space-x-2 mb-2">
        <span className="text-lg">{getNodeIcon()}</span>
        <div className="flex-1">
          <div className="text-sm font-semibold text-white truncate" title={node.name}>
            {node.name}
          </div>
          <div className="text-xs text-sis-gray-400">
            {getNodeTypeName()}
          </div>
        </div>
        
        {/* Status indicators */}
        <div className="flex items-center space-x-1">
          {/* Verification status */}
          <div className={`w-2 h-2 rounded-full ${
            node.verificationStatus === 'verified' ? 'bg-green-400' :
            node.verificationStatus === 'failed' ? 'bg-red-400' :
            'bg-sis-gray-500'
          }`} title={`Verification: ${node.verificationStatus}`} />
          
          {/* Hazard level indicator */}
          <div className={`w-2 h-2 rounded-full ${
            hazardLevel === 'safe' ? 'bg-green-400' :
            hazardLevel === 'warning' ? 'bg-yellow-400' :
            'bg-red-400'
          }`} title={`Safety: ${hazardLevel}`} />
        </div>
      </div>
      
      {/* Key Properties */}
      <div className="space-y-1">
        {node.clockFrequency && (
          <div className="flex justify-between text-xs">
            <span className="text-sis-gray-400">Freq:</span>
            <span className="text-white">{node.clockFrequency} MHz</span>
          </div>
        )}
        {node.powerConsumption && (
          <div className="flex justify-between text-xs">
            <span className="text-sis-gray-400">Power:</span>
            <span className="text-white">{node.powerConsumption} mW</span>
          </div>
        )}
        {node.gateCount && (
          <div className="flex justify-between text-xs">
            <span className="text-sis-gray-400">Gates:</span>
            <span className="text-white">{node.gateCount.toLocaleString()}</span>
          </div>
        )}
      </div>
      
      {/* Input Handles */}
      {node.ports
        .filter(port => port.type === 'input')
        .map((port, index) => (
          <Handle
            key={`input-${port.id}`}
            type="target"
            position={Position.Left}
            id={port.id}
            style={{
              top: `${30 + index * 20}px`,
              background: getPortColor(port.dataType),
              width: '8px',
              height: '8px',
              border: '2px solid #1e293b',
            }}
            title={`${port.name} (${port.dataType}${port.bitWidth ? `, ${port.bitWidth}-bit` : ''})`}
          />
        ))}
      
      {/* Output Handles */}
      {node.ports
        .filter(port => port.type === 'output')
        .map((port, index) => (
          <Handle
            key={`output-${port.id}`}
            type="source"
            position={Position.Right}
            id={port.id}
            style={{
              top: `${30 + index * 20}px`,
              background: getPortColor(port.dataType),
              width: '8px',
              height: '8px',
              border: '2px solid #1e293b',
            }}
            title={`${port.name} (${port.dataType}${port.bitWidth ? `, ${port.bitWidth}-bit` : ''})`}
          />
        ))}
      
      {/* Bidirectional Handles */}
      {node.ports
        .filter(port => port.type === 'bidirectional')
        .map((port, index) => (
          <React.Fragment key={`bidirectional-${port.id}`}>
            <Handle
              type="target"
              position={Position.Bottom}
              id={`${port.id}_in`}
              style={{
                left: `${40 + index * 20}px`,
                background: getPortColor(port.dataType),
                width: '8px',
                height: '8px',
                border: '2px solid #1e293b',
              }}
              title={`${port.name} Input (${port.dataType})`}
            />
            <Handle
              type="source"
              position={Position.Top}
              id={`${port.id}_out`}
              style={{
                left: `${40 + index * 20}px`,
                background: getPortColor(port.dataType),
                width: '8px',
                height: '8px',
                border: '2px solid #1e293b',
              }}
              title={`${port.name} Output (${port.dataType})`}
            />
          </React.Fragment>
        ))}
    </div>
  )
}

// Helper function to get port color based on data type
function getPortColor(dataType: string): string {
  switch (dataType) {
    case 'clock': return '#f59e0b' // yellow
    case 'reset': return '#ef4444' // red
    case 'power': return '#10b981' // green
    case 'data': return '#3b82f6' // blue
    case 'digital': return '#8b5cf6' // purple
    case 'analog': return '#f97316' // orange
    default: return '#64748b' // gray
  }
}

export default HardwareNode