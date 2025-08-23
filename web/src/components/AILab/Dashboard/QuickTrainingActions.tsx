/**
 * Quick Training Actions
 * Fast access training templates and common operations
 */

import React, { useState } from 'react';
import {
  Zap,
  Copy,
  Layers,
  FileText,
  RefreshCw,
  Download,
  Upload,
  Settings,
  GitBranch,
  Users,
  Mic,
  Image,
  Code,
  MessageSquare,
  Clock
} from 'lucide-react';

interface TrainingTemplate {
  id: string;
  name: string;
  description: string;
  icon: React.ComponentType<any>;
  category: 'quick-start' | 'aurag' | 'fine-tune' | 'collaboration';
  estimatedTime: string;
  recommended?: boolean;
}

const TRAINING_TEMPLATES: TrainingTemplate[] = [
  {
    id: 'quick-train',
    name: 'Quick Train',
    description: 'Start training from natural language description',
    icon: Zap,
    category: 'quick-start',
    estimatedTime: '< 1 min',
    recommended: true
  },
  {
    id: 'clone-modify',
    name: 'Clone & Modify',
    description: 'Fork an existing model and customize',
    icon: Copy,
    category: 'fine-tune',
    estimatedTime: '2-3 hours'
  },
  {
    id: 'aurag-builder',
    name: 'AURAG Builder',
    description: 'Create knowledge-augmented model with philosophical lenses',
    icon: Layers,
    category: 'aurag',
    estimatedTime: '4-6 hours',
    recommended: true
  },
  {
    id: 'dataset-upload',
    name: 'Dataset Upload',
    description: 'Import and prepare custom training data',
    icon: Upload,
    category: 'quick-start',
    estimatedTime: 'Variable'
  },
  {
    id: 'fine-tune-existing',
    name: 'Fine-Tune Model',
    description: 'Adapt pre-trained model to specific domain',
    icon: RefreshCw,
    category: 'fine-tune',
    estimatedTime: '1-2 hours'
  },
  {
    id: 'collaborative-training',
    name: 'Team Training',
    description: 'Start collaborative training session',
    icon: Users,
    category: 'collaboration',
    estimatedTime: 'Variable'
  }
];

interface QuickTrainingActionsProps {
  onActionSelect: (templateId: string) => void;
}

export const QuickTrainingActions: React.FC<QuickTrainingActionsProps> = ({ onActionSelect }) => {
  const [selectedCategory, setSelectedCategory] = useState<'all' | 'quick-start' | 'aurag' | 'fine-tune' | 'collaboration'>('all');
  const [showNaturalLanguage, setShowNaturalLanguage] = useState(false);
  const [nlPrompt, setNlPrompt] = useState('');

  const filteredTemplates = selectedCategory === 'all' 
    ? TRAINING_TEMPLATES 
    : TRAINING_TEMPLATES.filter(t => t.category === selectedCategory);

  const handleNaturalLanguageSubmit = () => {
    if (nlPrompt.trim()) {
      console.log('Natural language training request:', nlPrompt);
      onActionSelect('nl-training');
      setNlPrompt('');
      setShowNaturalLanguage(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Natural Language Interface */}
      <div className="card p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white flex items-center space-x-2">
            <MessageSquare className="w-5 h-5 text-sis-blue-400" />
            <span>Natural Language Training</span>
          </h3>
          <button
            onClick={() => setShowNaturalLanguage(!showNaturalLanguage)}
            className="text-sis-gray-400 hover:text-white transition-colors"
          >
            {showNaturalLanguage ? 'Hide' : 'Show'}
          </button>
        </div>

        {showNaturalLanguage && (
          <div className="space-y-4">
            <textarea
              value={nlPrompt}
              onChange={(e) => setNlPrompt(e.target.value)}
              placeholder="Describe what you want to train... e.g., 'Train a sentiment analyzer for customer reviews using BERT with 10 epochs'"
              className="w-full h-24 px-4 py-3 bg-sis-gray-800 border border-sis-gray-600 rounded-lg text-white placeholder-sis-gray-400 resize-none focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
            />
            
            <div className="flex items-center space-x-3">
              <button
                onClick={handleNaturalLanguageSubmit}
                disabled={!nlPrompt.trim()}
                className="btn-primary px-4 py-2 disabled:bg-sis-gray-600 disabled:cursor-not-allowed"
              >
                Start Training
              </button>
              
              <button className="p-2 bg-sis-gray-700 text-sis-gray-300 rounded-lg hover:bg-sis-gray-600 transition-colors">
                <Mic className="w-5 h-5" />
              </button>
              
              <button className="p-2 bg-sis-gray-700 text-sis-gray-300 rounded-lg hover:bg-sis-gray-600 transition-colors">
                <Image className="w-5 h-5" />
              </button>
              
              <button className="p-2 bg-sis-gray-700 text-sis-gray-300 rounded-lg hover:bg-sis-gray-600 transition-colors">
                <Code className="w-5 h-5" />
              </button>
            </div>

            <div className="text-xs text-sis-gray-400">
              Tip: You can also use voice commands, upload sketches, or paste code snippets
            </div>
          </div>
        )}
      </div>

      {/* Category Filter */}
      <div className="flex items-center space-x-2">
        {['all', 'quick-start', 'aurag', 'fine-tune', 'collaboration'].map(category => (
          <button
            key={category}
            onClick={() => setSelectedCategory(category as any)}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              selectedCategory === category
                ? 'bg-sis-blue-600 text-white'
                : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
            }`}
          >
            {category.charAt(0).toUpperCase() + category.slice(1).replace('-', ' ')}
          </button>
        ))}
      </div>

      {/* Training Templates Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {filteredTemplates.map(template => {
          const IconComponent = template.icon;
          return (
            <button
              key={template.id}
              onClick={() => onActionSelect(template.id)}
              className="card p-6 text-left hover:border-sis-blue-500 transition-all hover:scale-105 relative"
            >
              {template.recommended && (
                <div className="absolute top-2 right-2 px-2 py-1 bg-green-600 text-white text-xs rounded-full">
                  Recommended
                </div>
              )}
              
              <div className="flex items-start space-x-4">
                <div className="flex-shrink-0">
                  <div className="w-12 h-12 bg-sis-blue-600/20 rounded-lg flex items-center justify-center">
                    <IconComponent className="w-6 h-6 text-sis-blue-400" />
                  </div>
                </div>
                
                <div className="flex-1 min-w-0">
                  <h4 className="text-white font-medium mb-1">{template.name}</h4>
                  <p className="text-sm text-sis-gray-400 mb-2">{template.description}</p>
                  <div className="flex items-center space-x-4 text-xs text-sis-gray-500">
                    <span>Est. {template.estimatedTime}</span>
                    <span className="capitalize">{template.category.replace('-', ' ')}</span>
                  </div>
                </div>
              </div>
            </button>
          );
        })}
      </div>

      {/* Recent Models Section */}
      <div className="card p-6">
        <h3 className="text-lg font-semibold text-white mb-4 flex items-center space-x-2">
          <Clock className="w-5 h-5 text-purple-400" />
          <span>Recent Models</span>
        </h3>
        
        <div className="space-y-3">
          <div className="flex items-center justify-between p-3 bg-sis-gray-800 rounded-lg">
            <div className="flex items-center space-x-3">
              <FileText className="w-5 h-5 text-sis-gray-400" />
              <div>
                <p className="text-white font-medium">Legal Document Analyzer</p>
                <p className="text-xs text-sis-gray-400">Last trained 2 hours ago</p>
              </div>
            </div>
            <button className="text-sis-blue-400 hover:text-sis-blue-300 transition-colors">
              Resume
            </button>
          </div>
          
          <div className="flex items-center justify-between p-3 bg-sis-gray-800 rounded-lg">
            <div className="flex items-center space-x-3">
              <FileText className="w-5 h-5 text-sis-gray-400" />
              <div>
                <p className="text-white font-medium">Medical Knowledge Assistant</p>
                <p className="text-xs text-sis-gray-400">Completed yesterday</p>
              </div>
            </div>
            <button className="text-sis-blue-400 hover:text-sis-blue-300 transition-colors">
              Clone
            </button>
          </div>
          
          <div className="flex items-center justify-between p-3 bg-sis-gray-800 rounded-lg">
            <div className="flex items-center space-x-3">
              <FileText className="w-5 h-5 text-sis-gray-400" />
              <div>
                <p className="text-white font-medium">Code Generation Model</p>
                <p className="text-xs text-sis-gray-400">Completed 3 days ago</p>
              </div>
            </div>
            <button className="text-sis-blue-400 hover:text-sis-blue-300 transition-colors">
              View
            </button>
          </div>
        </div>
      </div>

      {/* Training Tips */}
      <div className="bg-sis-blue-900/30 border border-sis-blue-500/30 p-4 rounded-lg">
        <h4 className="font-medium text-sis-blue-300 mb-2">Training Tips</h4>
        <ul className="space-y-1 text-sm text-sis-blue-200">
          <li>• Use AURAG Builder for knowledge-intensive tasks requiring reasoning</li>
          <li>• Apple Silicon optimization is automatic for MLX-compatible models</li>
          <li>• Enable collaborative mode for team training sessions</li>
          <li>• Monitor resource usage to optimize batch sizes and learning rates</li>
        </ul>
      </div>
    </div>
  );
};

export default QuickTrainingActions;