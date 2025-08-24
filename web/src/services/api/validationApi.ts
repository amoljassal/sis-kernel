/**
 * Model Validation API Service
 * Real API integration for model testing and validation
 */

export interface PerformanceMetrics {
  accuracy: number;
  precision: number;
  recall: number;
  f1Score: number;
  latency: number;
  throughput: number;
  memoryUsage: number;
  energyEfficiency: number;
}

export interface ValidationTest {
  id: string;
  name: string;
  description: string;
  status: 'pending' | 'running' | 'passed' | 'failed' | 'warning';
  duration: number;
  category: 'accuracy' | 'performance' | 'robustness' | 'bias' | 'security';
  score?: number;
  details?: string;
  progress?: number;
  startTime?: string;
  endTime?: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  version: string;
  architecture: string;
  parameters: string;
  datasetSize: string;
  trainingTime: string;
  lastValidated: string;
  status: 'active' | 'deprecated' | 'archived';
}

export interface TestSuite {
  id: string;
  name: string;
  description: string;
  tests: ValidationTest[];
  modelId: string;
  createdAt: string;
  completedAt?: string;
  overallScore?: number;
}

export interface TestConfiguration {
  testIds: string[];
  datasetId?: string;
  batchSize?: number;
  timeout?: number;
  customParameters?: Record<string, any>;
}

export interface TestResult {
  testId: string;
  modelId: string;
  status: ValidationTest['status'];
  score?: number;
  metrics?: Record<string, number>;
  details?: string;
  artifacts?: {
    confusion_matrix?: number[][];
    roc_curve?: { fpr: number[]; tpr: number[]; auc: number };
    error_analysis?: { errors: Array<{ input: string; expected: string; actual: string; confidence: number }> };
    performance_breakdown?: Record<string, number>;
  };
}

class ValidationApiService {
  private baseUrl = process.env.VITE_API_BASE_URL || 'http://localhost:3001/api';
  private wsUrl = process.env.VITE_WS_URL || 'ws://localhost:3001';
  
  // WebSocket for real-time test progress
  private ws: WebSocket | null = null;
  private progressCallbacks: Map<string, (progress: number, test: ValidationTest) => void> = new Map();

  constructor() {
    this.initWebSocket();
  }

  private initWebSocket() {
    try {
      this.ws = new WebSocket(`${this.wsUrl}/validation`);
      
      this.ws.onopen = () => {
        console.log('Validation WebSocket connected');
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
        console.log('Validation WebSocket disconnected');
        // Attempt reconnection after delay
        setTimeout(() => this.initWebSocket(), 5000);
      };

      this.ws.onerror = (error) => {
        console.error('Validation WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to initialize WebSocket:', error);
    }
  }

  private handleWebSocketMessage(data: any) {
    switch (data.type) {
      case 'test_progress':
        const callback = this.progressCallbacks.get(data.testId);
        if (callback) {
          callback(data.progress, data.test);
        }
        // Emit custom event for components to listen
        window.dispatchEvent(new CustomEvent('validation_progress', {
          detail: { testId: data.testId, progress: data.progress, test: data.test }
        }));
        break;
      case 'test_complete':
        window.dispatchEvent(new CustomEvent('validation_complete', {
          detail: { testId: data.testId, result: data.result }
        }));
        break;
      default:
        console.log('Unknown validation WebSocket message:', data.type);
    }
  }

  /**
   * Get all available models for testing
   */
  async getModels(): Promise<ModelInfo[]> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/models`);
      if (!response.ok) {
        throw new Error(`Failed to fetch models: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching models:', error);
      return this.getMockModels();
    }
  }

  /**
   * Get model information by ID
   */
  async getModelInfo(modelId: string): Promise<ModelInfo> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/models/${modelId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch model info: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching model info:', error);
      return this.getMockModels()[0];
    }
  }

  /**
   * Get available validation tests
   */
  async getAvailableTests(): Promise<ValidationTest[]> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/tests`);
      if (!response.ok) {
        throw new Error(`Failed to fetch tests: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching tests:', error);
      return this.getMockTests();
    }
  }

  /**
   * Start validation tests for a model
   */
  async startValidation(modelId: string, config: TestConfiguration): Promise<TestSuite> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/start`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ modelId, config }),
      });

      if (!response.ok) {
        throw new Error(`Failed to start validation: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error starting validation:', error);
      throw error;
    }
  }

  /**
   * Get test suite status
   */
  async getTestSuite(suiteId: string): Promise<TestSuite> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/suites/${suiteId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch test suite: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching test suite:', error);
      throw error;
    }
  }

  /**
   * Get test results with detailed analysis
   */
  async getTestResults(testId: string): Promise<TestResult> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/results/${testId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch test results: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching test results:', error);
      throw error;
    }
  }

  /**
   * Cancel a running test
   */
  async cancelTest(testId: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/cancel/${testId}`, {
        method: 'POST',
      });
      if (!response.ok) {
        throw new Error(`Failed to cancel test: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error canceling test:', error);
      throw error;
    }
  }

  /**
   * Export validation report
   */
  async exportReport(suiteId: string, format: 'pdf' | 'json' | 'csv' = 'pdf'): Promise<Blob> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/export/${suiteId}?format=${format}`);
      if (!response.ok) {
        throw new Error(`Failed to export report: ${response.statusText}`);
      }
      return await response.blob();
    } catch (error) {
      console.error('Error exporting report:', error);
      throw error;
    }
  }

  /**
   * Get performance benchmarks for comparison
   */
  async getPerformanceBenchmarks(architecture: string): Promise<PerformanceMetrics[]> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/benchmarks?architecture=${architecture}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch benchmarks: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Error fetching benchmarks:', error);
      return [];
    }
  }

  /**
   * Run custom test with user-provided data
   */
  async runCustomTest(modelId: string, testData: {
    name: string;
    inputs: any[];
    expectedOutputs: any[];
    testType: 'accuracy' | 'performance' | 'robustness';
  }): Promise<TestResult> {
    try {
      const response = await fetch(`${this.baseUrl}/validation/custom`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ modelId, testData }),
      });

      if (!response.ok) {
        throw new Error(`Failed to run custom test: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error running custom test:', error);
      throw error;
    }
  }

  /**
   * Subscribe to test progress updates
   */
  subscribeToProgress(testId: string, callback: (progress: number, test: ValidationTest) => void): () => void {
    this.progressCallbacks.set(testId, callback);
    
    return () => {
      this.progressCallbacks.delete(testId);
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
    this.progressCallbacks.clear();
  }

  // Mock data for offline/development use
  private getMockModels(): ModelInfo[] {
    return [
      {
        id: 'model-1',
        name: 'Legal Document Analyzer',
        version: 'v2.1.3',
        architecture: 'BERT-Large + AURAG',
        parameters: '340M',
        datasetSize: '2.4M documents',
        trainingTime: '14.2 hours',
        lastValidated: '2 minutes ago',
        status: 'active'
      },
      {
        id: 'model-2',
        name: 'Medical Knowledge Assistant',
        version: 'v1.5.2',
        architecture: 'GPT-4 Fine-tuned',
        parameters: '1.2B',
        datasetSize: '850K medical records',
        trainingTime: '22.7 hours',
        lastValidated: '1 hour ago',
        status: 'active'
      }
    ];
  }

  private getMockTests(): ValidationTest[] {
    return [
      {
        id: 'accuracy-test',
        name: 'Accuracy Validation',
        description: 'Test model accuracy against benchmark dataset',
        status: 'passed',
        duration: 45,
        category: 'accuracy',
        score: 94.7,
        details: 'Exceeds target accuracy of 90%'
      },
      {
        id: 'latency-test',
        name: 'Inference Latency',
        description: 'Measure response time for single inference',
        status: 'passed',
        duration: 12,
        category: 'performance',
        score: 89.2,
        details: '< 50ms average response time'
      },
      {
        id: 'bias-test',
        name: 'Bias Detection',
        description: 'Evaluate model for demographic and cultural bias',
        status: 'warning',
        duration: 78,
        category: 'bias',
        score: 76.3,
        details: 'Minor bias detected in gender-related content'
      },
      {
        id: 'adversarial-test',
        name: 'Adversarial Robustness',
        description: 'Test resilience against adversarial attacks',
        status: 'pending',
        duration: 0,
        category: 'robustness'
      },
      {
        id: 'security-test',
        name: 'Security Validation',
        description: 'Check for potential security vulnerabilities',
        status: 'pending',
        duration: 0,
        category: 'security'
      },
      {
        id: 'memory-test',
        name: 'Memory Efficiency',
        description: 'Analyze memory usage patterns during inference',
        status: 'passed',
        duration: 23,
        category: 'performance',
        score: 91.8,
        details: 'Optimal memory utilization'
      }
    ];
  }
}

// Export singleton instance
export const validationApi = new ValidationApiService();