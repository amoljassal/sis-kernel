import React, { useState } from 'react';
import { Server, Activity, Settings } from 'lucide-react';
import TrainingResourceManager from '../../components/AILab/Infrastructure/TrainingResourceManager';
import InfrastructureMonitor from '../../components/AILab/Infrastructure/InfrastructureMonitor';

type InfrastructureMode = 'resources' | 'monitor';

const Hardware: React.FC = () => {
  const [mode, setMode] = useState<InfrastructureMode>('resources');

  return (
    <div className="min-h-screen bg-sis-gray-950 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Mode Toggle */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center space-x-6">
            <div className="flex items-center space-x-2">
              <Server className="w-6 h-6 text-sis-blue-400" />
              <h1 className="text-2xl font-bold text-white">Training Infrastructure</h1>
            </div>
            
            <div className="flex items-center bg-sis-gray-800 rounded-lg p-1">
              <button
                onClick={() => setMode('resources')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  mode === 'resources'
                    ? 'bg-sis-blue-600 text-white'
                    : 'text-sis-gray-300 hover:text-white'
                }`}
              >
                Resource Manager
              </button>
              <button
                onClick={() => setMode('monitor')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  mode === 'monitor'
                    ? 'bg-sis-blue-600 text-white'
                    : 'text-sis-gray-300 hover:text-white'
                }`}
              >
                Infrastructure Monitor
              </button>
            </div>
          </div>
          
          <button className="p-2 bg-sis-gray-800 text-sis-gray-300 rounded-lg hover:bg-sis-gray-700 transition-colors">
            <Settings className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        {mode === 'resources' ? (
          <TrainingResourceManager />
        ) : (
          <InfrastructureMonitor />
        )}
      </div>
    </div>
  );
};

export default Hardware;