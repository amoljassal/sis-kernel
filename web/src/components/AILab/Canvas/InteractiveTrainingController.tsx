/**
 * Interactive Training Controller
 * Real-time training parameter adjustment with live feedback
 */

import React, { useState, useEffect } from 'react';
import {
  Play,
  Pause,
  Square,
  Settings,
  Zap,
  TrendingUp,
  Clock,
  Target,
  Sliders,
  RefreshCw,
  Save
} from 'lucide-react';

interface TrainingParameters {
  learningRate: number;
  batchSize: number;
  epochs: number;
  optimizer: 'adam' | 'sgd' | 'adamw';
  schedulerType: 'cosine' | 'linear' | 'exponential';
  warmupSteps: number;
  weightDecay: number;
  gradientClipping: number;
}

interface TrainingMetrics {
  epoch: number;
  loss: number;
  accuracy: number;
  learningRate: number;
  timeElapsed: number;
  estimatedTimeRemaining: number;
  gpuUtilization: number;
  memoryUsage: number;
}

interface TrainingControllerProps {
  modelName?: string;
  initialParams?: Partial<TrainingParameters>;
  onParameterChange?: (params: TrainingParameters) => void;
  onTrainingStart?: () => void;
  onTrainingStop?: () => void;
  onTrainingPause?: () => void;
  isTraining?: boolean;
  metrics?: TrainingMetrics;
}

const DEFAULT_PARAMS: TrainingParameters = {
  learningRate: 0.001,
  batchSize: 32,
  epochs: 10,
  optimizer: 'adamw',
  schedulerType: 'cosine',
  warmupSteps: 100,
  weightDecay: 0.01,
  gradientClipping: 1.0
};

export const InteractiveTrainingController: React.FC<TrainingControllerProps> = ({
  modelName,
  initialParams = DEFAULT_PARAMS,
  onParameterChange,
  onTrainingStart,
  onTrainingStop,
  onTrainingPause,
  isTraining,
  metrics
}) => {
  const [params, setParams] = useState<TrainingParameters>(() => ({
    ...DEFAULT_PARAMS,
    ...(initialParams || {})
  }));
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [presets, setPresets] = useState<Record<string, TrainingParameters>>({
    fast: { ...DEFAULT_PARAMS, learningRate: 0.01, epochs: 5, batchSize: 64 },
    balanced: { ...DEFAULT_PARAMS },
    precise: { ...DEFAULT_PARAMS, learningRate: 0.0001, epochs: 20, batchSize: 16 }
  });

  useEffect(() => {
    onParameterChange?.(params);
  }, [params, onParameterChange]);

  const updateParam = <K extends keyof TrainingParameters>(
    key: K,
    value: TrainingParameters[K]
  ) => {
    setParams(prev => ({ ...prev, [key]: value }));
  };

  const applyPreset = (presetName: string) => {
    if (presets[presetName]) {
      setParams(presets[presetName]);
    }
  };

  const formatTime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const getOptimizationSuggestions = () => {
    const suggestions = [];
    
    if (params.learningRate > 0.01) {
      suggestions.push({
        type: 'warning',
        message: 'Learning rate might be too high for stable training',
        suggestion: 'Try reducing to 0.001-0.01'
      });
    }
    
    if (params.batchSize > 128) {
      suggestions.push({
        type: 'info',
        message: 'Large batch size may require learning rate adjustment',
        suggestion: 'Consider scaling learning rate with batch size'
      });
    }
    
    if (params.epochs > 50) {
      suggestions.push({
        type: 'warning',
        message: 'High epoch count may lead to overfitting',
        suggestion: 'Monitor validation loss carefully'
      });
    }

    return suggestions;
  };

  const suggestions = getOptimizationSuggestions();

  return (
    <div className="space-y-6">
      {/* Training Control Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-white mb-1">Training Controller</h3>
          <p className="text-sm text-sis-gray-400">Model: {modelName}</p>
        </div>
        
        <div className="flex items-center space-x-3">
          {!isTraining ? (
            <button
              onClick={() => onTrainingStart && onTrainingStart()}
              className="btn-primary px-6 py-2 flex items-center space-x-2"
            >
              <Play className="w-4 h-4" />
              <span>Start Training</span>
            </button>
          ) : (
            <>
              <button
                onClick={() => onTrainingPause && onTrainingPause()}
                className="btn-secondary px-4 py-2 flex items-center space-x-2"
              >
                <Pause className="w-4 h-4" />
                <span>Pause</span>
              </button>
              <button
                onClick={() => onTrainingStop && onTrainingStop()}
                className="btn-danger px-4 py-2 flex items-center space-x-2"
              >
                <Square className="w-4 h-4" />
                <span>Stop</span>
              </button>
            </>
          )}
        </div>
      </div>

      {/* Training Metrics (when training) */}
      {isTraining && metrics && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="card p-4 text-center">
            <div className="text-2xl font-bold text-white mb-1">
              {metrics.epoch}/{params.epochs}
            </div>
            <div className="text-sm text-sis-gray-400">Epochs</div>
          </div>
          
          <div className="card p-4 text-center">
            <div className="text-2xl font-bold text-white mb-1">
              {metrics.loss.toFixed(4)}
            </div>
            <div className="text-sm text-sis-gray-400">Loss</div>
          </div>
          
          <div className="card p-4 text-center">
            <div className="text-2xl font-bold text-white mb-1">
              {(metrics.accuracy * 100).toFixed(1)}%
            </div>
            <div className="text-sm text-sis-gray-400">Accuracy</div>
          </div>
          
          <div className="card p-4 text-center">
            <div className="text-2xl font-bold text-white mb-1">
              {formatTime(metrics.estimatedTimeRemaining)}
            </div>
            <div className="text-sm text-sis-gray-400">Time Left</div>
          </div>
        </div>
      )}

      {/* Training Presets */}
      <div className="card p-4">
        <h4 className="text-white font-medium mb-3 flex items-center space-x-2">
          <Zap className="w-4 h-4 text-sis-blue-400" />
          <span>Training Presets</span>
        </h4>
        
        <div className="grid grid-cols-3 gap-3">
          {Object.entries(presets).map(([presetName, presetParams]) => (
            <button
              key={presetName}
              onClick={() => applyPreset(presetName)}
              disabled={isTraining}
              className="p-3 bg-sis-gray-800 rounded-lg hover:bg-sis-gray-700 transition-colors disabled:opacity-50 text-left"
            >
              <div className="text-white font-medium capitalize mb-1">{presetName}</div>
              <div className="text-xs text-sis-gray-400">
                LR: {presetParams.learningRate}, Epochs: {presetParams.epochs}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Basic Parameters */}
      <div className="card p-6">
        <div className="flex items-center justify-between mb-4">
          <h4 className="text-white font-medium flex items-center space-x-2">
            <Sliders className="w-4 h-4 text-sis-green-400" />
            <span>Training Parameters</span>
          </h4>
          
          <button
            onClick={() => setShowAdvanced(!showAdvanced)}
            className="text-sm text-sis-blue-400 hover:text-sis-blue-300 transition-colors"
          >
            {showAdvanced ? 'Hide Advanced' : 'Show Advanced'}
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Learning Rate */}
          <div>
            <label className="block text-sm font-medium text-sis-gray-300 mb-2">
              Learning Rate: {params.learningRate}
            </label>
            <input
              type="range"
              min="0.0001"
              max="0.1"
              step="0.0001"
              value={params.learningRate}
              onChange={(e) => updateParam('learningRate', parseFloat(e.target.value))}
              disabled={isTraining}
              className="w-full h-2 bg-sis-gray-700 rounded-lg appearance-none cursor-pointer slider"
            />
            <div className="flex justify-between text-xs text-sis-gray-500 mt-1">
              <span>0.0001</span>
              <span>Conservative</span>
              <span>Aggressive</span>
              <span>0.1</span>
            </div>
          </div>

          {/* Batch Size */}
          <div>
            <label className="block text-sm font-medium text-sis-gray-300 mb-2">
              Batch Size: {params.batchSize}
            </label>
            <input
              type="range"
              min="8"
              max="256"
              step="8"
              value={params.batchSize}
              onChange={(e) => updateParam('batchSize', parseInt(e.target.value))}
              disabled={isTraining}
              className="w-full h-2 bg-sis-gray-700 rounded-lg appearance-none cursor-pointer slider"
            />
            <div className="flex justify-between text-xs text-sis-gray-500 mt-1">
              <span>8</span>
              <span>Small</span>
              <span>Large</span>
              <span>256</span>
            </div>
          </div>

          {/* Epochs */}
          <div>
            <label className="block text-sm font-medium text-sis-gray-300 mb-2">
              Epochs: {params.epochs}
            </label>
            <input
              type="range"
              min="1"
              max="100"
              step="1"
              value={params.epochs}
              onChange={(e) => updateParam('epochs', parseInt(e.target.value))}
              disabled={isTraining}
              className="w-full h-2 bg-sis-gray-700 rounded-lg appearance-none cursor-pointer slider"
            />
            <div className="flex justify-between text-xs text-sis-gray-500 mt-1">
              <span>1</span>
              <span>Quick</span>
              <span>Thorough</span>
              <span>100</span>
            </div>
          </div>

          {/* Optimizer */}
          <div>
            <label className="block text-sm font-medium text-sis-gray-300 mb-2">
              Optimizer
            </label>
            <select
              value={params.optimizer}
              onChange={(e) => updateParam('optimizer', e.target.value as typeof params.optimizer)}
              disabled={isTraining}
              className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
            >
              <option value="adamw">AdamW (Recommended)</option>
              <option value="adam">Adam</option>
              <option value="sgd">SGD</option>
            </select>
          </div>
        </div>

        {/* Advanced Parameters */}
        {showAdvanced && (
          <div className="mt-6 pt-6 border-t border-sis-gray-700">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* Scheduler Type */}
              <div>
                <label className="block text-sm font-medium text-sis-gray-300 mb-2">
                  Learning Rate Scheduler
                </label>
                <select
                  value={params.schedulerType}
                  onChange={(e) => updateParam('schedulerType', e.target.value as typeof params.schedulerType)}
                  disabled={isTraining}
                  className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                >
                  <option value="cosine">Cosine Annealing</option>
                  <option value="linear">Linear</option>
                  <option value="exponential">Exponential</option>
                </select>
              </div>

              {/* Warmup Steps */}
              <div>
                <label className="block text-sm font-medium text-sis-gray-300 mb-2">
                  Warmup Steps: {params.warmupSteps}
                </label>
                <input
                  type="range"
                  min="0"
                  max="1000"
                  step="10"
                  value={params.warmupSteps}
                  onChange={(e) => updateParam('warmupSteps', parseInt(e.target.value))}
                  disabled={isTraining}
                  className="w-full h-2 bg-sis-gray-700 rounded-lg appearance-none cursor-pointer slider"
                />
              </div>

              {/* Weight Decay */}
              <div>
                <label className="block text-sm font-medium text-sis-gray-300 mb-2">
                  Weight Decay: {params.weightDecay}
                </label>
                <input
                  type="range"
                  min="0"
                  max="0.1"
                  step="0.001"
                  value={params.weightDecay}
                  onChange={(e) => updateParam('weightDecay', parseFloat(e.target.value))}
                  disabled={isTraining}
                  className="w-full h-2 bg-sis-gray-700 rounded-lg appearance-none cursor-pointer slider"
                />
              </div>

              {/* Gradient Clipping */}
              <div>
                <label className="block text-sm font-medium text-sis-gray-300 mb-2">
                  Gradient Clipping: {params.gradientClipping}
                </label>
                <input
                  type="range"
                  min="0.1"
                  max="5.0"
                  step="0.1"
                  value={params.gradientClipping}
                  onChange={(e) => updateParam('gradientClipping', parseFloat(e.target.value))}
                  disabled={isTraining}
                  className="w-full h-2 bg-sis-gray-700 rounded-lg appearance-none cursor-pointer slider"
                />
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Optimization Suggestions */}
      {suggestions.length > 0 && (
        <div className="card p-4">
          <h4 className="text-white font-medium mb-3 flex items-center space-x-2">
            <Target className="w-4 h-4 text-sis-orange-400" />
            <span>Optimization Suggestions</span>
          </h4>
          
          <div className="space-y-3">
            {suggestions.map((suggestion, index) => (
              <div
                key={index}
                className={`p-3 rounded-lg border ${
                  suggestion.type === 'warning' 
                    ? 'bg-yellow-900/30 border-yellow-500/30' 
                    : 'bg-blue-900/30 border-blue-500/30'
                }`}
              >
                <div className="text-sm">
                  <div className="text-white font-medium mb-1">{suggestion.message}</div>
                  <div className="text-sis-gray-300">{suggestion.suggestion}</div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Progress Visualization (when training) */}
      {isTraining && metrics && (
        <div className="card p-6">
          <h4 className="text-white font-medium mb-4 flex items-center space-x-2">
            <TrendingUp className="w-4 h-4 text-sis-purple-400" />
            <span>Training Progress</span>
          </h4>
          
          <div className="space-y-4">
            {/* Overall Progress */}
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="text-sis-gray-300">Overall Progress</span>
                <span className="text-white">{((metrics.epoch / params.epochs) * 100).toFixed(1)}%</span>
              </div>
              <div className="w-full bg-sis-gray-700 rounded-full h-3">
                <div 
                  className="h-3 bg-gradient-to-r from-sis-blue-500 to-sis-green-500 rounded-full transition-all duration-300"
                  style={{ width: `${(metrics.epoch / params.epochs) * 100}%` }}
                />
              </div>
            </div>

            {/* Resource Utilization */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-sis-gray-300">GPU Utilization</span>
                  <span className="text-white">{metrics.gpuUtilization}%</span>
                </div>
                <div className="w-full bg-sis-gray-700 rounded-full h-2">
                  <div 
                    className="h-2 bg-sis-blue-500 rounded-full transition-all"
                    style={{ width: `${metrics.gpuUtilization}%` }}
                  />
                </div>
              </div>
              
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-sis-gray-300">Memory Usage</span>
                  <span className="text-white">{metrics.memoryUsage}%</span>
                </div>
                <div className="w-full bg-sis-gray-700 rounded-full h-2">
                  <div 
                    className="h-2 bg-sis-green-500 rounded-full transition-all"
                    style={{ width: `${metrics.memoryUsage}%` }}
                  />
                </div>
              </div>
            </div>

            {/* Training Stats */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <div className="text-sis-gray-400">Current LR</div>
                <div className="text-white font-mono">{metrics.learningRate.toExponential(2)}</div>
              </div>
              <div>
                <div className="text-sis-gray-400">Time Elapsed</div>
                <div className="text-white font-mono">{formatTime(metrics.timeElapsed)}</div>
              </div>
              <div>
                <div className="text-sis-gray-400">Steps/Sec</div>
                <div className="text-white font-mono">{(60 / (metrics.timeElapsed / metrics.epoch || 1)).toFixed(1)}</div>
              </div>
              <div>
                <div className="text-sis-gray-400">ETA</div>
                <div className="text-white font-mono">{formatTime(metrics.estimatedTimeRemaining)}</div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Save Configuration */}
      <div className="flex justify-end">
        <button
          onClick={() => {
            // Save current configuration as a preset
            const presetName = prompt('Enter preset name:');
            if (presetName) {
              setPresets(prev => ({ ...prev, [presetName]: { ...params } }));
            }
          }}
          className="btn-secondary px-4 py-2 flex items-center space-x-2"
        >
          <Save className="w-4 h-4" />
          <span>Save as Preset</span>
        </button>
      </div>
    </div>
  );
};

export default InteractiveTrainingController;