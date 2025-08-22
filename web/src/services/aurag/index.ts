/**
 * SIS AURAG Services
 * Exported interfaces for the Unified RAG system
 */

// Core service exports
export { UnifiedRAGService, createRAGService } from './unified-rag-service';
export { BrowserRAGService, createBrowserRAGService } from './browser-rag-service';
export { ScoringAlgorithms } from './scoring-algorithms';
export { ConfidenceScorer } from './confidence-scorer';
export { DocumentProcessor } from './document-processor';
export { ContextBuilder } from './context-builder';

// Type exports
export * from './types';

// Import for factory function
import { createBrowserRAGService } from './browser-rag-service';

// Example usage and factory functions
export const AURAG = {
    // Create a new RAG service instance (browser-compatible)
    createService: (config?: any) => createBrowserRAGService(config),
    
    // Default configurations
    configs: {
        development: {
            maxContextItems: 8,
            tokenBudget: 1000,
            defaultPhilosophicalLens: 'analytical',
            providers: {
                embedding: 'ollama',
                llm: 'ollama'
            }
        },
        production: {
            maxContextItems: 12,
            tokenBudget: 1500,
            defaultPhilosophicalLens: 'analytical', 
            providers: {
                embedding: 'openai',
                llm: 'claude'
            }
        }
    },
    
    // Available reasoning modes
    reasoningModes: ['analytical', 'creative', 'philosophical', 'pragmatic', 'personal'] as const,
    
    // Available lens types
    lensTypes: [
        'analytical', 'creative', 'ethical', 'practical', 'personal',
        'scientific', 'artistic', 'business', 'academic', 'intuitive'
    ] as const
};