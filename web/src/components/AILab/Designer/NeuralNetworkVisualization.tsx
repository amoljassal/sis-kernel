/**
 * Neural Network Visualization
 * 3D visualization of neural network architectures using Three.js
 */

import React, { useEffect, useRef } from 'react';
import { Layers, Cpu, Eye, Settings } from 'lucide-react';

interface NeuralNetworkVisualizationProps {
  className?: string;
  height: number;
}

// Simulated network structure for visualization
interface NetworkLayer {
  id: string;
  type: 'input' | 'hidden' | 'output' | 'attention' | 'embedding';
  name: string;
  nodeCount: number;
  activation?: string;
  position: { x: number; y: number; z: number };
}

interface NetworkConnection {
  from: string;
  to: string;
  weight: number;
  active?: boolean;
}

const SAMPLE_NETWORK: NetworkLayer[] = [
  {
    id: 'input',
    type: 'input',
    name: 'Input Layer',
    nodeCount: 768,
    position: { x: -4, y: 0, z: 0 }
  },
  {
    id: 'embedding',
    type: 'embedding',
    name: 'Embedding',
    nodeCount: 768,
    activation: 'linear',
    position: { x: -2, y: 0, z: 0 }
  },
  {
    id: 'attention1',
    type: 'attention',
    name: 'Self-Attention',
    nodeCount: 768,
    activation: 'softmax',
    position: { x: 0, y: 0, z: 0 }
  },
  {
    id: 'ffn1',
    type: 'hidden',
    name: 'Feed Forward',
    nodeCount: 3072,
    activation: 'gelu',
    position: { x: 2, y: 0, z: 0 }
  },
  {
    id: 'output',
    type: 'output',
    name: 'Output Layer',
    nodeCount: 50000,
    activation: 'softmax',
    position: { x: 4, y: 0, z: 0 }
  }
];

export const NeuralNetworkVisualization: React.FC<NeuralNetworkVisualizationProps> = ({ 
  className = '', 
  height 
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [selectedLayer, setSelectedLayer] = React.useState<string | null>(null);
  const [animationActive, setAnimationActive] = React.useState(false);
  const [visualizationMode, setVisualizationMode] = React.useState<'topology' | 'activations' | 'weights'>('topology');

  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    canvas.width = canvas.offsetWidth;
    canvas.height = height;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = '#111827';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Render neural network
    renderNeuralNetwork(ctx, canvas.width, canvas.height);
  }, [height, selectedLayer, animationActive, visualizationMode]);

  const renderNeuralNetwork = (ctx: CanvasRenderingContext2D, width: number, height: number) => {
    const centerX = width / 2;
    const centerY = height / 2;
    const scale = Math.min(width, height) / 10;

    // Render connections first
    if (visualizationMode === 'topology' || visualizationMode === 'weights') {
      ctx.strokeStyle = visualizationMode === 'weights' ? '#60A5FA' : '#374151';
      ctx.lineWidth = 1;
      
      for (let i = 0; i < SAMPLE_NETWORK.length - 1; i++) {
        const from = SAMPLE_NETWORK[i];
        const to = SAMPLE_NETWORK[i + 1];
        
        const fromX = centerX + from.position.x * scale;
        const fromY = centerY + from.position.y * scale;
        const toX = centerX + to.position.x * scale;
        const toY = centerY + to.position.y * scale;
        
        // Draw connection with animation if active
        if (animationActive) {
          const gradient = ctx.createLinearGradient(fromX, fromY, toX, toY);
          gradient.addColorStop(0, '#3B82F6');
          gradient.addColorStop(0.5, '#60A5FA');
          gradient.addColorStop(1, '#93C5FD');
          ctx.strokeStyle = gradient;
          ctx.lineWidth = 2;
        }
        
        ctx.beginPath();
        ctx.moveTo(fromX, fromY);
        ctx.lineTo(toX, toY);
        ctx.stroke();
      }
    }

    // Render layers
    SAMPLE_NETWORK.forEach((layer, index) => {
      const x = centerX + layer.position.x * scale;
      const y = centerY + layer.position.y * scale;
      
      // Layer representation based on type
      let color = '#6B7280';
      let size = Math.log(layer.nodeCount + 1) * 8;
      
      switch (layer.type) {
        case 'input':
          color = '#10B981';
          break;
        case 'embedding':
          color = '#8B5CF6';
          break;
        case 'attention':
          color = '#F59E0B';
          break;
        case 'hidden':
          color = '#3B82F6';
          break;
        case 'output':
          color = '#EF4444';
          break;
      }
      
      // Highlight selected layer
      if (selectedLayer === layer.id) {
        ctx.strokeStyle = '#FBBF24';
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.arc(x, y, size + 5, 0, 2 * Math.PI);
        ctx.stroke();
      }
      
      // Draw layer node
      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.arc(x, y, size, 0, 2 * Math.PI);
      ctx.fill();
      
      // Add activation animation
      if (animationActive && visualizationMode === 'activations') {
        const pulseSize = size + Math.sin(Date.now() / 200 + index) * 5;
        ctx.fillStyle = color + '40'; // Semi-transparent
        ctx.beginPath();
        ctx.arc(x, y, pulseSize, 0, 2 * Math.PI);
        ctx.fill();
      }
      
      // Layer label
      ctx.fillStyle = '#FFFFFF';
      ctx.font = '12px monospace';
      ctx.textAlign = 'center';
      ctx.fillText(layer.name, x, y + size + 20);
      ctx.fillText(`${layer.nodeCount}`, x, y + size + 35);
    });
  };

  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!canvasRef.current) return;
    
    const canvas = canvasRef.current;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    
    const centerX = canvas.width / 2;
    const centerY = canvas.height / 2;
    const scale = Math.min(canvas.width, canvas.height) / 10;
    
    // Check if click is on a layer
    SAMPLE_NETWORK.forEach(layer => {
      const layerX = centerX + layer.position.x * scale;
      const layerY = centerY + layer.position.y * scale;
      const size = Math.log(layer.nodeCount + 1) * 8;
      
      const distance = Math.sqrt((x - layerX) ** 2 + (y - layerY) ** 2);
      if (distance <= size + 5) {
        setSelectedLayer(selectedLayer === layer.id ? null : layer.id);
      }
    });
  };

  const selectedLayerInfo = SAMPLE_NETWORK.find(layer => layer.id === selectedLayer);

  return (
    <div className={`bg-sis-gray-950 flex flex-col ${className}`}>
      {/* Controls */}
      <div className="p-4 border-b border-sis-gray-700">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Neural Network Topology</h3>
          <div className="flex items-center space-x-2">
            <button
              onClick={() => setAnimationActive(!animationActive)}
              className={`p-2 rounded-md transition-colors ${
                animationActive 
                  ? 'bg-sis-blue-600 text-white' 
                  : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
              }`}
              title="Toggle Animation"
            >
              <Eye className="w-4 h-4" />
            </button>
            <button className="p-2 bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600 rounded-md transition-colors">
              <Settings className="w-4 h-4" />
            </button>
          </div>
        </div>
        
        {/* Visualization Mode */}
        <div className="flex space-x-2">
          {[
            { id: 'topology', name: 'Topology', icon: Layers },
            { id: 'activations', name: 'Activations', icon: Cpu },
            { id: 'weights', name: 'Weights', icon: Settings }
          ].map(mode => {
            const IconComponent = mode.icon;
            return (
              <button
                key={mode.id}
                onClick={() => setVisualizationMode(mode.id as any)}
                className={`flex items-center space-x-2 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                  visualizationMode === mode.id
                    ? 'bg-sis-blue-600 text-white'
                    : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
                }`}
              >
                <IconComponent className="w-4 h-4" />
                <span>{mode.name}</span>
              </button>
            );
          })}
        </div>
      </div>
      
      {/* Visualization Canvas */}
      <div className="flex-1 relative">
        <canvas
          ref={canvasRef}
          onClick={handleCanvasClick}
          className="w-full h-full cursor-pointer"
          style={{ height }}
        />
        
        {/* Layer Info Overlay */}
        {selectedLayerInfo && (
          <div className="absolute top-4 right-4 bg-sis-gray-800 border border-sis-gray-600 rounded-lg p-4 min-w-64">
            <h4 className="text-white font-medium mb-3">{selectedLayerInfo.name}</h4>
            
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-sis-gray-400">Type:</span>
                <span className="text-white capitalize">{selectedLayerInfo.type}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-sis-gray-400">Nodes:</span>
                <span className="text-white font-mono">{selectedLayerInfo.nodeCount.toLocaleString()}</span>
              </div>
              {selectedLayerInfo.activation && (
                <div className="flex justify-between">
                  <span className="text-sis-gray-400">Activation:</span>
                  <span className="text-white font-mono">{selectedLayerInfo.activation}</span>
                </div>
              )}
              <div className="flex justify-between">
                <span className="text-sis-gray-400">Parameters:</span>
                <span className="text-white font-mono">
                  {(selectedLayerInfo.nodeCount * 768).toLocaleString()}
                </span>
              </div>
            </div>
            
            <button
              onClick={() => setSelectedLayer(null)}
              className="w-full mt-3 px-3 py-2 bg-sis-gray-700 text-sis-gray-300 rounded-md hover:bg-sis-gray-600 transition-colors text-sm"
            >
              Close
            </button>
          </div>
        )}
      </div>
      
      {/* Network Stats */}
      <div className="p-4 border-t border-sis-gray-700">
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-sis-gray-400">Total Layers:</span>
            <span className="text-white font-mono ml-2">{SAMPLE_NETWORK.length}</span>
          </div>
          <div>
            <span className="text-sis-gray-400">Total Parameters:</span>
            <span className="text-white font-mono ml-2">
              {SAMPLE_NETWORK.reduce((acc, layer) => acc + layer.nodeCount, 0).toLocaleString()}
            </span>
          </div>
          <div>
            <span className="text-sis-gray-400">Memory Usage:</span>
            <span className="text-white font-mono ml-2">2.1 GB</span>
          </div>
          <div>
            <span className="text-sis-gray-400">Inference Time:</span>
            <span className="text-white font-mono ml-2">~45ms</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default NeuralNetworkVisualization;