/**
 * Simple MLX Training Interface Component
 * Browser-compatible demo version
 */

import React, { useState } from 'react';
import { Play, Square, Upload, Settings, TrendingUp } from 'lucide-react';

interface TrainingSession {
  id: string;
  description: string;
  status: 'running' | 'completed' | 'failed' | 'paused';
  progress?: {
    epoch: number;
    loss: number;
    accuracy?: number;
    eta?: string;
  };
  startTime: number;
}

export const TrainingInterface: React.FC = () => {
  const [description, setDescription] = useState('');
  const [datasetPath, setDatasetPath] = useState('./sample_dataset.txt');
  const [sessions, setSessions] = useState<TrainingSession[]>([]);
  const [isTraining, setIsTraining] = useState(false);

  const handleStartTraining = async () => {
    if (!description.trim()) return;

    const newSession: TrainingSession = {
      id: `training_${Date.now()}`,
      description,
      status: 'running',
      startTime: Date.now()
    };

    setSessions(prev => [newSession, ...prev]);
    setIsTraining(true);

    // Simulate training progress
    let epoch = 0;
    const totalEpochs = 10;
    
    const progressInterval = setInterval(() => {
      epoch++;
      const loss = Math.max(0.1, 2.0 - (epoch / totalEpochs) * 1.8 + Math.random() * 0.2);
      const accuracy = Math.min(0.95, (epoch / totalEpochs) * 0.85 + Math.random() * 0.1);
      
      setSessions(prev => prev.map(session => 
        session.id === newSession.id 
          ? {
              ...session,
              progress: {
                epoch,
                loss,
                accuracy,
                eta: epoch < totalEpochs ? `${(totalEpochs - epoch) * 30}s` : undefined
              },
              status: epoch >= totalEpochs ? 'completed' : 'running'
            }
          : session
      ));

      if (epoch >= totalEpochs) {
        clearInterval(progressInterval);
        setIsTraining(false);
      }
    }, 3000);

    setDescription('');
  };

  return (
    <div className="max-w-6xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="glass rounded-lg p-6">
        <h1 className="text-3xl font-bold text-white mb-2">
          SIS MLX Training Lab
        </h1>
        <p className="text-sis-gray-300">
          Apple Silicon optimized AI model training with natural language interface
        </p>
        
        <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div className="bg-green-500/20 p-3 rounded border border-green-500/30">
            <div className="font-medium text-green-400">Status</div>
            <div className="text-green-300">Ready</div>
          </div>
          <div className="bg-sis-blue-500/20 p-3 rounded border border-sis-blue-500/30">
            <div className="font-medium text-sis-blue-400">Platform</div>
            <div className="text-sis-blue-300">Apple Silicon</div>
          </div>
          <div className="bg-purple-500/20 p-3 rounded border border-purple-500/30">
            <div className="font-medium text-purple-400">Framework</div>
            <div className="text-purple-300">MLX</div>
          </div>
          <div className="bg-orange-500/20 p-3 rounded border border-orange-500/30">
            <div className="font-medium text-orange-400">Mode</div>
            <div className="text-orange-300">Demo</div>
          </div>
        </div>
      </div>

      {/* Training Interface */}
      <div className="glass rounded-lg p-6">
        <h2 className="text-xl font-semibold text-white mb-4">
          Natural Language Training
        </h2>
        
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-sis-gray-300 mb-1">
              Training Description
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Describe what you want to train... e.g., 'Train a text classifier for sentiment analysis with 5 epochs and learning rate 0.001'"
              rows={3}
              className="w-full px-3 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-md text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-sis-gray-300 mb-1">
              Dataset Path
            </label>
            <input
              type="text"
              value={datasetPath}
              onChange={(e) => setDatasetPath(e.target.value)}
              className="w-full px-3 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-md text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
            />
          </div>
          
          <div className="flex space-x-3">
            <button
              onClick={handleStartTraining}
              disabled={isTraining || !description.trim()}
              className="btn-primary disabled:bg-sis-gray-600 disabled:cursor-not-allowed flex items-center space-x-2"
            >
              <Play className="w-4 h-4" />
              <span>{isTraining ? 'Training...' : 'Start Training'}</span>
            </button>
            
            <button className="btn-secondary flex items-center space-x-2">
              <Upload className="w-4 h-4" />
              <span>Upload Dataset</span>
            </button>
            
            <button className="btn-secondary flex items-center space-x-2">
              <Settings className="w-4 h-4" />
              <span>Advanced Settings</span>
            </button>
          </div>
        </div>
      </div>

      {/* Training Sessions */}
      {sessions.length > 0 && (
        <div className="glass rounded-lg p-6">
          <h2 className="text-xl font-semibold text-white mb-4 flex items-center space-x-2">
            <TrendingUp className="w-5 h-5" />
            <span>Training Sessions</span>
          </h2>
          
          <div className="space-y-4">
            {sessions.map((session) => (
              <div key={session.id} className="bg-sis-gray-800/50 p-4 rounded-md">
                <div className="flex items-center justify-between mb-2">
                  <h3 className="font-medium text-white">{session.description}</h3>
                  <span className={`px-2 py-1 rounded text-xs font-medium ${
                    session.status === 'running' ? 'bg-yellow-600 text-yellow-100' :
                    session.status === 'completed' ? 'bg-green-600 text-green-100' :
                    session.status === 'failed' ? 'bg-red-600 text-red-100' :
                    'bg-sis-gray-600 text-sis-gray-100'
                  }`}>
                    {session.status.toUpperCase()}
                  </span>
                </div>
                
                {session.progress && (
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div>
                      <span className="text-sis-gray-400">Epoch:</span>
                      <span className="ml-1 font-medium text-white">{session.progress.epoch}/10</span>
                    </div>
                    <div>
                      <span className="text-sis-gray-400">Loss:</span>
                      <span className="ml-1 font-medium text-white">{session.progress.loss.toFixed(4)}</span>
                    </div>
                    <div>
                      <span className="text-sis-gray-400">Accuracy:</span>
                      <span className="ml-1 font-medium text-white">
                        {session.progress.accuracy ? (session.progress.accuracy * 100).toFixed(1) + '%' : 'N/A'}
                      </span>
                    </div>
                    <div>
                      <span className="text-sis-gray-400">ETA:</span>
                      <span className="ml-1 font-medium text-white">{session.progress.eta || 'Complete'}</span>
                    </div>
                  </div>
                )}
                
                {session.status === 'running' && (
                  <div className="mt-2">
                    <div className="w-full bg-sis-gray-700 rounded-full h-2">
                      <div 
                        className="bg-sis-blue-500 h-2 rounded-full transition-all duration-500"
                        style={{ width: `${session.progress ? (session.progress.epoch / 10) * 100 : 0}%` }}
                      />
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Info Panel */}
      <div className="bg-sis-blue-900/30 border border-sis-blue-500/30 p-4 rounded-md">
        <h3 className="font-medium text-sis-blue-300 mb-2">Demo Mode</h3>
        <p className="text-sis-blue-200 text-sm">
          This is a demonstration interface. In production, this would connect to real MLX training pipelines 
          on Apple Silicon hardware with actual Python script generation and model training.
        </p>
      </div>
    </div>
  );
};

export default TrainingInterface;