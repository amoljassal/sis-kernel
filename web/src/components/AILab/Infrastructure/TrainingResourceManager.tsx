/**
 * Training Resource Manager
 * Centralized management of compute resources for AI training
 */

import React, { useState, useEffect } from 'react';
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
  RefreshCw
} from 'lucide-react';

interface ComputeResource {
  id: string;
  name: string;
  type: 'local' | 'cloud' | 'distributed';
  provider: string;
  status: 'available' | 'busy' | 'offline' | 'maintenance';
  capabilities: {
    gpus: number;
    gpuMemory: number;
    cpuCores: number;
    systemMemory: number;
    storage: number;
    bandwidth: number;
  };
  utilization: {
    gpu: number;
    cpu: number;
    memory: number;
    storage: number;
    power: number;
  };
  pricing?: {
    costPerHour: number;
    currency: string;
  };
  location: string;
  lastUpdated: Date;
}

interface TrainingJob {
  id: string;
  modelName: string;
  status: 'queued' | 'running' | 'completed' | 'failed' | 'paused';
  resourceId: string;
  startTime: Date;
  estimatedDuration: number;
  progress: number;
  resourceUsage: {
    gpu: number;
    cpu: number;
    memory: number;
  };
}

const SAMPLE_RESOURCES: ComputeResource[] = [
  {
    id: 'local-m1-pro',
    name: 'MacBook Pro M1 Max',
    type: 'local',
    provider: 'Apple Silicon',
    status: 'busy',
    capabilities: {
      gpus: 1,
      gpuMemory: 32,
      cpuCores: 10,
      systemMemory: 64,
      storage: 2048,
      bandwidth: 1000
    },
    utilization: {
      gpu: 87,
      cpu: 45,
      memory: 62,
      storage: 34,
      power: 65
    },
    location: 'Local',
    lastUpdated: new Date()
  },
  {
    id: 'aws-p4d-24xlarge',
    name: 'AWS p4d.24xlarge',
    type: 'cloud',
    provider: 'Amazon Web Services',
    status: 'available',
    capabilities: {
      gpus: 8,
      gpuMemory: 320,
      cpuCores: 96,
      systemMemory: 1152,
      storage: 8000,
      bandwidth: 4000
    },
    utilization: {
      gpu: 0,
      cpu: 5,
      memory: 8,
      storage: 12,
      power: 15
    },
    pricing: {
      costPerHour: 32.77,
      currency: 'USD'
    },
    location: 'us-west-2',
    lastUpdated: new Date()
  },
  {
    id: 'gcp-a100-cluster',
    name: 'GCP A100 Cluster',
    type: 'distributed',
    provider: 'Google Cloud Platform',
    status: 'maintenance',
    capabilities: {
      gpus: 16,
      gpuMemory: 640,
      cpuCores: 192,
      systemMemory: 2304,
      storage: 16000,
      bandwidth: 8000
    },
    utilization: {
      gpu: 0,
      cpu: 0,
      memory: 0,
      storage: 8,
      power: 0
    },
    pricing: {
      costPerHour: 45.50,
      currency: 'USD'
    },
    location: 'us-central1',
    lastUpdated: new Date()
  }
];

const SAMPLE_JOBS: TrainingJob[] = [
  {
    id: 'job-001',
    modelName: 'Legal Document Analyzer v2.1',
    status: 'running',
    resourceId: 'local-m1-pro',
    startTime: new Date(Date.now() - 1000 * 60 * 45),
    estimatedDuration: 120,
    progress: 67,
    resourceUsage: {
      gpu: 87,
      cpu: 45,
      memory: 62
    }
  }
];

export const TrainingResourceManager: React.FC = () => {
  const [resources, setResources] = useState<ComputeResource[]>(SAMPLE_RESOURCES);
  const [jobs, setJobs] = useState<TrainingJob[]>(SAMPLE_JOBS);
  const [selectedResource, setSelectedResource] = useState<ComputeResource | null>(null);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [filterStatus, setFilterStatus] = useState<'all' | 'available' | 'busy' | 'offline'>('all');

  // Real-time updates simulation
  useEffect(() => {
    const interval = setInterval(() => {
      setResources(prev => prev.map(resource => ({
        ...resource,
        utilization: {
          ...resource.utilization,
          gpu: resource.status === 'busy' ? 
            Math.max(50, Math.min(95, resource.utilization.gpu + (Math.random() - 0.5) * 10)) : 
            Math.max(0, Math.min(20, resource.utilization.gpu + (Math.random() - 0.5) * 5)),
          cpu: resource.status === 'busy' ? 
            Math.max(30, Math.min(80, resource.utilization.cpu + (Math.random() - 0.5) * 8)) : 
            Math.max(0, Math.min(15, resource.utilization.cpu + (Math.random() - 0.5) * 3)),
          memory: resource.status === 'busy' ? 
            Math.max(40, Math.min(85, resource.utilization.memory + (Math.random() - 0.5) * 6)) : 
            Math.max(0, Math.min(25, resource.utilization.memory + (Math.random() - 0.5) * 4))
        },
        lastUpdated: new Date()
      })));

      setJobs(prev => prev.map(job => 
        job.status === 'running' ? {
          ...job,
          progress: Math.min(100, job.progress + Math.random() * 2)
        } : job
      ));
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  const filteredResources = resources.filter(resource => 
    filterStatus === 'all' || resource.status === filterStatus
  );

  const getStatusIcon = (status: ComputeResource['status']) => {
    switch (status) {
      case 'available': return <CheckCircle className="w-4 h-4 text-green-400" />;
      case 'busy': return <Activity className="w-4 h-4 text-blue-400 animate-pulse" />;
      case 'offline': return <AlertTriangle className="w-4 h-4 text-red-400" />;
      case 'maintenance': return <Clock className="w-4 h-4 text-yellow-400" />;
    }
  };

  const getStatusColor = (status: ComputeResource['status']) => {
    switch (status) {
      case 'available': return 'bg-green-900/30 border-green-500/30';
      case 'busy': return 'bg-blue-900/30 border-blue-500/30';
      case 'offline': return 'bg-red-900/30 border-red-500/30';
      case 'maintenance': return 'bg-yellow-900/30 border-yellow-500/30';
    }
  };

  const ResourceCard: React.FC<{ resource: ComputeResource }> = ({ resource }) => (
    <div 
      className={`card p-6 cursor-pointer transition-all hover:scale-105 border ${getStatusColor(resource.status)}`}
      onClick={() => setSelectedResource(resource)}
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
                  value > 80 ? 'bg-red-500' : value > 60 ? 'bg-yellow-500' : 'bg-green-500'
                }`}
                style={{ width: `${value}%` }}
              />
            </div>
            <span className="text-xs text-sis-gray-300 w-8 text-right">{value.toFixed(0)}%</span>
          </div>
        ))}
      </div>
    </div>
  );

  const JobCard: React.FC<{ job: TrainingJob }> = ({ job }) => {
    const resource = resources.find(r => r.id === job.resourceId);
    const elapsedTime = Math.floor((Date.now() - job.startTime.getTime()) / 1000 / 60);
    
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
            <span className="text-sis-gray-300">{job.progress.toFixed(1)}%</span>
          </div>
          <div className="w-full bg-sis-gray-700 rounded-full h-2">
            <div 
              className="h-2 bg-gradient-to-r from-blue-500 to-green-500 rounded-full transition-all"
              style={{ width: `${job.progress}%` }}
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
        <div>
          <h2 className="text-2xl font-bold text-white mb-2">Training Resource Manager</h2>
          <p className="text-sis-gray-400">Manage compute resources for AI model training</p>
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
          
          <button className="btn-primary px-4 py-2 flex items-center space-x-2">
            <Cloud className="w-4 h-4" />
            <span>Add Resource</span>
          </button>
        </div>
      </div>

      {/* Resource Overview */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="card p-4 text-center">
          <div className="text-2xl font-bold text-white mb-1">
            {resources.filter(r => r.status === 'available').length}
          </div>
          <div className="text-sm text-sis-gray-400">Available</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="text-2xl font-bold text-blue-400 mb-1">
            {resources.filter(r => r.status === 'busy').length}
          </div>
          <div className="text-sm text-sis-gray-400">In Use</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="text-2xl font-bold text-green-400 mb-1">
            {jobs.filter(j => j.status === 'running').length}
          </div>
          <div className="text-sm text-sis-gray-400">Active Jobs</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="text-2xl font-bold text-purple-400 mb-1">
            ${resources.filter(r => r.pricing).reduce((sum, r) => sum + (r.pricing?.costPerHour || 0), 0).toFixed(2)}
          </div>
          <div className="text-sm text-sis-gray-400">Total Cost/hr</div>
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
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setSelectedResource(null)}>
          <div className="bg-sis-gray-900 rounded-lg p-6 max-w-2xl w-full mx-4" onClick={e => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-bold text-white">{selectedResource.name}</h3>
              <button
                onClick={() => setSelectedResource(null)}
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
                    <span className="text-white">{selectedResource.capabilities.gpus} × {selectedResource.capabilities.gpuMemory}GB</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sis-gray-400">CPU Cores</span>
                    <span className="text-white">{selectedResource.capabilities.cpuCores}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sis-gray-400">System Memory</span>
                    <span className="text-white">{selectedResource.capabilities.systemMemory}GB</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sis-gray-400">Storage</span>
                    <span className="text-white">{selectedResource.capabilities.storage}GB</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sis-gray-400">Network</span>
                    <span className="text-white">{selectedResource.capabilities.bandwidth}Mbps</span>
                  </div>
                </div>
              </div>
              
              <div>
                <h4 className="text-lg font-semibold text-white mb-3">Current Utilization</h4>
                <div className="space-y-3">
                  {Object.entries(selectedResource.utilization).map(([key, value]) => (
                    <div key={key}>
                      <div className="flex justify-between text-sm mb-1">
                        <span className="text-sis-gray-400 capitalize">{key}</span>
                        <span className="text-white">{value.toFixed(1)}%</span>
                      </div>
                      <div className="w-full bg-sis-gray-700 rounded-full h-2">
                        <div 
                          className={`h-2 rounded-full ${
                            value > 80 ? 'bg-red-500' : value > 60 ? 'bg-yellow-500' : 'bg-green-500'
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
              <button className="btn-primary flex-1">Start Training Job</button>
              <button className="btn-secondary">Monitor Performance</button>
              <button className="btn-secondary">Configure</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default TrainingResourceManager;