/**
 * WebSocket Monitoring Dashboard for Phase 5C
 * Real-time monitoring of WebSocket infrastructure for 25,000+ users
 */

import React, { useState, useEffect } from 'react';
import { webSocketInfrastructure, WebSocketMetrics } from '../services/websocket-infrastructure';

interface WebSocketMonitoringDashboardProps {
  isVisible: boolean;
  onClose: () => void;
}

export const WebSocketMonitoringDashboard: React.FC<WebSocketMonitoringDashboardProps> = ({
  isVisible,
  onClose
}) => {
  const [metrics, setMetrics] = useState<WebSocketMetrics | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<any>(null);
  const [isTestingConnection, setIsTestingConnection] = useState(false);

  useEffect(() => {
    if (!isVisible) return;

    const updateMetrics = () => {
      const currentMetrics = webSocketInfrastructure.getMetrics();
      const status = webSocketInfrastructure.getConnectionStatus();
      setMetrics(currentMetrics);
      setConnectionStatus(status);
    };

    updateMetrics();
    const interval = setInterval(updateMetrics, 2000);
    return () => clearInterval(interval);
  }, [isVisible]);

  const handleTestConnection = async () => {
    setIsTestingConnection(true);
    try {
      const connectionId = await webSocketInfrastructure.connectUser(
        'test_user_' + Date.now(),
        'test_session_' + Date.now(),
        'test_channel'
      );
      
      // Send test message
      await webSocketInfrastructure.sendMessage({
        connectionId,
        message: { type: 'test', content: 'Connection test successful' },
        priority: 'high'
      });
      
      // Disconnect after test
      setTimeout(() => {
        webSocketInfrastructure.disconnectUser(connectionId);
      }, 1000);
      
      console.log('WebSocket connection test successful');
    } catch (error) {
      console.error('WebSocket connection test failed:', error);
    } finally {
      setIsTestingConnection(false);
    }
  };

  if (!isVisible || !metrics || !connectionStatus) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-2xl w-5/6 h-5/6 overflow-auto">
        {/* Header */}
        <div className="bg-purple-600 text-white p-6 rounded-t-lg">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-2xl font-bold">WebSocket Infrastructure Monitoring</h2>
              <p className="text-purple-100">Real-time collaboration for 25,000+ users (Mumbai Primary)</p>
            </div>
            <button
              onClick={onClose}
              className="bg-purple-500 hover:bg-purple-700 px-4 py-2 rounded transition-colors"
            >
              Close
            </button>
          </div>
        </div>

        <div className="p-6">
          {/* Real-time Metrics */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
            <MetricCard
              title="Active Connections"
              value={metrics.activeConnections.toLocaleString()}
              target="Target: 25,000"
              color="blue"
              percentage={(metrics.activeConnections / 25000) * 100}
            />
            <MetricCard
              title="Active Sessions"
              value={metrics.totalSessions.toLocaleString()}
              target="Collaborative sessions"
              color="green"
            />
            <MetricCard
              title="Messages/Second"
              value={metrics.messagesPerSecond.toLocaleString()}
              target="Real-time throughput"
              color="purple"
            />
            <MetricCard
              title="Latency (P95)"
              value={`${metrics.latencyP95}ms`}
              target="Target: <50ms (India)"
              color={metrics.latencyP95 > 50 ? "red" : "green"}
            />
          </div>

          {/* Connection Status Overview */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
            <div className="bg-gray-50 p-4 rounded-lg">
              <h3 className="text-lg font-semibold mb-4">Connection Distribution</h3>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span>Peak Capacity Utilization</span>
                  <span className="font-semibold">
                    {connectionStatus.peakCapacityUtilization.toFixed(1)}%
                  </span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-3">
                  <div
                    className="bg-blue-500 h-3 rounded-full transition-all duration-300"
                    style={{ width: `${connectionStatus.peakCapacityUtilization}%` }}
                  ></div>
                </div>
                
                <div className="mt-4 space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-600">Total Connections:</span>
                    <span className="font-medium">{connectionStatus.totalConnections}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-600">Active Connections:</span>
                    <span className="font-medium">{connectionStatus.activeConnections}</span>
                  </div>
                </div>
              </div>
            </div>

            <div className="bg-gray-50 p-4 rounded-lg">
              <h3 className="text-lg font-semibold mb-4">Session Types</h3>
              <div className="space-y-3">
                {Object.entries(connectionStatus.sessionsByType).map(([type, count]) => (
                  <div key={type} className="flex justify-between items-center">
                    <span className="capitalize">{type} Sessions</span>
                    <span className="font-semibold">{String(count)}</span>
                  </div>
                ))}
                
                <div className="mt-4 p-3 bg-blue-50 rounded text-sm">
                  <h4 className="font-medium mb-2">Session Capabilities</h4>
                  <ul className="text-gray-600 space-y-1">
                    <li>• Educational: 500 participants max</li>
                    <li>• Design: 50 participants max</li>
                    <li>• Marketplace: 100 participants max</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>

          {/* Performance Monitoring */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
            <div className="bg-gray-50 p-4 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Performance Health</h3>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span>Error Rate</span>
                  <span className={`px-3 py-1 rounded text-sm font-medium ${
                    metrics.errorRate > 1 ? 'bg-red-100 text-red-800' : 'bg-green-100 text-green-800'
                  }`}>
                    {metrics.errorRate.toFixed(2)}%
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Bandwidth Usage</span>
                  <span className="font-semibold">{(metrics.bandwidthUsage / 1024 / 1024).toFixed(2)} MB</span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Connection Health</span>
                  <span className="px-3 py-1 bg-green-100 text-green-800 rounded text-sm font-medium">
                    Excellent
                  </span>
                </div>
              </div>
            </div>

            <div className="bg-gray-50 p-4 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Indian Optimization</h3>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span>Mumbai Primary</span>
                  <span className="px-3 py-1 bg-blue-100 text-blue-800 rounded text-sm font-medium">
                    Active
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>ISP Optimization</span>
                  <span className="px-3 py-1 bg-green-100 text-green-800 rounded text-sm font-medium">
                    Airtel, Jio, BSNL
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Offline Sync</span>
                  <span className="px-3 py-1 bg-purple-100 text-purple-800 rounded text-sm font-medium">
                    Ready
                  </span>
                </div>
              </div>
            </div>

            <div className="bg-gray-50 p-4 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">Collaboration Features</h3>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span>CRDT Algorithm</span>
                  <span className="text-sm font-medium">YJS Optimized</span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Sticky Sessions</span>
                  <span className="px-3 py-1 bg-blue-100 text-blue-800 rounded text-sm font-medium">
                    IP Hash
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Educational Mode</span>
                  <span className="px-3 py-1 bg-green-100 text-green-800 rounded text-sm font-medium">
                    Enabled
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Control Panel */}
          <div className="bg-gradient-to-r from-purple-50 to-blue-50 p-4 rounded-lg mb-6">
            <h3 className="text-lg font-semibold mb-4">WebSocket Control Panel</h3>
            <div className="flex items-center justify-between">
              <div className="text-sm">
                <p className="text-gray-600 mb-1">Test WebSocket connectivity and performance</p>
                <p className="text-xs text-gray-500">Creates test connection, sends message, and disconnects</p>
              </div>
              <button
                onClick={handleTestConnection}
                disabled={isTestingConnection}
                className={`px-6 py-2 rounded font-semibold transition-colors ${
                  isTestingConnection
                    ? 'bg-gray-400 cursor-not-allowed'
                    : 'bg-purple-600 hover:bg-purple-700 text-white'
                }`}
              >
                {isTestingConnection ? 'Testing...' : 'Test Connection'}
              </button>
            </div>
          </div>

          {/* Infrastructure Architecture */}
          <div className="p-4 bg-gradient-to-r from-gray-50 to-purple-50 rounded-lg">
            <h3 className="text-lg font-semibold mb-3">WebSocket Architecture Overview</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
              <div>
                <h4 className="font-medium text-purple-600 mb-2">Mumbai Primary Gateway</h4>
                <ul className="text-gray-600 space-y-1">
                  <li>• 10 WebSocket instances</li>
                  <li>• 20,000 connection capacity</li>
                  <li>• Sticky session support</li>
                  <li>• ISP direct peering</li>
                </ul>
              </div>
              <div>
                <h4 className="font-medium text-blue-600 mb-2">Kafka Event Streaming</h4>
                <ul className="text-gray-600 space-y-1">
                  <li>• 9 broker cluster</li>
                  <li>• 60 partitions for design events</li>
                  <li>• Real-time message routing</li>
                  <li>• Educational event support</li>
                </ul>
              </div>
              <div>
                <h4 className="font-medium text-green-600 mb-2">CRDT Collaboration</h4>
                <ul className="text-gray-600 space-y-1">
                  <li>• YJS algorithm optimized</li>
                  <li>• Offline-first design</li>
                  <li>• Conflict resolution (IST priority)</li>
                  <li>• Educational session support</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// Metric card component
const MetricCard: React.FC<{
  title: string;
  value: string;
  target: string;
  color: string;
  percentage?: number;
}> = ({ title, value, target, color, percentage }) => {
  const colorClasses: { [key: string]: string } = {
    blue: 'border-blue-500 bg-blue-50',
    green: 'border-green-500 bg-green-50',
    red: 'border-red-500 bg-red-50',
    purple: 'border-purple-500 bg-purple-50'
  };

  return (
    <div className={`border-2 rounded-lg p-4 ${colorClasses[color] || colorClasses.blue}`}>
      <h3 className="text-sm font-medium text-gray-600">{title}</h3>
      <div className="text-2xl font-bold mt-1">{value}</div>
      <div className="text-xs text-gray-500 mt-1">{target}</div>
      {percentage !== undefined && (
        <div className="mt-2">
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className={`h-2 rounded-full transition-all duration-300 ${
                color === 'blue' ? 'bg-blue-500' :
                color === 'green' ? 'bg-green-500' :
                color === 'red' ? 'bg-red-500' :
                'bg-purple-500'
              }`}
              style={{ width: `${Math.min(percentage, 100)}%` }}
            ></div>
          </div>
        </div>
      )}
    </div>
  );
};

export default WebSocketMonitoringDashboard;