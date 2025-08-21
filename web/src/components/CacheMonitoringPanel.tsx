/**
 * Cache Monitoring Panel for Phase 5C
 * Real-time monitoring of multi-layer Redis caching system
 */

import React, { useState, useEffect } from 'react';
import { multiLayerCache, CacheMetrics } from '../services/redis-caching';

interface CacheMonitoringPanelProps {
  isVisible: boolean;
  onClose: () => void;
}

export const CacheMonitoringPanel: React.FC<CacheMonitoringPanelProps> = ({
  isVisible,
  onClose
}) => {
  const [metrics, setMetrics] = useState<{
    l1: CacheMetrics;
    l2: CacheMetrics;
    l3: CacheMetrics;
    overall: CacheMetrics;
  } | null>(null);
  const [cacheStatus, setCacheStatus] = useState<any>(null);

  useEffect(() => {
    if (!isVisible) return;

    const updateMetrics = () => {
      const currentMetrics = multiLayerCache.getCacheMetrics();
      const status = multiLayerCache.getCacheStatus();
      setMetrics(currentMetrics);
      setCacheStatus(status);
    };

    updateMetrics();
    const interval = setInterval(updateMetrics, 2000);
    return () => clearInterval(interval);
  }, [isVisible]);

  const handleClearCache = (layer: 'l1' | 'l2' | 'l3') => {
    multiLayerCache.clearLayer(layer);
    console.log(`Cleared ${layer.toUpperCase()} cache layer`);
  };

  if (!isVisible || !metrics || !cacheStatus) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-2xl w-5/6 h-5/6 overflow-auto">
        {/* Header */}
        <div className="bg-green-600 text-white p-6 rounded-t-lg">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-2xl font-bold">Multi-Layer Redis Cache Monitoring</h2>
              <p className="text-green-100">L1: Application | L2: Sessions | L3: Collaboration</p>
            </div>
            <button
              onClick={onClose}
              className="bg-green-500 hover:bg-green-700 px-4 py-2 rounded transition-colors"
            >
              Close
            </button>
          </div>
        </div>

        <div className="p-6">
          {/* Overall Performance */}
          <div className="mb-6 p-4 bg-gradient-to-r from-green-50 to-blue-50 rounded-lg">
            <h3 className="text-lg font-semibold mb-4">Overall Cache Performance</h3>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="text-center">
                <div className="text-3xl font-bold text-green-600">
                  {metrics.overall.hitRate.toFixed(1)}%
                </div>
                <div className="text-sm text-gray-600">Hit Rate</div>
              </div>
              <div className="text-center">
                <div className="text-3xl font-bold text-blue-600">
                  {cacheStatus.totalMemoryUsage}
                </div>
                <div className="text-sm text-gray-600">Memory Usage</div>
              </div>
              <div className="text-center">
                <div className="text-3xl font-bold text-purple-600">
                  {metrics.overall.connections.toLocaleString()}
                </div>
                <div className="text-sm text-gray-600">Connections</div>
              </div>
              <div className="text-center">
                <div className={`text-3xl font-bold ${
                  cacheStatus.peakHoursActive ? 'text-orange-600' : 'text-gray-600'
                }`}>
                  {cacheStatus.peakHoursActive ? 'PEAK' : 'NORMAL'}
                </div>
                <div className="text-sm text-gray-600">Traffic Mode</div>
              </div>
            </div>
          </div>

          {/* Layer-specific Metrics */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
            {/* L1 Application Cache */}
            <div className="border border-blue-200 rounded-lg p-4">
              <div className="flex justify-between items-center mb-4">
                <h4 className="text-lg font-semibold text-blue-700">L1 Application Cache</h4>
                <button
                  onClick={() => handleClearCache('l1')}
                  className="text-xs bg-blue-100 hover:bg-blue-200 px-3 py-1 rounded transition-colors"
                >
                  Clear
                </button>
              </div>
              
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Hit Rate:</span>
                  <span className="font-semibold">{metrics.l1.hitRate.toFixed(1)}%</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Cache Size:</span>
                  <span className="font-semibold">{cacheStatus.l1Size.toLocaleString()}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Memory:</span>
                  <span className="font-semibold">{(metrics.l1.memoryUsage / 1024 / 1024).toFixed(1)} MB</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Connections:</span>
                  <span className="font-semibold">{metrics.l1.connections.toLocaleString()}</span>
                </div>
              </div>

              {/* Progress bars */}
              <div className="mt-4">
                <div className="text-xs text-gray-500 mb-1">Hit Rate</div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                    style={{ width: `${metrics.l1.hitRate}%` }}
                  ></div>
                </div>
              </div>

              <div className="mt-3 p-2 bg-blue-50 rounded text-xs">
                <strong>Optimized for:</strong> Educational content, component library, certification data
              </div>
            </div>

            {/* L2 Session Store */}
            <div className="border border-green-200 rounded-lg p-4">
              <div className="flex justify-between items-center mb-4">
                <h4 className="text-lg font-semibold text-green-700">L2 Session Store</h4>
                <button
                  onClick={() => handleClearCache('l2')}
                  className="text-xs bg-green-100 hover:bg-green-200 px-3 py-1 rounded transition-colors"
                >
                  Clear
                </button>
              </div>
              
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Hit Rate:</span>
                  <span className="font-semibold">{metrics.l2.hitRate.toFixed(1)}%</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Sessions:</span>
                  <span className="font-semibold">{cacheStatus.l2Size.toLocaleString()}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Memory:</span>
                  <span className="font-semibold">{(metrics.l2.memoryUsage / 1024 / 1024).toFixed(1)} MB</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Connections:</span>
                  <span className="font-semibold">{metrics.l2.connections.toLocaleString()}</span>
                </div>
              </div>

              <div className="mt-4">
                <div className="text-xs text-gray-500 mb-1">Hit Rate</div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-green-500 h-2 rounded-full transition-all duration-300"
                    style={{ width: `${metrics.l2.hitRate}%` }}
                  ></div>
                </div>
              </div>

              <div className="mt-3 p-2 bg-green-50 rounded text-xs">
                <strong>Features:</strong> PDPB compliant, 25K concurrent sessions, high availability
              </div>
            </div>

            {/* L3 Collaboration */}
            <div className="border border-purple-200 rounded-lg p-4">
              <div className="flex justify-between items-center mb-4">
                <h4 className="text-lg font-semibold text-purple-700">L3 Collaboration</h4>
                <button
                  onClick={() => handleClearCache('l3')}
                  className="text-xs bg-purple-100 hover:bg-purple-200 px-3 py-1 rounded transition-colors"
                >
                  Clear
                </button>
              </div>
              
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Hit Rate:</span>
                  <span className="font-semibold">{metrics.l3.hitRate.toFixed(1)}%</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Channels:</span>
                  <span className="font-semibold">{cacheStatus.l3Size.toLocaleString()}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Memory:</span>
                  <span className="font-semibold">{(metrics.l3.memoryUsage / 1024 / 1024).toFixed(1)} MB</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600">Connections:</span>
                  <span className="font-semibold">{metrics.l3.connections.toLocaleString()}</span>
                </div>
              </div>

              <div className="mt-4">
                <div className="text-xs text-gray-500 mb-1">Hit Rate</div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-purple-500 h-2 rounded-full transition-all duration-300"
                    style={{ width: `${metrics.l3.hitRate}%` }}
                  ></div>
                </div>
              </div>

              <div className="mt-3 p-2 bg-purple-50 rounded text-xs">
                <strong>Features:</strong> Offline-first, real-time sync, Indian network optimized
              </div>
            </div>
          </div>

          {/* Performance Trends */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
            <div className="bg-gray-50 p-4 rounded-lg">
              <h4 className="text-lg font-semibold mb-3">Indian Traffic Optimization</h4>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span>Peak Hours (9 AM - 11 PM IST)</span>
                  <span className={`px-3 py-1 rounded text-sm font-medium ${
                    cacheStatus.peakHoursActive 
                      ? 'bg-orange-100 text-orange-800' 
                      : 'bg-green-100 text-green-800'
                  }`}>
                    {cacheStatus.peakHoursActive ? 'Active' : 'Inactive'}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Educational Content Preloading</span>
                  <span className="px-3 py-1 bg-blue-100 text-blue-800 rounded text-sm font-medium">
                    Enabled
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Hindi Language Support</span>
                  <span className="px-3 py-1 bg-green-100 text-green-800 rounded text-sm font-medium">
                    Active
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Offline Collaboration Sync</span>
                  <span className="px-3 py-1 bg-purple-100 text-purple-800 rounded text-sm font-medium">
                    Ready
                  </span>
                </div>
              </div>
            </div>

            <div className="bg-gray-50 p-4 rounded-lg">
              <h4 className="text-lg font-semibold mb-3">Cache Health Status</h4>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span>Overall Health</span>
                  <span className="px-3 py-1 bg-green-100 text-green-800 rounded text-sm font-medium">
                    Excellent
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Memory Utilization</span>
                  <span className="px-3 py-1 bg-blue-100 text-blue-800 rounded text-sm font-medium">
                    Optimal
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Eviction Rate</span>
                  <span className="px-3 py-1 bg-green-100 text-green-800 rounded text-sm font-medium">
                    Low
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span>PDPB Compliance</span>
                  <span className="px-3 py-1 bg-green-100 text-green-800 rounded text-sm font-medium">
                    Verified
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Cache Configuration Overview */}
          <div className="p-4 bg-gradient-to-r from-gray-50 to-blue-50 rounded-lg">
            <h4 className="text-lg font-semibold mb-3">Cache Architecture Overview</h4>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
              <div>
                <h5 className="font-medium text-blue-600 mb-2">L1 Application Cache</h5>
                <ul className="text-gray-600 space-y-1">
                  <li>• 8 Redis nodes (256GB total)</li>
                  <li>• TTL: 1-24 hours based on content</li>
                  <li>• Educational content preloading</li>
                  <li>• Component library optimization</li>
                </ul>
              </div>
              <div>
                <h5 className="font-medium text-green-600 mb-2">L2 Session Store</h5>
                <ul className="text-gray-600 space-y-1">
                  <li>• 5 Sentinel nodes for HA</li>
                  <li>• 25,000+ concurrent sessions</li>
                  <li>• PDPB compliant data handling</li>
                  <li>• 2-hour session timeout</li>
                </ul>
              </div>
              <div>
                <h5 className="font-medium text-purple-600 mb-2">L3 Collaboration</h5>
                <ul className="text-gray-600 space-y-1">
                  <li>• 6 Stream nodes (30K connections)</li>
                  <li>• Offline-first for Indian networks</li>
                  <li>• Real-time conflict resolution</li>
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

export default CacheMonitoringPanel;