/**
 * Browser-compatible MLX Training Interface
 * Communicates with backend training service via API
 */

export interface TrainingConfig {
  modelName: string;
  description: string;
  datasetPath: string;
  outputPath: string;
  epochs: number;
  learningRate: number;
  batchSize: number;
  maxLength: number;
}

export interface TrainingProgress {
  epoch: number;
  loss: number;
  accuracy?: number;
  status: 'running' | 'completed' | 'failed' | 'paused';
  eta?: string;
}

export interface TrainingResult {
  success: boolean;
  modelPath?: string;
  metrics?: {
    finalLoss: number;
    finalAccuracy?: number;
    trainingTime: number;
  };
  error?: string;
}

export class BrowserMLXTrainingPipeline {
  private baseUrl: string;

  constructor(baseUrl: string = '/api/mlx') {
    this.baseUrl = baseUrl;
  }

  /**
   * Initialize MLX training environment (browser mock)
   */
  async initialize(): Promise<void> {
    try {
      // Check if backend is available
      const response = await fetch(`${this.baseUrl}/status`);
      if (!response.ok) {
        console.warn('MLX backend not available, using mock mode');
      }
    } catch (error) {
      console.warn('MLX backend not available, using mock mode');
    }
  }

  /**
   * Parse natural language training description into configuration
   */
  parseTrainingDescription(description: string): Partial<TrainingConfig> {
    const config: Partial<TrainingConfig> = {
      description,
      epochs: 10,
      learningRate: 0.001,
      batchSize: 32,
      maxLength: 512
    };

    // Extract parameters from natural language
    const epochsMatch = description.match(/(\d+)\s*epochs?/i);
    if (epochsMatch) {
      config.epochs = parseInt(epochsMatch[1]);
    }

    const lrMatch = description.match(/learning\s*rate\s*[of]?\s*([\d.e-]+)/i);
    if (lrMatch) {
      config.learningRate = parseFloat(lrMatch[1]);
    }

    const batchMatch = description.match(/batch\s*size\s*[of]?\s*(\d+)/i);
    if (batchMatch) {
      config.batchSize = parseInt(batchMatch[1]);
    }

    const lengthMatch = description.match(/max\s*length\s*[of]?\s*(\d+)/i);
    if (lengthMatch) {
      config.maxLength = parseInt(lengthMatch[1]);
    }

    // Extract model type
    if (description.toLowerCase().includes('gpt') || description.toLowerCase().includes('language model')) {
      config.modelName = 'custom-gpt';
    } else if (description.toLowerCase().includes('classifier')) {
      config.modelName = 'classifier';
    } else if (description.toLowerCase().includes('embedding')) {
      config.modelName = 'embedding';
    } else {
      config.modelName = 'general-model';
    }

    return config;
  }

  /**
   * Start training with natural language description
   */
  async startTraining(description: string, datasetPath: string): Promise<string> {
    const config = this.parseTrainingDescription(description);
    const trainingId = `training_${Date.now()}`;
    
    const fullConfig: TrainingConfig = {
      modelName: config.modelName || 'custom-model',
      description,
      datasetPath,
      outputPath: `/models/${trainingId}`,
      epochs: config.epochs || 10,
      learningRate: config.learningRate || 0.001,
      batchSize: config.batchSize || 32,
      maxLength: config.maxLength || 512
    };

    try {
      // Try to call backend API
      const response = await fetch(`${this.baseUrl}/train`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          trainingId,
          config: fullConfig
        })
      });

      if (response.ok) {
        const result = await response.json();
        return result.trainingId;
      } else {
        throw new Error('Backend training service not available');
      }
    } catch (error) {
      // Fallback to mock mode
      console.warn('Using mock training mode:', error);
      this.startMockTraining(trainingId, fullConfig);
      return trainingId;
    }
  }

  /**
   * Mock training for demonstration when backend is not available
   */
  private startMockTraining(trainingId: string, config: TrainingConfig): void {
    let currentEpoch = 0;
    const totalEpochs = config.epochs;
    const startTime = Date.now();

    const progressKey = `training_progress_${trainingId}`;
    const resultKey = `training_result_${trainingId}`;

    const updateProgress = () => {
      currentEpoch++;
      const loss = Math.max(0.1, 2.0 - (currentEpoch / totalEpochs) * 1.8 + Math.random() * 0.2);
      const accuracy = Math.min(0.95, (currentEpoch / totalEpochs) * 0.85 + Math.random() * 0.1);
      
      const progress: TrainingProgress = {
        epoch: currentEpoch,
        loss,
        accuracy,
        status: currentEpoch < totalEpochs ? 'running' : 'completed',
        eta: currentEpoch < totalEpochs ? `${Math.round((totalEpochs - currentEpoch) * 2)}s` : undefined
      };

      localStorage.setItem(progressKey, JSON.stringify(progress));

      if (currentEpoch >= totalEpochs) {
        const result: TrainingResult = {
          success: true,
          modelPath: config.outputPath,
          metrics: {
            finalLoss: loss,
            finalAccuracy: accuracy,
            trainingTime: Date.now() - startTime
          }
        };
        localStorage.setItem(resultKey, JSON.stringify(result));
      } else {
        setTimeout(updateProgress, 2000); // Update every 2 seconds
      }
    };

    // Start mock training
    setTimeout(updateProgress, 1000);
  }

  /**
   * Get training progress
   */
  async getTrainingProgress(trainingId: string): Promise<TrainingProgress | null> {
    try {
      // Try backend first
      const response = await fetch(`${this.baseUrl}/progress/${trainingId}`);
      if (response.ok) {
        return await response.json();
      }
    } catch (error) {
      // Fallback to localStorage for mock mode
    }

    // Check localStorage for mock progress
    const progressKey = `training_progress_${trainingId}`;
    const progressData = localStorage.getItem(progressKey);
    return progressData ? JSON.parse(progressData) : null;
  }

  /**
   * Get training result
   */
  async getTrainingResult(trainingId: string): Promise<TrainingResult | null> {
    try {
      // Try backend first
      const response = await fetch(`${this.baseUrl}/result/${trainingId}`);
      if (response.ok) {
        return await response.json();
      }
    } catch (error) {
      // Fallback to localStorage for mock mode
    }

    // Check localStorage for mock result
    const resultKey = `training_result_${trainingId}`;
    const resultData = localStorage.getItem(resultKey);
    return resultData ? JSON.parse(resultData) : null;
  }

  /**
   * Stop active training
   */
  async stopTraining(trainingId: string): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/stop/${trainingId}`, {
        method: 'POST'
      });
      if (response.ok) {
        return true;
      }
    } catch (error) {
      // Mock mode - just update status
      const progressKey = `training_progress_${trainingId}`;
      const progressData = localStorage.getItem(progressKey);
      if (progressData) {
        const progress = JSON.parse(progressData);
        progress.status = 'paused';
        localStorage.setItem(progressKey, JSON.stringify(progress));
        return true;
      }
    }
    return false;
  }

  /**
   * List all training sessions
   */
  async listTrainingSessions(): Promise<string[]> {
    try {
      const response = await fetch(`${this.baseUrl}/sessions`);
      if (response.ok) {
        const result = await response.json();
        return result.sessions;
      }
    } catch (error) {
      // Mock mode - check localStorage
      const sessions: string[] = [];
      for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        if (key && key.startsWith('training_progress_')) {
          const trainingId = key.replace('training_progress_', '');
          sessions.push(trainingId);
        }
      }
      return sessions;
    }
    return [];
  }
}

export const createBrowserMLXTrainingPipeline = (baseUrl?: string): BrowserMLXTrainingPipeline => {
  return new BrowserMLXTrainingPipeline(baseUrl);
};