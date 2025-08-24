/**
 * Training Operations Center
 * Main command center for AI model training operations
 */

import React, { useState } from 'react';
import { useTrainingOperations } from '../../../hooks/useTrainingOperations';
import type { TrainingParameters, TrainingSession } from '../../../services/api/trainingApi';
import QuickTrainingActions from './QuickTrainingActions';
import { 
  Brain, 
  Activity, 
  Clock, 
  Cpu, 
  Database, 
  Zap, 
  TrendingUp,
  Play,
  Pause,
  Square,
  BarChart3,
  Users,
  GitBranch,
  Download,
  Upload,
  Settings,
  AlertCircle,
  RefreshCw,
  Loader2
} from 'lucide-react';

// Types are now imported from the API service

// Sample data moved to API service - using real API integration

export const TrainingOperationsCenter: React.FC = () => {
  const {
    sessions: trainingSessions,
    modelStats,
    computeResources,
    activityEvents,
    loading,
    error,
    refreshData,
    startTraining,
    pauseTraining,
    stopTraining,
    resumeTraining,
    exportMetrics
  } = useTrainingOperations();
  
  const [selectedSession, setSelectedSession] = useState<string | null>(null);
  const [isVoiceEnabled, setIsVoiceEnabled] = useState(false);
  const [showNewTrainingForm, setShowNewTrainingForm] = useState(false);

  // Real-time updates are now handled by the useTrainingOperations hook

  const getStatusColor = (status: TrainingSession['status']) => {
    switch (status) {
      case 'running': return 'text-green-400 bg-green-900/30';
      case 'queued': return 'text-yellow-400 bg-yellow-900/30';
      case 'completed': return 'text-blue-400 bg-blue-900/30';
      case 'failed': return 'text-red-400 bg-red-900/30';
      case 'paused': return 'text-gray-400 bg-gray-900/30';
      default: return 'text-gray-400 bg-gray-900/30';
    }
  };

  const getResourceColor = (usage: number) => {
    if (usage > 80) return 'text-red-400';
    if (usage > 60) return 'text-yellow-400';
    return 'text-green-400';
  };

  const handleStartTraining = () => {
    setShowNewTrainingForm(true);
  };

  const handleQuickStart = async (templateId: string) => {
    // Quick start training with predefined parameters based on template
    const quickStartParams: Record<string, TrainingParameters> = {
      'quick-train': {
        modelName: 'Quick Training Model',
        architecture: 'transformer',
        dataset: 'sample_dataset',
        epochs: 10,
        batchSize: 16,
        learningRate: 0.001,
        optimizer: 'adam'
      },
      'aurag-builder': {
        modelName: 'AURAG Knowledge Model',
        architecture: 'rag',
        dataset: 'knowledge_base',
        epochs: 25,
        batchSize: 8,
        learningRate: 0.0005,
        optimizer: 'adamw'
      },
      'clone-modify': {
        modelName: 'Cloned Model Variant',
        architecture: 'pretrained',
        dataset: 'fine_tuning_set',
        epochs: 5,
        batchSize: 32,
        learningRate: 0.00001,
        optimizer: 'adam'
      }
    };

    const params = quickStartParams[templateId];
    if (params) {
      try {
        await startTraining(params);
      } catch (error) {
        console.error('Failed to start quick training:', error);
      }
    }
  };


  const handlePauseSession = async (sessionId: string) => {
    try {
      await pauseTraining(sessionId);
    } catch (error) {
      console.error('Failed to pause session:', error);
    }
  };

  const handleStopSession = async (sessionId: string) => {
    try {
      await stopTraining(sessionId);
    } catch (error) {
      console.error('Failed to stop session:', error);
    }
  };

  const handleExportMetrics = async (sessionId: string) => {
    try {
      await exportMetrics(sessionId, 'json');
    } catch (error) {
      console.error('Failed to export metrics:', error);
    }
  };

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="text-center py-8">
        <div className="flex items-center justify-center space-x-4 mb-4">
          <h1 className="text-4xl font-bold text-gradient">
            AI Training Operations Center
          </h1>
          <button
            onClick={refreshData}
            disabled={loading}
            className="p-2 bg-sis-gray-700 text-sis-gray-300 rounded-lg hover:bg-sis-gray-600 disabled:opacity-50 transition-colors"
            title="Refresh Data"
          >
            {loading ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : (
              <RefreshCw className="w-5 h-5" />
            )}
          </button>
        </div>
        <p className="text-sis-gray-400 text-lg max-w-2xl mx-auto">
          Centralized command center for all AI model training operations.
          Monitor, manage, and optimize your training pipeline in real-time.
        </p>
        {error && (
          <div className="mt-4 max-w-2xl mx-auto p-3 bg-red-900/30 border border-red-500/30 rounded-lg">
            <p className="text-red-300 text-sm">
              <AlertCircle className="w-4 h-4 inline mr-2" />
              {error}
            </p>
          </div>
        )}
      </div>

      {/* Key Metrics Grid */}
      {modelStats ? (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
          <div className="card p-6">
            <div className="flex items-center justify-between mb-4">
              <Brain className="w-8 h-8 text-sis-blue-400" />
              <span className="text-2xl font-bold text-white">{modelStats.totalModels}</span>
            </div>
            <h3 className="text-sis-gray-300 font-medium">Total Models</h3>
          </div>

          <div className="card p-6">
            <div className="flex items-center justify-between mb-4">
              <Activity className="w-8 h-8 text-green-400" />
              <span className="text-2xl font-bold text-green-400">{modelStats.activeTraining}</span>
            </div>
            <h3 className="text-sis-gray-300 font-medium">Active Training</h3>
          </div>

          <div className="card p-6">
            <div className="flex items-center justify-between mb-4">
              <TrendingUp className="w-8 h-8 text-blue-400" />
              <span className="text-2xl font-bold text-blue-400">{modelStats.completedToday}</span>
            </div>
            <h3 className="text-sis-gray-300 font-medium">Completed Today</h3>
          </div>

          <div className="card p-6">
            <div className="flex items-center justify-between mb-4">
              <AlertCircle className="w-8 h-8 text-red-400" />
              <span className="text-2xl font-bold text-red-400">{modelStats.failedToday}</span>
            </div>
            <h3 className="text-sis-gray-300 font-medium">Failed Today</h3>
          </div>

          <div className="card p-6">
            <div className="flex items-center justify-between mb-4">
              <BarChart3 className="w-8 h-8 text-purple-400" />
              <span className="text-2xl font-bold text-purple-400">
                {(modelStats.averageAccuracy * 100).toFixed(1)}%
              </span>
            </div>
            <h3 className="text-sis-gray-300 font-medium">Avg Accuracy</h3>
          </div>

          <div className="card p-6">
            <div className="flex items-center justify-between mb-4">
              <Clock className="w-8 h-8 text-orange-400" />
              <span className="text-2xl font-bold text-orange-400">{modelStats.totalTrainingHours}h</span>
            </div>
            <h3 className="text-sis-gray-300 font-medium">Training Hours</h3>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
          {Array.from({ length: 6 }).map((_, index) => (
            <div key={index} className="card p-6 animate-pulse">
              <div className="flex items-center justify-between mb-4">
                <div className="w-8 h-8 bg-sis-gray-600 rounded" />
                <div className="w-16 h-8 bg-sis-gray-600 rounded" />
              </div>
              <div className="w-20 h-4 bg-sis-gray-600 rounded" />
            </div>
          ))}
        </div>
      )}

      {/* Quick Actions */}
      <div className="mb-6">
        <QuickTrainingActions onActionSelect={handleQuickStart} />
      </div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Training Sessions - 2 columns */}
        <div className="lg:col-span-2 card p-6">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-xl font-bold text-white flex items-center space-x-2">
              <Zap className="w-5 h-5 text-yellow-400" />
              <span>Active Training Sessions</span>
            </h2>
            <button
              onClick={handleStartTraining}
              className="btn-primary px-4 py-2 text-sm flex items-center space-x-2"
            >
              <Play className="w-4 h-4" />
              <span>New Training</span>
            </button>
          </div>

          <div className="space-y-4">
            {trainingSessions.map(session => (
              <div
                key={session.id}
                className={`bg-sis-gray-800 rounded-lg p-4 border ${
                  selectedSession === session.id 
                    ? 'border-sis-blue-500' 
                    : 'border-sis-gray-700'
                } cursor-pointer transition-colors`}
                onClick={() => setSelectedSession(session.id)}
              >
                <div className="flex items-start justify-between mb-3">
                  <div>
                    <h3 className="text-white font-medium">{session.modelName}</h3>
                    <div className="flex items-center space-x-4 mt-1 text-sm text-sis-gray-400">
                      <span className="capitalize">{session.type}</span>
                      <span>Epoch {session.metrics.epoch}/{session.metrics.totalEpochs}</span>
                      <span>LR: {session.metrics.learningRate}</span>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(session.status)}`}>
                      {session.status.toUpperCase()}
                    </span>
                    {session.status === 'running' && (
                      <div className="flex space-x-1">
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handlePauseSession(session.id);
                          }}
                          className="p-1 text-yellow-400 hover:bg-sis-gray-700 rounded"
                        >
                          <Pause className="w-4 h-4" />
                        </button>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleStopSession(session.id);
                          }}
                          className="p-1 text-red-400 hover:bg-sis-gray-700 rounded"
                        >
                          <Square className="w-4 h-4" />
                        </button>
                      </div>
                    )}
                  </div>
                </div>

                {/* Progress Bar */}
                <div className="mb-3">
                  <div className="flex justify-between text-xs text-sis-gray-400 mb-1">
                    <span>Progress</span>
                    <span>{session.progress.toFixed(1)}%</span>
                  </div>
                  <div className="w-full bg-sis-gray-700 rounded-full h-2">
                    <div
                      className={`h-2 rounded-full transition-all duration-500 ${
                        session.status === 'running' ? 'bg-green-500 animate-pulse' :
                        session.status === 'completed' ? 'bg-blue-500' :
                        session.status === 'failed' ? 'bg-red-500' :
                        'bg-yellow-500'
                      }`}
                      style={{ width: `${session.progress}%` }}
                    />
                  </div>
                </div>

                {/* Metrics */}
                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div>
                    <span className="text-sis-gray-400">Loss:</span>
                    <span className="text-white ml-2 font-mono">{session.metrics.loss.toFixed(4)}</span>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">Accuracy:</span>
                    <span className="text-green-400 ml-2 font-mono">
                      {(session.metrics.accuracy * 100).toFixed(1)}%
                    </span>
                  </div>
                  {session.estimatedCompletion && session.status === 'running' && (
                    <div>
                      <span className="text-sis-gray-400">ETA:</span>
                      <span className="text-white ml-2 font-mono">
                        {new Date(session.estimatedCompletion).toLocaleTimeString()}
                      </span>
                    </div>
                  )}
                </div>

                {/* Resource Usage */}
                {session.status === 'running' && (
                  <div className="mt-3 pt-3 border-t border-sis-gray-700 flex items-center space-x-4 text-xs">
                    <div className="flex items-center space-x-1">
                      <Cpu className="w-3 h-3 text-sis-gray-400" />
                      <span className={getResourceColor(session.resources.gpu)}>
                        GPU: {session.resources.gpu.toFixed(0)}%
                      </span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <Database className="w-3 h-3 text-sis-gray-400" />
                      <span className={getResourceColor(session.resources.memory)}>
                        MEM: {session.resources.memory.toFixed(0)}%
                      </span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <Brain className="w-3 h-3 text-sis-gray-400" />
                      <span className={getResourceColor(session.resources.neuralEngine)}>
                        ANE: {session.resources.neuralEngine.toFixed(0)}%
                      </span>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Resource Monitor - 1 column */}
        <div className="card p-6">
          <h2 className="text-xl font-bold text-white mb-6 flex items-center space-x-2">
            <Cpu className="w-5 h-5 text-blue-400" />
            <span>Compute Resources</span>
          </h2>

          <div className="space-y-4">
            {computeResources.map((resource, index) => (
              <div key={index} className="bg-sis-gray-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-white font-medium text-sm">{resource.name}</h3>
                  <span className={`text-sm font-mono ${getResourceColor(resource.usage)}`}>
                    {resource.usage.toFixed(0)}%
                  </span>
                </div>

                {/* Usage Bar */}
                <div className="w-full bg-sis-gray-700 rounded-full h-2 mb-3">
                  <div
                    className={`h-2 rounded-full transition-all duration-500 ${
                      resource.usage > 80 ? 'bg-red-500' :
                      resource.usage > 60 ? 'bg-yellow-500' :
                      'bg-green-500'
                    }`}
                    style={{ width: `${resource.usage}%` }}
                  />
                </div>

                {/* Detailed Stats */}
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <span className="text-sis-gray-400">Temp:</span>
                    <span className={`ml-1 ${
                      resource.temperature > 75 ? 'text-red-400' : 'text-white'
                    }`}>
                      {resource.temperature}°C
                    </span>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">Memory:</span>
                    <span className="text-white ml-1">{resource.memory}%</span>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">Power:</span>
                    <span className="text-white ml-1">{resource.power}W</span>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">Type:</span>
                    <span className="text-white ml-1 capitalize">{resource.type}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Quick Actions */}
          <div className="mt-6 space-y-3">
            <button className="w-full btn-secondary text-sm py-2 flex items-center justify-center space-x-2">
              <GitBranch className="w-4 h-4" />
              <span>Distributed Training</span>
            </button>
            <button 
              onClick={() => selectedSession && handleExportMetrics(selectedSession)}
              disabled={!selectedSession}
              className="w-full btn-secondary text-sm py-2 flex items-center justify-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Download className="w-4 h-4" />
              <span>Export Metrics</span>
            </button>
            <button 
              onClick={() => setIsVoiceEnabled(!isVoiceEnabled)}
              className={`w-full text-sm py-2 flex items-center justify-center space-x-2 rounded-lg transition-colors ${
                isVoiceEnabled 
                  ? 'bg-purple-600 text-white hover:bg-purple-700' 
                  : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
              }`}
            >
              <Activity className="w-4 h-4" />
              <span>Voice Commands {isVoiceEnabled ? 'ON' : 'OFF'}</span>
            </button>
          </div>
        </div>
      </div>

      {/* Recent Activity Timeline */}
      <div className="card p-6">
        <h2 className="text-xl font-bold text-white mb-6 flex items-center space-x-2">
          <Clock className="w-5 h-5 text-purple-400" />
          <span>Training Timeline</span>
        </h2>

        <div className="space-y-3">
          {activityEvents.length > 0 ? (
            activityEvents.slice(0, 10).map((event) => {
              const getEventColor = (type: string) => {
                switch (type) {
                  case 'training_started': return 'bg-green-400';
                  case 'training_completed': return 'bg-blue-400';
                  case 'training_failed': return 'bg-red-400';
                  case 'model_queued': return 'bg-yellow-400';
                  case 'dataset_updated': return 'bg-purple-400';
                  default: return 'bg-gray-400';
                }
              };

              const formatTimeAgo = (timestamp: string) => {
                const now = new Date();
                const eventTime = new Date(timestamp);
                const diffMs = now.getTime() - eventTime.getTime();
                const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
                const diffMinutes = Math.floor(diffMs / (1000 * 60));
                
                if (diffHours > 0) {
                  return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
                } else if (diffMinutes > 0) {
                  return `${diffMinutes} minute${diffMinutes > 1 ? 's' : ''} ago`;
                } else {
                  return 'Just now';
                }
              };

              return (
                <div key={event.id} className="flex items-start space-x-3">
                  <div className={`w-2 h-2 ${getEventColor(event.type)} rounded-full mt-2`} />
                  <div className="flex-1">
                    <div className="flex items-center justify-between">
                      <span className="text-white font-medium">{event.modelName} - {event.type.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}</span>
                      <span className="text-sis-gray-400 text-sm">{formatTimeAgo(event.timestamp)}</span>
                    </div>
                    <p className="text-sis-gray-400 text-sm mt-1">
                      {event.message}
                    </p>
                    {event.details && (
                      <div className="mt-1 text-xs text-sis-gray-500">
                        {Object.entries(event.details).map(([key, value]) => (
                          <span key={key} className="mr-3">
                            {key}: {String(value)}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              );
            })
          ) : (
            <div className="text-center py-8 text-sis-gray-500">
              <Clock className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p>No recent training activity</p>
              <p className="text-sm mt-1">Start a training session to see activity updates here</p>
            </div>
          )}
        </div>
      </div>

      {/* Voice Command Info */}
      {isVoiceEnabled && (
        <div className="bg-purple-900/30 border border-purple-500/30 p-4 rounded-lg">
          <h3 className="font-medium text-purple-300 mb-2">Voice Commands Active</h3>
          <p className="text-purple-200 text-sm mb-3">
            You can now control training operations with voice commands.
          </p>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
            <div className="bg-purple-800/30 p-2 rounded">
              <span className="text-purple-300">Start training [model name]</span>
            </div>
            <div className="bg-purple-800/30 p-2 rounded">
              <span className="text-purple-300">Show performance metrics</span>
            </div>
            <div className="bg-purple-800/30 p-2 rounded">
              <span className="text-purple-300">Pause current training</span>
            </div>
            <div className="bg-purple-800/30 p-2 rounded">
              <span className="text-purple-300">Export results</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default TrainingOperationsCenter;