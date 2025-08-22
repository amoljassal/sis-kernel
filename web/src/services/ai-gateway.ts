// AI Gateway - Unified entry point for all AI services
// Implements recommendations from Gemini, ChatGPT, Claude, and Grok
// Phase 6C: Production-Grade AI Infrastructure

import { EventEmitter } from 'events';
// import Redis from 'ioredis'; // TODO: Install ioredis package
import { MockRedis } from './ai-gateway-mock';
import crypto from 'crypto';
import { modelQuantization } from './model-quantization';
import { treeSitterDebugger } from './tree-sitter-debugger';

// Circuit Breaker states
enum CircuitState {
  CLOSED = 'CLOSED',
  OPEN = 'OPEN',
  HALF_OPEN = 'HALF_OPEN'
}

// Request priority levels
enum RequestPriority {
  LOW = 'LOW',
  MEDIUM = 'MEDIUM',
  HIGH = 'HIGH',
  CRITICAL = 'CRITICAL'
}

interface AIRequest {
  id: string;
  type: 'debug' | 'generate' | 'analyze' | 'multimodal';
  input: any;
  context?: any;
  priority?: RequestPriority;
  userId: string;
  timestamp: Date;
}

interface AIResponse {
  id: string;
  requestId: string;
  result: any;
  confidence: number;
  model: string;
  latency: number;
  cached: boolean;
  cost?: number;
}

interface BatchRequest {
  request: AIRequest;
  resolve: (response: AIResponse) => void;
  reject: (error: Error) => void;
}

// Circuit Breaker implementation (Claude's recommendation)
class CircuitBreaker {
  private state: CircuitState = CircuitState.CLOSED;
  private failureCount: number = 0;
  private successCount: number = 0;
  private lastFailureTime?: Date;
  private readonly failureThreshold: number = 5;
  private readonly successThreshold: number = 3;
  private readonly timeout: number = 60000; // 60 seconds
  private readonly eventEmitter: EventEmitter;

  constructor() {
    this.eventEmitter = new EventEmitter();
  }

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    if (this.state === CircuitState.OPEN) {
      if (!this.shouldAttemptReset()) {
        throw new Error('Circuit breaker is OPEN - Service unavailable');
      }
      this.state = CircuitState.HALF_OPEN;
    }

    try {
      const result = await fn();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private shouldAttemptReset(): boolean {
    return this.lastFailureTime 
      ? Date.now() - this.lastFailureTime.getTime() > this.timeout 
      : false;
  }

  private onSuccess(): void {
    this.failureCount = 0;
    
    if (this.state === CircuitState.HALF_OPEN) {
      this.successCount++;
      if (this.successCount >= this.successThreshold) {
        this.state = CircuitState.CLOSED;
        this.successCount = 0;
        this.eventEmitter.emit('stateChange', CircuitState.CLOSED);
      }
    }
  }

  private onFailure(): void {
    this.failureCount++;
    this.lastFailureTime = new Date();
    
    if (this.failureCount >= this.failureThreshold) {
      this.state = CircuitState.OPEN;
      this.eventEmitter.emit('stateChange', CircuitState.OPEN);
    }
  }

  getState(): CircuitState {
    return this.state;
  }

  on(event: string, listener: (...args: any[]) => void): void {
    this.eventEmitter.on(event, listener);
  }
}

// Request Batcher (ChatGPT's recommendation)
class RequestBatcher {
  private queue: Map<string, BatchRequest[]> = new Map();
  private batchTimeout?: NodeJS.Timeout;
  private readonly maxBatchSize: number = 10;
  private readonly batchWindow: number = 100; // 100ms

  async add(request: AIRequest): Promise<AIResponse> {
    return new Promise((resolve, reject) => {
      const key = this.getBatchKey(request);
      
      if (!this.queue.has(key)) {
        this.queue.set(key, []);
      }
      
      this.queue.get(key)!.push({ request, resolve, reject });
      this.scheduleBatch();
    });
  }

  private getBatchKey(request: AIRequest): string {
    // Group by type and priority for batching
    return `${request.type}_${request.priority || RequestPriority.MEDIUM}`;
  }

  private scheduleBatch(): void {
    if (this.batchTimeout) return;
    
    this.batchTimeout = setTimeout(() => {
      this.processBatches();
      this.batchTimeout = undefined;
    }, this.batchWindow);
  }

  private async processBatches(): Promise<void> {
    const batches = Array.from(this.queue.entries());
    this.queue.clear();

    for (const [key, batch] of batches) {
      if (batch.length === 0) continue;
      
      try {
        const batchToProcess = batch.splice(0, this.maxBatchSize);
        const responses = await this.processBatch(batchToProcess);
        
        batchToProcess.forEach((item, index) => {
          item.resolve(responses[index]);
        });
        
        // Re-queue remaining items
        if (batch.length > 0) {
          this.queue.set(key, batch);
          this.scheduleBatch();
        }
      } catch (error) {
        batch.forEach(item => item.reject(error as Error));
      }
    }
  }

  private async processBatch(batch: BatchRequest[]): Promise<AIResponse[]> {
    // This would call the actual AI service
    // For now, returning mock responses
    return batch.map(item => ({
      id: crypto.randomUUID(),
      requestId: item.request.id,
      result: { processed: true },
      confidence: 0.95,
      model: 'claude-3.5-sonnet',
      latency: Math.random() * 200,
      cached: false,
      cost: 0.001
    }));
  }
}

// Multi-level Cache (Gemini's recommendation)
class MultiLevelCache {
  private redis: MockRedis;
  private semanticCache: Map<string, { embedding: number[], response: AIResponse }> = new Map();
  private readonly l1TTL: number = 300; // 5 minutes
  // private readonly l2TTL: number = 3600; // 1 hour

  constructor() {
    this.redis = new MockRedis({
      host: process.env.REDIS_HOST || 'localhost',
      port: parseInt(process.env.REDIS_PORT || '6379'),
      maxRetriesPerRequest: 3
    });
  }

  async get(request: AIRequest): Promise<AIResponse | null> {
    // L1: Exact match cache
    const l1Key = this.getL1Key(request);
    const l1Result = await this.redis.get(l1Key);
    
    if (l1Result) {
      const response = JSON.parse(l1Result) as AIResponse;
      response.cached = true;
      return response;
    }

    // L2: Semantic similarity cache (Gemini's innovation)
    const embedding = await this.getEmbedding(request.input);
    const similarResponse = this.findSimilarResponse(embedding);
    
    if (similarResponse && similarResponse.confidence > 0.85) {
      similarResponse.cached = true;
      return similarResponse;
    }

    return null;
  }

  async set(request: AIRequest, response: AIResponse): Promise<void> {
    // L1: Store exact match
    const l1Key = this.getL1Key(request);
    await this.redis.setex(l1Key, this.l1TTL, JSON.stringify(response));

    // L2: Store semantic embedding
    const embedding = await this.getEmbedding(request.input);
    this.semanticCache.set(request.id, { embedding, response });
    
    // Cleanup old semantic cache entries
    if (this.semanticCache.size > 1000) {
      const firstKey = this.semanticCache.keys().next().value!;
      this.semanticCache.delete(firstKey);
    }
  }

  private getL1Key(request: AIRequest): string {
    const hash = crypto.createHash('sha256')
      .update(JSON.stringify({
        type: request.type,
        input: request.input,
        context: request.context
      }))
      .digest('hex');
    return `ai:cache:${hash}`;
  }

  private async getEmbedding(_input: any): Promise<number[]> {
    // This would call an embedding service
    // For now, returning mock embedding
    return Array(768).fill(0).map(() => Math.random());
  }

  private findSimilarResponse(embedding: number[]): AIResponse | null {
    let bestMatch: AIResponse | null = null;
    let bestSimilarity = 0;

    for (const [, cached] of this.semanticCache) {
      const similarity = this.cosineSimilarity(embedding, cached.embedding);
      if (similarity > bestSimilarity && similarity > 0.85) {
        bestSimilarity = similarity;
        bestMatch = { ...cached.response };
      }
    }

    if (bestMatch) {
      bestMatch.confidence = bestSimilarity;
    }

    return bestMatch;
  }

  private cosineSimilarity(a: number[], b: number[]): number {
    let dotProduct = 0;
    let normA = 0;
    let normB = 0;
    
    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }
    
    return dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
  }

  async cleanup(): Promise<void> {
    await this.redis.quit();
  }
}

// Model Router (Grok's recommendation)
class ModelRouter {
  private readonly models = {
    simple: {
      name: 'claude-3-haiku',
      costPerToken: 0.00001,
      maxTokens: 1024,
      latency: 100
    },
    medium: {
      name: 'claude-3.5-sonnet',
      costPerToken: 0.00003,
      maxTokens: 8192,
      latency: 300
    },
    complex: {
      name: 'claude-3-opus',
      costPerToken: 0.00015,
      maxTokens: 32768,
      latency: 1000
    }
  };

  selectModel(request: AIRequest): typeof this.models[keyof typeof this.models] {
    // Grok's intelligent routing based on complexity
    const complexity = this.assessComplexity(request);
    
    if (complexity < 0.3) return this.models.simple;
    if (complexity < 0.7) return this.models.medium;
    return this.models.complex;
  }

  private assessComplexity(request: AIRequest): number {
    let complexity = 0;

    // Type-based complexity
    const typeComplexity: Record<string, number> = {
      debug: 0.2,
      analyze: 0.5,
      generate: 0.7,
      multimodal: 0.9
    };
    complexity += typeComplexity[request.type] || 0.5;

    // Context size
    if (request.context) {
      const contextSize = JSON.stringify(request.context).length;
      complexity += Math.min(contextSize / 10000, 0.3);
    }

    // Input size
    const inputSize = JSON.stringify(request.input).length;
    complexity += Math.min(inputSize / 5000, 0.2);

    return Math.min(complexity, 1.0);
  }
}

// Main AI Gateway class
export class AIGateway {
  private circuitBreakers: Map<string, CircuitBreaker> = new Map();
  private requestBatcher: RequestBatcher;
  private cache: MultiLevelCache;
  private modelRouter: ModelRouter;
  private metrics: {
    totalRequests: number;
    cacheHits: number;
    averageLatency: number;
    totalCost: number;
    errorRate: number;
  };

  constructor() {
    this.requestBatcher = new RequestBatcher();
    this.cache = new MultiLevelCache();
    this.modelRouter = new ModelRouter();
    this.metrics = {
      totalRequests: 0,
      cacheHits: 0,
      averageLatency: 0,
      totalCost: 0,
      errorRate: 0
    };

    // Initialize circuit breakers for each model
    ['simple', 'medium', 'complex'].forEach(level => {
      const breaker = new CircuitBreaker();
      breaker.on('stateChange', (state) => {
        console.log(`Circuit breaker for ${level} model changed to ${state}`);
      });
      this.circuitBreakers.set(level, breaker);
    });
  }

  async processRequest(request: AIRequest): Promise<AIResponse> {
    const startTime = Date.now();
    this.metrics.totalRequests++;

    try {
      // Check cache first
      const cached = await this.cache.get(request);
      if (cached) {
        this.metrics.cacheHits++;
        this.updateMetrics(Date.now() - startTime, 0, false);
        return cached;
      }

      // Select optimal model
      const model = this.modelRouter.selectModel(request);
      const complexity = model === this.modelRouter['models'].simple ? 'simple' :
                        model === this.modelRouter['models'].medium ? 'medium' : 'complex';
      
      // Get circuit breaker for this model
      const circuitBreaker = this.circuitBreakers.get(complexity)!;

      // Process through circuit breaker
      const response = await circuitBreaker.execute(async () => {
        // Batch the request
        const batchedResponse = await this.requestBatcher.add(request);
        
        // Enhance response with model info
        batchedResponse.model = model.name;
        batchedResponse.cost = (model.costPerToken * 1000); // Mock cost calculation
        
        return batchedResponse;
      });

      // Cache the response
      await this.cache.set(request, response);

      // Update metrics
      const latency = Date.now() - startTime;
      this.updateMetrics(latency, response.cost || 0, false);

      return response;

    } catch (error) {
      this.updateMetrics(Date.now() - startTime, 0, true);
      throw error;
    }
  }

  private updateMetrics(latency: number, cost: number, isError: boolean): void {
    // Update running average latency
    this.metrics.averageLatency = 
      (this.metrics.averageLatency * (this.metrics.totalRequests - 1) + latency) / 
      this.metrics.totalRequests;
    
    this.metrics.totalCost += cost;
    
    if (isError) {
      this.metrics.errorRate = 
        (this.metrics.errorRate * (this.metrics.totalRequests - 1) + 1) / 
        this.metrics.totalRequests;
    }
  }

  getMetrics() {
    return {
      ...this.metrics,
      cacheHitRate: this.metrics.totalRequests > 0 
        ? (this.metrics.cacheHits / this.metrics.totalRequests) * 100 
        : 0
    };
  }

  async processMultimodal(input: {
    text?: string;
    voice?: ArrayBuffer;
    sketch?: HTMLCanvasElement;
  }): Promise<AIResponse> {
    // Grok's multimodal fusion approach with quantization
    const fusedRequest: AIRequest = {
      id: crypto.randomUUID(),
      type: 'multimodal',
      input: {
        text: input.text,
        voice: input.voice ? await this.processVoice(input.voice) : null,
        sketch: input.sketch ? await this.processSketch(input.sketch) : null
      },
      userId: 'system',
      timestamp: new Date(),
      priority: RequestPriority.HIGH
    };

    return this.processRequest(fusedRequest);
  }

  // Grok's Tree-sitter integration for client-side debugging
  async debugCode(code: string, language: string): Promise<AIResponse> {
    const startTime = Date.now();
    
    try {
      // Use Tree-sitter for fast client-side analysis
      const debugResult = await treeSitterDebugger.analyzeCode(code, language, {
        includeOptimizations: true,
        maxAnalysisTime: 10 // 10ms target
      });

      // If Tree-sitter analysis is sufficient, return immediately
      if (debugResult.analysisTime < 10 && debugResult.syntaxErrors.length === 0) {
        return {
          id: crypto.randomUUID(),
          requestId: crypto.randomUUID(),
          result: debugResult,
          confidence: 0.95,
          model: 'tree-sitter-local',
          latency: Date.now() - startTime,
          cached: false,
          cost: 0 // Free local analysis
        };
      }

      // For complex cases, fall back to AI processing with quantized models
      const request: AIRequest = {
        id: crypto.randomUUID(),
        type: 'debug',
        input: { code, language, localAnalysis: debugResult },
        userId: 'system',
        timestamp: new Date(),
        priority: RequestPriority.HIGH
      };

      const aiResponse = await this.processRequest(request);
      
      // Enhance AI response with local analysis
      aiResponse.result = {
        ...aiResponse.result,
        localAnalysis: debugResult,
        hybridAnalysis: true
      };

      return aiResponse;

    } catch (error) {
      console.error('Code debugging failed:', error);
      throw error;
    }
  }

  // Quantized model processing with Grok's optimization
  async processWithQuantization(request: AIRequest): Promise<AIResponse> {
    const startTime = Date.now();
    
    try {
      // Determine optimal model based on request complexity
      const model = this.modelRouter.selectModel(request);
      const modelId = model.name;
      const task = this.getTaskFromRequestType(request.type);

      // Use quantized model for 40% faster inference
      const result = await modelQuantization.processWithQuantizedModel(
        modelId,
        task,
        request.input
      );

      return {
        id: crypto.randomUUID(),
        requestId: request.id,
        result: result.result,
        confidence: 0.92, // Slightly lower due to quantization
        model: `${result.model}-quantized`,
        latency: Date.now() - startTime,
        cached: false,
        cost: result.cost || model.costPerToken * 1000
      };

    } catch (error) {
      console.error('Quantized processing failed:', error);
      throw error;
    }
  }

  private getTaskFromRequestType(type: string): string {
    const taskMapping = {
      debug: 'code-analysis',
      generate: 'text-generation',
      analyze: 'classification',
      multimodal: 'text-generation'
    };
    
    return taskMapping[type as keyof typeof taskMapping] || 'text-generation';
  }

  private async processVoice(_audio: ArrayBuffer): Promise<string> {
    // Process voice with Whisper (mock)
    return 'Transcribed audio text';
  }

  private async processSketch(_canvas: HTMLCanvasElement): Promise<string> {
    // Process sketch with computer vision (mock)
    return 'Analyzed sketch description';
  }

  async cleanup(): Promise<void> {
    await this.cache.cleanup();
    await modelQuantization.cleanup();
    await treeSitterDebugger.cleanup();
  }
}

// Export types and classes
export type {
  AIRequest,
  AIResponse,
  RequestPriority,
  CircuitBreaker,
  RequestBatcher,
  MultiLevelCache,
  ModelRouter
};