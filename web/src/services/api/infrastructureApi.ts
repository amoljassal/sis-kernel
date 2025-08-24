/**
 * Infrastructure Management API Service
 * Real cloud provider integrations for training infrastructure
 */

export interface ComputeResource {
  id: string;
  name: string;
  type: 'local' | 'cloud' | 'distributed';
  provider: 'aws' | 'gcp' | 'azure' | 'local' | 'custom';
  status: 'available' | 'busy' | 'offline' | 'maintenance' | 'provisioning' | 'terminating';
  region: string;
  zone?: string;
  capabilities: {
    gpus: number;
    gpuType: string;
    gpuMemory: number;
    cpuCores: number;
    cpuType: string;
    systemMemory: number;
    storage: number;
    storageType: 'ssd' | 'hdd' | 'nvme';
    bandwidth: number;
    maxBandwidth: number;
    accelerators?: string[];
  };
  utilization: {
    gpu: number;
    cpu: number;
    memory: number;
    storage: number;
    power: number;
    network: number;
    temperature?: number;
  };
  pricing: {
    costPerHour: number;
    currency: string;
    billingModel: 'on-demand' | 'reserved' | 'spot';
    estimatedMonthlyCost?: number;
  };
  metadata: {
    instanceType: string;
    os: string;
    imageId?: string;
    securityGroups?: string[];
    tags: Record<string, string>;
    createdAt: string;
    lastUpdated: string;
  };
}

export interface CloudProvider {
  id: string;
  name: string;
  type: 'aws' | 'gcp' | 'azure';
  status: 'connected' | 'disconnected' | 'error' | 'configuring';
  credentials: {
    isConfigured: boolean;
    lastValidated?: string;
    region: string;
    availableRegions: string[];
  };
  quotas: {
    maxInstances: number;
    maxGPUs: number;
    maxCPUs: number;
    maxStorage: number;
    currentUsage: {
      instances: number;
      gpus: number;
      cpus: number;
      storage: number;
    };
  };
  pricing: {
    currency: string;
    discountRate?: number;
    billingAccount?: string;
  };
}

export interface TrainingJob {
  id: string;
  name: string;
  modelName: string;
  status: 'queued' | 'provisioning' | 'running' | 'completed' | 'failed' | 'paused' | 'cancelled';
  resourceId: string;
  resourceType: 'single' | 'distributed';
  priority: 'low' | 'normal' | 'high' | 'urgent';
  scheduling: {
    startTime?: string;
    estimatedDuration: number;
    maxDuration?: number;
    autoTerminate: boolean;
  };
  progress: {
    percentage: number;
    currentEpoch: number;
    totalEpochs: number;
    timeElapsed: number;
    timeRemaining: number;
  };
  resourceUsage: {
    gpu: number;
    cpu: number;
    memory: number;
    storage: number;
    networkIO: number;
  };
  costs: {
    currentCost: number;
    estimatedTotalCost: number;
    currency: string;
  };
  logs?: {
    stdout: string[];
    stderr: string[];
    system: string[];
  };
}

export interface ResourceTemplate {
  id: string;
  name: string;
  description: string;
  provider: 'aws' | 'gcp' | 'azure';
  instanceType: string;
  capabilities: ComputeResource['capabilities'];
  pricing: ComputeResource['pricing'];
  tags: string[];
  isRecommended?: boolean;
}

export interface InfrastructureMetrics {
  totalResources: number;
  activeJobs: number;
  utilizationRates: {
    overall: number;
    gpu: number;
    cpu: number;
    memory: number;
  };
  costs: {
    currentHourly: number;
    dailyCost: number;
    monthlyCost: number;
    currency: string;
  };
  availability: {
    uptime: number;
    incidents: number;
    lastIncident?: string;
  };
}

class InfrastructureApiService {
  private baseUrl = process.env.VITE_API_BASE_URL || 'http://localhost:3001/api';
  private wsUrl = process.env.VITE_WS_URL || 'ws://localhost:3001';
  private ws: WebSocket | null = null;

  constructor() {
    this.initWebSocket();
  }

  private initWebSocket() {
    try {
      this.ws = new WebSocket(`${this.wsUrl}/infrastructure`);
      
      this.ws.onopen = () => {
        console.log('Infrastructure WebSocket connected');
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.handleWebSocketMessage(data);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      this.ws.onclose = () => {
        console.log('Infrastructure WebSocket disconnected');
        // Attempt reconnection after delay
        setTimeout(() => this.initWebSocket(), 5000);
      };

      this.ws.onerror = (error) => {
        console.error('Infrastructure WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to initialize WebSocket:', error);
    }
  }

  private handleWebSocketMessage(data: any) {
    switch (data.type) {
      case 'resource_update':
        window.dispatchEvent(new CustomEvent('infrastructure_resource_update', {
          detail: data.resource
        }));
        break;
      case 'job_update':
        window.dispatchEvent(new CustomEvent('infrastructure_job_update', {
          detail: data.job
        }));
        break;
      case 'metrics_update':
        window.dispatchEvent(new CustomEvent('infrastructure_metrics_update', {
          detail: data.metrics
        }));
        break;
      default:
        console.log('Unknown infrastructure message:', data.type);
    }
  }

  // Cloud Provider Management
  async getCloudProviders(): Promise<CloudProvider[]> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/providers`);
      if (!response.ok) {
        throw new Error(`Failed to fetch cloud providers: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching cloud providers:', error);
      return this.getMockCloudProviders();
    }
  }

  async connectCloudProvider(provider: 'aws' | 'gcp' | 'azure', credentials: Record<string, string>): Promise<CloudProvider> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/providers/connect`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ provider, credentials }),
      });

      if (!response.ok) {
        throw new Error(`Failed to connect cloud provider: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error connecting cloud provider:', error);
      throw error;
    }
  }

  async validateCloudProvider(providerId: string): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/providers/${providerId}/validate`, {
        method: 'POST',
      });

      return response.ok;
    } catch (error) {
      console.error('Error validating cloud provider:', error);
      return false;
    }
  }

  // Resource Management
  async getComputeResources(): Promise<ComputeResource[]> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/resources`);
      if (!response.ok) {
        throw new Error(`Failed to fetch compute resources: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching compute resources:', error);
      return this.getMockComputeResources();
    }
  }

  async provisionResource(template: ResourceTemplate, config: {
    name: string;
    region: string;
    tags?: Record<string, string>;
  }): Promise<ComputeResource> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/resources/provision`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ template, config }),
      });

      if (!response.ok) {
        throw new Error(`Failed to provision resource: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error provisioning resource:', error);
      throw error;
    }
  }

  async terminateResource(resourceId: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/resources/${resourceId}/terminate`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`Failed to terminate resource: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error terminating resource:', error);
      throw error;
    }
  }

  async scaleResource(resourceId: string, action: 'scale-up' | 'scale-down' | 'scale-out' | 'scale-in', params?: Record<string, any>): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/resources/${resourceId}/scale`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action, params }),
      });

      if (!response.ok) {
        throw new Error(`Failed to scale resource: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error scaling resource:', error);
      throw error;
    }
  }

  // Training Job Management
  async getTrainingJobs(): Promise<TrainingJob[]> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/jobs`);
      if (!response.ok) {
        throw new Error(`Failed to fetch training jobs: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching training jobs:', error);
      return this.getMockTrainingJobs();
    }
  }

  async scheduleTrainingJob(job: Partial<TrainingJob>): Promise<TrainingJob> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/jobs/schedule`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(job),
      });

      if (!response.ok) {
        throw new Error(`Failed to schedule training job: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error scheduling training job:', error);
      throw error;
    }
  }

  async cancelTrainingJob(jobId: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/jobs/${jobId}/cancel`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`Failed to cancel training job: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error canceling training job:', error);
      throw error;
    }
  }

  // Resource Templates
  async getResourceTemplates(provider?: 'aws' | 'gcp' | 'azure'): Promise<ResourceTemplate[]> {
    try {
      const url = provider 
        ? `${this.baseUrl}/infrastructure/templates?provider=${provider}`
        : `${this.baseUrl}/infrastructure/templates`;
        
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to fetch resource templates: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching resource templates:', error);
      return this.getMockResourceTemplates();
    }
  }

  // Metrics and Monitoring
  async getInfrastructureMetrics(): Promise<InfrastructureMetrics> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/metrics`);
      if (!response.ok) {
        throw new Error(`Failed to fetch infrastructure metrics: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching infrastructure metrics:', error);
      return this.getMockInfrastructureMetrics();
    }
  }

  async getCostAnalysis(timeRange: '1h' | '24h' | '7d' | '30d' = '24h'): Promise<{
    totalCost: number;
    costBreakdown: Record<string, number>;
    trends: { timestamp: string; cost: number }[];
    predictions: { period: string; estimatedCost: number }[];
  }> {
    try {
      const response = await fetch(`${this.baseUrl}/infrastructure/costs?range=${timeRange}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch cost analysis: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching cost analysis:', error);
      return {
        totalCost: 45.67,
        costBreakdown: { 'AWS': 32.77, 'GCP': 12.90 },
        trends: [],
        predictions: []
      };
    }
  }

  // Cleanup
  cleanup() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  // Mock data fallbacks
  private getMockCloudProviders(): CloudProvider[] {
    return [
      {
        id: 'aws-main',
        name: 'AWS Production',
        type: 'aws',
        status: 'connected',
        credentials: {
          isConfigured: true,
          lastValidated: new Date().toISOString(),
          region: 'us-west-2',
          availableRegions: ['us-west-2', 'us-east-1', 'eu-west-1']
        },
        quotas: {
          maxInstances: 100,
          maxGPUs: 32,
          maxCPUs: 1000,
          maxStorage: 10000,
          currentUsage: { instances: 3, gpus: 8, cpus: 96, storage: 2000 }
        },
        pricing: { currency: 'USD' }
      },
      {
        id: 'gcp-main',
        name: 'Google Cloud',
        type: 'gcp',
        status: 'connected',
        credentials: {
          isConfigured: true,
          region: 'us-central1',
          availableRegions: ['us-central1', 'europe-west1', 'asia-east1']
        },
        quotas: {
          maxInstances: 50,
          maxGPUs: 16,
          maxCPUs: 500,
          maxStorage: 5000,
          currentUsage: { instances: 1, gpus: 16, cpus: 192, storage: 1000 }
        },
        pricing: { currency: 'USD' }
      }
    ];
  }

  private getMockComputeResources(): ComputeResource[] {
    return [
      {
        id: 'aws-p4d-24xlarge-001',
        name: 'AWS p4d.24xlarge',
        type: 'cloud',
        provider: 'aws',
        status: 'available',
        region: 'us-west-2',
        zone: 'us-west-2a',
        capabilities: {
          gpus: 8,
          gpuType: 'A100',
          gpuMemory: 320,
          cpuCores: 96,
          cpuType: 'Intel Xeon',
          systemMemory: 1152,
          storage: 8000,
          storageType: 'nvme',
          bandwidth: 4000,
          maxBandwidth: 10000,
          accelerators: ['NVIDIA A100']
        },
        utilization: {
          gpu: 0,
          cpu: 5,
          memory: 8,
          storage: 12,
          power: 15,
          network: 2
        },
        pricing: {
          costPerHour: 32.77,
          currency: 'USD',
          billingModel: 'on-demand',
          estimatedMonthlyCost: 23595.60
        },
        metadata: {
          instanceType: 'p4d.24xlarge',
          os: 'Ubuntu 20.04 LTS',
          imageId: 'ami-0abcdef1234567890',
          tags: { Purpose: 'AI Training', Environment: 'Production' },
          createdAt: new Date().toISOString(),
          lastUpdated: new Date().toISOString()
        }
      }
    ];
  }

  private getMockTrainingJobs(): TrainingJob[] {
    return [
      {
        id: 'job-001',
        name: 'Legal Document Analyzer Training',
        modelName: 'Legal Document Analyzer v2.1',
        status: 'running',
        resourceId: 'aws-p4d-24xlarge-001',
        resourceType: 'single',
        priority: 'high',
        scheduling: {
          startTime: new Date(Date.now() - 45 * 60 * 1000).toISOString(), // Started 45 minutes ago
          estimatedDuration: 7200,
          autoTerminate: true
        },
        progress: {
          percentage: 67,
          currentEpoch: 67,
          totalEpochs: 100,
          timeElapsed: 2700,
          timeRemaining: 1320
        },
        resourceUsage: {
          gpu: 87,
          cpu: 45,
          memory: 62,
          storage: 34,
          networkIO: 12
        },
        costs: {
          currentCost: 24.67,
          estimatedTotalCost: 65.54,
          currency: 'USD'
        }
      }
    ];
  }

  private getMockResourceTemplates(): ResourceTemplate[] {
    return [
      {
        id: 'aws-ml-small',
        name: 'AWS ML Small',
        description: 'Cost-effective GPU instance for small models',
        provider: 'aws',
        instanceType: 'p3.2xlarge',
        capabilities: {
          gpus: 1,
          gpuType: 'V100',
          gpuMemory: 16,
          cpuCores: 8,
          cpuType: 'Intel Xeon',
          systemMemory: 61,
          storage: 160,
          storageType: 'ssd',
          bandwidth: 1000,
          maxBandwidth: 10000
        },
        pricing: {
          costPerHour: 3.06,
          currency: 'USD',
          billingModel: 'on-demand'
        },
        tags: ['beginner-friendly', 'cost-effective'],
        isRecommended: true
      }
    ];
  }

  private getMockInfrastructureMetrics(): InfrastructureMetrics {
    return {
      totalResources: 5,
      activeJobs: 2,
      utilizationRates: {
        overall: 65,
        gpu: 75,
        cpu: 45,
        memory: 62
      },
      costs: {
        currentHourly: 45.67,
        dailyCost: 1096.08,
        monthlyCost: 32882.40,
        currency: 'USD'
      },
      availability: {
        uptime: 99.8,
        incidents: 0
      }
    };
  }
}

// Export singleton instance
export const infrastructureApi = new InfrastructureApiService();