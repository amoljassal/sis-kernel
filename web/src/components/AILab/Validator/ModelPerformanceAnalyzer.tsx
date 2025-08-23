/**
 * Model Performance Analyzer
 * Comprehensive validation and performance analysis for AI models
 */

import React, { useState, useEffect, useRef } from 'react';
import {
  Activity,
  Target,
  Zap,
  CheckCircle,
  AlertTriangle,
  XCircle,
  BarChart3,
  TrendingUp,
  Clock,
  Database,
  Cpu,
  Monitor,
  Settings,
  Download,
  Play,
  Pause,
  RotateCcw
} from 'lucide-react';

interface PerformanceMetrics {
  accuracy: number;
  precision: number;
  recall: number;
  f1Score: number;
  latency: number;
  throughput: number;
  memoryUsage: number;
  energyEfficiency: number;
}

interface ValidationTest {
  id: string;
  name: string;
  description: string;
  status: 'pending' | 'running' | 'passed' | 'failed' | 'warning';
  duration: number;
  category: 'accuracy' | 'performance' | 'robustness' | 'bias' | 'security';
  score?: number;
  details?: string;
}

interface ModelInfo {
  name: string;
  version: string;
  architecture: string;
  parameters: string;
  datasetSize: string;
  trainingTime: string;
  lastValidated: string;
}

const SAMPLE_MODEL: ModelInfo = {
  name: 'Legal Document Analyzer',
  version: 'v2.1.3',
  architecture: 'BERT-Large + AURAG',
  parameters: '340M',
  datasetSize: '2.4M documents',
  trainingTime: '14.2 hours',
  lastValidated: '2 minutes ago'
};

const VALIDATION_TESTS: ValidationTest[] = [
  {
    id: 'accuracy-test',
    name: 'Accuracy Validation',
    description: 'Test model accuracy against benchmark dataset',
    status: 'passed',
    duration: 45,
    category: 'accuracy',
    score: 94.7,
    details: 'Exceeds target accuracy of 90%'
  },
  {
    id: 'latency-test',
    name: 'Inference Latency',
    description: 'Measure response time for single inference',
    status: 'passed',
    duration: 12,
    category: 'performance',
    score: 89.2,
    details: '< 50ms average response time'
  },
  {
    id: 'bias-test',
    name: 'Bias Detection',
    description: 'Evaluate model for demographic and cultural bias',
    status: 'warning',
    duration: 78,
    category: 'bias',
    score: 76.3,
    details: 'Minor bias detected in gender-related content'
  },
  {
    id: 'adversarial-test',
    name: 'Adversarial Robustness',
    description: 'Test resilience against adversarial attacks',
    status: 'running',
    duration: 0,
    category: 'robustness'
  },
  {
    id: 'security-test',
    name: 'Security Validation',
    description: 'Check for potential security vulnerabilities',
    status: 'pending',
    duration: 0,
    category: 'security'
  },
  {
    id: 'memory-test',
    name: 'Memory Efficiency',
    description: 'Analyze memory usage patterns during inference',
    status: 'passed',
    duration: 23,
    category: 'performance',
    score: 91.8,
    details: 'Optimal memory utilization'
  }
];

export const ModelPerformanceAnalyzer: React.FC = () => {
  const [validationTests, setValidationTests] = useState<ValidationTest[]>(VALIDATION_TESTS);
  const [selectedCategory, setSelectedCategory] = useState<'all' | 'accuracy' | 'performance' | 'robustness' | 'bias' | 'security'>('all');
  const [isRunning, setIsRunning] = useState(false);
  const [metrics, setMetrics] = useState<PerformanceMetrics>({
    accuracy: 94.7,
    precision: 92.1,
    recall: 96.8,
    f1Score: 94.4,
    latency: 42,
    throughput: 1240,
    memoryUsage: 2.3,
    energyEfficiency: 87.2
  });

  // Simulate real-time test updates
  useEffect(() => {
    const interval = setInterval(() => {
      setValidationTests(prev => prev.map(test => {
        if (test.status === 'running' && Math.random() < 0.1) {
          const passed = Math.random() > 0.2;
          return {
            ...test,
            status: passed ? 'passed' : 'failed',
            duration: test.duration + Math.floor(Math.random() * 30),
            score: passed ? 80 + Math.random() * 20 : 60 + Math.random() * 20,
            details: passed ? 'Test completed successfully' : 'Test failed - needs attention'
          };
        }
        if (test.status === 'running') {
          return { ...test, duration: test.duration + 1 };
        }
        return test;
      }));
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  const filteredTests = selectedCategory === 'all' 
    ? validationTests 
    : validationTests.filter(test => test.category === selectedCategory);

  const getStatusIcon = (status: ValidationTest['status']) => {
    switch (status) {
      case 'passed': return <CheckCircle className="w-5 h-5 text-green-400" />;
      case 'warning': return <AlertTriangle className="w-5 h-5 text-yellow-400" />;
      case 'failed': return <XCircle className="w-5 h-5 text-red-400" />;
      case 'running': return <Activity className="w-5 h-5 text-blue-400 animate-pulse" />;
      default: return <Clock className="w-5 h-5 text-gray-400" />;
    }
  };

  const getStatusColor = (status: ValidationTest['status']) => {
    switch (status) {
      case 'passed': return 'bg-green-900/30 border-green-500/30';
      case 'warning': return 'bg-yellow-900/30 border-yellow-500/30';
      case 'failed': return 'bg-red-900/30 border-red-500/30';
      case 'running': return 'bg-blue-900/30 border-blue-500/30';
      default: return 'bg-gray-900/30 border-gray-500/30';
    }
  };

  const runAllTests = () => {
    setIsRunning(true);
    setValidationTests(prev => prev.map(test => ({
      ...test,
      status: test.status === 'failed' ? 'running' : test.status,
      duration: test.status === 'failed' ? 0 : test.duration
    })));
    
    setTimeout(() => setIsRunning(false), 5000);
  };

  const overallScore = validationTests
    .filter(test => test.score !== undefined)
    .reduce((sum, test) => sum + (test.score || 0), 0) / 
    validationTests.filter(test => test.score !== undefined).length;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white mb-2">Model Performance Analyzer</h1>
          <p className="text-sis-gray-400">Comprehensive validation and performance analysis</p>
        </div>
        
        <div className="flex items-center space-x-3">
          <button
            onClick={runAllTests}
            disabled={isRunning}
            className="btn-primary px-4 py-2 flex items-center space-x-2 disabled:opacity-50"
          >
            {isRunning ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
            <span>{isRunning ? 'Running...' : 'Run All Tests'}</span>
          </button>
          
          <button className="p-2 bg-sis-gray-700 text-sis-gray-300 rounded-lg hover:bg-sis-gray-600 transition-colors">
            <Download className="w-5 h-5" />
          </button>
          
          <button className="p-2 bg-sis-gray-700 text-sis-gray-300 rounded-lg hover:bg-sis-gray-600 transition-colors">
            <Settings className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Model Overview */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 card p-6">
          <h3 className="text-lg font-semibold text-white mb-4 flex items-center space-x-2">
            <Target className="w-5 h-5 text-sis-blue-400" />
            <span>Model Information</span>
          </h3>
          
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="text-sm text-sis-gray-400">Model Name</label>
              <p className="text-white font-medium">{SAMPLE_MODEL.name}</p>
            </div>
            <div>
              <label className="text-sm text-sis-gray-400">Version</label>
              <p className="text-white font-medium">{SAMPLE_MODEL.version}</p>
            </div>
            <div>
              <label className="text-sm text-sis-gray-400">Architecture</label>
              <p className="text-white font-medium">{SAMPLE_MODEL.architecture}</p>
            </div>
            <div>
              <label className="text-sm text-sis-gray-400">Parameters</label>
              <p className="text-white font-medium">{SAMPLE_MODEL.parameters}</p>
            </div>
            <div>
              <label className="text-sm text-sis-gray-400">Dataset Size</label>
              <p className="text-white font-medium">{SAMPLE_MODEL.datasetSize}</p>
            </div>
            <div>
              <label className="text-sm text-sis-gray-400">Last Validated</label>
              <p className="text-white font-medium">{SAMPLE_MODEL.lastValidated}</p>
            </div>
          </div>
        </div>

        <div className="card p-6">
          <h3 className="text-lg font-semibold text-white mb-4 flex items-center space-x-2">
            <BarChart3 className="w-5 h-5 text-green-400" />
            <span>Overall Score</span>
          </h3>
          
          <div className="text-center">
            <div className="text-4xl font-bold text-white mb-2">
              {overallScore.toFixed(1)}
            </div>
            <div className="text-sm text-sis-gray-400 mb-4">Out of 100</div>
            
            <div className="w-full bg-sis-gray-700 rounded-full h-3">
              <div 
                className="h-3 rounded-full bg-gradient-to-r from-green-500 to-blue-500"
                style={{ width: `${overallScore}%` }}
              ></div>
            </div>
          </div>
        </div>
      </div>

      {/* Performance Metrics */}
      <div className="card p-6">
        <h3 className="text-lg font-semibold text-white mb-4 flex items-center space-x-2">
          <TrendingUp className="w-5 h-5 text-purple-400" />
          <span>Performance Metrics</span>
        </h3>
        
        <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
          <div className="text-center">
            <div className="text-2xl font-bold text-white mb-1">{metrics.accuracy}%</div>
            <div className="text-sm text-sis-gray-400">Accuracy</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-white mb-1">{metrics.f1Score}%</div>
            <div className="text-sm text-sis-gray-400">F1 Score</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-white mb-1">{metrics.latency}ms</div>
            <div className="text-sm text-sis-gray-400">Latency</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-white mb-1">{metrics.throughput}/s</div>
            <div className="text-sm text-sis-gray-400">Throughput</div>
          </div>
        </div>
      </div>

      {/* Category Filter */}
      <div className="flex items-center space-x-2">
        {['all', 'accuracy', 'performance', 'robustness', 'bias', 'security'].map(category => (
          <button
            key={category}
            onClick={() => setSelectedCategory(category as any)}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              selectedCategory === category
                ? 'bg-sis-blue-600 text-white'
                : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
            }`}
          >
            {category.charAt(0).toUpperCase() + category.slice(1)}
          </button>
        ))}
      </div>

      {/* Validation Tests */}
      <div className="space-y-4">
        {filteredTests.map(test => (
          <div key={test.id} className={`card p-6 border ${getStatusColor(test.status)}`}>
            <div className="flex items-start justify-between">
              <div className="flex items-start space-x-4">
                <div className="flex-shrink-0 mt-1">
                  {getStatusIcon(test.status)}
                </div>
                
                <div className="flex-1">
                  <div className="flex items-center space-x-3 mb-2">
                    <h4 className="text-white font-medium">{test.name}</h4>
                    {test.score && (
                      <span className="px-2 py-1 bg-sis-blue-600 text-white text-xs rounded-full">
                        {test.score.toFixed(1)}/100
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-sis-gray-400 mb-2">{test.description}</p>
                  {test.details && (
                    <p className="text-xs text-sis-gray-500">{test.details}</p>
                  )}
                </div>
              </div>
              
              <div className="text-right">
                <div className="text-sm text-sis-gray-400">
                  {test.status === 'running' ? `${test.duration}s` : `${test.duration}s`}
                </div>
                <div className="text-xs text-sis-gray-500 capitalize mt-1">
                  {test.category}
                </div>
              </div>
            </div>
            
            {test.status === 'running' && (
              <div className="mt-4">
                <div className="w-full bg-sis-gray-700 rounded-full h-2">
                  <div className="h-2 bg-blue-500 rounded-full animate-pulse" style={{width: '45%'}}></div>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default ModelPerformanceAnalyzer;