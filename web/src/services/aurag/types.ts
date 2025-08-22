/**
 * SIS AURAG Type Definitions
 * Core types for the Unified RAG system
 */

// ====== Core Response Types ======

export interface RAGResponse {
    responseText: string;
    confidenceScore: number;
    contextItems: ContextItem[];
    philosophicalLens: string;
    processingTimeMs: number;
    sourcesUsed: string[];
    queryId?: number;
}

export interface DocumentProcessingResult {
    documentId: number;
    chunksCreated: number;
    conceptsExtracted: number;
    processingTimeMs: number;
    success: boolean;
    errorMessage?: string;
}

// ====== Context and Scoring Types ======

export interface ContextItem {
    kind: string; // "memory", "entity", "relationship", "document"
    text: string;
    score: number;
    meta: Record<string, any>;
    
    // Scoring breakdown for transparency  
    relevanceScore?: number;
    recencyScore?: number;
    priorityScore?: number;
    centralityScore?: number;
    lensBonusScore?: number;
}

export interface ContextResult {
    contextItems: ContextItem[];
    totalRelevance: number;
    processingTimeMs: number;
    cacheHit: boolean;
}

export interface ScoringWeights {
    relevance: number;
    recency: number;
    priority: number;
    centrality: number;
    lensBonus: number;
}

// ====== Confidence Scoring Types ======

export interface ConfidenceResult {
    overall: number;
    factors: {
        Sr: number; // Retrieval quality
        Sl: number; // Lens alignment  
        Sa: number; // Answer-evidence alignment
    };
    details: {
        retrieval: RetrievalDetails;
        answer: AnswerDetails;
        mode: string;
        provider: string;
        providerPrior: number;
    };
}

export interface RetrievalDetails {
    similarity: number;
    coverage: number;
    diversity: number;
    recency: number;
    provenance: number;
    contextItems: number;
    totalTokens: number;
}

export interface AnswerDetails {
    alignment: number;
    citation: number;
    hedgingPenalty: number;
    hedgeCount: number;
    sentenceCount: number;
}

// ====== Document Processing Types ======

export interface DocumentChunk {
    id: number;
    documentId: number;
    chunkIndex: number;
    content: string;
    contentHash: string;
    tokenCount: number;
    startChar: number;
    endChar: number;
    contentType: string;
    complexity: number;
    sectionTitle?: string;
    embedding?: number[];
    embeddingModel?: string;
}

export interface Document {
    id: number;
    userId: number;
    title: string;
    sourceType: string;
    author?: string;
    content: string;
    contentHash: string;
    tags: string[];
    rating?: number;
    notes?: string;
    processed: boolean;
    chunkCount?: number;
    conceptCount?: number;
    createdAt: Date;
    updatedAt: Date;
}

// ====== Knowledge Graph Types ======

export interface LearnedConcept {
    id: number;
    userId: number;
    name: string;
    description: string;
    confidenceScore: number;
    importanceScore: number;
    personalNotes?: string;
    tags: string[];
    encounterCount: number;
    firstEncountered: Date;
    lastReinforced: Date;
}

export interface ConceptRelationship {
    id: number;
    fromConceptId: number;
    toConceptId: number;
    relationshipType: string;
    strength: number;
    notes?: string;
    evidence?: string;
    createdAt: Date;
}

export interface KnowledgeGraph {
    id: number;
    userId: number;
    totalDocuments: number;
    totalChunks: number;
    totalConcepts: number;
    totalRelationships: number;
    knowledgeDepth: number;
    learningVelocity: number;
    graphDensity: number;
    lastUpdated: Date;
}

// ====== Memory Templates Types ======

export interface MemoryTemplate {
    id: number;
    userId: number;
    templateType: string;
    name: string;
    description: string;
    structure: Record<string, any>;
    lenses: string[];
    isActive: boolean;
    createdAt: Date;
    updatedAt: Date;
}

export interface StructuredMemoryEntry {
    id: number;
    userId: number;
    templateId: number;
    title: string;
    structuredData: Record<string, any>;
    fullText: string;
    extractedConcepts: string[];
    createdAt: Date;
    updatedAt: Date;
}

// ====== Training Types (New for our enhancement) ======

export interface TrainingSpec {
    id?: number;
    userId: number;
    description: string;
    auragType: string;
    capabilities: string[];
    trainingMethod: 'fine_tuning' | 'lora' | 'full_training';
    baseModel: string;
    hyperparameters: Record<string, any>;
    dataRequirements: DataRequirements;
    evaluationCriteria: EvaluationCriteria;
    createdAt?: Date;
}

export interface DataRequirements {
    minExamples: number;
    syntheticGeneration: boolean;
    augmentationStrategy: string;
    coverageRequirements: string[];
}

export interface EvaluationCriteria {
    metrics: string[];
    benchmarks: string[];
    qualityThresholds: Record<string, number>;
}

export interface TrainingJob {
    id: number;
    specId: number;
    status: 'pending' | 'running' | 'completed' | 'failed';
    progress: number;
    currentStage: string;
    startedAt?: Date;
    completedAt?: Date;
    metrics: Record<string, any>;
    errorMessage?: string;
}

export interface AURAG {
    id: number;
    userId: number;
    name: string;
    description: string;
    capabilities: string[];
    version: string;
    modelPath: string;
    configPath: string;
    status: 'training' | 'ready' | 'deployed' | 'archived';
    performance: Record<string, number>;
    createdAt: Date;
    deployedAt?: Date;
}

// ====== Philosophical Lens Types ======

export interface PhilosophicalLens {
    id: number;
    slug: string;
    name: string;
    description: string;
    defaultWeight: number;
    associatedAxioms: string[];
    isActive: boolean;
    createdAt: Date;
}

export const REASONING_MODES = [
    'analytical',
    'creative', 
    'philosophical',
    'pragmatic',
    'personal'
] as const;

export type ReasoningMode = typeof REASONING_MODES[number];

// ====== LLM Integration Types ======

export interface LLMProvider {
    name: string;
    endpoint: string;
    apiKey?: string;
    model: string;
    maxTokens: number;
    temperature: number;
    isLocal: boolean;
}

export interface LLMMessage {
    role: 'system' | 'user' | 'assistant';
    content: string;
}

export interface LLMResponse {
    success: boolean;
    content: string;
    usage?: {
        promptTokens: number;
        completionTokens: number;
        totalTokens: number;
    };
    model?: string;
    processingTimeMs: number;
    error?: string;
}

// ====== Cache Types ======

export interface CacheEntry<T> {
    data: T;
    timestamp: number;
    ttl: number;
    key: string;
}

export interface CacheManager {
    get<T>(key: string): Promise<T | null>;
    set<T>(key: string, value: T, ttl?: number): Promise<void>;
    delete(key: string): Promise<void>;
    clear(): Promise<void>;
}

// ====== Performance Monitoring Types ======

export interface QueryPerformanceMetrics {
    queryType: string;
    executionTimeMs: number;
    rowsScanned: number;
    rowsReturned: number;
    cacheHit: boolean;
    optimizationUsed: string;
    timestamp: Date;
}

export interface PerformanceProfile {
    latency: PerformanceMetrics;
    memory: MemoryMetrics;
    throughput: ThroughputMetrics;
    cognitiveLoad: CognitiveLoadMetrics;
}

export interface PerformanceMetrics {
    average: number;
    p50: number;
    p95: number;
    p99: number;
    max: number;
    samples: number;
}

export interface MemoryMetrics {
    workingMemory: number;
    longTermMemory: number;
    cacheSize: number;
    memoryLeaks: boolean;
}

export interface ThroughputMetrics {
    requestsPerSecond: number;
    documentsPerMinute: number;
    queriesPerMinute: number;
}

export interface CognitiveLoadMetrics {
    complexityScore: number;
    parallelTasks: number;
    contextSwitching: number;
    resourceUtilization: number;
}

// ====== Error Types ======

export class AURAGError extends Error {
    constructor(
        message: string,
        public code: string,
        public context?: Record<string, any>
    ) {
        super(message);
        this.name = 'AURAGError';
    }
}

export class TrainingError extends AURAGError {
    constructor(message: string, context?: Record<string, any>) {
        super(message, 'TRAINING_ERROR', context);
        this.name = 'TrainingError';
    }
}

export class ProcessingError extends AURAGError {
    constructor(message: string, context?: Record<string, any>) {
        super(message, 'PROCESSING_ERROR', context);
        this.name = 'ProcessingError';
    }
}

export class ConfigurationError extends AURAGError {
    constructor(message: string, context?: Record<string, any>) {
        super(message, 'CONFIGURATION_ERROR', context);
        this.name = 'ConfigurationError';
    }
}