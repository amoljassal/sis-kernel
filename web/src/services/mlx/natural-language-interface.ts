/**
 * Natural Language Training Interface
 * Converts natural language descriptions into MLX training configurations
 */

import { BrowserMLXTrainingPipeline, TrainingConfig } from './browser-training-interface';

export interface TrainingIntent {
  type: 'fine-tune' | 'train-from-scratch' | 'transfer-learning' | 'classify' | 'generate';
  domain: string;
  task: string;
  confidence: number;
}

export interface ParsedTrainingRequest {
  intent: TrainingIntent;
  config: Partial<TrainingConfig>;
  suggestions: string[];
  warnings: string[];
}

export class NaturalLanguageTrainingInterface {
  private mlxPipeline: BrowserMLXTrainingPipeline;
  private trainingPatterns: Map<string, RegExp>;

  constructor(mlxPipeline: BrowserMLXTrainingPipeline) {
    this.mlxPipeline = mlxPipeline;
    this.initializePatterns();
  }

  /**
   * Initialize pattern matching for natural language parsing
   */
  private initializePatterns(): void {
    this.trainingPatterns = new Map([
      // Training types
      ['fine-tune', /fine[- ]?tun(e|ing)|adapt|customize|refine/i],
      ['train-from-scratch', /train from scratch|build new|create new|start fresh/i],
      ['transfer-learning', /transfer learn|pre[- ]?trained|use existing/i],
      ['classify', /classif|categoriz|label|predict class/i],
      ['generate', /generat|creat|writ|produc|synthesiz/i],

      // Domains
      ['text', /text|language|nlp|natural language/i],
      ['code', /code|programming|software|development/i],
      ['medical', /medical|health|clinical|diagnosis/i],
      ['legal', /legal|law|contract|compliance/i],
      ['financial', /financial|finance|trading|investment/i],
      ['academic', /academic|research|scientific|paper/i],

      // Parameters
      ['epochs', /(\d+)\s*epochs?/i],
      ['learning-rate', /learning[- ]?rate[:\s]*([0-9.e-]+)/i],
      ['batch-size', /batch[- ]?size[:\s]*(\d+)/i],
      ['max-length', /max[- ]?length[:\s]*(\d+)/i],

      // Model architectures
      ['transformer', /transformer|gpt|bert|attention/i],
      ['rnn', /rnn|lstm|gru|recurrent/i],
      ['cnn', /cnn|convolution|conv/i],
      ['linear', /linear|dense|fully[- ]?connected/i]
    ]);
  }

  /**
   * Parse natural language training request
   */
  parseTrainingRequest(description: string): ParsedTrainingRequest {
    const intent = this.extractIntent(description);
    const config = this.extractConfiguration(description);
    const suggestions = this.generateSuggestions(description, intent);
    const warnings = this.validateRequest(description, config);

    return {
      intent,
      config,
      suggestions,
      warnings
    };
  }

  /**
   * Extract training intent from description
   */
  private extractIntent(description: string): TrainingIntent {
    let type: TrainingIntent['type'] = 'train-from-scratch';
    let domain = 'general';
    let task = 'general-purpose';
    let confidence = 0.5;

    // Determine training type
    for (const [key, pattern] of this.trainingPatterns) {
      if (['fine-tune', 'train-from-scratch', 'transfer-learning', 'classify', 'generate'].includes(key)) {
        if (pattern.test(description)) {
          type = key as TrainingIntent['type'];
          confidence += 0.2;
          break;
        }
      }
    }

    // Determine domain
    for (const [key, pattern] of this.trainingPatterns) {
      if (['text', 'code', 'medical', 'legal', 'financial', 'academic'].includes(key)) {
        if (pattern.test(description)) {
          domain = key;
          confidence += 0.2;
          break;
        }
      }
    }

    // Extract specific task
    const taskKeywords = [
      'classification', 'generation', 'translation', 'summarization',
      'question answering', 'sentiment analysis', 'named entity recognition',
      'code completion', 'bug detection', 'documentation'
    ];

    for (const keyword of taskKeywords) {
      if (description.toLowerCase().includes(keyword.toLowerCase())) {
        task = keyword;
        confidence += 0.1;
        break;
      }
    }

    return { type, domain, task, confidence: Math.min(confidence, 1.0) };
  }

  /**
   * Extract training configuration from description
   */
  private extractConfiguration(description: string): Partial<TrainingConfig> {
    const config: Partial<TrainingConfig> = {};

    // Extract epochs
    const epochsMatch = description.match(this.trainingPatterns.get('epochs')!);
    if (epochsMatch) {
      config.epochs = parseInt(epochsMatch[1]);
    }

    // Extract learning rate
    const lrMatch = description.match(this.trainingPatterns.get('learning-rate')!);
    if (lrMatch) {
      config.learningRate = parseFloat(lrMatch[1]);
    }

    // Extract batch size
    const batchMatch = description.match(this.trainingPatterns.get('batch-size')!);
    if (batchMatch) {
      config.batchSize = parseInt(batchMatch[1]);
    }

    // Extract max length
    const lengthMatch = description.match(this.trainingPatterns.get('max-length')!);
    if (lengthMatch) {
      config.maxLength = parseInt(lengthMatch[1]);
    }

    // Generate model name based on description
    config.modelName = this.generateModelName(description);
    config.description = description;

    return config;
  }

  /**
   * Generate appropriate model name based on description
   */
  private generateModelName(description: string): string {
    const timestamp = new Date().toISOString().slice(0, 10);
    
    if (this.trainingPatterns.get('classify')!.test(description)) {
      return `classifier-${timestamp}`;
    } else if (this.trainingPatterns.get('generate')!.test(description)) {
      return `generator-${timestamp}`;
    } else if (description.toLowerCase().includes('gpt')) {
      return `custom-gpt-${timestamp}`;
    } else if (description.toLowerCase().includes('bert')) {
      return `custom-bert-${timestamp}`;
    } else {
      return `custom-model-${timestamp}`;
    }
  }

  /**
   * Generate helpful suggestions based on the request
   */
  private generateSuggestions(description: string, intent: TrainingIntent): string[] {
    const suggestions: string[] = [];

    // Intent-based suggestions
    if (intent.type === 'fine-tune') {
      suggestions.push('Consider using a pre-trained model as your starting point');
      suggestions.push('Fine-tuning typically requires fewer epochs than training from scratch');
    } else if (intent.type === 'train-from-scratch') {
      suggestions.push('Training from scratch may require a large dataset');
      suggestions.push('Consider starting with more epochs (20-50) for better convergence');
    }

    // Domain-specific suggestions
    if (intent.domain === 'code') {
      suggestions.push('Use code-specific tokenization and consider syntax highlighting');
      suggestions.push('Include diverse programming languages in your dataset');
    } else if (intent.domain === 'medical') {
      suggestions.push('Ensure compliance with healthcare data regulations');
      suggestions.push('Consider using medical terminology embeddings');
    }

    // Parameter suggestions
    if (!description.includes('batch')) {
      suggestions.push('Consider specifying batch size based on your dataset size');
    }
    if (!description.includes('learning')) {
      suggestions.push('Specify learning rate for better control over training');
    }

    return suggestions;
  }

  /**
   * Validate training request and generate warnings
   */
  private validateRequest(description: string, config: Partial<TrainingConfig>): string[] {
    const warnings: string[] = [];

    // Check for missing critical information
    if (!description.includes('dataset') && !description.includes('data')) {
      warnings.push('No dataset mentioned - you will need to specify a dataset path');
    }

    // Parameter validation
    if (config.epochs && config.epochs > 100) {
      warnings.push('High epoch count may lead to overfitting');
    }
    if (config.learningRate && config.learningRate > 0.01) {
      warnings.push('High learning rate may cause training instability');
    }
    if (config.batchSize && config.batchSize > 128) {
      warnings.push('Large batch size may require significant memory');
    }

    // Architecture warnings
    if (description.toLowerCase().includes('transformer') && 
        config.maxLength && config.maxLength > 2048) {
      warnings.push('Long sequences with transformers may be computationally expensive');
    }

    return warnings;
  }

  /**
   * Convert natural language to training configuration and start training
   */
  async trainFromDescription(description: string, datasetPath: string): Promise<{
    trainingId: string;
    parsedRequest: ParsedTrainingRequest;
  }> {
    const parsedRequest = this.parseTrainingRequest(description);

    // Validate that we have enough information
    if (parsedRequest.warnings.length > 5) {
      throw new Error('Too many warnings detected. Please provide more specific training requirements.');
    }

    if (parsedRequest.intent.confidence < 0.3) {
      throw new Error('Could not understand training intent. Please provide clearer description.');
    }

    // Start training
    const trainingId = await this.mlxPipeline.startTraining(description, datasetPath);

    return {
      trainingId,
      parsedRequest
    };
  }

  /**
   * Generate example training descriptions
   */
  getExampleDescriptions(): string[] {
    return [
      'Fine-tune a GPT model for code generation with 20 epochs and learning rate 0.0001',
      'Train a text classifier for sentiment analysis from scratch with batch size 64',
      'Create a medical text summarization model using transfer learning',
      'Build a legal document classifier with 15 epochs and max length 1024',
      'Train a code completion model for Python with 30 epochs',
      'Fine-tune BERT for academic paper classification with learning rate 0.00005',
      'Create a financial news generator with transformer architecture',
      'Train a bug detection model for JavaScript code from scratch'
    ];
  }

  /**
   * Get training recommendations based on task type
   */
  getRecommendations(taskType: string): {
    epochs: number;
    learningRate: number;
    batchSize: number;
    architecture: string;
  } {
    const recommendations = {
      'classification': {
        epochs: 15,
        learningRate: 0.0001,
        batchSize: 32,
        architecture: 'transformer-encoder'
      },
      'generation': {
        epochs: 25,
        learningRate: 0.00005,
        batchSize: 16,
        architecture: 'transformer-decoder'
      },
      'fine-tuning': {
        epochs: 10,
        learningRate: 0.00001,
        batchSize: 8,
        architecture: 'pre-trained-base'
      },
      'from-scratch': {
        epochs: 50,
        learningRate: 0.001,
        batchSize: 64,
        architecture: 'custom-transformer'
      }
    };

    return recommendations[taskType as keyof typeof recommendations] || recommendations['from-scratch'];
  }
}

export const createNaturalLanguageInterface = (mlxPipeline: BrowserMLXTrainingPipeline): NaturalLanguageTrainingInterface => {
  return new NaturalLanguageTrainingInterface(mlxPipeline);
};