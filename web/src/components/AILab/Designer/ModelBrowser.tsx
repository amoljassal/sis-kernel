/**
 * Model Browser
 * Transforms design browser into AI model management interface
 */

import React, { useState } from 'react';
import { Search, Filter, Download, Share2, Star, Clock, Brain, Database } from 'lucide-react';

interface AIModel {
  id: string;
  name: string;
  description: string;
  type: 'transformer' | 'aurag' | 'custom' | 'fine-tuned';
  framework: 'mlx' | 'pytorch' | 'tensorflow';
  size: string;
  parameters: string;
  accuracy?: number;
  domain: string;
  created: string;
  lastTrained?: string;
  status: 'trained' | 'training' | 'failed' | 'draft';
  tags: string[];
}

const SAMPLE_MODELS: AIModel[] = [
  {
    id: '1',
    name: 'Legal Document Analyzer',
    description: 'AURAG-powered model for legal document analysis and summarization',
    type: 'aurag',
    framework: 'mlx',
    size: '2.3 GB',
    parameters: '7B',
    accuracy: 0.94,
    domain: 'Legal',
    created: '2024-01-15',
    lastTrained: '2024-01-20',
    status: 'trained',
    tags: ['legal', 'analysis', 'aurag', 'production']
  },
  {
    id: '2',
    name: 'Medical Knowledge Assistant',
    description: 'Specialized model for medical question answering with philosophical lens integration',
    type: 'transformer',
    framework: 'mlx',
    size: '1.8 GB',
    parameters: '3B',
    accuracy: 0.89,
    domain: 'Medical',
    created: '2024-01-10',
    lastTrained: '2024-01-18',
    status: 'trained',
    tags: ['medical', 'qa', 'healthcare', 'production']
  },
  {
    id: '3',
    name: 'Code Generation Model',
    description: 'Fine-tuned model for Python code generation and debugging assistance',
    type: 'fine-tuned',
    framework: 'mlx',
    size: '5.2 GB',
    parameters: '13B',
    accuracy: 0.87,
    domain: 'Software',
    created: '2024-01-08',
    status: 'training',
    tags: ['code', 'python', 'debugging', 'development']
  },
  {
    id: '4',
    name: 'Financial Analysis AURAG',
    description: 'AURAG system for financial document processing and market analysis',
    type: 'aurag',
    framework: 'mlx',
    size: '3.1 GB',
    parameters: '7B',
    domain: 'Finance',
    created: '2024-01-12',
    status: 'draft',
    tags: ['finance', 'analysis', 'aurag', 'experimental']
  },
  {
    id: '5',
    name: 'Academic Research Assistant',
    description: 'Multi-modal model for academic research and paper analysis',
    type: 'custom',
    framework: 'mlx',
    size: '4.7 GB',
    parameters: '11B',
    accuracy: 0.92,
    domain: 'Academic',
    created: '2024-01-14',
    lastTrained: '2024-01-19',
    status: 'trained',
    tags: ['research', 'academic', 'multimodal', 'production']
  }
];

interface ModelBrowserProps {
  className?: string;
}

export const ModelBrowser: React.FC<ModelBrowserProps> = ({ className = '' }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedFilter, setSelectedFilter] = useState<'all' | 'aurag' | 'transformer' | 'fine-tuned' | 'custom'>('all');
  const [selectedDomain, setSelectedDomain] = useState<'all' | string>('all');
  const [sortBy, setSortBy] = useState<'created' | 'accuracy' | 'size'>('created');
  const [selectedModel, setSelectedModel] = useState<string | null>(null);

  const filteredModels = SAMPLE_MODELS.filter(model => {
    const matchesSearch = model.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         model.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         model.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()));
    
    const matchesFilter = selectedFilter === 'all' || model.type === selectedFilter;
    const matchesDomain = selectedDomain === 'all' || model.domain === selectedDomain;
    
    return matchesSearch && matchesFilter && matchesDomain;
  }).sort((a, b) => {
    switch (sortBy) {
      case 'accuracy':
        return (b.accuracy || 0) - (a.accuracy || 0);
      case 'size':
        return parseFloat(b.size) - parseFloat(a.size);
      case 'created':
      default:
        return new Date(b.created).getTime() - new Date(a.created).getTime();
    }
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'trained': return 'text-green-400 bg-green-900/30';
      case 'training': return 'text-yellow-400 bg-yellow-900/30';
      case 'failed': return 'text-red-400 bg-red-900/30';
      case 'draft': return 'text-sis-gray-400 bg-sis-gray-700/30';
      default: return 'text-sis-gray-400 bg-sis-gray-700/30';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'aurag': return <Database className="w-4 h-4" />;
      case 'transformer': return <Brain className="w-4 h-4" />;
      default: return <Brain className="w-4 h-4" />;
    }
  };

  const domains = [...new Set(SAMPLE_MODELS.map(m => m.domain))];

  return (
    <div className={`bg-sis-gray-900 flex flex-col ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-sis-gray-700">
        <h2 className="text-lg font-semibold text-white mb-4">Model Library</h2>
        
        {/* Search */}
        <div className="relative mb-4">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-sis-gray-400" />
          <input
            type="text"
            placeholder="Search models, descriptions, or tags..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-md text-white placeholder-sis-gray-400 text-sm focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
          />
        </div>
        
        {/* Filters */}
        <div className="space-y-3">
          {/* Model Type Filter */}
          <div>
            <label className="block text-xs text-sis-gray-400 mb-2">Model Type</label>
            <div className="flex flex-wrap gap-2">
              {['all', 'aurag', 'transformer', 'fine-tuned', 'custom'].map(filter => (
                <button
                  key={filter}
                  onClick={() => setSelectedFilter(filter as any)}
                  className={`px-3 py-1 rounded-full text-xs font-medium transition-colors ${
                    selectedFilter === filter
                      ? 'bg-sis-blue-600 text-white'
                      : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
                  }`}
                >
                  {filter.charAt(0).toUpperCase() + filter.slice(1).replace('-', ' ')}
                </button>
              ))}
            </div>
          </div>
          
          {/* Domain Filter */}
          <div>
            <label className="block text-xs text-sis-gray-400 mb-2">Domain</label>
            <select
              value={selectedDomain}
              onChange={(e) => setSelectedDomain(e.target.value)}
              className="w-full px-3 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-md text-white text-sm focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
            >
              <option value="all">All Domains</option>
              {domains.map(domain => (
                <option key={domain} value={domain}>{domain}</option>
              ))}
            </select>
          </div>
          
          {/* Sort */}
          <div>
            <label className="block text-xs text-sis-gray-400 mb-2">Sort By</label>
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              className="w-full px-3 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-md text-white text-sm focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
            >
              <option value="created">Date Created</option>
              <option value="accuracy">Accuracy</option>
              <option value="size">Model Size</option>
            </select>
          </div>
        </div>
      </div>
      
      {/* Model List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {filteredModels.map(model => (
          <div
            key={model.id}
            onClick={() => setSelectedModel(selectedModel === model.id ? null : model.id)}
            className={`bg-sis-gray-800 rounded-lg p-4 cursor-pointer transition-colors border overflow-hidden ${
              selectedModel === model.id
                ? 'border-sis-blue-500 bg-sis-blue-900/10'
                : 'border-sis-gray-600 hover:border-sis-gray-500'
            }`}
          >
            <div className="mb-3">
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-start space-x-3 flex-1 min-w-0">
                  <div className="flex-shrink-0 mt-1">
                    {getTypeIcon(model.type)}
                  </div>
                  <h3 className="text-white font-medium break-words flex-1">{model.name}</h3>
                </div>
                <div className={`px-2 py-1 rounded-full text-xs font-medium flex-shrink-0 ml-2 ${getStatusColor(model.status)}`}>
                  {model.status.toUpperCase()}
                </div>
              </div>
              <p className="text-sm text-sis-gray-400 break-words pl-7">
                {model.description}
              </p>
            </div>
            
            {/* Model Stats */}
            <div className="space-y-1 mb-3 text-sm">
              <div className="flex flex-wrap items-center gap-x-4">
                <div>
                  <span className="text-sis-gray-400">Framework:</span>
                  <span className="text-white ml-1 font-mono text-xs">{model.framework.toUpperCase()}</span>
                </div>
                <div>
                  <span className="text-sis-gray-400">Size:</span>
                  <span className="text-white ml-1 font-mono text-xs">{model.size}</span>
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-x-4">
                <div>
                  <span className="text-sis-gray-400">Parameters:</span>
                  <span className="text-white ml-1 font-mono text-xs">{model.parameters}</span>
                </div>
                {model.accuracy && (
                  <div>
                    <span className="text-sis-gray-400">Accuracy:</span>
                    <span className="text-green-400 ml-1 font-mono text-xs">{(model.accuracy * 100).toFixed(1)}%</span>
                  </div>
                )}
              </div>
            </div>
            
            {/* Tags */}
            <div className="flex flex-wrap gap-1 mb-3 max-w-full">
              {model.tags.slice(0, 4).map(tag => (
                <span
                  key={tag}
                  className="px-2 py-1 bg-sis-gray-700 text-sis-gray-300 text-xs rounded-full flex-shrink-0"
                >
                  {tag}
                </span>
              ))}
              {model.tags.length > 4 && (
                <span className="px-2 py-1 bg-sis-gray-600 text-sis-gray-400 text-xs rounded-full flex-shrink-0">
                  +{model.tags.length - 4}
                </span>
              )}
            </div>
            
            {/* Expanded Details */}
            {selectedModel === model.id && (
              <div className="border-t border-sis-gray-700 pt-3 mt-3 space-y-3">
                <div className="space-y-2 text-sm">
                  <div className="flex flex-wrap gap-x-4">
                    <div>
                      <span className="text-sis-gray-400">Created:</span>
                      <span className="text-white ml-1 text-xs">{new Date(model.created).toLocaleDateString()}</span>
                    </div>
                    {model.lastTrained && (
                      <div>
                        <span className="text-sis-gray-400">Last Trained:</span>
                        <span className="text-white ml-1 text-xs">{new Date(model.lastTrained).toLocaleDateString()}</span>
                      </div>
                    )}
                  </div>
                  <div className="flex flex-wrap gap-x-4">
                    <div>
                      <span className="text-sis-gray-400">Domain:</span>
                      <span className="text-white ml-1 text-xs">{model.domain}</span>
                    </div>
                    <div>
                      <span className="text-sis-gray-400">Type:</span>
                      <span className="text-white ml-1 text-xs capitalize">{model.type.replace('-', ' ')}</span>
                    </div>
                  </div>
                </div>
                
                {/* Actions */}
                <div className="flex flex-wrap gap-2 pt-2">
                  <button className="flex items-center space-x-1 px-2 py-1 bg-sis-blue-600 text-white rounded-md hover:bg-sis-blue-700 transition-colors text-xs flex-shrink-0">
                    <Download className="w-3 h-3" />
                    <span>Export</span>
                  </button>
                  <button className="flex items-center space-x-1 px-2 py-1 bg-sis-gray-700 text-sis-gray-300 rounded-md hover:bg-sis-gray-600 transition-colors text-xs flex-shrink-0">
                    <Share2 className="w-3 h-3" />
                    <span>Share</span>
                  </button>
                  {model.type === 'aurag' && (
                    <button className="flex items-center space-x-1 px-2 py-1 bg-purple-600 text-white rounded-md hover:bg-purple-700 transition-colors text-xs flex-shrink-0">
                      <Brain className="w-3 h-3" />
                      <span>Test</span>
                    </button>
                  )}
                </div>
              </div>
            )}
          </div>
        ))}
        
        {filteredModels.length === 0 && (
          <div className="text-center py-12">
            <Database className="w-12 h-12 text-sis-gray-600 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-white mb-2">No models found</h3>
            <p className="text-sis-gray-400">
              Try adjusting your search or filter criteria
            </p>
          </div>
        )}
      </div>
      
      {/* Footer */}
      <div className="p-4 border-t border-sis-gray-700">
        <div className="flex items-center justify-between text-sm text-sis-gray-400">
          <span>{filteredModels.length} models shown</span>
          <span>Total size: {SAMPLE_MODELS.reduce((acc, model) => acc + parseFloat(model.size), 0).toFixed(1)} GB</span>
        </div>
      </div>
    </div>
  );
};

export default ModelBrowser;