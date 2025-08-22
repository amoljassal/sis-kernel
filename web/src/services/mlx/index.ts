/**
 * SIS MLX Services
 * Apple Silicon optimized AI training services
 */

// Core service exports (browser-compatible only)
export { BrowserMLXTrainingPipeline, createBrowserMLXTrainingPipeline } from './browser-training-interface';
export { createNaturalLanguageInterface } from './natural-language-interface';

// Type exports
export type { TrainingConfig, TrainingProgress, TrainingResult } from './browser-training-interface';

// Factory function for creating the appropriate training pipeline
export const MLX = {
    // Create training pipeline (browser API interface for frontend)
    createTrainingPipeline: (scriptsDir?: string) => {
        // Always use browser interface in frontend environment
        return createBrowserMLXTrainingPipeline();
    },

    // Default configurations
    configs: {
        development: {
            epochs: 5,
            learningRate: 0.001,
            batchSize: 16,
            maxLength: 256
        },
        production: {
            epochs: 20,
            learningRate: 0.0001,
            batchSize: 32,
            maxLength: 512
        }
    },

    // Model types
    modelTypes: [
        'custom-gpt',
        'classifier', 
        'embedding',
        'fine-tuned-model',
        'general-model'
    ] as const
};