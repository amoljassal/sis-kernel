/**
 * MLX Training Interface Component
 * Natural language interface for training AI models on Apple Silicon
 */

import React, { useState, useEffect, useRef } from 'react';
import { Play, Square, Upload, FileText, Settings, TrendingUp, Clock, CheckCircle, AlertCircle } from 'lucide-react';
import { createBrowserMLXTrainingPipeline } from '../../services/mlx/browser-training-interface';
import { createNaturalLanguageInterface, ParsedTrainingRequest } from '../../services/mlx/natural-language-interface';
import type { TrainingProgress, TrainingResult } from '../../services/mlx/browser-training-interface';

interface TrainingSession {
  id: string;
  description: string;
  status: 'running' | 'completed' | 'failed' | 'paused';
  progress?: TrainingProgress;
  result?: TrainingResult;
  startTime: number;
}

export const TrainingInterface: React.FC = () => {
  const [description, setDescription] = useState('');
  const [datasetPath, setDatasetPath] = useState('');
  const [parsedRequest, setParsedRequest] = useState<ParsedTrainingRequest | null>(null);
  const [activeSessions, setActiveSessions] = useState<TrainingSession[]>([]);
  const [isTraining, setIsTraining] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);

  const mlxPipeline = useRef(createBrowserMLXTrainingPipeline());
  const nlInterface = useRef(createNaturalLanguageInterface(mlxPipeline.current));
  const progressInterval = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    initializeMLX();
    return () => {
      if (progressInterval.current) {
        clearInterval(progressInterval.current);
      }
    };
  }, []);

  const initializeMLX = async () => {
    try {
      await mlxPipeline.current.initialize();
      console.log('MLX Training Pipeline initialized');
    } catch (error) {
      console.error('Failed to initialize MLX:', error);
    }
  };

  const handleDescriptionChange = (value: string) => {
    setDescription(value);
    
    if (value.length > 10) {
      try {
        const parsed = nlInterface.current.parseTrainingRequest(value);
        setParsedRequest(parsed);
      } catch (error) {
        setParsedRequest(null);
      }
    } else {
      setParsedRequest(null);
    }
  };

  const startTraining = async () => {
    if (!description || !datasetPath) {
      alert('Please provide both description and dataset path');
      return;
    }

    try {
      setIsTraining(true);
      const { trainingId, parsedRequest: request } = await nlInterface.current.trainFromDescription(
        description,
        datasetPath
      );

      const newSession: TrainingSession = {
        id: trainingId,
        description,
        status: 'running',
        startTime: Date.now()
      };

      setActiveSessions(prev => [...prev, newSession]);
      
      // Start monitoring progress
      startProgressMonitoring(trainingId);
      
      // Clear form
      setDescription('');
      setDatasetPath('');
      setParsedRequest(null);
    } catch (error) {
      alert(`Failed to start training: ${error}`);
    } finally {
      setIsTraining(false);
    }
  };

  const startProgressMonitoring = (trainingId: string) => {
    if (progressInterval.current) {
      clearInterval(progressInterval.current);
    }

    progressInterval.current = setInterval(async () => {
      try {
        const progress = await mlxPipeline.current.getTrainingProgress(trainingId);
        const result = await mlxPipeline.current.getTrainingResult(trainingId);

        setActiveSessions(prev => prev.map(session => {
          if (session.id === trainingId) {
            const updated = { ...session };
            
            if (progress) {
              updated.progress = progress;
              updated.status = progress.status;
            }
            
            if (result) {
              updated.result = result;
              updated.status = result.success ? 'completed' : 'failed';
            }
            
            return updated;
          }
          return session;
        }));

        // Stop monitoring if training is complete
        if (result) {
          clearInterval(progressInterval.current!);
        }
      } catch (error) {
        console.error('Failed to fetch training progress:', error);
      }
    }, 2000);
  };

  const stopTraining = async (trainingId: string) => {
    try {
      await mlxPipeline.current.stopTraining(trainingId);
      setActiveSessions(prev => prev.map(session =>
        session.id === trainingId 
          ? { ...session, status: 'paused' as const }
          : session
      ));
    } catch (error) {
      console.error('Failed to stop training:', error);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running':
        return <TrendingUp className="h-4 w-4 text-blue-400 animate-pulse" />;
      case 'completed':
        return <CheckCircle className="h-4 w-4 text-green-400" />;
      case 'failed':
        return <AlertCircle className="h-4 w-4 text-red-400" />;
      case 'paused':
        return <Clock className="h-4 w-4 text-yellow-400" />;
      default:
        return <Clock className="h-4 w-4 text-gray-400" />;
    }
  };

  const formatDuration = (startTime: number) => {
    const duration = Date.now() - startTime;
    const minutes = Math.floor(duration / 60000);
    const seconds = Math.floor((duration % 60000) / 1000);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  const exampleDescriptions = nlInterface.current.getExampleDescriptions();

  return (
    <div className="max-w-6xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="text-center">
        <h1 className="text-3xl font-bold text-white mb-2">MLX Training Laboratory</h1>
        <p className="text-gray-400">Train AI models with natural language on Apple Silicon</p>
      </div>

      {/* Training Form */}
      <div className="card p-6">
        <h2 className="text-xl font-semibold text-white mb-4">Start New Training</h2>
        
        <div className="space-y-4">
          {/* Training Description */}
          <div>
            <label className="label">Describe what you want to train</label>
            <textarea
              value={description}
              onChange={(e) => handleDescriptionChange(e.target.value)}
              placeholder="E.g., 'Fine-tune a GPT model for code generation with 20 epochs and learning rate 0.0001'"
              className="input h-24 resize-none"
              disabled={isTraining}
            />
          </div>

          {/* Dataset Path */}
          <div>
            <label className="label">Dataset Path</label>
            <div className="flex space-x-2">
              <input
                type="text"
                value={datasetPath}
                onChange={(e) => setDatasetPath(e.target.value)}
                placeholder="/path/to/your/dataset.json"
                className="input flex-1"
                disabled={isTraining}
              />
              <button className="btn-secondary px-3">
                <Upload className="h-4 w-4" />
              </button>
            </div>
          </div>

          {/* Parsed Request Display */}
          {parsedRequest && (
            <div className="bg-gray-800 rounded-lg p-4 border border-gray-600">
              <h3 className="text-sm font-medium text-white mb-2">Parsed Training Intent</h3>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                <div>
                  <span className="text-gray-400">Type:</span>
                  <span className="ml-2 text-blue-400 capitalize">{parsedRequest.intent.type}</span>
                </div>
                <div>
                  <span className="text-gray-400">Domain:</span>
                  <span className="ml-2 text-green-400 capitalize">{parsedRequest.intent.domain}</span>
                </div>
                <div>
                  <span className="text-gray-400">Confidence:</span>
                  <span className="ml-2 text-yellow-400">{Math.round(parsedRequest.intent.confidence * 100)}%</span>
                </div>
              </div>
              
              {parsedRequest.suggestions.length > 0 && (
                <div className="mt-3">
                  <span className="text-gray-400 text-xs">Suggestions:</span>
                  <ul className="text-xs text-gray-300 mt-1 space-y-1">
                    {parsedRequest.suggestions.slice(0, 3).map((suggestion, idx) => (
                      <li key={idx} className="flex items-start">
                        <span className="text-blue-400 mr-1">•</span>
                        {suggestion}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
              
              {parsedRequest.warnings.length > 0 && (
                <div className="mt-3">
                  <span className="text-red-400 text-xs">Warnings:</span>
                  <ul className="text-xs text-red-300 mt-1 space-y-1">
                    {parsedRequest.warnings.slice(0, 2).map((warning, idx) => (
                      <li key={idx} className="flex items-start">
                        <span className="text-red-400 mr-1">!</span>
                        {warning}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          {/* Training Controls */}
          <div className="flex justify-between items-center">
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="btn-secondary"
            >
              <Settings className="h-4 w-4 mr-2" />
              Advanced Options
            </button>
            
            <button
              onClick={startTraining}
              disabled={isTraining || !description || !datasetPath}
              className="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isTraining ? (
                <>
                  <TrendingUp className="h-4 w-4 mr-2 animate-pulse" />
                  Starting Training...
                </>
              ) : (
                <>
                  <Play className="h-4 w-4 mr-2" />
                  Start Training
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Example Descriptions */}
      <div className="card p-6">
        <h3 className="text-lg font-semibold text-white mb-3">Example Training Descriptions</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {exampleDescriptions.map((example, idx) => (
            <button
              key={idx}
              onClick={() => setDescription(example)}
              className="text-left p-3 bg-gray-800 hover:bg-gray-700 rounded-lg border border-gray-600 transition-colors"
              disabled={isTraining}
            >
              <div className="text-sm text-gray-300">{example}</div>
            </button>
          ))}
        </div>
      </div>

      {/* Active Training Sessions */}
      {activeSessions.length > 0 && (
        <div className="card p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Active Training Sessions</h3>
          <div className="space-y-4">
            {activeSessions.map((session) => (
              <div key={session.id} className="bg-gray-800 rounded-lg p-4 border border-gray-600">
                <div className="flex justify-between items-start mb-3">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2 mb-1">
                      {getStatusIcon(session.status)}
                      <span className="text-white font-medium">{session.id}</span>
                      <span className="text-xs text-gray-400">
                        {formatDuration(session.startTime)}
                      </span>
                    </div>
                    <p className="text-sm text-gray-300">{session.description}</p>
                  </div>
                  
                  {session.status === 'running' && (
                    <button
                      onClick={() => stopTraining(session.id)}
                      className="btn-danger px-3 py-1 text-xs"
                    >
                      <Square className="h-3 w-3 mr-1" />
                      Stop
                    </button>
                  )}
                </div>
                
                {session.progress && (
                  <div className="space-y-2">
                    <div className="flex justify-between text-xs text-gray-400">
                      <span>Epoch {session.progress.epoch}</span>
                      <span>Loss: {session.progress.loss.toFixed(4)}</span>
                    </div>
                    {session.progress.eta && (
                      <div className="text-xs text-gray-500">ETA: {session.progress.eta}</div>
                    )}
                  </div>
                )}
                
                {session.result && (
                  <div className="mt-3 p-3 bg-gray-900 rounded border border-gray-700">
                    <div className="text-xs text-gray-400">
                      {session.result.success ? (
                        <div className="text-green-400">
                          Training completed successfully
                          {session.result.modelPath && (
                            <div className="mt-1">Model saved: {session.result.modelPath}</div>
                          )}
                        </div>
                      ) : (
                        <div className="text-red-400">
                          Training failed: {session.result.error}
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default TrainingInterface;