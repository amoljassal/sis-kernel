// Phase 6B: Global Infrastructure Dashboard
// Real-time monitoring and management of global deployment
// @ts-nocheck

import React, { useState, useEffect } from 'react';
import { 
  GlobeAltIcon, 
  CloudIcon, 
  ServerIcon, 
  ShieldCheckIcon,
  ChartBarIcon,
  CheckCircleIcon,
  XCircleIcon,
  CogIcon
} from '@heroicons/react/24/outline';
import GlobalDeploymentManager from '../services/global-deployment-manager';
import EdgeCDNManager from '../services/edge-cdn-manager';
import DataReplicationManager from '../services/data-replication-manager';
import GDPRComplianceManager from '../services/gdpr-compliance-manager';
import GlobalLoadBalancer from '../services/global-load-balancer';

interface GlobalInfrastructureDashboardProps {
  isOpen: boolean;
  onClose: () => void;
}

const GlobalInfrastructureDashboard: React.FC<GlobalInfrastructureDashboardProps> = ({
  isOpen,
  onClose
}) => {
  const [activeTab, setActiveTab] = useState('overview');
  const [deploymentManager] = useState(() => new GlobalDeploymentManager());
  const [cdnManager] = useState(() => new EdgeCDNManager());
  const [replicationManager] = useState(() => new DataReplicationManager());
  const [complianceManager] = useState(() => new GDPRComplianceManager());
  const [loadBalancer] = useState(() => new GlobalLoadBalancer(deploymentManager));

  const [globalStatus, setGlobalStatus] = useState<any>(null);
  const [cdnStatus, setCdnStatus] = useState<any>(null);
  const [replicationStatus, setReplicationStatus] = useState<any>(null);
  const [complianceStatus, setComplianceStatus] = useState<any>(null);
  const [loadBalancerStatus, setLoadBalancerStatus] = useState<any>(null);

  useEffect(() => {
    if (!isOpen) return;

    // Initialize data
    setGlobalStatus(deploymentManager.getGlobalStatus());
    setCdnStatus(cdnManager.getGlobalCDNStatus());
    setReplicationStatus(replicationManager.getGlobalMetrics());
    setComplianceStatus(complianceManager.getComplianceStatus());
    setLoadBalancerStatus(loadBalancer.getGlobalStatus());

    // Set up event listeners
    deploymentManager.onMetricsUpdate((status: any) => setGlobalStatus(status));
    cdnManager.onMetricsUpdate((status: any) => setCdnStatus(status));
    replicationManager.onMetricsUpdate((metrics: any) => setReplicationStatus(metrics));
    complianceManager.onMetricsUpdate((status: any) => setComplianceStatus(status));
    loadBalancer.onTrafficEvent((status: any) => setLoadBalancerStatus(loadBalancer.getGlobalStatus()));

    return () => {
      deploymentManager.destroy();
      cdnManager.destroy();
      replicationManager.destroy();
      complianceManager.destroy();
      loadBalancer.destroy();
    };
  }, [isOpen]);

  if (!isOpen) return null;

  const tabs = [
    { id: 'overview', name: 'Global Overview', icon: GlobeAltIcon },
    { id: 'regions', name: 'Region Status', icon: CloudIcon },
    { id: 'cdn', name: 'Edge CDN', icon: ServerIcon },
    { id: 'replication', name: 'Data Sync', icon: ChartBarIcon },
    { id: 'compliance', name: 'Compliance', icon: ShieldCheckIcon },
    { id: 'loadbalancer', name: 'Load Balancing', icon: CogIcon }
  ];

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-7xl h-5/6 flex flex-col">
        <div className="flex items-center justify-between p-6 border-b">
          <div className="flex items-center space-x-3">
            <GlobeAltIcon className="h-8 w-8 text-blue-600" />
            <h2 className="text-2xl font-bold text-gray-900">Global Infrastructure</h2>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            <XCircleIcon className="h-6 w-6" />
          </button>
        </div>

        <div className="flex flex-1">
          {/* Sidebar */}
          <div className="w-64 bg-gray-50 border-r">
            <nav className="p-4 space-y-2">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`w-full flex items-center space-x-3 px-3 py-2 rounded-md text-left ${
                    activeTab === tab.id
                      ? 'bg-blue-100 text-blue-700'
                      : 'text-gray-600 hover:bg-gray-100'
                  }`}
                >
                  <tab.icon className="h-5 w-5" />
                  <span>{tab.name}</span>
                </button>
              ))}
            </nav>
          </div>

          {/* Main Content */}
          <div className="flex-1 p-6 overflow-auto">
            {activeTab === 'overview' && (
              <GlobalOverviewTab 
                globalStatus={globalStatus}
                cdnStatus={cdnStatus}
                replicationStatus={replicationStatus}
                complianceStatus={complianceStatus}
                loadBalancerStatus={loadBalancerStatus}
              />
            )}
            {activeTab === 'regions' && (
              <RegionStatusTab globalStatus={globalStatus} />
            )}
            {activeTab === 'cdn' && (
              <CDNStatusTab cdnStatus={cdnStatus} />
            )}
            {activeTab === 'replication' && (
              <ReplicationStatusTab replicationStatus={replicationStatus} />
            )}
            {activeTab === 'compliance' && (
              <ComplianceStatusTab complianceStatus={complianceStatus} />
            )}
            {activeTab === 'loadbalancer' && (
              <LoadBalancerStatusTab loadBalancerStatus={loadBalancerStatus} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

// Global Overview Tab
const GlobalOverviewTab: React.FC<{
  globalStatus: any;
  cdnStatus: any;
  replicationStatus: any;
  complianceStatus: any;
  loadBalancerStatus: any;
}> = ({ globalStatus, cdnStatus, complianceStatus }) => (
  <div className="space-y-6">
    <h3 className="text-xl font-semibold text-gray-900">Global Infrastructure Overview</h3>
    
    {/* Key Metrics */}
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <div className="bg-blue-50 p-6 rounded-lg">
        <div className="flex items-center">
          <CloudIcon className="h-8 w-8 text-blue-600" />
          <div className="ml-4">
            <p className="text-sm font-medium text-blue-600">Active Regions</p>
            <p className="text-2xl font-bold text-blue-900">
              {globalStatus?.activeRegions || 0}/{globalStatus?.totalRegions || 0}
            </p>
          </div>
        </div>
      </div>

      <div className="bg-green-50 p-6 rounded-lg">
        <div className="flex items-center">
          <ServerIcon className="h-8 w-8 text-green-600" />
          <div className="ml-4">
            <p className="text-sm font-medium text-green-600">Edge Locations</p>
            <p className="text-2xl font-bold text-green-900">52</p>
          </div>
        </div>
      </div>

      <div className="bg-purple-50 p-6 rounded-lg">
        <div className="flex items-center">
          <ChartBarIcon className="h-8 w-8 text-purple-600" />
          <div className="ml-4">
            <p className="text-sm font-medium text-purple-600">Cache Hit Rate</p>
            <p className="text-2xl font-bold text-purple-900">
              {cdnStatus?.cacheHitRate?.toFixed(1) || 0}%
            </p>
          </div>
        </div>
      </div>

      <div className="bg-yellow-50 p-6 rounded-lg">
        <div className="flex items-center">
          <ShieldCheckIcon className="h-8 w-8 text-yellow-600" />
          <div className="ml-4">
            <p className="text-sm font-medium text-yellow-600">Compliance Score</p>
            <p className="text-2xl font-bold text-yellow-900">
              {complianceStatus?.complianceScore || 0}%
            </p>
          </div>
        </div>
      </div>
    </div>

    {/* Global Health Status */}
    <div className="bg-white border rounded-lg p-6">
      <h4 className="text-lg font-medium text-gray-900 mb-4">Global Health Status</h4>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="text-center">
          <div className="text-3xl font-bold text-green-600">
            {globalStatus?.globalHealth?.toFixed(1) || 0}%
          </div>
          <div className="text-sm text-gray-600">Overall Health</div>
        </div>
        <div className="text-center">
          <div className="text-3xl font-bold text-blue-600">
            {globalStatus?.averageLatency?.toFixed(0) || 0}ms
          </div>
          <div className="text-sm text-gray-600">Average Latency</div>
        </div>
        <div className="text-center">
          <div className="text-3xl font-bold text-purple-600">
            {globalStatus?.totalUsers?.toLocaleString() || 0}
          </div>
          <div className="text-sm text-gray-600">Active Users</div>
        </div>
      </div>
    </div>

    {/* Traffic Distribution */}
    <div className="bg-white border rounded-lg p-6">
      <h4 className="text-lg font-medium text-gray-900 mb-4">Traffic Distribution</h4>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div>
          <h5 className="font-medium text-gray-700 mb-2">By Region</h5>
          <div className="space-y-2">
            {globalStatus?.regions?.slice(0, 5).map((region: any, index: number) => (
              <div key={region.regionId} className="flex items-center justify-between">
                <span className="text-sm text-gray-600">{region.name}</span>
                <div className="flex items-center space-x-2">
                  <div className="w-20 bg-gray-200 rounded-full h-2">
                    <div 
                      className="bg-blue-600 h-2 rounded-full"
                      style={{ width: `${region.load}%` }}
                    ></div>
                  </div>
                  <span className="text-sm font-medium">{region.load}%</span>
                </div>
              </div>
            ))}
          </div>
        </div>
        <div>
          <h5 className="font-medium text-gray-700 mb-2">CDN Performance</h5>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Total Requests</span>
              <span className="text-sm font-medium">{cdnStatus?.totalRequests?.toLocaleString() || 0}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Bandwidth Saved</span>
              <span className="text-sm font-medium">{cdnStatus?.bandwidthSaved || '0 GB'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Error Rate</span>
              <span className="text-sm font-medium">{cdnStatus?.errorRate?.toFixed(2) || 0}%</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
);

// Region Status Tab
const RegionStatusTab: React.FC<{ globalStatus: any }> = ({ globalStatus }) => (
  <div className="space-y-6">
    <h3 className="text-xl font-semibold text-gray-900">Region Status</h3>
    
    <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
      {globalStatus?.regions?.map((region: any) => (
        <div key={region.regionId} className="bg-white border rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h4 className="font-medium text-gray-900">{region.name}</h4>
            <div className={`px-2 py-1 rounded-full text-xs font-medium ${
              region.status === 'active' ? 'bg-green-100 text-green-800' :
              region.status === 'deploying' ? 'bg-yellow-100 text-yellow-800' :
              'bg-red-100 text-red-800'
            }`}>
              {region.status}
            </div>
          </div>
          
          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Health</span>
              <span className="text-sm font-medium">{region.health}%</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Load</span>
              <span className="text-sm font-medium">{region.load}%</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Latency</span>
              <span className="text-sm font-medium">{region.latency}ms</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Users</span>
              <span className="text-sm font-medium">{region.traffic?.users?.toLocaleString() || 0}</span>
            </div>
          </div>

          <div className="mt-4 pt-4 border-t">
            <h5 className="text-sm font-medium text-gray-700 mb-2">Infrastructure</h5>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div>Web: {region.instances?.webServers || 0}</div>
              <div>DB: {region.instances?.databases || 0}</div>
              <div>Cache: {region.instances?.cacheNodes || 0}</div>
              <div>AI: {region.instances?.aiServices || 0}</div>
            </div>
          </div>
        </div>
      ))}
    </div>
  </div>
);

// CDN Status Tab
const CDNStatusTab: React.FC<{ cdnStatus: any }> = ({ cdnStatus }) => (
  <div className="space-y-6">
    <h3 className="text-xl font-semibold text-gray-900">Edge CDN Status</h3>
    
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <div className="bg-blue-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-blue-600">Total Requests</p>
        <p className="text-2xl font-bold text-blue-900">{cdnStatus?.totalRequests?.toLocaleString() || 0}</p>
      </div>
      <div className="bg-green-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-green-600">Cache Hit Rate</p>
        <p className="text-2xl font-bold text-green-900">{cdnStatus?.cacheHitRate?.toFixed(1) || 0}%</p>
      </div>
      <div className="bg-purple-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-purple-600">Bandwidth Saved</p>
        <p className="text-2xl font-bold text-purple-900">{cdnStatus?.bandwidthSaved || '0 GB'}</p>
      </div>
      <div className="bg-orange-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-orange-600">Avg Latency</p>
        <p className="text-2xl font-bold text-orange-900">{cdnStatus?.averageLatency?.toFixed(0) || 0}ms</p>
      </div>
    </div>

    <div className="bg-white border rounded-lg p-6">
      <h4 className="text-lg font-medium text-gray-900 mb-4">Top Performing Locations</h4>
      <div className="space-y-3">
        {cdnStatus?.topLocations?.map((location: string, index: number) => (
          <div key={location} className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className="text-sm font-medium text-gray-500">#{index + 1}</div>
              <div className="text-sm text-gray-900">{location}</div>
            </div>
            <CheckCircleIcon className="h-5 w-5 text-green-500" />
          </div>
        ))}
      </div>
    </div>
  </div>
);

// Replication Status Tab
const ReplicationStatusTab: React.FC<{ replicationStatus: any }> = ({ replicationStatus }) => (
  <div className="space-y-6">
    <h3 className="text-xl font-semibold text-gray-900">Data Replication Status</h3>
    
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <div className="bg-blue-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-blue-600">Total Operations</p>
        <p className="text-2xl font-bold text-blue-900">{replicationStatus?.totalOperations?.toLocaleString() || 0}</p>
      </div>
      <div className="bg-green-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-green-600">Success Rate</p>
        <p className="text-2xl font-bold text-green-900">
          {replicationStatus?.totalOperations ? 
            ((replicationStatus.successfulOperations / replicationStatus.totalOperations) * 100).toFixed(1) : 0}%
        </p>
      </div>
      <div className="bg-yellow-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-yellow-600">Avg Latency</p>
        <p className="text-2xl font-bold text-yellow-900">{replicationStatus?.averageLatency?.toFixed(0) || 0}ms</p>
      </div>
      <div className="bg-red-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-red-600">Conflicts</p>
        <p className="text-2xl font-bold text-red-900">{replicationStatus?.conflictsDetected || 0}</p>
      </div>
    </div>

    <div className="bg-white border rounded-lg p-6">
      <h4 className="text-lg font-medium text-gray-900 mb-4">Sync Performance</h4>
      <div className="space-y-4">
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-600">Data Transferred</span>
          <span className="text-sm font-medium">
            {replicationStatus?.dataTransferred ? 
              (replicationStatus.dataTransferred / (1024 * 1024 * 1024)).toFixed(2) + ' GB' : '0 GB'}
          </span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-600">Conflicts Resolved</span>
          <span className="text-sm font-medium">{replicationStatus?.conflictsResolved || 0}</span>
        </div>
      </div>
    </div>
  </div>
);

// Compliance Status Tab
const ComplianceStatusTab: React.FC<{ complianceStatus: any }> = ({ complianceStatus }) => (
  <div className="space-y-6">
    <h3 className="text-xl font-semibold text-gray-900">GDPR & Compliance Status</h3>
    
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <div className="bg-green-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-green-600">Compliance Score</p>
        <p className="text-2xl font-bold text-green-900">{complianceStatus?.complianceScore || 0}%</p>
      </div>
      <div className="bg-blue-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-blue-600">Data Subjects</p>
        <p className="text-2xl font-bold text-blue-900">{complianceStatus?.dataSubjects || 0}</p>
      </div>
      <div className="bg-yellow-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-yellow-600">Pending Requests</p>
        <p className="text-2xl font-bold text-yellow-900">{complianceStatus?.pendingRequests || 0}</p>
      </div>
      <div className="bg-purple-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-purple-600">Active Regions</p>
        <p className="text-2xl font-bold text-purple-900">{complianceStatus?.regions?.length || 0}</p>
      </div>
    </div>

    <div className="bg-white border rounded-lg p-6">
      <h4 className="text-lg font-medium text-gray-900 mb-4">Regional Compliance</h4>
      <div className="space-y-3">
        {complianceStatus?.regions?.map((region: string) => (
          <div key={region} className="flex items-center justify-between">
            <span className="text-sm text-gray-900">{region}</span>
            <div className="flex items-center space-x-2">
              <CheckCircleIcon className="h-5 w-5 text-green-500" />
              <span className="text-sm text-green-600">Compliant</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  </div>
);

// Load Balancer Status Tab
const LoadBalancerStatusTab: React.FC<{ loadBalancerStatus: any }> = ({ loadBalancerStatus }) => (
  <div className="space-y-6">
    <h3 className="text-xl font-semibold text-gray-900">Load Balancer Status</h3>
    
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <div className="bg-blue-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-blue-600">Active Nodes</p>
        <p className="text-2xl font-bold text-blue-900">
          {loadBalancerStatus?.activeNodes || 0}/{loadBalancerStatus?.totalNodes || 0}
        </p>
      </div>
      <div className="bg-green-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-green-600">Total Capacity</p>
        <p className="text-2xl font-bold text-green-900">{loadBalancerStatus?.totalCapacity?.toLocaleString() || 0}</p>
      </div>
      <div className="bg-yellow-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-yellow-600">Current Load</p>
        <p className="text-2xl font-bold text-yellow-900">{loadBalancerStatus?.totalLoad?.toLocaleString() || 0}</p>
      </div>
      <div className="bg-purple-50 p-4 rounded-lg">
        <p className="text-sm font-medium text-purple-600">Avg Health</p>
        <p className="text-2xl font-bold text-purple-900">{loadBalancerStatus?.averageHealth?.toFixed(1) || 0}%</p>
      </div>
    </div>

    <div className="bg-white border rounded-lg p-6">
      <h4 className="text-lg font-medium text-gray-900 mb-4">Node Status</h4>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {loadBalancerStatus?.nodes?.slice(0, 8).map((node: any) => (
          <div key={node.regionId} className="flex items-center justify-between p-3 border rounded">
            <div>
              <div className="font-medium text-gray-900">{node.regionId}</div>
              <div className="text-sm text-gray-600">Load: {node.currentLoad}%</div>
            </div>
            <div className="flex items-center space-x-2">
              <div className={`w-3 h-3 rounded-full ${
                node.status === 'active' ? 'bg-green-500' :
                node.status === 'draining' ? 'bg-yellow-500' :
                'bg-red-500'
              }`}></div>
              <span className="text-sm">{node.status}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  </div>
);

export default GlobalInfrastructureDashboard;