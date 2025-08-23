import React, { useState } from 'react';
import { Shield, FileText, Settings } from 'lucide-react';
import ModelPerformanceAnalyzer from '../../components/AILab/Validator/ModelPerformanceAnalyzer';
import ValidationReports from '../../components/AILab/Validator/ValidationReports';

type ValidatorMode = 'analyzer' | 'reports';

const Validator: React.FC = () => {
  const [mode, setMode] = useState<ValidatorMode>('analyzer');

  return (
    <div className="min-h-screen bg-sis-gray-950 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Mode Toggle */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center space-x-6">
            <div className="flex items-center space-x-2">
              <Shield className="w-6 h-6 text-sis-blue-400" />
              <h1 className="text-2xl font-bold text-white">Model Validator</h1>
            </div>
            
            <div className="flex items-center bg-sis-gray-800 rounded-lg p-1">
              <button
                onClick={() => setMode('analyzer')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  mode === 'analyzer'
                    ? 'bg-sis-blue-600 text-white'
                    : 'text-sis-gray-300 hover:text-white'
                }`}
              >
                Performance Analyzer
              </button>
              <button
                onClick={() => setMode('reports')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  mode === 'reports'
                    ? 'bg-sis-blue-600 text-white'
                    : 'text-sis-gray-300 hover:text-white'
                }`}
              >
                Validation Reports
              </button>
            </div>
          </div>
          
          <button className="p-2 bg-sis-gray-800 text-sis-gray-300 rounded-lg hover:bg-sis-gray-700 transition-colors">
            <Settings className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        {mode === 'analyzer' ? (
          <ModelPerformanceAnalyzer />
        ) : (
          <ValidationReports />
        )}
      </div>
    </div>
  );
};

export default Validator;