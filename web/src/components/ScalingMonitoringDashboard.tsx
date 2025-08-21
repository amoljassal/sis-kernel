/**
 * Phase 5C Scaling Monitoring Dashboard
 * Real-time monitoring of database scaling for 25,000+ users
 */

import React, { useState, useEffect } from 'react';
import { databaseScalingService, ScalingMetrics, ConnectionPool, ShardInfo } from '../services/database-scaling';

interface ScalingDashboardProps {
  isVisible: boolean;
  onClose: () => void;
}

export const ScalingMonitoringDashboard: React.FC<ScalingDashboardProps> = ({
  isVisible,
  onClose
}) => {
  const [metrics, setMetrics] = useState<ScalingMetrics | null>(null);
  const [scalingStatus, setScalingStatus] = useState(databaseScalingService.getScalingStatus());
  const [connectionPools, setConnectionPools] = useState<ConnectionPool[]>([]);
  const [shards, setShards] = useState<ShardInfo[]>([]);
  const [scalingTarget, setScalingTarget] = useState(25000);
  const [isScaling, setIsScaling] = useState(false);

  // Real-time data updates
  useEffect(() => {
    if (!isVisible) return;

    const updateInterval = setInterval(async () => {
      const currentMetrics = await databaseScalingService.getCurrentMetrics();
      const status = databaseScalingService.getScalingStatus();
      const pools = databaseScalingService.getConnectionPoolStatus();
      const shardInfo = databaseScalingService.getShardDistribution();

      setMetrics(currentMetrics);
      setScalingStatus(status);
      setConnectionPools(pools);
      setShards(shardInfo);
    }, 2000);

    return () => clearInterval(updateInterval);
  }, [isVisible]);

  const handleStartScaling = async () => {
    setIsScaling(true);
    try {
      const success = await databaseScalingService.scaleToTargetUsers(scalingTarget);
      if (success) {
        console.log('✅ Scaling completed successfully');
      } else {
        console.log('❌ Scaling failed');
      }
    } catch (error) {
      console.error('Scaling error:', error);
    } finally {
      setIsScaling(false);
    }
  };

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-2xl w-5/6 h-5/6 overflow-auto">
        {/* Header */}
        <div className="bg-blue-600 text-white p-6 rounded-t-lg">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-2xl font-bold">Phase 5C Database Scaling Dashboard</h2>
              <p className="text-blue-100">Target: 25,000+ Concurrent Indian Users</p>
            </div>
            <button
              onClick={onClose}
              className="bg-blue-500 hover:bg-blue-700 px-4 py-2 rounded transition-colors"
            >
              Close
            </button>
          </div>
        </div>

        <div className="p-6">
          {/* Scaling Control Panel */}
          <div className="mb-6 p-4 bg-gray-50 rounded-lg">
            <h3 className="text-lg font-semibold mb-4">🚀 Scaling Control</h3>
            <div className="flex items-center gap-4">
              <div>
                <label className="block text-sm font-medium mb-1">Target Users:</label>
                <input
                  type="number"
                  value={scalingTarget}
                  onChange={(e) => setScalingTarget(Number(e.target.value))}
                  className="border rounded px-3 py-2 w-24"
                  min="1000"
                  max="50000"
                  step="1000"
                />
              </div>
              <button
                onClick={handleStartScaling}
                disabled={isScaling || scalingStatus.inProgress}
                className={`px-6 py-2 rounded font-semibold transition-colors ${
                  isScaling || scalingStatus.inProgress
                    ? 'bg-gray-400 cursor-not-allowed'
                    : 'bg-green-600 hover:bg-green-700 text-white'
                }`}
              >
                {isScaling ? 'Scaling...' : 'Start Scaling'}
              </button>
            </div>
          </div>

          {/* Current Metrics */}
          {metrics && (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
              <MetricCard
                title="Current Users"
                value={metrics.currentUsers.toLocaleString()}
                target={`Target: ${metrics.targetUsers.toLocaleString()}`}
                color="blue"
              />
              <MetricCard
                title="Database Load"
                value={`${metrics.databaseLoad}%`}
                target="Target: <80%"
                color={metrics.databaseLoad > 80 ? "red" : "green"}
              />
              <MetricCard
                title="Cache Hit Rate"
                value={`${metrics.cacheHitRate}%`}
                target="Target: >85%"
                color={metrics.cacheHitRate < 85 ? "red" : "green"}
              />
              <MetricCard
                title="Latency (P95)"
                value={`${metrics.latencyP95}ms`}
                target="Target: <50ms (India)"
                color={metrics.latencyP95 > 50 ? "red" : "green"}
              />
              <MetricCard
                title="Error Rate"
                value={`${metrics.errorRate}%`}
                target="Target: <0.1%"
                color={metrics.errorRate > 0.1 ? "red" : "green"}
              />
              <MetricCard
                title="Scaling Progress"
                value={`${Math.round(scalingStatus.progress)}%`}
                target={`${scalingStatus.currentCapacity.toLocaleString()} / ${scalingStatus.targetCapacity.toLocaleString()}`}
                color="purple"
              />
            </div>
          )}

          {/* Scaling Progress Bar */}
          <div className="mb-6 p-4 bg-gray-50 rounded-lg">
            <h3 className="text-lg font-semibold mb-3">📊 Scaling Progress</h3>
            <div className="w-full bg-gray-200 rounded-full h-4">
              <div
                className="bg-blue-600 h-4 rounded-full transition-all duration-300"
                style={{ width: `${scalingStatus.progress}%` }}
              ></div>
            </div>
            <div className="mt-2 text-sm text-gray-600">
              {scalingStatus.inProgress ? (
                <span className="text-blue-600 font-medium">⏳ Scaling in progress...</span>
              ) : (
                <span>Current capacity: {scalingStatus.currentCapacity.toLocaleString()} users</span>
              )}
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Connection Pools Status */}
            <div className="bg-gray-50 p-4 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">🔄 Connection Pools</h3>
              <div className="space-y-2">
                {connectionPools.map(pool => (
                  <div key={pool.id} className="flex justify-between items-center bg-white p-3 rounded">
                    <div>
                      <span className="font-medium">{pool.id}</span>
                      <span className="text-sm text-gray-500 ml-2">({pool.region})</span>
                    </div>
                    <div className="text-right">
                      <div className="text-sm">
                        {pool.activeConnections} / {pool.maxConnections}
                      </div>
                      <div className={`text-xs px-2 py-1 rounded ${getStatusColor(pool.status)}`}>
                        {pool.status}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Shard Distribution */}
            <div className="bg-gray-50 p-4 rounded-lg">
              <h3 className="text-lg font-semibold mb-3">🗂️ Shard Distribution</h3>
              <div className="space-y-2">
                {shards.map(shard => (
                  <div key={shard.id} className="bg-white p-3 rounded">
                    <div className="flex justify-between items-center">
                      <div>
                        <span className="font-medium">{shard.id}</span>
                        <span className="text-sm text-gray-500 ml-2">({shard.region})</span>
                      </div>
                      <div className="text-right">
                        <div className="text-sm">{shard.userRange}</div>
                        <div className="text-xs text-gray-500">
                          Load: {Math.round((shard.currentLoad / shard.maxCapacity) * 100)}%
                        </div>
                      </div>
                    </div>
                    <div className="mt-2">
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-blue-500 h-2 rounded-full"
                          style={{ width: `${(shard.currentLoad / shard.maxCapacity) * 100}%` }}
                        ></div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Infrastructure Overview */}
          <div className="mt-6 p-4 bg-gradient-to-r from-blue-50 to-purple-50 rounded-lg">
            <h3 className="text-lg font-semibold mb-3">🏗️ Infrastructure Overview</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
              <div>
                <h4 className="font-medium text-blue-600">Database Scaling</h4>
                <ul className="text-gray-600 mt-1 space-y-1">
                  <li>• Primary: Mumbai (AP-SOUTH-1)</li>
                  <li>• Read Replicas: 4x r6g.4xlarge</li>
                  <li>• Connection Pool: 15,000 max</li>
                  <li>• Sharding: 3 horizontal shards</li>
                </ul>
              </div>
              <div>
                <h4 className="font-medium text-purple-600">Caching Layers</h4>
                <ul className="text-gray-600 mt-1 space-y-1">
                  <li>• L1: 8 nodes (256GB total)</li>
                  <li>• L2: 5 sentinel nodes</li>
                  <li>• L3: 6 stream nodes</li>
                  <li>• Hit Rate: {metrics?.cacheHitRate}%</li>
                </ul>
              </div>
              <div>
                <h4 className="font-medium text-green-600">Network & Performance</h4>
                <ul className="text-gray-600 mt-1 space-y-1">
                  <li>• WebSocket: 10 instances</li>
                  <li>• Kafka: 9 brokers</li>
                  <li>• Target Latency: &lt;50ms (India)</li>
                  <li>• Uptime SLA: 99.95%</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// Helper component for metric cards
const MetricCard: React.FC<{
  title: string;
  value: string;
  target: string;
  color: string;
}> = ({ title, value, target, color }) => {
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
    </div>
  );
};

// Helper function for status colors
const getStatusColor = (status: string): string => {
  switch (status) {
    case 'active':
      return 'bg-green-100 text-green-800';
    case 'scaling':
      return 'bg-blue-100 text-blue-800';
    case 'maintenance':
      return 'bg-yellow-100 text-yellow-800';
    default:
      return 'bg-gray-100 text-gray-800';
  }
};

export default ScalingMonitoringDashboard;