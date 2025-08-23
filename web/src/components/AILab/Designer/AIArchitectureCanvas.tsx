/**
 * AI Architecture Canvas
 * Transforms hardware design canvas into neural network architecture designer
 */

import React, { useState, useRef, useCallback } from 'react';
import { Play, Square, Save, Download, Upload, Zap } from 'lucide-react';

interface AINode {
  id: string;
  type: string;
  name: string;
  position: { x: number; y: number };
  inputs: { name: string; type: string; connected?: string }[];
  outputs: { name: string; type: string; connected?: string[] }[];
  parameters: Record<string, any>;
  category: string;
}

interface AIConnection {
  id: string;
  from: { nodeId: string; output: string };
  to: { nodeId: string; input: string };
  tensorShape?: string;
  dataType?: string;
}

interface TrainingMetrics {
  loss: number;
  accuracy: number;
  epoch: number;
  isTraining: boolean;
}

interface AIArchitectureCanvasProps {
  className?: string;
}

export const AIArchitectureCanvas: React.FC<AIArchitectureCanvasProps> = ({ className = '' }) => {
  const canvasRef = useRef<HTMLDivElement>(null);
  const [nodes, setNodes] = useState<AINode[]>([]);
  const [connections, setConnections] = useState<AIConnection[]>([]);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [trainingMetrics, setTrainingMetrics] = useState<TrainingMetrics>({
    loss: 0,
    accuracy: 0,
    epoch: 0,
    isTraining: false
  });

  // Handle component drop from palette
  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    
    if (!canvasRef.current) return;
    
    const rect = canvasRef.current.getBoundingClientRect();
    const componentData = JSON.parse(e.dataTransfer.getData('application/json'));
    
    const newNode: AINode = {
      id: `node_${Date.now()}`,
      type: componentData.id,
      name: componentData.name,
      position: {
        x: (e.clientX - rect.left - pan.x) / zoom,
        y: (e.clientY - rect.top - pan.y) / zoom
      },
      inputs: componentData.inputs || [],
      outputs: componentData.outputs || [],
      parameters: componentData.parameters?.reduce((acc: any, param: any) => {
        acc[param.name] = param.default;
        return acc;
      }, {}) || {},
      category: componentData.category
    };

    setNodes(prev => [...prev, newNode]);
  }, [pan, zoom]);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  };

  // Node drag handling
  const handleNodeMouseDown = (e: React.MouseEvent, nodeId: string) => {
    e.preventDefault();
    e.stopPropagation();
    
    setSelectedNode(nodeId);
    setIsDragging(true);
    
    const rect = e.currentTarget.getBoundingClientRect();
    setDragOffset({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top
    });
  };

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDragging || !selectedNode || !canvasRef.current) return;
    
    const rect = canvasRef.current.getBoundingClientRect();
    const newX = (e.clientX - rect.left - dragOffset.x - pan.x) / zoom;
    const newY = (e.clientY - rect.top - dragOffset.y - pan.y) / zoom;
    
    setNodes(prev => prev.map(node => 
      node.id === selectedNode 
        ? { ...node, position: { x: newX, y: newY } }
        : node
    ));
  }, [isDragging, selectedNode, dragOffset, pan, zoom]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  React.useEffect(() => {
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [handleMouseMove, handleMouseUp]);

  // Training simulation
  const startTraining = () => {
    setTrainingMetrics(prev => ({ ...prev, isTraining: true, epoch: 0 }));
    
    const interval = setInterval(() => {
      setTrainingMetrics(prev => {
        const newEpoch = prev.epoch + 1;
        const newLoss = Math.max(0.1, 2.0 - (newEpoch / 50) * 1.8 + Math.random() * 0.2);
        const newAccuracy = Math.min(0.95, (newEpoch / 50) * 0.85 + Math.random() * 0.1);
        
        if (newEpoch >= 50) {
          clearInterval(interval);
          return { ...prev, isTraining: false };
        }
        
        return {
          epoch: newEpoch,
          loss: newLoss,
          accuracy: newAccuracy,
          isTraining: true
        };
      });
    }, 1000);
  };

  const stopTraining = () => {
    setTrainingMetrics(prev => ({ ...prev, isTraining: false }));
  };

  // Validate architecture
  const validateArchitecture = () => {
    const issues: string[] = [];
    
    // Check for unconnected nodes
    const unconnectedNodes = nodes.filter(node => 
      node.inputs.some(input => !input.connected) || 
      node.outputs.some(output => !output.connected || output.connected.length === 0)
    );
    
    if (unconnectedNodes.length > 0) {
      issues.push(`${unconnectedNodes.length} nodes have unconnected ports`);
    }
    
    // Check for cycles (simplified)
    if (connections.length > nodes.length) {
      issues.push('Potential cycles detected in architecture');
    }
    
    return issues;
  };

  const validationIssues = validateArchitecture();

  const exportArchitecture = () => {
    const architecture = {
      nodes,
      connections,
      metadata: {
        created: new Date().toISOString(),
        version: '1.0',
        framework: 'SIS-MLX'
      }
    };
    
    const blob = new Blob([JSON.stringify(architecture, null, 2)], {
      type: 'application/json'
    });
    
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'ai-architecture.json';
    a.click();
    
    URL.revokeObjectURL(url);
  };

  return (
    <div className={`relative bg-sis-gray-950 ${className}`}>
      {/* Toolbar */}
      <div className="absolute top-4 left-4 z-10 bg-sis-gray-800 rounded-lg p-2 flex items-center space-x-2">
        <button
          onClick={startTraining}
          disabled={trainingMetrics.isTraining || nodes.length === 0}
          className="p-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:bg-sis-gray-600 disabled:cursor-not-allowed transition-colors"
          title="Start Training"
        >
          <Play className="w-4 h-4" />
        </button>
        
        <button
          onClick={stopTraining}
          disabled={!trainingMetrics.isTraining}
          className="p-2 bg-red-600 text-white rounded-md hover:bg-red-700 disabled:bg-sis-gray-600 disabled:cursor-not-allowed transition-colors"
          title="Stop Training"
        >
          <Square className="w-4 h-4" />
        </button>
        
        <div className="w-px h-8 bg-sis-gray-600" />
        
        <button
          onClick={exportArchitecture}
          disabled={nodes.length === 0}
          className="p-2 bg-sis-blue-600 text-white rounded-md hover:bg-sis-blue-700 disabled:bg-sis-gray-600 disabled:cursor-not-allowed transition-colors"
          title="Export Architecture"
        >
          <Download className="w-4 h-4" />
        </button>
        
        <button
          className="p-2 bg-sis-gray-600 text-white rounded-md hover:bg-sis-gray-700 transition-colors"
          title="Save Architecture"
        >
          <Save className="w-4 h-4" />
        </button>
      </div>

      {/* Training Metrics Overlay */}
      {trainingMetrics.isTraining && (
        <div className="absolute top-4 right-4 z-10 bg-sis-gray-800 rounded-lg p-4 min-w-64">
          <div className="flex items-center space-x-2 mb-3">
            <Zap className="w-4 h-4 text-yellow-400 animate-pulse" />
            <span className="text-white font-medium">Training Active</span>
          </div>
          
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-sis-gray-400">Epoch:</span>
              <span className="text-white font-mono">{trainingMetrics.epoch}/50</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sis-gray-400">Loss:</span>
              <span className="text-white font-mono">{trainingMetrics.loss.toFixed(4)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sis-gray-400">Accuracy:</span>
              <span className="text-white font-mono">{(trainingMetrics.accuracy * 100).toFixed(1)}%</span>
            </div>
          </div>
          
          <div className="mt-3">
            <div className="w-full bg-sis-gray-700 rounded-full h-2">
              <div 
                className="bg-sis-blue-500 h-2 rounded-full transition-all duration-1000"
                style={{ width: `${(trainingMetrics.epoch / 50) * 100}%` }}
              />
            </div>
          </div>
        </div>
      )}

      {/* Validation Issues */}
      {validationIssues.length > 0 && (
        <div className="absolute bottom-4 left-4 z-10 bg-yellow-900/80 border border-yellow-600 rounded-lg p-3 max-w-md">
          <div className="text-yellow-400 font-medium text-sm mb-2">Architecture Issues:</div>
          <ul className="text-yellow-300 text-sm space-y-1">
            {validationIssues.map((issue, index) => (
              <li key={index}>• {issue}</li>
            ))}
          </ul>
        </div>
      )}

      {/* Canvas */}
      <div
        ref={canvasRef}
        className="w-full h-full overflow-hidden relative"
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        style={{
          backgroundImage: `
            radial-gradient(circle at 1px 1px, rgba(255,255,255,0.15) 1px, transparent 0)
          `,
          backgroundSize: `${20 * zoom}px ${20 * zoom}px`,
          backgroundPosition: `${pan.x}px ${pan.y}px`
        }}
      >
        <div
          className="relative"
          style={{
            transform: `scale(${zoom}) translate(${pan.x}px, ${pan.y}px)`,
            transformOrigin: '0 0'
          }}
        >
          {/* Render Connections */}
          <svg className="absolute inset-0 pointer-events-none" style={{ zIndex: 1 }}>
            {connections.map(connection => {
              const fromNode = nodes.find(n => n.id === connection.from.nodeId);
              const toNode = nodes.find(n => n.id === connection.to.nodeId);
              
              if (!fromNode || !toNode) return null;
              
              const fromX = fromNode.position.x + 200; // Node width
              const fromY = fromNode.position.y + 50;  // Approximate output position
              const toX = toNode.position.x;
              const toY = toNode.position.y + 50;      // Approximate input position
              
              return (
                <line
                  key={connection.id}
                  x1={fromX}
                  y1={fromY}
                  x2={toX}
                  y2={toY}
                  stroke="#3B82F6"
                  strokeWidth={2}
                  strokeDasharray={trainingMetrics.isTraining ? "5,5" : "none"}
                  className={trainingMetrics.isTraining ? "animate-pulse" : ""}
                />
              );
            })}
          </svg>

          {/* Render Nodes */}
          {nodes.map(node => (
            <div
              key={node.id}
              className={`absolute bg-sis-gray-800 border-2 rounded-lg p-3 cursor-move min-w-48 ${
                selectedNode === node.id
                  ? 'border-sis-blue-500 shadow-lg shadow-sis-blue-500/25'
                  : 'border-sis-gray-600 hover:border-sis-gray-500'
              }`}
              style={{
                left: node.position.x,
                top: node.position.y,
                zIndex: selectedNode === node.id ? 10 : 2
              }}
              onMouseDown={(e) => handleNodeMouseDown(e, node.id)}
            >
              <div className="text-white font-medium text-sm mb-2">{node.name}</div>
              
              <div className="text-xs text-sis-gray-400 mb-2">
                Category: {node.category}
              </div>
              
              {/* Input Ports */}
              <div className="space-y-1">
                {node.inputs.map((input, index) => (
                  <div key={index} className="flex items-center space-x-2">
                    <div className="w-2 h-2 bg-sis-blue-400 rounded-full flex-shrink-0" />
                    <span className="text-xs text-sis-gray-300">{input.name}</span>
                  </div>
                ))}
              </div>
              
              {/* Output Ports */}
              <div className="mt-2 space-y-1">
                {node.outputs.map((output, index) => (
                  <div key={index} className="flex items-center justify-end space-x-2">
                    <span className="text-xs text-sis-gray-300">{output.name}</span>
                    <div className="w-2 h-2 bg-sis-green-400 rounded-full flex-shrink-0" />
                  </div>
                ))}
              </div>
              
              {/* Parameters Preview */}
              {Object.keys(node.parameters).length > 0 && (
                <div className="mt-2 pt-2 border-t border-sis-gray-700">
                  <div className="text-xs text-sis-gray-400">
                    {Object.keys(node.parameters).length} parameters configured
                  </div>
                </div>
              )}
            </div>
          ))}

          {/* Welcome Message */}
          {nodes.length === 0 && (
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="text-center text-sis-gray-500 max-w-md">
                <div className="text-6xl mb-4">AI</div>
                <h3 className="text-lg font-medium mb-2">AI Architecture Canvas</h3>
                <p className="text-sm">
                  Drag components from the palette to build your neural network architecture.
                  Connect components to create training pipelines and AI agents.
                </p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Zoom Controls */}
      <div className="absolute bottom-4 right-4 z-10 bg-sis-gray-800 rounded-lg p-2 flex flex-col space-y-2">
        <button
          onClick={() => setZoom(prev => Math.min(2, prev * 1.2))}
          className="p-2 text-white hover:bg-sis-gray-700 rounded transition-colors"
        >
          +
        </button>
        <div className="text-white text-xs text-center px-2">
          {Math.round(zoom * 100)}%
        </div>
        <button
          onClick={() => setZoom(prev => Math.max(0.5, prev / 1.2))}
          className="p-2 text-white hover:bg-sis-gray-700 rounded transition-colors"
        >
          -
        </button>
      </div>
    </div>
  );
};

export default AIArchitectureCanvas;