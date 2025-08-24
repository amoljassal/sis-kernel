/**
 * Model Browser
 * Browse and discover AI models and datasets
 */

import React, { useState, useEffect } from 'react';
import { marketplaceApi, MarketplaceModel } from '../../../services/api/marketplaceApi';
import {
  Brain,
  Database,
  Download,
  Star,
  Users,
  Calendar,
  Tag,
  Search,
  Filter,
  TrendingUp,
  Award,
  Shield,
  Zap,
  Clock,
  Eye
} from 'lucide-react';

interface ModelInfo {
  id: string;
  name: string;
  description: string;
  type: 'foundation' | 'fine-tuned' | 'specialized' | 'research';
  category: 'nlp' | 'vision' | 'multimodal' | 'audio' | 'code' | 'reasoning';
  provider: string;
  author: string;
  version: string;
  downloads: number;
  rating: number;
  reviewCount: number;
  size: number; // in GB
  parameters: string;
  license: 'open' | 'commercial' | 'research';
  pricing?: {
    type: 'free' | 'paid' | 'usage-based';
    cost: number;
    unit: string;
  };
  capabilities: string[];
  tags: string[];
  publishedAt: Date;
  lastUpdated: Date;
  verified: boolean;
  featured: boolean;
  performance: {
    accuracy?: number;
    speed: number;
    efficiency: number;
  };
}

interface Dataset {
  id: string;
  name: string;
  description: string;
  category: 'text' | 'image' | 'audio' | 'video' | 'structured' | 'multimodal';
  size: number; // in GB
  samples: number;
  provider: string;
  license: 'open' | 'commercial' | 'research';
  pricing?: {
    type: 'free' | 'paid';
    cost: number;
  };
  tags: string[];
  publishedAt: Date;
  downloads: number;
  rating: number;
  reviewCount: number;
  verified: boolean;
}

const SAMPLE_MODELS: ModelInfo[] = [
  {
    id: 'model-001',
    name: 'LegalBERT Pro',
    description: 'Advanced legal document analysis model with AURAG enhancement for constitutional interpretation and case law reasoning',
    type: 'fine-tuned',
    category: 'nlp',
    provider: 'SIS Labs',
    author: 'Legal AI Team',
    version: '2.1.3',
    downloads: 15420,
    rating: 4.8,
    reviewCount: 127,
    size: 2.4,
    parameters: '340M',
    license: 'commercial',
    pricing: {
      type: 'usage-based',
      cost: 0.05,
      unit: 'per 1K tokens'
    },
    capabilities: ['Document Analysis', 'Legal Reasoning', 'Citation Extraction', 'Contract Review'],
    tags: ['legal', 'bert', 'aurag', 'constitutional-law', 'contract-analysis'],
    publishedAt: new Date('2024-01-15'),
    lastUpdated: new Date('2024-01-20'),
    verified: true,
    featured: true,
    performance: {
      accuracy: 94.7,
      speed: 89,
      efficiency: 92
    }
  },
  {
    id: 'model-002',
    name: 'MedViT-Large',
    description: 'Vision transformer for medical imaging analysis with specialized radiology training',
    type: 'specialized',
    category: 'vision',
    provider: 'MedAI Collective',
    author: 'Dr. Sarah Chen',
    version: '1.5.0',
    downloads: 8900,
    rating: 4.6,
    reviewCount: 89,
    size: 5.2,
    parameters: '1.2B',
    license: 'research',
    capabilities: ['X-ray Analysis', 'CT Scan Processing', 'Anomaly Detection', 'Report Generation'],
    tags: ['medical', 'vision', 'radiology', 'healthcare', 'transformer'],
    publishedAt: new Date('2024-01-10'),
    lastUpdated: new Date('2024-01-18'),
    verified: true,
    featured: false,
    performance: {
      accuracy: 91.2,
      speed: 76,
      efficiency: 85
    }
  },
  {
    id: 'model-003',
    name: 'CodeGen-Python',
    description: 'Specialized code generation model trained on Python codebases with focus on ML/AI libraries',
    type: 'specialized',
    category: 'code',
    provider: 'DevAI Labs',
    author: 'CodeAI Team',
    version: '3.0.1',
    downloads: 23150,
    rating: 4.9,
    reviewCount: 201,
    size: 1.8,
    parameters: '220M',
    license: 'open',
    capabilities: ['Code Generation', 'Bug Detection', 'Documentation', 'Refactoring'],
    tags: ['python', 'code-generation', 'ml', 'ai', 'open-source'],
    publishedAt: new Date('2024-01-08'),
    lastUpdated: new Date('2024-01-19'),
    verified: true,
    featured: true,
    performance: {
      accuracy: 87.3,
      speed: 94,
      efficiency: 89
    }
  }
];

const SAMPLE_DATASETS: Dataset[] = [
  {
    id: 'dataset-001',
    name: 'Legal Documents Corpus',
    description: 'Comprehensive collection of legal documents, contracts, and case law with annotations',
    category: 'text',
    size: 45.2,
    samples: 2400000,
    provider: 'Legal Data Commons',
    license: 'commercial',
    pricing: {
      type: 'paid',
      cost: 299
    },
    tags: ['legal', 'contracts', 'case-law', 'annotations'],
    publishedAt: new Date('2024-01-12'),
    downloads: 892,
    rating: 4.7,
    reviewCount: 34,
    verified: true
  },
  {
    id: 'dataset-002',
    name: 'Medical Imaging Dataset',
    description: 'Curated medical imaging dataset with expert annotations for radiology training',
    category: 'image',
    size: 120.5,
    samples: 150000,
    provider: 'MedData Institute',
    license: 'research',
    tags: ['medical', 'radiology', 'x-ray', 'ct-scan', 'annotations'],
    publishedAt: new Date('2024-01-05'),
    downloads: 456,
    rating: 4.8,
    reviewCount: 67,
    verified: true
  }
];

export const ModelBrowser: React.FC = () => {
  const [models, setModels] = useState<MarketplaceModel[]>([]);
  const [datasets, setDatasets] = useState<MarketplaceModel[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'models' | 'datasets'>('models');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedLicense, setSelectedLicense] = useState<string>('all');
  const [sortBy, setSortBy] = useState<'popular' | 'rating' | 'recent'>('popular');
  const [showFilters, setShowFilters] = useState(false);

  // Load marketplace data
  useEffect(() => {
    loadMarketplaceData();
  }, [activeTab, selectedCategory, selectedLicense, sortBy, searchQuery]);

  const loadMarketplaceData = async () => {
    setLoading(true);
    setError(null);
    try {
      const filters = {
        type: activeTab === 'models' ? 'model' as const : 'dataset' as const,
        category: selectedCategory !== 'all' ? selectedCategory : undefined,
        search: searchQuery || undefined,
        sort: sortBy,
      };

      const data = await marketplaceApi.getMarketplaceModels(filters);
      
      if (activeTab === 'models') {
        setModels(data.filter(item => item.type === 'model'));
      } else {
        setDatasets(data.filter(item => item.type === 'dataset'));
      }
    } catch (err) {
      setError('Failed to load marketplace data');
      console.error('Error loading marketplace data:', err);
    } finally {
      setLoading(false);
    }
  };

  // Data is now filtered and sorted on the API side
  const filteredModels = models;
  const filteredDatasets = datasets;

  const ModelCard: React.FC<{ model: MarketplaceModel }> = ({ model }) => (
    <div className="card p-6 hover:border-sis-blue-500 transition-all">
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1">
          <div className="flex items-center space-x-2 mb-2">
            <h3 className="text-lg font-semibold text-white">{model.name}</h3>
            {model.verified && <Shield className="w-4 h-4 text-green-400" />}
            {model.featured && <Award className="w-4 h-4 text-yellow-400" />}
          </div>
          <p className="text-sm text-sis-gray-400 mb-3 line-clamp-2">{model.description}</p>
          
          <div className="flex items-center space-x-4 text-xs text-sis-gray-500 mb-3">
            <span>by {model.author.name}</span>
            <span>v{model.version}</span>
            <span>{model.parameters} params</span>
            <span>{model.size}GB</span>
          </div>

          <div className="flex flex-wrap gap-2 mb-3">
            {model.capabilities.slice(0, 3).map(cap => (
              <span key={cap} className="text-xs px-2 py-1 bg-sis-blue-600/20 text-sis-blue-300 rounded">
                {cap}
              </span>
            ))}
            {model.capabilities.length > 3 && (
              <span className="text-xs px-2 py-1 bg-sis-gray-700 text-sis-gray-300 rounded">
                +{model.capabilities.length - 3}
              </span>
            )}
          </div>
        </div>

        <div className="text-right ml-4">
          {model.pricing ? (
            <div className="text-lg font-bold text-sis-blue-400">
              ${model.pricing.cost}{model.pricing.unit}
            </div>
          ) : (
            <div className="text-lg font-bold text-green-400">Free</div>
          )}
          <div className="text-xs text-sis-gray-400 capitalize">{model.license}</div>
        </div>
      </div>

      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-4 text-sm">
          <div className="flex items-center space-x-1">
            <Star className="w-4 h-4 text-yellow-400 fill-current" />
            <span className="text-white">{model.metrics.rating}</span>
            <span className="text-sis-gray-400">({model.metrics.reviews})</span>
          </div>
          <div className="flex items-center space-x-1">
            <Download className="w-4 h-4 text-sis-gray-400" />
            <span className="text-white">{model.metrics.downloads.toLocaleString()}</span>
          </div>
          <div className="flex items-center space-x-1">
            <Clock className="w-4 h-4 text-sis-gray-400" />
            <span className="text-sis-gray-400">{new Date(model.updatedAt).toLocaleDateString()}</span>
          </div>
        </div>
        
        <div className="flex items-center space-x-1">
          <span className="text-xs px-2 py-1 bg-sis-purple-600/20 text-sis-purple-300 rounded capitalize">
            {model.type}
          </span>
          <span className="text-xs px-2 py-1 bg-sis-green-600/20 text-sis-green-300 rounded capitalize">
            {model.category}
          </span>
        </div>
      </div>

      {model.performance && (
        <div className="space-y-2 mb-4">
          <div className="text-xs text-sis-gray-400 mb-1">Performance Metrics</div>
          <div className="grid grid-cols-3 gap-3 text-xs">
            {model.performance.accuracy && (
              <div className="text-center">
                <div className="text-white font-medium">{model.performance.accuracy}%</div>
                <div className="text-sis-gray-400">Accuracy</div>
              </div>
            )}
            <div className="text-center">
              <div className="text-white font-medium">{model.performance.speed}%</div>
              <div className="text-sis-gray-400">Speed</div>
            </div>
            <div className="text-center">
              <div className="text-white font-medium">{model.performance.efficiency}%</div>
              <div className="text-sis-gray-400">Efficiency</div>
            </div>
          </div>
        </div>
      )}

      <div className="flex space-x-2">
        <button className="flex-1 btn-primary text-sm py-2 flex items-center justify-center space-x-2">
          <Download className="w-4 h-4" />
          <span>Download</span>
        </button>
        <button className="btn-secondary text-sm py-2 px-4 flex items-center space-x-2">
          <Eye className="w-4 h-4" />
          <span>Details</span>
        </button>
      </div>
    </div>
  );

  const DatasetCard: React.FC<{ dataset: Dataset }> = ({ dataset }) => (
    <div className="card p-6 hover:border-sis-blue-500 transition-all">
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1">
          <div className="flex items-center space-x-2 mb-2">
            <h3 className="text-lg font-semibold text-white">{dataset.name}</h3>
            {dataset.verified && <Shield className="w-4 h-4 text-green-400" />}
          </div>
          <p className="text-sm text-sis-gray-400 mb-3 line-clamp-2">{dataset.description}</p>
          
          <div className="flex items-center space-x-4 text-xs text-sis-gray-500 mb-3">
            <span>{dataset.samples.toLocaleString()} samples</span>
            <span>{dataset.size}GB</span>
            <span>by {dataset.provider}</span>
          </div>
        </div>

        <div className="text-right ml-4">
          {dataset.pricing ? (
            <div className="text-lg font-bold text-sis-blue-400">
              ${dataset.pricing.cost}
            </div>
          ) : (
            <div className="text-lg font-bold text-green-400">Free</div>
          )}
          <div className="text-xs text-sis-gray-400 capitalize">{dataset.license}</div>
        </div>
      </div>

      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-4 text-sm">
          <div className="flex items-center space-x-1">
            <Star className="w-4 h-4 text-yellow-400 fill-current" />
            <span className="text-white">{dataset.rating}</span>
            <span className="text-sis-gray-400">({dataset.reviewCount})</span>
          </div>
          <div className="flex items-center space-x-1">
            <Download className="w-4 h-4 text-sis-gray-400" />
            <span className="text-white">{dataset.downloads.toLocaleString()}</span>
          </div>
        </div>
        
        <span className="text-xs px-2 py-1 bg-sis-orange-600/20 text-sis-orange-300 rounded capitalize">
          {dataset.category}
        </span>
      </div>

      <div className="flex flex-wrap gap-2 mb-4">
        {dataset.tags.map(tag => (
          <span key={tag} className="text-xs px-2 py-1 bg-sis-gray-700 text-sis-gray-300 rounded">
            {tag}
          </span>
        ))}
      </div>

      <div className="flex space-x-2">
        <button className="flex-1 btn-primary text-sm py-2 flex items-center justify-center space-x-2">
          <Download className="w-4 h-4" />
          <span>Download</span>
        </button>
        <button className="btn-secondary text-sm py-2 px-4 flex items-center space-x-2">
          <Eye className="w-4 h-4" />
          <span>Preview</span>
        </button>
      </div>
    </div>
  );

  return (
    <div className="space-y-6">
      {/* Search and Filters */}
      <div className="space-y-4">
        <div className="flex items-center space-x-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-sis-gray-400" />
            <input
              type="text"
              placeholder="Search models, datasets, or capabilities..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-lg text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
            />
          </div>
          
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`px-4 py-2 rounded-lg flex items-center space-x-2 transition-colors ${
              showFilters ? 'bg-sis-blue-600 text-white' : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
            }`}
          >
            <Filter className="w-4 h-4" />
            <span>Filters</span>
          </button>
        </div>

        {showFilters && (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 p-4 bg-sis-gray-800 rounded-lg">
            <div>
              <label className="block text-sm text-sis-gray-300 mb-2">Category</label>
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                className="w-full bg-sis-gray-700 border border-sis-gray-600 rounded text-white text-sm px-3 py-2"
              >
                <option value="all">All Categories</option>
                <option value="nlp">Natural Language</option>
                <option value="vision">Computer Vision</option>
                <option value="audio">Audio Processing</option>
                <option value="code">Code Generation</option>
                <option value="multimodal">Multimodal</option>
                <option value="reasoning">Reasoning</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm text-sis-gray-300 mb-2">License</label>
              <select
                value={selectedLicense}
                onChange={(e) => setSelectedLicense(e.target.value)}
                className="w-full bg-sis-gray-700 border border-sis-gray-600 rounded text-white text-sm px-3 py-2"
              >
                <option value="all">All Licenses</option>
                <option value="open">Open Source</option>
                <option value="commercial">Commercial</option>
                <option value="research">Research Only</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm text-sis-gray-300 mb-2">Sort By</label>
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as any)}
                className="w-full bg-sis-gray-700 border border-sis-gray-600 rounded text-white text-sm px-3 py-2"
              >
                <option value="popular">Most Popular</option>
                <option value="rating">Highest Rated</option>
                <option value="recent">Most Recent</option>
                <option value="size">Smallest Size</option>
              </select>
            </div>
          </div>
        )}
      </div>

      {/* Tab Navigation */}
      <div className="flex items-center space-x-2">
        <button
          onClick={() => setActiveTab('models')}
          className={`px-4 py-2 rounded-lg flex items-center space-x-2 transition-colors ${
            activeTab === 'models'
              ? 'bg-sis-blue-600 text-white'
              : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
          }`}
        >
          <Brain className="w-4 h-4" />
          <span>Models ({filteredModels.length})</span>
        </button>
        
        <button
          onClick={() => setActiveTab('datasets')}
          className={`px-4 py-2 rounded-lg flex items-center space-x-2 transition-colors ${
            activeTab === 'datasets'
              ? 'bg-sis-blue-600 text-white'
              : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
          }`}
        >
          <Database className="w-4 h-4" />
          <span>Datasets ({filteredDatasets.length})</span>
        </button>
      </div>

      {/* Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {activeTab === 'models' ? (
          filteredModels.length > 0 ? (
            filteredModels.map(model => (
              <ModelCard key={model.id} model={model} />
            ))
          ) : (
            <div className="col-span-2 text-center py-12">
              <Brain className="w-12 h-12 text-sis-gray-600 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-white mb-2">No models found</h3>
              <p className="text-sis-gray-400">Try adjusting your search or filters</p>
            </div>
          )
        ) : (
          filteredDatasets.length > 0 ? (
            filteredDatasets.map(dataset => (
              <DatasetCard key={dataset.id} dataset={dataset} />
            ))
          ) : (
            <div className="col-span-2 text-center py-12">
              <Database className="w-12 h-12 text-sis-gray-600 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-white mb-2">No datasets found</h3>
              <p className="text-sis-gray-400">Try adjusting your search or filters</p>
            </div>
          )
        )}
      </div>
    </div>
  );
};

export default ModelBrowser;