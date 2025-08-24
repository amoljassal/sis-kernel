/**
 * Training Resource Manager
 * Centralized management of compute resources for AI training
 */

import React, { useState } from 'react';
import { useInfrastructureManager } from '../../../hooks/useInfrastructureManager';
import { TrainingJob } from '../../../services/api/infrastructureApi';
import {
  Cpu,
  HardDrive,
  Zap,
  Thermometer,
  Monitor,
  Server,
  Cloud,
  Activity,
  Settings,
  AlertTriangle,
  CheckCircle,
  Clock,
  BarChart3,
  RefreshCw,
  Plus,
  Loader2,
  DollarSign,
  MapPin,
  Gauge,
  Link,
  Power,
  Scale
} from 'lucide-react';

// Types are now imported from the API service

// Sample data moved to API service

export const TrainingResourceManager: React.FC = () => {
  const {
    resources,
    cloudProviders,
    trainingJobs,
    resourceTemplates,
    metrics,
    selectedResource,
    loading,
    error,
    refreshData,
    selectResource,
    connectCloudProvider,
    provisionResource,
    terminateResource,
    scaleResource,
    scheduleTrainingJob,
    getCostAnalysis
  } = useInfrastructureManager();
  
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [filterStatus, setFilterStatus] = useState<'all' | 'available' | 'busy' | 'offline'>('all');
  const [showProvisionModal, setShowProvisionModal] = useState(false);
  const [showCloudProviderModal, setShowCloudProviderModal] = useState(false);
  const [costAnalysis, setCostAnalysis] = useState<any>(null);

  // Load cost analysis on mount
  React.useEffect(() => {
    const loadCostAnalysis = async () => {
      try {
        const analysis = await getCostAnalysis('24h');
        setCostAnalysis(analysis);
      } catch (error) {
        console.error('Failed to load cost analysis:', error);
      }
    };
    
    loadCostAnalysis();
  }, [getCostAnalysis]);

  const handleProvisionResource = async (template: any, config: any) => {
    try {
      await provisionResource(template, config);
      setShowProvisionModal(false);
    } catch (error) {
      console.error('Failed to provision resource:', error);
    }
  };

  const handleConnectProvider = async (provider: 'aws' | 'gcp' | 'azure', credentials: Record<string, string>) => {
    try {
      await connectCloudProvider(provider, credentials);
      setShowCloudProviderModal(false);
    } catch (error) {
      console.error('Failed to connect cloud provider:', error);
    }
  };

  const filteredResources = resources.filter(resource => 
    filterStatus === 'all' || resource.status === filterStatus
  );

  const jobs = trainingJobs; // For compatibility with existing code

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'available': return <CheckCircle className="w-4 h-4 text-green-400" />;
      case 'busy': return <Activity className="w-4 h-4 text-blue-400 animate-pulse" />;
      case 'offline': return <AlertTriangle className="w-4 h-4 text-red-400" />;
      case 'maintenance': return <Clock className="w-4 h-4 text-yellow-400" />;
      case 'provisioning': return <Loader2 className="w-4 h-4 text-blue-400 animate-spin" />;
      case 'terminating': return <Power className="w-4 h-4 text-red-400" />;
      default: return <AlertTriangle className="w-4 h-4 text-gray-400" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'available': return 'bg-green-900/30 border-green-500/30';
      case 'busy': return 'bg-blue-900/30 border-blue-500/30';
      case 'offline': return 'bg-red-900/30 border-red-500/30';
      case 'maintenance': return 'bg-yellow-900/30 border-yellow-500/30';
      case 'provisioning': return 'bg-blue-900/30 border-blue-500/30';
      case 'terminating': return 'bg-red-900/30 border-red-500/30';
      default: return 'bg-gray-900/30 border-gray-500/30';
    }
  };

  const getProviderColor = (provider: string) => {
    switch (provider) {
      case 'aws': return 'text-orange-400';
      case 'gcp': return 'text-blue-400';
      case 'azure': return 'text-cyan-400';
      default: return 'text-gray-400';
    }
  };

  const ResourceCard: React.FC<{ resource: any }> = ({ resource }) => (
    <div 
      className={`card p-6 cursor-pointer transition-all hover:scale-105 border ${getStatusColor(resource.status)}`}
      onClick={() => selectResource(resource)}
    >
      <div className="flex items-start justify-between mb-4">
        <div>
          <h3 className="text-white font-semibold mb-1">{resource.name}</h3>
          <p className="text-sm text-sis-gray-400">{resource.provider}</p>
          <div className="flex items-center space-x-2 mt-2">
            {getStatusIcon(resource.status)}
            <span className="text-sm text-sis-gray-300 capitalize">{resource.status}</span>
          </div>
        </div>
        
        <div className="text-right">
          <div className="text-xs text-sis-gray-500 mb-1">{resource.type.toUpperCase()}</div>
          <div className="text-xs text-sis-gray-400">{resource.location}</div>
          {resource.pricing && (
            <div className="text-xs text-green-400 mt-1">
              ${resource.pricing.costPerHour}/hr
            </div>
          )}
        </div>
      </div>

      {/* Resource Specs */}
      <div className="grid grid-cols-2 gap-3 mb-4 text-sm">
        <div className="flex items-center space-x-2">
          <Monitor className="w-4 h-4 text-sis-blue-400" />
          <span className="text-sis-gray-300">{resource.capabilities.gpus} GPU</span>
        </div>
        <div className="flex items-center space-x-2">
          <Cpu className="w-4 h-4 text-sis-green-400" />
          <span className="text-sis-gray-300">{resource.capabilities.cpuCores} CPU</span>
        </div>
        <div className="flex items-center space-x-2">
          <HardDrive className="w-4 h-4 text-sis-purple-400" />
          <span className="text-sis-gray-300">{resource.capabilities.systemMemory}GB RAM</span>
        </div>
        <div className="flex items-center space-x-2">
          <Server className="w-4 h-4 text-sis-orange-400" />
          <span className="text-sis-gray-300">{resource.capabilities.storage}GB</span>
        </div>
      </div>

      {/* Utilization Bars */}
      <div className="space-y-2">
        {Object.entries(resource.utilization).slice(0, 3).map(([key, value]) => (
          <div key={key} className="flex items-center space-x-2">
            <span className="text-xs text-sis-gray-400 w-12 capitalize">{key}</span>
            <div className="flex-1 bg-sis-gray-700 rounded-full h-1.5">
              <div 
                className={`h-1.5 rounded-full ${
                  (value as number) > 80 ? 'bg-red-500' : (value as number) > 60 ? 'bg-yellow-500' : 'bg-green-500'
                }`}
                style={{ width: `${value}%` }}
              />
            </div>
            <span className="text-xs text-sis-gray-300 w-8 text-right">{(value as number).toFixed(0)}%</span>
          </div>
        ))}
      </div>
    </div>
  );

  const JobCard: React.FC<{ job: TrainingJob }> = ({ job }) => {
    const resource = resources.find(r => r.id === job.resourceId);
    
    /**
     * IMPORTANT: This component handles startTime data defensively to prevent runtime errors
     * The TrainingJob interface specifies startTime as optional and located at job.scheduling.startTime
     * However, for backward compatibility, we also check job.startTime (legacy format)
     * If no startTime is found, we estimate it from job.progress.timeElapsed
     */
    const getStartTime = () => {
      // Check multiple possible locations for startTime for backward compatibility
      const startTimeStr = job.scheduling?.startTime || job.startTime;
      
      if (!startTimeStr) {
        // If no startTime is available, estimate based on progress
        const timeElapsed = job.progress?.timeElapsed || 0;
        const estimatedStartTime = Date.now() - (timeElapsed * 1000);
        return new Date(estimatedStartTime);
      }
      
      // Handle both string and Date object formats
      return typeof startTimeStr === 'string' ? new Date(startTimeStr) : startTimeStr;
    };
    
    const startTime = getStartTime();
    
    // Calculate elapsed time with error protection
    const calculateElapsedTime = () => {
      try {
        if (!startTime || !startTime.getTime) {
          return 0;
        }
        const elapsed = Math.floor((Date.now() - startTime.getTime()) / 1000 / 60);
        return elapsed >= 0 ? elapsed : 0;
      } catch (error) {
        console.error('Error calculating elapsed time:', error);
        return 0;
      }
    };
    
    const elapsedTime = calculateElapsedTime();
    
    return (
      <div className="card p-4 border border-sis-gray-700">
        <div className="flex items-start justify-between mb-3">
          <div>
            <h4 className="text-white font-medium mb-1">{job.modelName}</h4>
            <p className="text-sm text-sis-gray-400">
              Running on {resource?.name}
            </p>
          </div>
          <div className="text-right">
            <div className={`text-xs font-medium ${
              job.status === 'running' ? 'text-blue-400' : 
              job.status === 'completed' ? 'text-green-400' : 'text-yellow-400'
            }`}>
              {job.status.toUpperCase()}
            </div>
            <div className="text-xs text-sis-gray-400 mt-1">
              {elapsedTime}m elapsed
            </div>
          </div>
        </div>

        <div className="space-y-2">
          <div className="flex items-center justify-between text-xs">
            <span className="text-sis-gray-400">Progress</span>
            <span className="text-sis-gray-300">{job.progress.percentage.toFixed(1)}%</span>
          </div>
          <div className="w-full bg-sis-gray-700 rounded-full h-2">
            <div 
              className="h-2 bg-gradient-to-r from-blue-500 to-green-500 rounded-full transition-all"
              style={{ width: `${job.progress.percentage}%` }}
            />
          </div>
        </div>

        <div className="grid grid-cols-3 gap-2 mt-3 text-xs">
          <div className="text-center">
            <div className="text-white">{job.resourceUsage.gpu}%</div>
            <div className="text-sis-gray-400">GPU</div>
          </div>
          <div className="text-center">
            <div className="text-white">{job.resourceUsage.cpu}%</div>
            <div className="text-sis-gray-400">CPU</div>
          </div>
          <div className="text-center">
            <div className="text-white">{job.resourceUsage.memory}%</div>
            <div className="text-sis-gray-400">Memory</div>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <div className="flex items-center space-x-4 mb-2">
            <h2 className="text-2xl font-bold text-white">Training Resource Manager</h2>
            {loading && <Loader2 className="w-5 h-5 animate-spin text-sis-blue-400" />}
          </div>
          <p className="text-sis-gray-400">Manage compute resources for AI model training across multiple cloud providers</p>
          
          {/* Error Display */}
          {error && (
            <div className="mt-3 p-3 bg-red-900/30 border border-red-500/30 rounded-lg">
              <p className="text-red-300 text-sm">
                <AlertTriangle className="w-4 h-4 inline mr-2" />
                {error}
              </p>
            </div>
          )}
          
          {/* Cloud Provider Status */}
          <div className="mt-3 flex items-center space-x-4">
            <span className="text-sm text-sis-gray-400">Connected Providers:</span>
            {cloudProviders.map(provider => (
              <div key={provider.id} className="flex items-center space-x-1">
                <div className={`w-2 h-2 rounded-full ${
                  provider.status === 'connected' ? 'bg-green-400' : 
                  provider.status === 'error' ? 'bg-red-400' : 'bg-yellow-400'
                }`} />
                <span className={`text-xs ${getProviderColor(provider.type)}`}>
                  {provider.name}
                </span>
              </div>
            ))}
            <button
              onClick={() => setShowCloudProviderModal(true)}
              className="text-xs text-sis-blue-400 hover:text-sis-blue-300 transition-colors flex items-center space-x-1"
            >
              <Link className="w-3 h-3" />
              <span>Connect Provider</span>
            </button>
          </div>
        </div>
        
        <div className="flex items-center space-x-3">
          <select
            value={filterStatus}
            onChange={(e) => setFilterStatus(e.target.value as any)}
            className="bg-sis-gray-800 border border-sis-gray-600 rounded text-white text-sm px-3 py-2"
          >
            <option value="all">All Resources</option>
            <option value="available">Available</option>
            <option value="busy">Busy</option>
            <option value="offline">Offline</option>
          </select>
          
          <div className="flex items-center bg-sis-gray-800 rounded-lg p-1">
            <button
              onClick={() => setViewMode('grid')}
              className={`p-2 rounded ${viewMode === 'grid' ? 'bg-sis-blue-600' : ''}`}
            >
              <BarChart3 className="w-4 h-4" />
            </button>
            <button
              onClick={() => setViewMode('list')}
              className={`p-2 rounded ${viewMode === 'list' ? 'bg-sis-blue-600' : ''}`}
            >
              <Activity className="w-4 h-4" />
            </button>
          </div>
          
          <button
            onClick={() => setShowProvisionModal(true)}
            disabled={loading}
            className="btn-primary px-4 py-2 flex items-center space-x-2 disabled:opacity-50"
          >
            <Plus className="w-4 h-4" />
            <span>Provision Resource</span>
          </button>
          
          <button
            onClick={refreshData}
            disabled={loading}
            className="p-2 bg-sis-gray-700 text-sis-gray-300 rounded-lg hover:bg-sis-gray-600 transition-colors disabled:opacity-50"
          >
            <RefreshCw className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Resource Overview */}
      <div className="grid grid-cols-1 md:grid-cols-6 gap-4">
        <div className="card p-4 text-center">
          <div className="flex items-center justify-center mb-2">
            <CheckCircle className="w-5 h-5 text-green-400 mr-2" />
            <div className="text-2xl font-bold text-white">
              {resources.filter(r => r.status === 'available').length}
            </div>
          </div>
          <div className="text-sm text-sis-gray-400">Available</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="flex items-center justify-center mb-2">
            <Activity className="w-5 h-5 text-blue-400 mr-2" />
            <div className="text-2xl font-bold text-blue-400">
              {resources.filter(r => r.status === 'busy').length}
            </div>
          </div>
          <div className="text-sm text-sis-gray-400">In Use</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="flex items-center justify-center mb-2">
            <Zap className="w-5 h-5 text-green-400 mr-2" />
            <div className="text-2xl font-bold text-green-400">
              {jobs.filter(j => j.status === 'running').length}
            </div>
          </div>
          <div className="text-sm text-sis-gray-400">Active Jobs</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="flex items-center justify-center mb-2">
            <DollarSign className="w-5 h-5 text-purple-400 mr-2" />
            <div className="text-2xl font-bold text-purple-400">
              ${costAnalysis?.currentHourly?.toFixed(2) || '0.00'}
            </div>
          </div>
          <div className="text-sm text-sis-gray-400">Current Cost/hr</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="flex items-center justify-center mb-2">
            <Gauge className="w-5 h-5 text-orange-400 mr-2" />
            <div className="text-2xl font-bold text-orange-400">
              {metrics?.utilizationRates.overall || 0}%
            </div>
          </div>
          <div className="text-sm text-sis-gray-400">Avg Utilization</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="flex items-center justify-center mb-2">
            <Monitor className="w-5 h-5 text-cyan-400 mr-2" />
            <div className="text-2xl font-bold text-cyan-400">
              {resources.reduce((sum, r) => sum + r.capabilities.gpus, 0)}
            </div>
          </div>
          <div className="text-sm text-sis-gray-400">Total GPUs</div>
        </div>
      </div>

      {/* Active Jobs */}
      {jobs.length > 0 && (
        <div>
          <h3 className="text-lg font-semibold text-white mb-4">Active Training Jobs</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {jobs.map(job => (
              <JobCard key={job.id} job={job} />
            ))}
          </div>
        </div>
      )}

      {/* Resources Grid */}
      <div>
        <h3 className="text-lg font-semibold text-white mb-4">Compute Resources</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredResources.map(resource => (
            <ResourceCard key={resource.id} resource={resource} />
          ))}
        </div>
        
        {filteredResources.length === 0 && (
          <div className="text-center py-12">
            <Server className="w-12 h-12 text-sis-gray-600 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-white mb-2">No resources found</h3>
            <p className="text-sis-gray-400">Try adjusting your filters or add new resources</p>
          </div>
        )}
      </div>

      {/* Resource Detail Modal */}
      {selectedResource && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => selectResource(null)}>
          <div className="bg-sis-gray-900 rounded-lg p-6 max-w-2xl w-full mx-4" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-bold text-white">{selectedResource.name}</h3>
              <button
                onClick={() => selectResource(null)}
                className="text-sis-gray-400 hover:text-white"
              >
                ×
              </button>
            </div>
            
            <div className="grid grid-cols-2 gap-6">
              <div>
                <h4 className="text-lg font-semibold text-white mb-3">Specifications</h4>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-sis-gray-400">GPUs</span>
                    <span className="text-white">
                      {selectedResource.capabilities.gpus}× {selectedResource.capabilities.gpuType || 'GPU'}
                      {selectedResource.capabilities.gpuMemory && ` (${selectedResource.capabilities.gpuMemory}GB)`}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sis-gray-400">CPU Cores</span>
                    <span className="text-white">
                      {selectedResource.capabilities.cpuCores} {selectedResource.capabilities.cpuType || 'vCPU'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sis-gray-400">System Memory</span>
                    <span className="text-white">{selectedResource.capabilities.systemMemory}GB RAM</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sis-gray-400">Storage</span>
                    <span className="text-white">
                      {selectedResource.capabilities.storage}GB {selectedResource.capabilities.storageType?.toUpperCase() || 'SSD'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sis-gray-400">Network</span>
                    <span className="text-white">{selectedResource.capabilities.bandwidth} Mbps</span>
                  </div>
                  {selectedResource.region && (
                    <div className="flex justify-between">
                      <span className="text-sis-gray-400">Region</span>
                      <span className="text-white">{selectedResource.region}</span>
                    </div>
                  )}
                  {selectedResource.pricing && (
                    <>
                      <div className="flex justify-between border-t border-sis-gray-700 pt-2 mt-2">
                        <span className="text-sis-gray-400">Cost per Hour</span>
                        <span className="text-green-400">
                          ${selectedResource.pricing.costPerHour} {selectedResource.pricing.currency}
                        </span>
                      </div>
                      {selectedResource.pricing.estimatedMonthlyCost && (
                        <div className="flex justify-between">
                          <span className="text-sis-gray-400">Est. Monthly</span>
                          <span className="text-green-400">
                            ${selectedResource.pricing.estimatedMonthlyCost.toFixed(2)}
                          </span>
                        </div>
                      )}
                    </>
                  )}
                </div>
              </div>
              
              <div>
                <h4 className="text-lg font-semibold text-white mb-3">Current Utilization</h4>
                <div className="space-y-3">
                  {Object.entries(selectedResource.utilization).map(([key, value]) => (
                    <div key={key}>
                      <div className="flex justify-between text-sm mb-1">
                        <span className="text-sis-gray-400 capitalize">{key}</span>
                        <span className="text-white">{(value as number).toFixed(1)}%</span>
                      </div>
                      <div className="w-full bg-sis-gray-700 rounded-full h-2">
                        <div 
                          className={`h-2 rounded-full ${
                            (value as number) > 80 ? 'bg-red-500' : (value as number) > 60 ? 'bg-yellow-500' : 'bg-green-500'
                          }`}
                          style={{ width: `${value}%` }}
                        />
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
            
            <div className="flex space-x-3 mt-6">
              <button 
                onClick={() => {
                  // Navigate to training job creation
                  selectResource(null);
                }}
                disabled={selectedResource.status !== 'available'}
                className="btn-primary flex-1 disabled:opacity-50"
              >
                Start Training Job
              </button>
              
              {selectedResource.type === 'cloud' && (
                <>
                  <button 
                    onClick={() => scaleResource(selectedResource.id, 'scale-up')}
                    disabled={loading || selectedResource.status === 'busy'}
                    className="btn-secondary disabled:opacity-50"
                  >
                    <Scale className="w-4 h-4 mr-1" />
                    Scale
                  </button>
                  
                  <button 
                    onClick={() => {
                      if (confirm('Are you sure you want to terminate this resource?')) {
                        terminateResource(selectedResource.id);
                        selectResource(null);
                      }
                    }}
                    disabled={loading || selectedResource.status === 'busy'}
                    className="btn-secondary text-red-400 hover:bg-red-900/20 disabled:opacity-50"
                  >
                    <Power className="w-4 h-4 mr-1" />
                    Terminate
                  </button>
                </>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Cloud Provider Connection Modal */}
      {showCloudProviderModal && (
        <CloudProviderModal
          onClose={() => setShowCloudProviderModal(false)}
          onConnect={handleConnectProvider}
          loading={loading}
        />
      )}

      {/* Resource Provisioning Modal */}
      {showProvisionModal && (
        <ResourceProvisionModal
          templates={resourceTemplates}
          cloudProviders={cloudProviders}
          onClose={() => setShowProvisionModal(false)}
          onProvision={handleProvisionResource}
          loading={loading}
        />
      )}
    </div>
  );
};

// Field interface
interface ProviderField {
  key: string;
  label: string;
  type: string;
  placeholder?: string;
}

// Cloud Provider Connection Modal Component
const CloudProviderModal: React.FC<{
  onClose: () => void;
  onConnect: (provider: 'aws' | 'gcp' | 'azure', credentials: Record<string, string>) => void;
  loading: boolean;
}> = ({ onClose, onConnect, loading }) => {
  const [selectedProvider, setSelectedProvider] = useState<'aws' | 'gcp' | 'azure'>('aws');
  const [credentials, setCredentials] = useState<Record<string, string>>({});

  const providerFields: Record<'aws' | 'gcp' | 'azure', ProviderField[]> = {
    aws: [
      { key: 'accessKeyId', label: 'Access Key ID', type: 'text' },
      { key: 'secretAccessKey', label: 'Secret Access Key', type: 'password' },
      { key: 'region', label: 'Default Region', type: 'text', placeholder: 'us-west-2' }
    ],
    gcp: [
      { key: 'projectId', label: 'Project ID', type: 'text' },
      { key: 'serviceAccountKey', label: 'Service Account Key (JSON)', type: 'textarea' },
      { key: 'region', label: 'Default Region', type: 'text', placeholder: 'us-central1' }
    ],
    azure: [
      { key: 'subscriptionId', label: 'Subscription ID', type: 'text' },
      { key: 'clientId', label: 'Client ID', type: 'text' },
      { key: 'clientSecret', label: 'Client Secret', type: 'password' },
      { key: 'tenantId', label: 'Tenant ID', type: 'text' }
    ]
  };

  const handleConnect = () => {
    onConnect(selectedProvider, credentials);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-sis-gray-800 rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Connect Cloud Provider</h3>
          <button
            onClick={onClose}
            className="text-sis-gray-400 hover:text-white transition-colors"
          >
            ×
          </button>
        </div>

        {/* Provider Selection */}
        <div className="mb-6">
          <label className="block text-sm text-sis-gray-400 mb-2">Select Provider</label>
          <div className="flex space-x-3">
            {(['aws', 'gcp', 'azure'] as const).map(provider => (
              <button
                key={provider}
                onClick={() => {
                  setSelectedProvider(provider);
                  setCredentials({});
                }}
                className={`px-4 py-2 rounded-lg border transition-colors ${
                  selectedProvider === provider
                    ? 'border-sis-blue-500 bg-sis-blue-900/20 text-sis-blue-300'
                    : 'border-sis-gray-600 text-sis-gray-300 hover:border-sis-gray-500'
                }`}
              >
                {provider.toUpperCase()}
              </button>
            ))}
          </div>
        </div>

        {/* Credentials Form */}
        <div className="space-y-4">
          {providerFields[selectedProvider].map(field => (
            <div key={field.key}>
              <label className="block text-sm text-sis-gray-400 mb-2">{field.label}</label>
              {field.type === 'textarea' ? (
                <textarea
                  value={credentials[field.key] || ''}
                  onChange={(e) => setCredentials(prev => ({
                    ...prev,
                    [field.key]: e.target.value
                  }))}
                  className="w-full bg-sis-gray-700 text-white px-3 py-2 rounded-lg border border-sis-gray-600 focus:border-sis-blue-500 focus:outline-none h-32"
                  placeholder={field.placeholder || ''}
                />
              ) : (
                <input
                  type={field.type}
                  value={credentials[field.key] || ''}
                  onChange={(e) => setCredentials(prev => ({
                    ...prev,
                    [field.key]: e.target.value
                  }))}
                  className="w-full bg-sis-gray-700 text-white px-3 py-2 rounded-lg border border-sis-gray-600 focus:border-sis-blue-500 focus:outline-none"
                  placeholder={field.placeholder || ''}
                />
              )}
            </div>
          ))}
        </div>

        <div className="flex items-center justify-end space-x-3 mt-6">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sis-gray-400 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleConnect}
            disabled={loading || !Object.values(credentials).every(v => v.trim())}
            className="btn-primary px-4 py-2 disabled:opacity-50"
          >
            {loading ? 'Connecting...' : 'Connect Provider'}
          </button>
        </div>
      </div>
    </div>
  );
};

// Resource Provisioning Modal Component
const ResourceProvisionModal: React.FC<{
  templates: any[];
  cloudProviders: any[];
  onClose: () => void;
  onProvision: (template: any, config: any) => void;
  loading: boolean;
}> = ({ templates, cloudProviders, onClose, onProvision, loading }) => {
  const [selectedTemplate, setSelectedTemplate] = useState<any>(null);
  const [config, setConfig] = useState({
    name: '',
    region: '',
    tags: {} as Record<string, string>
  });

  const handleProvision = () => {
    if (selectedTemplate && config.name && config.region) {
      onProvision(selectedTemplate, config);
    }
  };

  const connectedProviders = cloudProviders.filter(p => p.status === 'connected');

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-sis-gray-800 rounded-lg p-6 w-full max-w-4xl max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Provision New Resource</h3>
          <button
            onClick={onClose}
            className="text-sis-gray-400 hover:text-white transition-colors"
          >
            ×
          </button>
        </div>

        {connectedProviders.length === 0 ? (
          <div className="text-center py-12">
            <Cloud className="w-12 h-12 text-sis-gray-600 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-white mb-2">No Cloud Providers Connected</h3>
            <p className="text-sis-gray-400 mb-4">Connect a cloud provider to provision resources</p>
            <button onClick={onClose} className="btn-primary">
              Connect Provider First
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Template Selection */}
            <div>
              <h4 className="text-lg font-semibold text-white mb-4">Select Resource Template</h4>
              <div className="space-y-3 max-h-96 overflow-y-auto">
                {templates.map(template => (
                  <div
                    key={template.id}
                    onClick={() => setSelectedTemplate(template)}
                    className={`p-4 rounded-lg border cursor-pointer transition-colors ${
                      selectedTemplate?.id === template.id
                        ? 'border-sis-blue-500 bg-sis-blue-900/20'
                        : 'border-sis-gray-600 hover:border-sis-gray-500'
                    }`}
                  >
                    <div className="flex items-start justify-between mb-2">
                      <h5 className="text-white font-medium">{template.name}</h5>
                      <span className={`text-xs px-2 py-1 rounded bg-sis-blue-600/20 text-sis-blue-300`}>
                        {template.provider.toUpperCase()}
                      </span>
                    </div>
                    <p className="text-sm text-sis-gray-400 mb-3">{template.description}</p>
                    
                    <div className="grid grid-cols-2 gap-2 text-xs text-sis-gray-300">
                      <div>GPUs: {template.capabilities.gpus}x {template.capabilities.gpuType}</div>
                      <div>CPU: {template.capabilities.cpuCores} cores</div>
                      <div>RAM: {template.capabilities.systemMemory}GB</div>
                      <div className="text-green-400">${template.pricing.costPerHour}/hr</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Configuration */}
            <div>
              <h4 className="text-lg font-semibold text-white mb-4">Configuration</h4>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-sis-gray-400 mb-2">Resource Name</label>
                  <input
                    type="text"
                    value={config.name}
                    onChange={(e) => setConfig(prev => ({ ...prev, name: e.target.value }))}
                    className="w-full bg-sis-gray-700 text-white px-3 py-2 rounded-lg border border-sis-gray-600 focus:border-sis-blue-500 focus:outline-none"
                    placeholder="my-training-resource"
                  />
                </div>
                
                <div>
                  <label className="block text-sm text-sis-gray-400 mb-2">Region</label>
                  <select
                    value={config.region}
                    onChange={(e) => setConfig(prev => ({ ...prev, region: e.target.value }))}
                    className="w-full bg-sis-gray-700 text-white px-3 py-2 rounded-lg border border-sis-gray-600 focus:border-sis-blue-500 focus:outline-none"
                  >
                    <option value="">Select Region</option>
                    <option value="us-west-2">US West 2 (Oregon)</option>
                    <option value="us-east-1">US East 1 (Virginia)</option>
                    <option value="eu-west-1">EU West 1 (Ireland)</option>
                    <option value="ap-southeast-1">Asia Pacific (Singapore)</option>
                  </select>
                </div>

                {selectedTemplate && (
                  <div className="mt-6 p-4 bg-sis-gray-900 rounded-lg">
                    <h5 className="text-white font-medium mb-3">Resource Summary</h5>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-sis-gray-400">Template:</span>
                        <span className="text-white">{selectedTemplate.name}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-sis-gray-400">Provider:</span>
                        <span className="text-white">{selectedTemplate.provider.toUpperCase()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-sis-gray-400">Cost/Hour:</span>
                        <span className="text-green-400">${selectedTemplate.pricing.costPerHour}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-sis-gray-400">Est. Daily:</span>
                        <span className="text-green-400">${(selectedTemplate.pricing.costPerHour * 24).toFixed(2)}</span>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {connectedProviders.length > 0 && (
          <div className="flex items-center justify-end space-x-3 mt-6">
            <button
              onClick={onClose}
              className="px-4 py-2 text-sis-gray-400 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleProvision}
              disabled={loading || !selectedTemplate || !config.name || !config.region}
              className="btn-primary px-4 py-2 disabled:opacity-50"
            >
              {loading ? 'Provisioning...' : `Provision Resource ($${selectedTemplate?.pricing.costPerHour || 0}/hr)`}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default TrainingResourceManager;