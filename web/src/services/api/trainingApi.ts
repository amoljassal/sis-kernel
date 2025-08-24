/**
 * Training API Service
 * Real API integration for training operations
 */

export interface TrainingSession {
  id: string;
  modelName: string;
  type: 'training' | 'fine-tuning' | 'evaluation';
  status: 'queued' | 'running' | 'completed' | 'failed' | 'paused';
  progress: number;
  startTime: string;
  estimatedCompletion?: string;
  metrics: {
    loss: number;
    accuracy: number;
    epoch: number;
    totalEpochs: number;
    learningRate: number;
  };
  resources: {
    gpu: number;
    memory: number;
    neuralEngine: number;
  };
}

export interface ModelStats {
  totalModels: number;
  activeTraining: number;
  completedToday: number;
  failedToday: number;
  averageAccuracy: number;
  totalTrainingHours: number;
}

export interface ComputeResource {
  type: 'gpu' | 'cpu' | 'neural-engine';
  name: string;
  usage: number;
  temperature: number;
  memory: number;
  power: number;
}

export interface TrainingParameters {
  modelName: string;
  architecture: string;
  dataset: string;
  epochs: number;
  batchSize: number;
  learningRate: number;
  optimizer: 'adam' | 'sgd' | 'adamw';
  schedulerType?: 'cosine' | 'linear' | 'exponential';
  warmupSteps?: number;
  weightDecay?: number;
  gradientClipping?: number;
}

export interface TrainingActivityEvent {
  id: string;
  type: 'training_started' | 'training_completed' | 'training_failed' | 'model_queued' | 'dataset_updated';
  modelName: string;
  message: string;
  timestamp: string;
  details?: Record<string, any>;
}

class TrainingApiService {
  private baseUrl = process.env.VITE_API_BASE_URL || 'http://localhost:3001/api';
  private wsUrl = process.env.VITE_WS_URL || 'ws://localhost:3001';
  private eventListeners: ((event: TrainingActivityEvent) => void)[] = [];

  // WebSocket connection for real-time updates
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  constructor() {
    this.initWebSocket();
  }

  private initWebSocket() {
    try {
      this.ws = new WebSocket(`${this.wsUrl}/training`);
      
      this.ws.onopen = () => {
        console.log('Training WebSocket connected');
        this.reconnectAttempts = 0;
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
        console.log('Training WebSocket disconnected');
        this.handleReconnection();
      };

      this.ws.onerror = (error) => {
        console.error('Training WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to initialize WebSocket:', error);
      this.handleReconnection();
    }
  }

  private handleWebSocketMessage(data: any) {
    switch (data.type) {
      case 'training_update':
        // Handle real-time training updates
        this.notifySessionUpdate(data.session);
        break;
      case 'activity_event':
        // Handle activity timeline events
        this.eventListeners.forEach(listener => listener(data.event));
        break;
      case 'resource_update':
        // Handle compute resource updates
        this.notifyResourceUpdate(data.resources);
        break;
      default:
        console.log('Unknown WebSocket message type:', data.type);
    }
  }

  private handleReconnection() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
      
      setTimeout(() => {
        console.log(`Attempting WebSocket reconnection (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
        this.initWebSocket();
      }, delay);
    } else {
      console.error('Max WebSocket reconnection attempts reached');
    }
  }

  private notifySessionUpdate(session: TrainingSession) {
    // Emit custom events for session updates
    window.dispatchEvent(new CustomEvent('training_session_update', {
      detail: session
    }));
  }

  private notifyResourceUpdate(resources: ComputeResource[]) {
    window.dispatchEvent(new CustomEvent('compute_resource_update', {
      detail: resources
    }));
  }

  // API Methods

  /**
   * Get all training sessions
   */
  async getTrainingSessions(): Promise<TrainingSession[]> {
    try {
      const response = await fetch(`${this.baseUrl}/training/sessions`);
      if (!response.ok) {
        throw new Error(`Failed to fetch training sessions: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching training sessions:', error);
      // Return fallback mock data if API is unavailable
      return this.getMockTrainingSessions();
    }
  }

  /**
   * Get model statistics
   */
  async getModelStats(): Promise<ModelStats> {
    try {
      const response = await fetch(`${this.baseUrl}/training/stats`);
      if (!response.ok) {
        throw new Error(`Failed to fetch model stats: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching model stats:', error);
      return this.getMockModelStats();
    }
  }

  /**
   * Get compute resource usage
   */
  async getComputeResources(): Promise<ComputeResource[]> {
    try {
      const response = await fetch(`${this.baseUrl}/training/resources`);
      if (!response.ok) {
        throw new Error(`Failed to fetch compute resources: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching compute resources:', error);
      return this.getMockComputeResources();
    }
  }

  /**
   * Start a new training session
   */
  async startTraining(parameters: TrainingParameters): Promise<TrainingSession> {
    try {
      const response = await fetch(`${this.baseUrl}/training/start`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(parameters),
      });

      if (!response.ok) {
        throw new Error(`Failed to start training: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error starting training:', error);
      throw error;
    }
  }

  /**
   * Pause a training session
   */
  async pauseTraining(sessionId: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/training/${sessionId}/pause`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`Failed to pause training: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error pausing training:', error);
      throw error;
    }
  }

  /**
   * Stop a training session
   */
  async stopTraining(sessionId: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/training/${sessionId}/stop`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`Failed to stop training: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error stopping training:', error);
      throw error;
    }
  }

  /**
   * Resume a paused training session
   */
  async resumeTraining(sessionId: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/training/${sessionId}/resume`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`Failed to resume training: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error resuming training:', error);
      throw error;
    }
  }

  /**
   * Get training activity timeline
   */
  async getTrainingActivity(): Promise<TrainingActivityEvent[]> {
    try {
      const response = await fetch(`${this.baseUrl}/training/activity`);
      if (!response.ok) {
        throw new Error(`Failed to fetch training activity: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching training activity:', error);
      return this.getMockActivityEvents();
    }
  }

  /**
   * Export training metrics
   */
  async exportMetrics(sessionId: string, format: 'json' | 'csv' | 'pdf' = 'json'): Promise<Blob> {
    try {
      const response = await fetch(`${this.baseUrl}/training/${sessionId}/export?format=${format}`);
      if (!response.ok) {
        throw new Error(`Failed to export metrics: ${response.statusText}`);
      }
      return await response.blob();
    } catch (error) {
      console.error('Error exporting metrics:', error);
      throw error;
    }
  }

  /**
   * Subscribe to activity events
   */
  subscribeToActivity(callback: (event: TrainingActivityEvent) => void) {
    this.eventListeners.push(callback);
    
    // Return unsubscribe function
    return () => {
      const index = this.eventListeners.indexOf(callback);
      if (index > -1) {
        this.eventListeners.splice(index, 1);
      }
    };
  }

  /**
   * Cleanup WebSocket connection
   */
  cleanup() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.eventListeners.length = 0;
  }

  // Mock data fallbacks for offline/development use
  private getMockTrainingSessions(): TrainingSession[] {
    return [
      {
        id: 'session-1',
        modelName: 'Legal Document Analyzer v2',
        type: 'fine-tuning',
        status: 'running',
        progress: 67,
        startTime: new Date(Date.now() - 3600000).toISOString(),
        estimatedCompletion: new Date(Date.now() + 1800000).toISOString(),
        metrics: {
          loss: 0.342,
          accuracy: 0.923,
          epoch: 67,
          totalEpochs: 100,
          learningRate: 0.0001
        },
        resources: {
          gpu: 78,
          memory: 65,
          neuralEngine: 82
        }
      },
      {
        id: 'session-2',
        modelName: 'Medical Knowledge Assistant',
        type: 'training',
        status: 'queued',
        progress: 0,
        startTime: new Date(Date.now() + 1800000).toISOString(),
        metrics: {
          loss: 0,
          accuracy: 0,
          epoch: 0,
          totalEpochs: 50,
          learningRate: 0.001
        },
        resources: {
          gpu: 0,
          memory: 0,
          neuralEngine: 0
        }
      }
    ];
  }

  private getMockModelStats(): ModelStats {
    return {
      totalModels: 42,
      activeTraining: 1,
      completedToday: 7,
      failedToday: 1,
      averageAccuracy: 0.934,
      totalTrainingHours: 156
    };
  }

  private getMockComputeResources(): ComputeResource[] {
    return [
      {
        type: 'neural-engine',
        name: 'Apple Neural Engine',
        usage: 82,
        temperature: 72,
        memory: 65,
        power: 45
      },
      {
        type: 'gpu',
        name: 'M3 Max GPU',
        usage: 78,
        temperature: 68,
        memory: 71,
        power: 62
      },
      {
        type: 'cpu',
        name: 'M3 Max CPU',
        usage: 34,
        temperature: 58,
        memory: 42,
        power: 28
      }
    ];
  }

  private getMockActivityEvents(): TrainingActivityEvent[] {
    return [
      {
        id: 'event-1',
        type: 'training_started',
        modelName: 'Legal Document Analyzer v2',
        message: 'Fine-tuning initiated with 10,000 legal documents dataset',
        timestamp: new Date(Date.now() - 3600000).toISOString(),
        details: { epochs: 100, batchSize: 32 }
      },
      {
        id: 'event-2',
        type: 'training_completed',
        modelName: 'Code Generation Model',
        message: 'Achieved 95.7% accuracy on test dataset with 0.156 loss',
        timestamp: new Date(Date.now() - 7200000).toISOString(),
        details: { accuracy: 0.957, loss: 0.156 }
      }
    ];
  }
}

// Export singleton instance
export const trainingApi = new TrainingApiService();