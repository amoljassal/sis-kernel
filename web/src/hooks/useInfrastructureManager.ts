/**
 * Infrastructure Manager Hook
 * Manages cloud infrastructure state and operations
 */

import { useState, useEffect, useCallback } from 'react';
import { 
  infrastructureApi, 
  ComputeResource, 
  CloudProvider, 
  TrainingJob, 
  ResourceTemplate,
  InfrastructureMetrics
} from '../services/api/infrastructureApi';

interface UseInfrastructureManagerReturn {
  // State
  resources: ComputeResource[];
  cloudProviders: CloudProvider[];
  trainingJobs: TrainingJob[];
  resourceTemplates: ResourceTemplate[];
  metrics: InfrastructureMetrics | null;
  selectedResource: ComputeResource | null;
  loading: boolean;
  error: string | null;
  
  // Actions
  refreshData: () => Promise<void>;
  selectResource: (resource: ComputeResource | null) => void;
  
  // Cloud Provider Management
  connectCloudProvider: (provider: 'aws' | 'gcp' | 'azure', credentials: Record<string, string>) => Promise<void>;
  validateCloudProvider: (providerId: string) => Promise<boolean>;
  
  // Resource Management
  provisionResource: (template: ResourceTemplate, config: {
    name: string;
    region: string;
    tags?: Record<string, string>;
  }) => Promise<void>;
  terminateResource: (resourceId: string) => Promise<void>;
  scaleResource: (resourceId: string, action: 'scale-up' | 'scale-down' | 'scale-out' | 'scale-in') => Promise<void>;
  
  // Job Management
  scheduleTrainingJob: (job: Partial<TrainingJob>) => Promise<void>;
  cancelTrainingJob: (jobId: string) => Promise<void>;
  
  // Cost Analysis
  getCostAnalysis: (timeRange?: '1h' | '24h' | '7d' | '30d') => Promise<{
    totalCost: number;
    costBreakdown: Record<string, number>;
    trends: { timestamp: string; cost: number }[];
    predictions: { period: string; estimatedCost: number }[];
  }>;
}

export const useInfrastructureManager = (): UseInfrastructureManagerReturn => {
  const [resources, setResources] = useState<ComputeResource[]>([]);
  const [cloudProviders, setCloudProviders] = useState<CloudProvider[]>([]);
  const [trainingJobs, setTrainingJobs] = useState<TrainingJob[]>([]);
  const [resourceTemplates, setResourceTemplates] = useState<ResourceTemplate[]>([]);
  const [metrics, setMetrics] = useState<InfrastructureMetrics | null>(null);
  const [selectedResource, setSelectedResource] = useState<ComputeResource | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load all infrastructure data
  const refreshData = useCallback(async () => {
    setLoading(true);
    setError(null);
    
    try {
      const [
        resourcesData,
        providersData,
        jobsData,
        templatesData,
        metricsData
      ] = await Promise.all([
        infrastructureApi.getComputeResources(),
        infrastructureApi.getCloudProviders(),
        infrastructureApi.getTrainingJobs(),
        infrastructureApi.getResourceTemplates(),
        infrastructureApi.getInfrastructureMetrics()
      ]);

      setResources(resourcesData);
      setCloudProviders(providersData);
      setTrainingJobs(jobsData);
      setResourceTemplates(templatesData);
      setMetrics(metricsData);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load infrastructure data';
      setError(errorMessage);
      console.error('Error loading infrastructure data:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Cloud Provider Management
  const connectCloudProvider = useCallback(async (
    provider: 'aws' | 'gcp' | 'azure', 
    credentials: Record<string, string>
  ) => {
    try {
      setError(null);
      setLoading(true);
      
      const connectedProvider = await infrastructureApi.connectCloudProvider(provider, credentials);
      
      // Update local state
      setCloudProviders(prev => {
        const existingIndex = prev.findIndex(p => p.type === provider);
        if (existingIndex >= 0) {
          const updated = [...prev];
          updated[existingIndex] = connectedProvider;
          return updated;
        } else {
          return [...prev, connectedProvider];
        }
      });
      
      // Refresh resources after connecting provider
      await refreshData();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to connect cloud provider';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [refreshData]);

  const validateCloudProvider = useCallback(async (providerId: string): Promise<boolean> => {
    try {
      setError(null);
      const isValid = await infrastructureApi.validateCloudProvider(providerId);
      
      // Update provider status
      setCloudProviders(prev => prev.map(provider => 
        provider.id === providerId 
          ? { ...provider, status: isValid ? 'connected' : 'error' }
          : provider
      ));
      
      return isValid;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to validate cloud provider';
      setError(errorMessage);
      return false;
    }
  }, []);

  // Resource Management
  const provisionResource = useCallback(async (
    template: ResourceTemplate, 
    config: { name: string; region: string; tags?: Record<string, string> }
  ) => {
    try {
      setError(null);
      setLoading(true);
      
      const newResource = await infrastructureApi.provisionResource(template, config);
      
      // Add new resource to state
      setResources(prev => [...prev, newResource]);
      
      // Refresh metrics
      const updatedMetrics = await infrastructureApi.getInfrastructureMetrics();
      setMetrics(updatedMetrics);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to provision resource';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const terminateResource = useCallback(async (resourceId: string) => {
    try {
      setError(null);
      await infrastructureApi.terminateResource(resourceId);
      
      // Update resource status locally
      setResources(prev => prev.map(resource => 
        resource.id === resourceId 
          ? { ...resource, status: 'terminating' }
          : resource
      ));
      
      // Remove after delay (simulating termination process)
      setTimeout(() => {
        setResources(prev => prev.filter(resource => resource.id !== resourceId));
        if (selectedResource?.id === resourceId) {
          setSelectedResource(null);
        }
      }, 5000);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to terminate resource';
      setError(errorMessage);
      throw err;
    }
  }, [selectedResource]);

  const scaleResource = useCallback(async (
    resourceId: string, 
    action: 'scale-up' | 'scale-down' | 'scale-out' | 'scale-in'
  ) => {
    try {
      setError(null);
      await infrastructureApi.scaleResource(resourceId, action);
      
      // Update resource status
      setResources(prev => prev.map(resource => 
        resource.id === resourceId 
          ? { ...resource, status: 'provisioning' }
          : resource
      ));
      
      // Refresh data after scaling
      setTimeout(() => refreshData(), 10000);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to scale resource';
      setError(errorMessage);
      throw err;
    }
  }, [refreshData]);

  // Job Management
  const scheduleTrainingJob = useCallback(async (job: Partial<TrainingJob>) => {
    try {
      setError(null);
      const scheduledJob = await infrastructureApi.scheduleTrainingJob(job);
      
      // Add new job to state
      setTrainingJobs(prev => [...prev, scheduledJob]);
      
      // Update resource status if specific resource targeted
      if (job.resourceId) {
        setResources(prev => prev.map(resource => 
          resource.id === job.resourceId 
            ? { ...resource, status: 'busy' }
            : resource
        ));
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to schedule training job';
      setError(errorMessage);
      throw err;
    }
  }, []);

  const cancelTrainingJob = useCallback(async (jobId: string) => {
    try {
      setError(null);
      await infrastructureApi.cancelTrainingJob(jobId);
      
      // Update job status
      setTrainingJobs(prev => prev.map(job => 
        job.id === jobId 
          ? { ...job, status: 'cancelled' }
          : job
      ));
      
      // Remove cancelled job after delay
      setTimeout(() => {
        setTrainingJobs(prev => prev.filter(job => job.id !== jobId));
      }, 3000);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to cancel training job';
      setError(errorMessage);
      throw err;
    }
  }, []);

  // Cost Analysis
  const getCostAnalysis = useCallback(async (timeRange: '1h' | '24h' | '7d' | '30d' = '24h') => {
    try {
      setError(null);
      return await infrastructureApi.getCostAnalysis(timeRange);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to get cost analysis';
      setError(errorMessage);
      throw err;
    }
  }, []);

  // Set up real-time updates
  useEffect(() => {
    const handleResourceUpdate = (event: CustomEvent) => {
      const updatedResource = event.detail as ComputeResource;
      setResources(prev => prev.map(resource => 
        resource.id === updatedResource.id ? updatedResource : resource
      ));
    };

    const handleJobUpdate = (event: CustomEvent) => {
      const updatedJob = event.detail as TrainingJob;
      setTrainingJobs(prev => prev.map(job => 
        job.id === updatedJob.id ? updatedJob : job
      ));
    };

    const handleMetricsUpdate = (event: CustomEvent) => {
      const updatedMetrics = event.detail as InfrastructureMetrics;
      setMetrics(updatedMetrics);
    };

    // Add event listeners
    window.addEventListener('infrastructure_resource_update', handleResourceUpdate as EventListener);
    window.addEventListener('infrastructure_job_update', handleJobUpdate as EventListener);
    window.addEventListener('infrastructure_metrics_update', handleMetricsUpdate as EventListener);

    return () => {
      window.removeEventListener('infrastructure_resource_update', handleResourceUpdate as EventListener);
      window.removeEventListener('infrastructure_job_update', handleJobUpdate as EventListener);
      window.removeEventListener('infrastructure_metrics_update', handleMetricsUpdate as EventListener);
    };
  }, []);

  // Load initial data on mount
  useEffect(() => {
    refreshData();
  }, [refreshData]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      infrastructureApi.cleanup();
    };
  }, []);

  return {
    resources,
    cloudProviders,
    trainingJobs,
    resourceTemplates,
    metrics,
    selectedResource,
    loading,
    error,
    refreshData,
    selectResource: setSelectedResource,
    connectCloudProvider,
    validateCloudProvider,
    provisionResource,
    terminateResource,
    scaleResource,
    scheduleTrainingJob,
    cancelTrainingJob,
    getCostAnalysis
  };
};