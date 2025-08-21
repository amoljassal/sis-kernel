// Auto-Scaling Monitoring Dashboard
// Real-time visualization of predictive scaling for Indian peak hours

import React, { useState, useEffect } from 'react';
import { predictiveAutoScaling } from '../services/predictive-autoscaling';
import type { PredictionResult } from '../services/predictive-autoscaling';
import {
  XMarkIcon
} from '@heroicons/react/24/outline';

interface AutoScalingDashboardProps {
  isVisible: boolean;
  onClose: () => void;
}

export const AutoScalingDashboard: React.FC<AutoScalingDashboardProps> = ({
  isVisible,
  onClose
}) => {
  const [scalingStatus, setScalingStatus] = useState<any>(null);
  const [predictions, setPredictions] = useState<PredictionResult[]>([]);
  const [historicalAnalysis, setHistoricalAnalysis] = useState<any>(null);
  const [activeTab, setActiveTab] = useState<'current' | 'predictions' | 'history'>('current');

  useEffect(() => {
    if (!isVisible) return;

    const updateData = () => {
      setScalingStatus(predictiveAutoScaling.getScalingStatus());
      setPredictions(predictiveAutoScaling.getPredictionsForNext24Hours());
      setHistoricalAnalysis(predictiveAutoScaling.getHistoricalAnalysis());
    };

    updateData();
    const interval = setInterval(updateData, 5000);

    // Listen for scaling events
    const handleScalingComplete = (event: any) => {
      console.log('Scaling completed:', event);
      updateData();
    };

    predictiveAutoScaling.on('scaling-complete', handleScalingComplete);

    return () => {
      clearInterval(interval);
      predictiveAutoScaling.off('scaling-complete', handleScalingComplete);
    };
  }, [isVisible]);

  if (!isVisible) return null;

  const formatTime = (date: Date): string => {
    return new Date(date).toLocaleTimeString('en-IN', {
      hour: '2-digit',
      minute: '2-digit',
      timeZone: 'Asia/Kolkata'
    });
  };

  const formatDateTime = (date: Date): string => {
    return new Date(date).toLocaleString('en-IN', {
      timeZone: 'Asia/Kolkata',
      day: '2-digit',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  const getLoadColor = (load: number): string => {
    if (load < 3000) return 'text-green-600';
    if (load < 7000) return 'text-yellow-600';
    if (load < 15000) return 'text-orange-600';
    return 'text-red-600';
  };

  const getResourceColor = (current: number, max: number): string => {
    const usage = (current / max) * 100;
    if (usage < 50) return 'bg-green-500';
    if (usage < 75) return 'bg-yellow-500';
    if (usage < 90) return 'bg-orange-500';
    return 'bg-red-500';
  };

  const ResourceBar: React.FC<{
    label: string;
    current: number;
    max: number;
    min: number;
  }> = ({ label, current, max, min }) => {
    const percentage = ((current - min) / (max - min)) * 100;
    
    return (
      <div className="mb-4">
        <div className="flex justify-between mb-1">
          <span className="text-sm font-medium">{label}</span>
          <span className="text-sm">
            {current}/{max} instances
          </span>
        </div>
        <div className="w-full bg-gray-200 rounded-full h-2.5">
          <div
            className={`h-2.5 rounded-full ${getResourceColor(current, max)}`}
            style={{ width: `${percentage}%` }}
          />
        </div>
      </div>
    );
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-11/12 h-5/6 max-w-7xl overflow-hidden">
        {/* Header */}
        <div className="bg-gradient-to-r from-purple-600 to-indigo-600 text-white p-4 flex justify-between items-center">
          <div>
            <h2 className="text-2xl font-bold">Predictive Auto-Scaling Dashboard</h2>
            <p className="text-sm opacity-90">Indian Peak Hours Optimization (9 AM - 11 PM IST)</p>
          </div>
          <button
            onClick={onClose}
            className="text-white hover:bg-white hover:bg-opacity-20 rounded-full p-2"
          >
            <XMarkIcon className="w-6 h-6" />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="border-b border-gray-200">
          <nav className="flex space-x-8 px-6 py-3">
            {['current', 'predictions', 'history'].map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab as any)}
                className={`pb-2 px-1 border-b-2 font-medium text-sm capitalize ${
                  activeTab === tab
                    ? 'border-purple-600 text-purple-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                }`}
              >
                {tab === 'current' ? 'Current Status' : tab}
              </button>
            ))}
          </nav>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto" style={{ height: 'calc(100% - 180px)' }}>
          {activeTab === 'current' && scalingStatus && (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Current Resources */}
              <div className="bg-white rounded-lg shadow p-6">
                <h3 className="text-lg font-semibold mb-4">Current Resources</h3>
                <ResourceBar
                  label="Web Servers"
                  current={scalingStatus.currentResources.webServers}
                  max={50}
                  min={3}
                />
                <ResourceBar
                  label="Database Replicas"
                  current={scalingStatus.currentResources.databaseReplicas}
                  max={10}
                  min={2}
                />
                <ResourceBar
                  label="Redis Nodes"
                  current={scalingStatus.currentResources.redisNodes}
                  max={20}
                  min={3}
                />
                <ResourceBar
                  label="WebSocket Gateways"
                  current={scalingStatus.currentResources.websocketGateways}
                  max={15}
                  min={2}
                />
                <ResourceBar
                  label="Kafka Brokers"
                  current={scalingStatus.currentResources.kafkaBrokers}
                  max={12}
                  min={3}
                />
              </div>

              {/* Scaling Status */}
              <div className="bg-white rounded-lg shadow p-6">
                <h3 className="text-lg font-semibold mb-4">Scaling Status</h3>
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span>Status:</span>
                    <span className={`font-semibold ${scalingStatus.isScaling ? 'text-yellow-600' : 'text-green-600'}`}>
                      {scalingStatus.isScaling ? 'Scaling in Progress' : 'Stable'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>Model Accuracy:</span>
                    <span className="font-semibold">{(scalingStatus.modelAccuracy * 100).toFixed(0)}%</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Last Scale:</span>
                    <span className="text-sm">{formatDateTime(scalingStatus.lastScaleTime)}</span>
                  </div>
                </div>

                {/* Recent Scaling Events */}
                <h4 className="text-md font-semibold mt-6 mb-3">Recent Events</h4>
                <div className="space-y-2 max-h-48 overflow-y-auto">
                  {scalingStatus.scalingHistory.slice(-5).reverse().map((event: any, index: number) => (
                    <div key={index} className="text-sm border-l-2 border-gray-300 pl-3">
                      <div className="flex justify-between">
                        <span className={`font-medium ${
                          event.direction === 'up' ? 'text-green-600' : 
                          event.direction === 'down' ? 'text-blue-600' : 
                          'text-yellow-600'
                        }`}>
                          Scale {event.direction}
                        </span>
                        <span className="text-gray-500">{formatTime(event.timestamp)}</span>
                      </div>
                      <div className="text-gray-600 text-xs">{event.reason}</div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Quick Actions */}
              <div className="bg-white rounded-lg shadow p-6 lg:col-span-2">
                <h3 className="text-lg font-semibold mb-4">Quick Actions</h3>
                <div className="flex space-x-4">
                  <button
                    onClick={() => predictiveAutoScaling.emergencyScale()}
                    className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
                  >
                    Emergency Scale
                  </button>
                  <button
                    onClick={() => predictiveAutoScaling.manualScale({ webServers: 10 })}
                    className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                  >
                    Manual Scale
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'predictions' && predictions.length > 0 && (
            <div>
              <h3 className="text-lg font-semibold mb-4">24-Hour Load Predictions</h3>
              
              {/* Prediction Chart */}
              <div className="bg-white rounded-lg shadow p-6 mb-6">
                <div className="h-64 flex items-end space-x-1">
                  {predictions.map((pred, index) => {
                    const maxLoad = Math.max(...predictions.map(p => p.predictedLoad));
                    const height = (pred.predictedLoad / maxLoad) * 100;
                    const hour = pred.timestamp.getHours();
                    const isPeak = (hour >= 9 && hour < 12) || (hour >= 19 && hour < 23);
                    
                    return (
                      <div
                        key={index}
                        className="flex-1 flex flex-col items-center"
                        title={`${formatTime(pred.timestamp)}: ${pred.predictedLoad} users`}
                      >
                        <div
                          className={`w-full ${isPeak ? 'bg-red-500' : 'bg-blue-500'} hover:opacity-75 transition-opacity`}
                          style={{ height: `${height}%` }}
                        />
                        <span className="text-xs mt-1">{hour}</span>
                      </div>
                    );
                  })}
                </div>
                <div className="flex justify-between mt-2 text-xs text-gray-600">
                  <span>Time (IST)</span>
                  <span>Peak Hours: 9-12 AM, 7-11 PM</span>
                </div>
              </div>

              {/* Prediction Table */}
              <div className="bg-white rounded-lg shadow overflow-hidden">
                <table className="min-w-full">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Time</th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Predicted Load</th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Confidence</th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Web Servers</th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">DB Replicas</th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Redis Nodes</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-200">
                    {predictions.slice(0, 12).map((pred, index) => (
                      <tr key={index} className="hover:bg-gray-50">
                        <td className="px-4 py-2 text-sm">{formatTime(pred.timestamp)}</td>
                        <td className={`px-4 py-2 text-sm font-medium ${getLoadColor(pred.predictedLoad)}`}>
                          {pred.predictedLoad.toLocaleString()}
                        </td>
                        <td className="px-4 py-2 text-sm">
                          {(pred.confidence * 100).toFixed(0)}%
                        </td>
                        <td className="px-4 py-2 text-sm">{pred.recommendedInstances.webServers}</td>
                        <td className="px-4 py-2 text-sm">{pred.recommendedInstances.databaseReplicas}</td>
                        <td className="px-4 py-2 text-sm">{pred.recommendedInstances.redisNodes}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {activeTab === 'history' && historicalAnalysis && (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Historical Analysis */}
              <div className="bg-white rounded-lg shadow p-6">
                <h3 className="text-lg font-semibold mb-4">Historical Analysis</h3>
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span>Average Daily Users:</span>
                    <span className="font-semibold">{Math.floor(historicalAnalysis.averageDaily).toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Peak Hour:</span>
                    <span className="font-semibold">{historicalAnalysis.peakHour}:00 IST</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Peak Load:</span>
                    <span className={`font-semibold ${getLoadColor(historicalAnalysis.peakLoad)}`}>
                      {Math.floor(historicalAnalysis.peakLoad).toLocaleString()} users
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>Low Hour:</span>
                    <span className="font-semibold">{historicalAnalysis.lowHour}:00 IST</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Low Load:</span>
                    <span className="font-semibold text-green-600">
                      {Math.floor(historicalAnalysis.lowLoad).toLocaleString()} users
                    </span>
                  </div>
                </div>
              </div>

              {/* Traffic Patterns */}
              <div className="bg-white rounded-lg shadow p-6">
                <h3 className="text-lg font-semibold mb-4">Indian Traffic Patterns</h3>
                <div className="space-y-4">
                  <div>
                    <h4 className="font-medium mb-2">Peak Hours</h4>
                    <div className="space-y-1 text-sm">
                      <div className="flex justify-between">
                        <span>Morning Peak:</span>
                        <span className="text-orange-600">9 AM - 12 PM</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Afternoon Peak:</span>
                        <span className="text-yellow-600">2 PM - 5 PM</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Evening Peak:</span>
                        <span className="text-red-600">7 PM - 11 PM</span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <h4 className="font-medium mb-2">Educational Seasons</h4>
                    <div className="space-y-1 text-sm">
                      <div className="flex justify-between">
                        <span>Exam Season:</span>
                        <span className="text-red-600">2.5x Traffic</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Admission Season:</span>
                        <span className="text-orange-600">2.0x Traffic</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Regular Classes:</span>
                        <span className="text-green-600">1.0x Traffic</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Scaling Efficiency */}
              <div className="bg-white rounded-lg shadow p-6 lg:col-span-2">
                <h3 className="text-lg font-semibold mb-4">Scaling Efficiency Metrics</h3>
                <div className="grid grid-cols-3 gap-4">
                  <div className="text-center">
                    <div className="text-3xl font-bold text-green-600">92%</div>
                    <div className="text-sm text-gray-600">Prediction Accuracy</div>
                  </div>
                  <div className="text-center">
                    <div className="text-3xl font-bold text-blue-600">45ms</div>
                    <div className="text-sm text-gray-600">Avg Response Time</div>
                  </div>
                  <div className="text-center">
                    <div className="text-3xl font-bold text-purple-600">₹2.3L</div>
                    <div className="text-sm text-gray-600">Monthly Savings</div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};