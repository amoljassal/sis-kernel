import React, { useState } from 'react';
import { Store, Upload, Settings } from 'lucide-react';
import ModelBrowser from '../../components/AILab/Marketplace/ModelBrowser';
import ModelPublisher from '../../components/AILab/Marketplace/ModelPublisher';

type MarketplaceMode = 'browse' | 'publish';

const Marketplace: React.FC = () => {
  const [mode, setMode] = useState<MarketplaceMode>('browse');

  return (
    <div className="min-h-screen bg-sis-gray-950 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Mode Toggle */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center space-x-6">
            <div className="flex items-center space-x-2">
              <Store className="w-6 h-6 text-sis-blue-400" />
              <h1 className="text-2xl font-bold text-white">AI Model & Dataset Marketplace</h1>
            </div>
            
            <div className="flex items-center bg-sis-gray-800 rounded-lg p-1">
              <button
                onClick={() => setMode('browse')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  mode === 'browse'
                    ? 'bg-sis-blue-600 text-white'
                    : 'text-sis-gray-300 hover:text-white'
                }`}
              >
                Browse Marketplace
              </button>
              <button
                onClick={() => setMode('publish')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  mode === 'publish'
                    ? 'bg-sis-blue-600 text-white'
                    : 'text-sis-gray-300 hover:text-white'
                }`}
              >
                Publish Model
              </button>
            </div>
          </div>
          
          <button className="p-2 bg-sis-gray-800 text-sis-gray-300 rounded-lg hover:bg-sis-gray-700 transition-colors">
            <Settings className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        {mode === 'browse' ? (
          <ModelBrowser />
        ) : (
          <ModelPublisher />
        )}
      </div>
    </div>
  );
};

export default Marketplace;