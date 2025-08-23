/**
 * AI Ethics Panel
 * Transforms hardware safety panel into AI ethics and bias detection
 */

import React, { useState, useEffect } from 'react';
import { Shield, AlertTriangle, CheckCircle, XCircle, Eye, Scale } from 'lucide-react';

interface EthicsCheck {
  id: string;
  name: string;
  description: string;
  status: 'passed' | 'warning' | 'failed' | 'pending';
  score?: number;
  details?: string[];
}

interface BiasMetric {
  category: string;
  score: number;
  threshold: number;
  status: 'safe' | 'warning' | 'critical';
}

interface AIEthicsPanelProps {
  className?: string;
}

export const AIEthicsPanel: React.FC<AIEthicsPanelProps> = ({ className = '' }) => {
  const [activeTab, setActiveTab] = useState<'overview' | 'bias' | 'privacy' | 'fairness'>('overview');
  const [ethicsChecks, setEthicsChecks] = useState<EthicsCheck[]>([
    {
      id: 'bias_detection',
      name: 'Bias Detection',
      description: 'Scans model outputs for demographic bias patterns',
      status: 'passed',
      score: 0.15,
      details: ['Gender bias: 0.12', 'Age bias: 0.08', 'Racial bias: 0.21']
    },
    {
      id: 'data_quality',
      name: 'Data Quality',
      description: 'Validates training data integrity and representativeness',
      status: 'warning',
      score: 0.73,
      details: ['Missing demographic groups', 'Potential data leakage detected', 'Label quality: 89%']
    },
    {
      id: 'privacy_compliance',
      name: 'Privacy Compliance',
      description: 'Checks for PII leakage and privacy violations',
      status: 'passed',
      score: 0.95,
      details: ['No PII detected in outputs', 'GDPR compliant', 'Anonymization verified']
    },
    {
      id: 'adversarial_robustness',
      name: 'Adversarial Robustness',
      description: 'Tests model resistance to adversarial attacks',
      status: 'warning',
      score: 0.68,
      details: ['Vulnerable to gradient attacks', 'Text perturbation resilience: 72%']
    },
    {
      id: 'transparency',
      name: 'Transparency',
      description: 'Evaluates model interpretability and explainability',
      status: 'failed',
      score: 0.34,
      details: ['Limited feature attribution', 'No attention visualization', 'Decision path unclear']
    }
  ]);

  const [biasMetrics, setBiasMetrics] = useState<BiasMetric[]>([
    { category: 'Gender', score: 0.12, threshold: 0.20, status: 'safe' },
    { category: 'Age', score: 0.08, threshold: 0.15, status: 'safe' },
    { category: 'Race/Ethnicity', score: 0.23, threshold: 0.20, status: 'warning' },
    { category: 'Religion', score: 0.05, threshold: 0.15, status: 'safe' },
    { category: 'Socioeconomic', score: 0.31, threshold: 0.25, status: 'critical' },
    { category: 'Geographic', score: 0.18, threshold: 0.20, status: 'safe' }
  ]);

  const [isScanning, setIsScanning] = useState(false);

  const runEthicsScan = async () => {
    setIsScanning(true);
    
    // Simulate ethics scanning process
    setTimeout(() => {
      setEthicsChecks(prev => prev.map(check => ({
        ...check,
        status: Math.random() > 0.3 ? 'passed' : Math.random() > 0.5 ? 'warning' : 'failed',
        score: Math.random()
      })));
      setIsScanning(false);
    }, 3000);
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'passed': return <CheckCircle className="w-4 h-4 text-green-400" />;
      case 'warning': return <AlertTriangle className="w-4 h-4 text-yellow-400" />;
      case 'failed': return <XCircle className="w-4 h-4 text-red-400" />;
      default: return <div className="w-4 h-4 rounded-full bg-sis-gray-600 animate-pulse" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'passed': return 'text-green-400';
      case 'warning': return 'text-yellow-400';
      case 'failed': return 'text-red-400';
      default: return 'text-sis-gray-400';
    }
  };

  const getBiasStatusColor = (status: string) => {
    switch (status) {
      case 'safe': return 'bg-green-500';
      case 'warning': return 'bg-yellow-500';
      case 'critical': return 'bg-red-500';
      default: return 'bg-sis-gray-500';
    }
  };

  const overallScore = ethicsChecks.reduce((acc, check) => acc + (check.score || 0), 0) / ethicsChecks.length;
  const overallStatus = overallScore > 0.8 ? 'safe' : overallScore > 0.6 ? 'warning' : 'critical';

  return (
    <div className={`bg-sis-gray-900 flex flex-col ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-sis-gray-700">
        <div className="flex items-center space-x-2 mb-3">
          <Scale className="w-5 h-5 text-sis-blue-400" />
          <h2 className="text-lg font-semibold text-white">AI Ethics & Safety</h2>
        </div>
        
        {/* Overall Score */}
        <div className={`text-2xl font-bold mb-2 ${
          overallStatus === 'safe' ? 'text-green-400' : 
          overallStatus === 'warning' ? 'text-yellow-400' : 'text-red-400'
        }`}>
          {Math.round(overallScore * 100)}/100
        </div>
        
        <button
          onClick={runEthicsScan}
          disabled={isScanning}
          className="w-full btn-primary text-sm py-2 disabled:bg-sis-gray-600"
        >
          {isScanning ? 'Scanning...' : 'Run Ethics Scan'}
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-sis-gray-700">
        {[
          { id: 'overview', name: 'Overview', icon: Shield },
          { id: 'bias', name: 'Bias', icon: Scale },
          { id: 'privacy', name: 'Privacy', icon: Eye },
          { id: 'fairness', name: 'Fairness', icon: CheckCircle }
        ].map(tab => {
          const IconComponent = tab.icon;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`flex-1 px-3 py-2 text-xs font-medium transition-colors ${
                activeTab === tab.id
                  ? 'bg-sis-gray-800 text-white border-b-2 border-sis-blue-500'
                  : 'text-sis-gray-400 hover:text-white hover:bg-sis-gray-800'
              }`}
            >
              <div className="flex items-center justify-center space-x-1">
                <IconComponent className="w-3 h-3" />
                <span className="hidden sm:inline">{tab.name}</span>
              </div>
            </button>
          );
        })}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {activeTab === 'overview' && (
          <div className="space-y-4">
            <div className="text-sm text-sis-gray-400 mb-4">
              Comprehensive ethics and safety validation for AI models
            </div>
            
            {ethicsChecks.map(check => (
              <div key={check.id} className="bg-sis-gray-800 rounded-lg p-3">
                <div className="flex items-start justify-between mb-2">
                  <div className="flex items-center space-x-2">
                    {getStatusIcon(check.status)}
                    <span className="text-white font-medium text-sm">{check.name}</span>
                  </div>
                  {check.score && (
                    <span className={`text-sm font-mono ${getStatusColor(check.status)}`}>
                      {Math.round(check.score * 100)}%
                    </span>
                  )}
                </div>
                
                <p className="text-xs text-sis-gray-400 mb-2">{check.description}</p>
                
                {check.details && (
                  <div className="space-y-1">
                    {check.details.map((detail, index) => (
                      <div key={index} className="text-xs text-sis-gray-500">
                        • {detail}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}

        {activeTab === 'bias' && (
          <div className="space-y-4">
            <div className="text-sm text-sis-gray-400 mb-4">
              Bias detection across demographic categories
            </div>
            
            {biasMetrics.map(metric => (
              <div key={metric.category} className="bg-sis-gray-800 rounded-lg p-3">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-white font-medium text-sm">{metric.category}</span>
                  <span className={`text-xs px-2 py-1 rounded-full ${
                    metric.status === 'safe' ? 'bg-green-900 text-green-300' :
                    metric.status === 'warning' ? 'bg-yellow-900 text-yellow-300' :
                    'bg-red-900 text-red-300'
                  }`}>
                    {metric.status.toUpperCase()}
                  </span>
                </div>
                
                <div className="flex items-center space-x-3 mb-2">
                  <div className="flex-1 bg-sis-gray-700 rounded-full h-2">
                    <div
                      className={`h-2 rounded-full ${getBiasStatusColor(metric.status)}`}
                      style={{ width: `${(metric.score / 0.5) * 100}%` }}
                    />
                  </div>
                  <span className="text-xs text-sis-gray-400 font-mono">
                    {metric.score.toFixed(3)}
                  </span>
                </div>
                
                <div className="text-xs text-sis-gray-500">
                  Threshold: {metric.threshold.toFixed(3)} 
                  {metric.score > metric.threshold && (
                    <span className="text-red-400 ml-2">EXCEEDS LIMIT</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}

        {activeTab === 'privacy' && (
          <div className="space-y-4">
            <div className="text-sm text-sis-gray-400 mb-4">
              Privacy and data protection compliance
            </div>
            
            <div className="grid gap-4">
              <div className="bg-sis-gray-800 rounded-lg p-3">
                <div className="flex items-center space-x-2 mb-2">
                  <CheckCircle className="w-4 h-4 text-green-400" />
                  <span className="text-white font-medium text-sm">GDPR Compliance</span>
                </div>
                <p className="text-xs text-sis-gray-400">
                  Model meets GDPR requirements for data processing and user rights
                </p>
              </div>
              
              <div className="bg-sis-gray-800 rounded-lg p-3">
                <div className="flex items-center space-x-2 mb-2">
                  <CheckCircle className="w-4 h-4 text-green-400" />
                  <span className="text-white font-medium text-sm">PII Detection</span>
                </div>
                <p className="text-xs text-sis-gray-400">
                  No personally identifiable information detected in model outputs
                </p>
              </div>
              
              <div className="bg-sis-gray-800 rounded-lg p-3">
                <div className="flex items-center space-x-2 mb-2">
                  <AlertTriangle className="w-4 h-4 text-yellow-400" />
                  <span className="text-white font-medium text-sm">Data Retention</span>
                </div>
                <p className="text-xs text-sis-gray-400">
                  Training data retention policy needs review
                </p>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'fairness' && (
          <div className="space-y-4">
            <div className="text-sm text-sis-gray-400 mb-4">
              Fairness metrics and demographic parity analysis
            </div>
            
            <div className="bg-sis-gray-800 rounded-lg p-3">
              <h3 className="text-white font-medium text-sm mb-3">Demographic Parity</h3>
              <div className="space-y-2">
                <div className="flex justify-between text-xs">
                  <span className="text-sis-gray-400">Overall Parity Score:</span>
                  <span className="text-white font-mono">0.72</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-sis-gray-400">Equalized Odds:</span>
                  <span className="text-yellow-400 font-mono">0.68</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-sis-gray-400">Calibration:</span>
                  <span className="text-green-400 font-mono">0.84</span>
                </div>
              </div>
            </div>
            
            <div className="bg-sis-gray-800 rounded-lg p-3">
              <h3 className="text-white font-medium text-sm mb-3">Recommendations</h3>
              <ul className="space-y-1 text-xs text-sis-gray-400">
                <li>• Increase representation in training data</li>
                <li>• Apply fairness constraints during training</li>
                <li>• Implement post-processing bias mitigation</li>
                <li>• Regular fairness auditing schedule</li>
              </ul>
            </div>
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="p-4 border-t border-sis-gray-700">
        <div className="text-xs text-sis-gray-500">
          Last scan: {new Date().toLocaleTimeString()}
        </div>
        <div className="text-xs text-sis-gray-400 mt-1">
          Ethics validation powered by SIS AURAG
        </div>
      </div>
    </div>
  );
};

export default AIEthicsPanel;