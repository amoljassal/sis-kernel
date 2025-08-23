/**
 * Training Operations Center
 * Main command center for AI model training operations
 */

import React, { useState, useEffect } from 'react';
import { useSelector } from 'react-redux';
import type { RootState } from '../../../store/store';
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
  AlertCircle
} from 'lucide-react';

interface TrainingSession {
  id: string;
  modelName: string;
  type: 'training' | 'fine-tuning' | 'evaluation';
  status: 'queued' | 'running' | 'completed' | 'failed' | 'paused';
  progress: number;
  startTime: string;
  estimatedCompletion?: string;
  metrics: {
    loss: number;
    accuracy: number;
    epoch: number;
    totalEpochs: number;
    learningRate: number;
  };
  resources: {
    gpu: number;
    memory: number;
    neuralEngine: number;
  };
}

interface ModelStats {
  totalModels: number;
  activeTraining: number;
  completedToday: number;
  failedToday: number;
  averageAccuracy: number;
  totalTrainingHours: number;
}

interface ComputeResource {
  type: 'gpu' | 'cpu' | 'neural-engine';
  name: string;
  usage: number;
  temperature: number;
  memory: number;
  power: number;
}

const SAMPLE_SESSIONS: TrainingSession[] = [
  {
    id: 'session-1',
    modelName: 'Legal Document Analyzer v2',
    type: 'fine-tuning',
    status: 'running',
    progress: 67,
    startTime: new Date(Date.now() - 3600000).toISOString(),
    estimatedCompletion: new Date(Date.now() + 1800000).toISOString(),
    metrics: {
      loss: 0.342,
      accuracy: 0.923,
      epoch: 67,
      totalEpochs: 100,
      learningRate: 0.0001
    },
    resources: {
      gpu: 78,
      memory: 65,
      neuralEngine: 82
    }
  },
  {
    id: 'session-2',
    modelName: 'Medical Knowledge Assistant',
    type: 'training',
    status: 'queued',
    progress: 0,
    startTime: new Date(Date.now() + 1800000).toISOString(),
    metrics: {
      loss: 0,
      accuracy: 0,
      epoch: 0,
      totalEpochs: 50,
      learningRate: 0.001
    },
    resources: {
      gpu: 0,
      memory: 0,
      neuralEngine: 0
    }
  },
  {
    id: 'session-3',
    modelName: 'Code Generation Model',
    type: 'evaluation',
    status: 'completed',
    progress: 100,
    startTime: new Date(Date.now() - 7200000).toISOString(),
    metrics: {
      loss: 0.156,
      accuracy: 0.957,
      epoch: 50,
      totalEpochs: 50,
      learningRate: 0.00005
    },
    resources: {
      gpu: 0,
      memory: 0,
      neuralEngine: 0
    }
  }
];

const COMPUTE_RESOURCES: ComputeResource[] = [
  {
    type: 'neural-engine',
    name: 'Apple Neural Engine',
    usage: 82,
    temperature: 72,
    memory: 65,
    power: 45
  },
  {
    type: 'gpu',
    name: 'M3 Max GPU',
    usage: 78,
    temperature: 68,
    memory: 71,
    power: 62
  },
  {
    type: 'cpu',
    name: 'M3 Max CPU',
    usage: 34,
    temperature: 58,
    memory: 42,
    power: 28
  }
];

export const TrainingOperationsCenter: React.FC = () => {
  const [trainingSessions, setTrainingSessions] = useState<TrainingSession[]>(SAMPLE_SESSIONS);
  const [selectedSession, setSelectedSession] = useState<string | null>(null);
  const [modelStats, setModelStats] = useState<ModelStats>({
    totalModels: 42,
    activeTraining: 1,
    completedToday: 7,
    failedToday: 1,
    averageAccuracy: 0.934,
    totalTrainingHours: 156
  });
  const [computeResources, setComputeResources] = useState<ComputeResource[]>(COMPUTE_RESOURCES);
  const [isVoiceEnabled, setIsVoiceEnabled] = useState(false);

  // Simulate real-time updates
  useEffect(() => {
    const interval = setInterval(() => {
      setTrainingSessions(prev => prev.map(session => {
        if (session.status === 'running') {
          const newProgress = Math.min(100, session.progress + Math.random() * 2);
          return {
            ...session,
            progress: newProgress,
            metrics: {
              ...session.metrics,
              epoch: Math.floor((newProgress / 100) * session.metrics.totalEpochs),
              loss: Math.max(0.1, session.metrics.loss - Math.random() * 0.01),
              accuracy: Math.min(0.99, session.metrics.accuracy + Math.random() * 0.001)
            },
            resources: {
              gpu: 70 + Math.random() * 20,
              memory: 60 + Math.random() * 20,
              neuralEngine: 75 + Math.random() * 20
            }
          };
        }
        return session;
      }));

      setComputeResources(prev => prev.map(resource => ({
        ...resource,
        usage: Math.max(0, Math.min(100, resource.usage + (Math.random() - 0.5) * 10)),
        temperature: Math.max(40, Math.min(90, resource.temperature + (Math.random() - 0.5) * 5))
      })));
    }, 2000);

    return () => clearInterval(interval);
  }, []);

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
    console.log('Starting new training session...');
  };

  const handleQuickAction = (templateId: string) => {
    console.log('Selected training template:', templateId);
    // Handle different template actions
    switch (templateId) {
      case 'quick-train':
        // Open natural language training interface
        break;
      case 'aurag-builder':
        // Navigate to AURAG builder
        break;
      case 'clone-modify':
        // Open model cloning interface
        break;
      default:
        console.log('Unknown template:', templateId);
    }
  };

  const handlePauseSession = (sessionId: string) => {
    setTrainingSessions(prev => prev.map(session => 
      session.id === sessionId 
        ? { ...session, status: 'paused' as const }
        : session
    ));
  };

  const handleStopSession = (sessionId: string) => {
    setTrainingSessions(prev => prev.map(session => 
      session.id === sessionId 
        ? { ...session, status: 'failed' as const, progress: session.progress }
        : session
    ));
  };

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="text-center py-8">
        <h1 className="text-4xl font-bold text-gradient mb-4">
          AI Training Operations Center
        </h1>
        <p className="text-sis-gray-400 text-lg max-w-2xl mx-auto">
          Centralized command center for all AI model training operations.
          Monitor, manage, and optimize your training pipeline in real-time.
        </p>
      </div>

      {/* Key Metrics Grid */}
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

      {/* Quick Actions */}
      <div className="mb-6">
        <QuickTrainingActions onActionSelect={handleQuickAction} />
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
            <button className="w-full btn-secondary text-sm py-2 flex items-center justify-center space-x-2">
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
          <div className="flex items-start space-x-3">
            <div className="w-2 h-2 bg-green-400 rounded-full mt-2" />
            <div className="flex-1">
              <div className="flex items-center justify-between">
                <span className="text-white font-medium">Legal Document Analyzer v2 - Training Started</span>
                <span className="text-sis-gray-400 text-sm">1 hour ago</span>
              </div>
              <p className="text-sis-gray-400 text-sm mt-1">
                Fine-tuning initiated with 10,000 legal documents dataset
              </p>
            </div>
          </div>

          <div className="flex items-start space-x-3">
            <div className="w-2 h-2 bg-blue-400 rounded-full mt-2" />
            <div className="flex-1">
              <div className="flex items-center justify-between">
                <span className="text-white font-medium">Code Generation Model - Evaluation Complete</span>
                <span className="text-sis-gray-400 text-sm">2 hours ago</span>
              </div>
              <p className="text-sis-gray-400 text-sm mt-1">
                Achieved 95.7% accuracy on test dataset with 0.156 loss
              </p>
            </div>
          </div>

          <div className="flex items-start space-x-3">
            <div className="w-2 h-2 bg-yellow-400 rounded-full mt-2" />
            <div className="flex-1">
              <div className="flex items-center justify-between">
                <span className="text-white font-medium">Medical Knowledge Assistant - Queued</span>
                <span className="text-sis-gray-400 text-sm">3 hours ago</span>
              </div>
              <p className="text-sis-gray-400 text-sm mt-1">
                Training scheduled for next available compute slot
              </p>
            </div>
          </div>

          <div className="flex items-start space-x-3">
            <div className="w-2 h-2 bg-purple-400 rounded-full mt-2" />
            <div className="flex-1">
              <div className="flex items-center justify-between">
                <span className="text-white font-medium">AURAG Knowledge Base Updated</span>
                <span className="text-sis-gray-400 text-sm">5 hours ago</span>
              </div>
              <p className="text-sis-gray-400 text-sm mt-1">
                Added 500 new documents to philosophical reasoning corpus
              </p>
            </div>
          </div>
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