/**
 * AI Component Palette
 * Transforms hardware component palette into AI model building blocks
 */

import React, { useState } from 'react';
import { Brain, Database, Search, Layers, Cpu, Zap } from 'lucide-react';

interface AIComponent {
  id: string;
  name: string;
  category: string;
  description: string;
  icon: React.ComponentType<any>;
  inputs: { name: string; type: string }[];
  outputs: { name: string; type: string }[];
  parameters?: { name: string; type: string; default: any }[];
}

const AI_COMPONENTS: AIComponent[] = [
  // AURAG Components
  {
    id: 'knowledge_base',
    name: 'Knowledge Base',
    category: 'AURAG',
    description: 'Document storage and retrieval system',
    icon: Database,
    inputs: [{ name: 'documents', type: 'Document[]' }],
    outputs: [{ name: 'knowledge', type: 'KnowledgeBase' }],
    parameters: [
      { name: 'chunkSize', type: 'number', default: 1000 },
      { name: 'overlap', type: 'number', default: 200 }
    ]
  },
  {
    id: 'vector_store',
    name: 'Vector Store',
    category: 'AURAG',
    description: 'Semantic embedding storage and search',
    icon: Search,
    inputs: [{ name: 'embeddings', type: 'Embedding[]' }],
    outputs: [{ name: 'vectorDB', type: 'VectorStore' }],
    parameters: [
      { name: 'dimensions', type: 'number', default: 384 },
      { name: 'similarity', type: 'string', default: 'cosine' }
    ]
  },
  {
    id: 'retrieval_engine',
    name: 'Retrieval Engine',
    category: 'AURAG',
    description: 'Context retrieval with philosophical lenses',
    icon: Brain,
    inputs: [
      { name: 'query', type: 'string' },
      { name: 'vectorStore', type: 'VectorStore' }
    ],
    outputs: [{ name: 'context', type: 'Context[]' }],
    parameters: [
      { name: 'topK', type: 'number', default: 5 },
      { name: 'lens', type: 'string', default: 'analytical' }
    ]
  },
  {
    id: 'generation_module',
    name: 'Generation Module',
    category: 'AURAG',
    description: 'Context-aware text generation',
    icon: Cpu,
    inputs: [
      { name: 'prompt', type: 'string' },
      { name: 'context', type: 'Context[]' }
    ],
    outputs: [{ name: 'response', type: 'string' }],
    parameters: [
      { name: 'temperature', type: 'number', default: 0.7 },
      { name: 'maxTokens', type: 'number', default: 512 }
    ]
  },

  // MLX Training Components
  {
    id: 'transformer_block',
    name: 'Transformer Block',
    category: 'MLX Neural',
    description: 'Self-attention transformer layer',
    icon: Layers,
    inputs: [{ name: 'input', type: 'Tensor[B,T,D]' }],
    outputs: [{ name: 'output', type: 'Tensor[B,T,D]' }],
    parameters: [
      { name: 'hiddenSize', type: 'number', default: 768 },
      { name: 'numHeads', type: 'number', default: 12 },
      { name: 'dropout', type: 'number', default: 0.1 }
    ]
  },
  {
    id: 'attention_head',
    name: 'Attention Head',
    category: 'MLX Neural',
    description: 'Multi-head self-attention mechanism',
    icon: Brain,
    inputs: [{ name: 'query', type: 'Tensor' }, { name: 'key', type: 'Tensor' }, { name: 'value', type: 'Tensor' }],
    outputs: [{ name: 'attended', type: 'Tensor' }],
    parameters: [
      { name: 'headSize', type: 'number', default: 64 },
      { name: 'dropout', type: 'number', default: 0.1 }
    ]
  },
  {
    id: 'embedding_layer',
    name: 'Embedding Layer',
    category: 'MLX Neural',
    description: 'Token to vector embedding',
    icon: Database,
    inputs: [{ name: 'tokens', type: 'int[]' }],
    outputs: [{ name: 'embeddings', type: 'Tensor[B,T,D]' }],
    parameters: [
      { name: 'vocabSize', type: 'number', default: 50000 },
      { name: 'embeddingDim', type: 'number', default: 768 }
    ]
  },

  // Training Components
  {
    id: 'data_loader',
    name: 'Data Loader',
    category: 'Training',
    description: 'Batch data loading and preprocessing',
    icon: Database,
    inputs: [{ name: 'dataset', type: 'Dataset' }],
    outputs: [{ name: 'batches', type: 'Batch[]' }],
    parameters: [
      { name: 'batchSize', type: 'number', default: 32 },
      { name: 'shuffle', type: 'boolean', default: true }
    ]
  },
  {
    id: 'optimizer',
    name: 'Optimizer',
    category: 'Training',
    description: 'Gradient-based parameter optimization',
    icon: Zap,
    inputs: [
      { name: 'parameters', type: 'Parameter[]' },
      { name: 'gradients', type: 'Gradient[]' }
    ],
    outputs: [{ name: 'updatedParams', type: 'Parameter[]' }],
    parameters: [
      { name: 'learningRate', type: 'number', default: 0.001 },
      { name: 'optimizer', type: 'string', default: 'adam' }
    ]
  },
  {
    id: 'loss_function',
    name: 'Loss Function',
    category: 'Training',
    description: 'Training objective and loss computation',
    icon: Brain,
    inputs: [
      { name: 'predictions', type: 'Tensor' },
      { name: 'targets', type: 'Tensor' }
    ],
    outputs: [{ name: 'loss', type: 'float' }],
    parameters: [
      { name: 'lossType', type: 'string', default: 'cross_entropy' }
    ]
  }
];

interface AIComponentPaletteProps {
  className?: string;
}

export const AIComponentPalette: React.FC<AIComponentPaletteProps> = ({ className = '' }) => {
  const [selectedCategory, setSelectedCategory] = useState<string>('All');
  const [searchQuery, setSearchQuery] = useState<string>('');

  const categories = ['All', ...Array.from(new Set(AI_COMPONENTS.map(c => c.category)))];

  const filteredComponents = AI_COMPONENTS.filter(component => {
    const matchesCategory = selectedCategory === 'All' || component.category === selectedCategory;
    const matchesSearch = component.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         component.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  const handleDragStart = (e: React.DragEvent, component: AIComponent) => {
    e.dataTransfer.setData('application/json', JSON.stringify(component));
    e.dataTransfer.effectAllowed = 'copy';
  };

  return (
    <div className={`bg-sis-gray-900 border-r border-sis-gray-700 flex flex-col ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-sis-gray-700">
        <h2 className="text-lg font-semibold text-white mb-3">AI Components</h2>
        
        {/* Search */}
        <input
          type="text"
          placeholder="Search components..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full px-3 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-md text-white placeholder-sis-gray-400 text-sm focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
        />
      </div>

      {/* Category Filter */}
      <div className="p-4 border-b border-sis-gray-700">
        <div className="flex flex-wrap gap-2">
          {categories.map(category => (
            <button
              key={category}
              onClick={() => setSelectedCategory(category)}
              className={`px-3 py-1 rounded-full text-xs font-medium transition-colors ${
                selectedCategory === category
                  ? 'bg-sis-blue-600 text-white'
                  : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
              }`}
            >
              {category}
            </button>
          ))}
        </div>
      </div>

      {/* Components List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {filteredComponents.map(component => {
          const IconComponent = component.icon;
          return (
            <div
              key={component.id}
              draggable
              onDragStart={(e) => handleDragStart(e, component)}
              className="bg-sis-gray-800 p-3 rounded-lg border border-sis-gray-600 cursor-move hover:border-sis-blue-500 transition-colors"
            >
              <div className="flex items-start space-x-3">
                <div className="flex-shrink-0">
                  <IconComponent className="w-5 h-5 text-sis-blue-400" />
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="text-sm font-medium text-white break-words">
                    {component.name}
                  </h3>
                  <p className="text-xs text-sis-gray-400 mt-1 break-words">
                    {component.description}
                  </p>
                  
                  {/* Component Info */}
                  <div className="mt-2 flex items-center space-x-4 text-xs text-sis-gray-500">
                    <span>In: {component.inputs.length}</span>
                    <span>Out: {component.outputs.length}</span>
                    {component.parameters && (
                      <span>Params: {component.parameters.length}</span>
                    )}
                  </div>

                  {/* Category Badge */}
                  <div className="mt-2">
                    <span className="inline-block px-2 py-1 bg-sis-gray-700 text-sis-gray-300 text-xs rounded-full">
                      {component.category}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* Footer */}
      <div className="p-4 border-t border-sis-gray-700">
        <div className="text-xs text-sis-gray-500">
          {filteredComponents.length} components available
        </div>
        <div className="text-xs text-sis-gray-400 mt-1">
          Drag components to canvas to build AI architecture
        </div>
      </div>
    </div>
  );
};

export default AIComponentPalette;