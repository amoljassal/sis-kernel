/**
 * Model Publisher
 * Publish and manage AI models and datasets in the marketplace
 */

import React, { useState } from 'react';
import {
  Upload,
  FileText,
  DollarSign,
  Tag,
  Settings,
  Eye,
  Save,
  Send,
  AlertTriangle,
  CheckCircle,
  Brain,
  Database,
  Image,
  Code,
  Mic,
  Video
} from 'lucide-react';

interface PublishingForm {
  name: string;
  description: string;
  type: 'model' | 'dataset';
  category: string;
  version: string;
  license: 'open' | 'commercial' | 'research';
  pricing: {
    type: 'free' | 'paid' | 'usage-based';
    cost: number;
    unit: string;
  };
  capabilities: string[];
  tags: string[];
  documentation: string;
  requirements: string[];
  modelFile?: File;
  configFile?: File;
  sampleData?: File;
  readme?: File;
}

const initialForm: PublishingForm = {
  name: '',
  description: '',
  type: 'model',
  category: '',
  version: '1.0.0',
  license: 'open',
  pricing: {
    type: 'free',
    cost: 0,
    unit: 'per use'
  },
  capabilities: [],
  tags: [],
  documentation: '',
  requirements: []
};

const MODEL_CATEGORIES = [
  { id: 'nlp', label: 'Natural Language Processing', icon: FileText },
  { id: 'vision', label: 'Computer Vision', icon: Eye },
  { id: 'audio', label: 'Audio Processing', icon: Mic },
  { id: 'video', label: 'Video Analysis', icon: Video },
  { id: 'code', label: 'Code Generation', icon: Code },
  { id: 'multimodal', label: 'Multimodal', icon: Brain },
  { id: 'reasoning', label: 'Reasoning & Logic', icon: Settings }
];

const DATASET_CATEGORIES = [
  { id: 'text', label: 'Text Data', icon: FileText },
  { id: 'image', label: 'Image Data', icon: Image },
  { id: 'audio', label: 'Audio Data', icon: Mic },
  { id: 'video', label: 'Video Data', icon: Video },
  { id: 'structured', label: 'Structured Data', icon: Database },
  { id: 'multimodal', label: 'Multimodal Data', icon: Brain }
];

export const ModelPublisher: React.FC = () => {
  const [form, setForm] = useState<PublishingForm>(initialForm);
  const [currentStep, setCurrentStep] = useState(1);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [validationErrors, setValidationErrors] = useState<string[]>([]);
  const [newCapability, setNewCapability] = useState('');
  const [newTag, setNewTag] = useState('');
  const [newRequirement, setNewRequirement] = useState('');

  const steps = [
    { id: 1, title: 'Basic Information', description: 'Name, description, and category' },
    { id: 2, title: 'Technical Details', description: 'Version, license, and capabilities' },
    { id: 3, title: 'Pricing & Distribution', description: 'Pricing model and access terms' },
    { id: 4, title: 'Files & Documentation', description: 'Upload files and documentation' },
    { id: 5, title: 'Review & Publish', description: 'Review and submit for approval' }
  ];

  const updateForm = (field: string, value: any) => {
    setForm(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const updateNested = (parent: string, field: string, value: any) => {
    setForm(prev => ({
      ...prev,
      [parent]: {
        ...prev[parent as keyof PublishingForm] as any,
        [field]: value
      }
    }));
  };

  const addToArray = (field: keyof PublishingForm, value: string) => {
    if (value.trim()) {
      setForm(prev => ({
        ...prev,
        [field]: [...(prev[field] as string[]), value.trim()]
      }));
    }
  };

  const removeFromArray = (field: keyof PublishingForm, index: number) => {
    setForm(prev => ({
      ...prev,
      [field]: (prev[field] as string[]).filter((_, i) => i !== index)
    }));
  };

  const validateStep = (step: number): boolean => {
    const errors: string[] = [];
    
    switch (step) {
      case 1:
        if (!form.name.trim()) errors.push('Name is required');
        if (!form.description.trim()) errors.push('Description is required');
        if (!form.category) errors.push('Category is required');
        break;
      case 2:
        if (!form.version.trim()) errors.push('Version is required');
        if (form.capabilities.length === 0) errors.push('At least one capability is required');
        break;
      case 3:
        if (form.pricing.type === 'paid' && form.pricing.cost <= 0) {
          errors.push('Price must be greater than 0 for paid models');
        }
        break;
      case 4:
        if (form.type === 'model' && !form.modelFile) {
          errors.push('Model file is required');
        }
        break;
    }
    
    setValidationErrors(errors);
    return errors.length === 0;
  };

  const handleNext = () => {
    if (validateStep(currentStep)) {
      setCurrentStep(prev => Math.min(prev + 1, steps.length));
    }
  };

  const handlePrevious = () => {
    setCurrentStep(prev => Math.max(prev - 1, 1));
  };

  const handleSubmit = async () => {
    if (!validateStep(currentStep)) return;
    
    setIsSubmitting(true);
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000));
      alert('Model/Dataset submitted successfully! It will be reviewed before publication.');
      setForm(initialForm);
      setCurrentStep(1);
    } catch (error) {
      alert('Failed to submit. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  const renderStep = () => {
    switch (currentStep) {
      case 1:
        return (
          <div className="space-y-6">
            <div>
              <label className="block text-sm font-medium text-white mb-2">Type</label>
              <div className="flex space-x-4">
                <button
                  type="button"
                  onClick={() => updateForm('type', 'model')}
                  className={`flex-1 p-4 rounded-lg border-2 transition-colors ${
                    form.type === 'model'
                      ? 'border-sis-blue-500 bg-sis-blue-500/10'
                      : 'border-sis-gray-600 hover:border-sis-gray-500'
                  }`}
                >
                  <Brain className="w-6 h-6 mx-auto mb-2 text-sis-blue-400" />
                  <div className="text-white font-medium">AI Model</div>
                  <div className="text-sm text-sis-gray-400">Trained AI model</div>
                </button>
                <button
                  type="button"
                  onClick={() => updateForm('type', 'dataset')}
                  className={`flex-1 p-4 rounded-lg border-2 transition-colors ${
                    form.type === 'dataset'
                      ? 'border-sis-blue-500 bg-sis-blue-500/10'
                      : 'border-sis-gray-600 hover:border-sis-gray-500'
                  }`}
                >
                  <Database className="w-6 h-6 mx-auto mb-2 text-sis-green-400" />
                  <div className="text-white font-medium">Dataset</div>
                  <div className="text-sm text-sis-gray-400">Training dataset</div>
                </button>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-white mb-2">Name</label>
              <input
                type="text"
                value={form.name}
                onChange={(e) => updateForm('name', e.target.value)}
                placeholder="Enter a descriptive name..."
                className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-3 text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-white mb-2">Description</label>
              <textarea
                value={form.description}
                onChange={(e) => updateForm('description', e.target.value)}
                placeholder="Describe what your model/dataset does and its key features..."
                rows={4}
                className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-3 text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500 resize-none"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-white mb-2">Category</label>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                {(form.type === 'model' ? MODEL_CATEGORIES : DATASET_CATEGORIES).map(category => {
                  const IconComponent = category.icon;
                  return (
                    <button
                      key={category.id}
                      type="button"
                      onClick={() => updateForm('category', category.id)}
                      className={`p-3 rounded-lg border transition-colors text-left ${
                        form.category === category.id
                          ? 'border-sis-blue-500 bg-sis-blue-500/10'
                          : 'border-sis-gray-600 hover:border-sis-gray-500'
                      }`}
                    >
                      <IconComponent className="w-5 h-5 mb-2 text-sis-blue-400" />
                      <div className="text-sm font-medium text-white">{category.label}</div>
                    </button>
                  );
                })}
              </div>
            </div>
          </div>
        );

      case 2:
        return (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-white mb-2">Version</label>
                <input
                  type="text"
                  value={form.version}
                  onChange={(e) => updateForm('version', e.target.value)}
                  placeholder="1.0.0"
                  className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-3 text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-white mb-2">License</label>
                <select
                  value={form.license}
                  onChange={(e) => updateForm('license', e.target.value)}
                  className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                >
                  <option value="open">Open Source</option>
                  <option value="commercial">Commercial</option>
                  <option value="research">Research Only</option>
                </select>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-white mb-2">Capabilities</label>
              <div className="flex space-x-2 mb-3">
                <input
                  type="text"
                  value={newCapability}
                  onChange={(e) => setNewCapability(e.target.value)}
                  placeholder="Add a capability..."
                  className="flex-1 bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-2 text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                />
                <button
                  type="button"
                  onClick={() => {
                    addToArray('capabilities', newCapability);
                    setNewCapability('');
                  }}
                  className="btn-primary px-4 py-2"
                >
                  Add
                </button>
              </div>
              <div className="flex flex-wrap gap-2">
                {form.capabilities.map((capability, index) => (
                  <span
                    key={index}
                    className="inline-flex items-center space-x-2 px-3 py-1 bg-sis-blue-600/20 text-sis-blue-300 rounded-full text-sm"
                  >
                    <span>{capability}</span>
                    <button
                      type="button"
                      onClick={() => removeFromArray('capabilities', index)}
                      className="text-sis-blue-200 hover:text-white"
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-white mb-2">Tags</label>
              <div className="flex space-x-2 mb-3">
                <input
                  type="text"
                  value={newTag}
                  onChange={(e) => setNewTag(e.target.value)}
                  placeholder="Add a tag..."
                  className="flex-1 bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-2 text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                />
                <button
                  type="button"
                  onClick={() => {
                    addToArray('tags', newTag);
                    setNewTag('');
                  }}
                  className="btn-primary px-4 py-2"
                >
                  Add
                </button>
              </div>
              <div className="flex flex-wrap gap-2">
                {form.tags.map((tag, index) => (
                  <span
                    key={index}
                    className="inline-flex items-center space-x-2 px-3 py-1 bg-sis-gray-700 text-sis-gray-300 rounded-full text-sm"
                  >
                    <span>{tag}</span>
                    <button
                      type="button"
                      onClick={() => removeFromArray('tags', index)}
                      className="text-sis-gray-200 hover:text-white"
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-white mb-2">Requirements</label>
              <div className="flex space-x-2 mb-3">
                <input
                  type="text"
                  value={newRequirement}
                  onChange={(e) => setNewRequirement(e.target.value)}
                  placeholder="e.g., Python 3.8+, PyTorch 1.9+"
                  className="flex-1 bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-2 text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                />
                <button
                  type="button"
                  onClick={() => {
                    addToArray('requirements', newRequirement);
                    setNewRequirement('');
                  }}
                  className="btn-primary px-4 py-2"
                >
                  Add
                </button>
              </div>
              <div className="space-y-2">
                {form.requirements.map((req, index) => (
                  <div
                    key={index}
                    className="flex items-center justify-between p-2 bg-sis-gray-800 rounded text-sm"
                  >
                    <span className="text-white">{req}</span>
                    <button
                      type="button"
                      onClick={() => removeFromArray('requirements', index)}
                      className="text-red-400 hover:text-red-300"
                    >
                      Remove
                    </button>
                  </div>
                ))}
              </div>
            </div>
          </div>
        );

      case 3:
        return (
          <div className="space-y-6">
            <div>
              <label className="block text-sm font-medium text-white mb-2">Pricing Model</label>
              <div className="space-y-3">
                <label className="flex items-center space-x-3 p-3 bg-sis-gray-800 rounded-lg cursor-pointer">
                  <input
                    type="radio"
                    name="pricing"
                    value="free"
                    checked={form.pricing.type === 'free'}
                    onChange={(e) => updateNested('pricing', 'type', e.target.value)}
                    className="text-sis-blue-500"
                  />
                  <div>
                    <div className="text-white font-medium">Free</div>
                    <div className="text-sm text-sis-gray-400">Open access for everyone</div>
                  </div>
                </label>
                
                <label className="flex items-center space-x-3 p-3 bg-sis-gray-800 rounded-lg cursor-pointer">
                  <input
                    type="radio"
                    name="pricing"
                    value="paid"
                    checked={form.pricing.type === 'paid'}
                    onChange={(e) => updateNested('pricing', 'type', e.target.value)}
                    className="text-sis-blue-500"
                  />
                  <div>
                    <div className="text-white font-medium">One-time Purchase</div>
                    <div className="text-sm text-sis-gray-400">Single payment for unlimited use</div>
                  </div>
                </label>
                
                <label className="flex items-center space-x-3 p-3 bg-sis-gray-800 rounded-lg cursor-pointer">
                  <input
                    type="radio"
                    name="pricing"
                    value="usage-based"
                    checked={form.pricing.type === 'usage-based'}
                    onChange={(e) => updateNested('pricing', 'type', e.target.value)}
                    className="text-sis-blue-500"
                  />
                  <div>
                    <div className="text-white font-medium">Usage-Based</div>
                    <div className="text-sm text-sis-gray-400">Pay per API call or token</div>
                  </div>
                </label>
              </div>
            </div>

            {form.pricing.type !== 'free' && (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <label className="block text-sm font-medium text-white mb-2">Price</label>
                  <div className="flex items-center">
                    <span className="text-white mr-2">$</span>
                    <input
                      type="number"
                      value={form.pricing.cost}
                      onChange={(e) => updateNested('pricing', 'cost', parseFloat(e.target.value) || 0)}
                      placeholder="0.00"
                      step="0.01"
                      min="0"
                      className="flex-1 bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-3 text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                    />
                  </div>
                </div>

                {form.pricing.type === 'usage-based' && (
                  <div>
                    <label className="block text-sm font-medium text-white mb-2">Unit</label>
                    <select
                      value={form.pricing.unit}
                      onChange={(e) => updateNested('pricing', 'unit', e.target.value)}
                      className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                    >
                      <option value="per use">per use</option>
                      <option value="per 1K tokens">per 1K tokens</option>
                      <option value="per hour">per hour</option>
                      <option value="per query">per query</option>
                    </select>
                  </div>
                )}
              </div>
            )}

            <div className="bg-sis-blue-900/30 border border-sis-blue-500/30 p-4 rounded-lg">
              <div className="flex items-start space-x-3">
                <DollarSign className="w-5 h-5 text-sis-blue-400 mt-0.5" />
                <div>
                  <h4 className="text-sis-blue-300 font-medium mb-1">Revenue Sharing</h4>
                  <p className="text-sm text-sis-blue-200">
                    You keep 80% of all revenue. Platform fee is 20% to cover hosting, 
                    processing, and marketplace operations.
                  </p>
                </div>
              </div>
            </div>
          </div>
        );

      case 4:
        return (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-white mb-2">
                  {form.type === 'model' ? 'Model File' : 'Dataset Archive'} *
                </label>
                <div className="border-2 border-dashed border-sis-gray-600 rounded-lg p-6 text-center hover:border-sis-gray-500 transition-colors">
                  <Upload className="w-8 h-8 text-sis-gray-400 mx-auto mb-2" />
                  <p className="text-sm text-sis-gray-400 mb-2">
                    Drop your {form.type} file here or click to browse
                  </p>
                  <input
                    type="file"
                    onChange={(e) => updateForm('modelFile', e.target.files?.[0])}
                    className="hidden"
                    id="model-file"
                  />
                  <label htmlFor="model-file" className="btn-secondary text-sm cursor-pointer">
                    Choose File
                  </label>
                  {form.modelFile && (
                    <p className="text-sm text-green-400 mt-2">{form.modelFile.name}</p>
                  )}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-white mb-2">Configuration File</label>
                <div className="border-2 border-dashed border-sis-gray-600 rounded-lg p-6 text-center hover:border-sis-gray-500 transition-colors">
                  <FileText className="w-8 h-8 text-sis-gray-400 mx-auto mb-2" />
                  <p className="text-sm text-sis-gray-400 mb-2">
                    Model config, hyperparameters, etc.
                  </p>
                  <input
                    type="file"
                    onChange={(e) => updateForm('configFile', e.target.files?.[0])}
                    className="hidden"
                    id="config-file"
                  />
                  <label htmlFor="config-file" className="btn-secondary text-sm cursor-pointer">
                    Choose File
                  </label>
                  {form.configFile && (
                    <p className="text-sm text-green-400 mt-2">{form.configFile.name}</p>
                  )}
                </div>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-white mb-2">Documentation</label>
              <textarea
                value={form.documentation}
                onChange={(e) => updateForm('documentation', e.target.value)}
                placeholder="Provide detailed documentation, usage examples, API reference..."
                rows={8}
                className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-3 text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500 resize-none font-mono text-sm"
              />
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-white mb-2">Sample Data</label>
                <div className="border-2 border-dashed border-sis-gray-600 rounded-lg p-6 text-center hover:border-sis-gray-500 transition-colors">
                  <Database className="w-8 h-8 text-sis-gray-400 mx-auto mb-2" />
                  <p className="text-sm text-sis-gray-400 mb-2">
                    Sample inputs/outputs for testing
                  </p>
                  <input
                    type="file"
                    onChange={(e) => updateForm('sampleData', e.target.files?.[0])}
                    className="hidden"
                    id="sample-file"
                  />
                  <label htmlFor="sample-file" className="btn-secondary text-sm cursor-pointer">
                    Choose File
                  </label>
                  {form.sampleData && (
                    <p className="text-sm text-green-400 mt-2">{form.sampleData.name}</p>
                  )}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-white mb-2">README</label>
                <div className="border-2 border-dashed border-sis-gray-600 rounded-lg p-6 text-center hover:border-sis-gray-500 transition-colors">
                  <FileText className="w-8 h-8 text-sis-gray-400 mx-auto mb-2" />
                  <p className="text-sm text-sis-gray-400 mb-2">
                    Markdown README file
                  </p>
                  <input
                    type="file"
                    onChange={(e) => updateForm('readme', e.target.files?.[0])}
                    className="hidden"
                    id="readme-file"
                    accept=".md,.txt"
                  />
                  <label htmlFor="readme-file" className="btn-secondary text-sm cursor-pointer">
                    Choose File
                  </label>
                  {form.readme && (
                    <p className="text-sm text-green-400 mt-2">{form.readme.name}</p>
                  )}
                </div>
              </div>
            </div>
          </div>
        );

      case 5:
        return (
          <div className="space-y-6">
            <div className="bg-sis-gray-800 p-6 rounded-lg">
              <h3 className="text-lg font-semibold text-white mb-4">Publication Summary</h3>
              
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <h4 className="text-sm font-medium text-sis-gray-300 mb-2">Basic Information</h4>
                  <div className="space-y-2 text-sm">
                    <div><span className="text-sis-gray-400">Name:</span> <span className="text-white">{form.name}</span></div>
                    <div><span className="text-sis-gray-400">Type:</span> <span className="text-white capitalize">{form.type}</span></div>
                    <div><span className="text-sis-gray-400">Category:</span> <span className="text-white capitalize">{form.category}</span></div>
                    <div><span className="text-sis-gray-400">Version:</span> <span className="text-white">{form.version}</span></div>
                    <div><span className="text-sis-gray-400">License:</span> <span className="text-white capitalize">{form.license}</span></div>
                  </div>
                </div>

                <div>
                  <h4 className="text-sm font-medium text-sis-gray-300 mb-2">Pricing</h4>
                  <div className="text-sm">
                    <div className="text-white capitalize">
                      {form.pricing.type === 'free' ? 'Free' : 
                       form.pricing.type === 'paid' ? `$${form.pricing.cost} one-time` :
                       `$${form.pricing.cost} ${form.pricing.unit}`
                      }
                    </div>
                  </div>
                </div>
              </div>

              <div className="mt-4">
                <h4 className="text-sm font-medium text-sis-gray-300 mb-2">Capabilities</h4>
                <div className="flex flex-wrap gap-2">
                  {form.capabilities.map((cap, index) => (
                    <span key={index} className="text-xs px-2 py-1 bg-sis-blue-600/20 text-sis-blue-300 rounded">
                      {cap}
                    </span>
                  ))}
                </div>
              </div>

              <div className="mt-4">
                <h4 className="text-sm font-medium text-sis-gray-300 mb-2">Tags</h4>
                <div className="flex flex-wrap gap-2">
                  {form.tags.map((tag, index) => (
                    <span key={index} className="text-xs px-2 py-1 bg-sis-gray-700 text-sis-gray-300 rounded">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            </div>

            <div className="bg-sis-yellow-900/30 border border-sis-yellow-500/30 p-4 rounded-lg">
              <div className="flex items-start space-x-3">
                <AlertTriangle className="w-5 h-5 text-sis-yellow-400 mt-0.5" />
                <div>
                  <h4 className="text-sis-yellow-300 font-medium mb-1">Review Process</h4>
                  <p className="text-sm text-sis-yellow-200">
                    Your {form.type} will be reviewed by our team before publication. This typically takes 1-3 business days. 
                    You'll be notified via email once the review is complete.
                  </p>
                </div>
              </div>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className="space-y-6">
      {/* Progress Steps */}
      <div className="flex items-center justify-between">
        {steps.map((step, index) => (
          <div key={step.id} className="flex items-center">
            <div className={`flex items-center justify-center w-8 h-8 rounded-full border-2 ${
              step.id === currentStep 
                ? 'border-sis-blue-500 bg-sis-blue-500 text-white' 
                : step.id < currentStep 
                ? 'border-green-500 bg-green-500 text-white'
                : 'border-sis-gray-600 text-sis-gray-400'
            }`}>
              {step.id < currentStep ? <CheckCircle className="w-4 h-4" /> : step.id}
            </div>
            {index < steps.length - 1 && (
              <div className={`w-12 h-0.5 ml-2 ${
                step.id < currentStep ? 'bg-green-500' : 'bg-sis-gray-600'
              }`} />
            )}
          </div>
        ))}
      </div>

      <div className="text-center">
        <h2 className="text-xl font-bold text-white">{steps[currentStep - 1].title}</h2>
        <p className="text-sis-gray-400">{steps[currentStep - 1].description}</p>
      </div>

      {/* Validation Errors */}
      {validationErrors.length > 0 && (
        <div className="bg-red-900/30 border border-red-500/30 p-4 rounded-lg">
          <div className="flex items-start space-x-3">
            <AlertTriangle className="w-5 h-5 text-red-400 mt-0.5" />
            <div>
              <h4 className="text-red-300 font-medium mb-1">Please fix the following errors:</h4>
              <ul className="text-sm text-red-200 space-y-1">
                {validationErrors.map((error, index) => (
                  <li key={index}>• {error}</li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      )}

      {/* Step Content */}
      <div className="bg-sis-gray-900 p-6 rounded-lg">
        {renderStep()}
      </div>

      {/* Navigation */}
      <div className="flex items-center justify-between">
        <button
          onClick={handlePrevious}
          disabled={currentStep === 1}
          className="btn-secondary px-6 py-2 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Previous
        </button>

        <div className="text-sm text-sis-gray-400">
          Step {currentStep} of {steps.length}
        </div>

        {currentStep < steps.length ? (
          <button
            onClick={handleNext}
            className="btn-primary px-6 py-2"
          >
            Next
          </button>
        ) : (
          <button
            onClick={handleSubmit}
            disabled={isSubmitting}
            className="btn-primary px-6 py-2 flex items-center space-x-2 disabled:opacity-50"
          >
            {isSubmitting ? (
              <>
                <div className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin" />
                <span>Submitting...</span>
              </>
            ) : (
              <>
                <Send className="w-4 h-4" />
                <span>Submit for Review</span>
              </>
            )}
          </button>
        )}
      </div>
    </div>
  );
};

export default ModelPublisher;