/**
 * SIS Scoring Algorithms
 * Extracted from sis-core Django implementation
 * 
 * Multi-dimensional scoring system for RAG context ranking
 * Implements the sophisticated scoring from apps/rag_context/scoring.py
 */

import { ScoringWeights, ReasoningMode } from './types';

// Hedging terms that indicate uncertainty (from confidence.py)
const HEDGE_TERMS = new Set([
    "might", "maybe", "perhaps", "possibly", "uncertain", "unsure",
    "not sure", "i think", "i believe", "it seems", "appears to be",
    "could be", "may be", "potentially", "likely", "probably"
]);

export class ScoringAlgorithms {
    private initialized = false;

    // Reasoning mode weight configurations (from scoring.py)
    private readonly REASONING_MODE_WEIGHTS: Record<ReasoningMode, ScoringWeights> = {
        analytical: {
            relevance: 0.5,
            recency: 0.2,
            priority: 0.2,
            centrality: 0.05,
            lensBonus: 0.05
        },
        creative: {
            relevance: 0.3,
            recency: 0.15,
            priority: 0.15,
            centrality: 0.3, // High centrality for surprising connections
            lensBonus: 0.1
        },
        philosophical: {
            relevance: 0.35,
            recency: 0.15,
            priority: 0.2,
            centrality: 0.1,
            lensBonus: 0.2 // Strong lens alignment for philosophical reasoning
        },
        pragmatic: {
            relevance: 0.4,
            recency: 0.3, // Recent is practical
            priority: 0.25, // Priorities matter for practical decisions
            centrality: 0.05,
            lensBonus: 0.0
        },
        personal: {
            relevance: 0.35,
            recency: 0.25,
            priority: 0.2,
            centrality: 0.1,
            lensBonus: 0.1
        }
    };

    async initialize(): Promise<void> {
        this.initialized = true;
    }

    /**
     * Calculate recency score using exponential decay
     * 
     * @param createdAt When the item was created
     * @param halfLifeDays Days for score to decay to 0.5 (default: 14)
     * @returns Score between 0.0 and 1.0 (1.0 = very recent)
     */
    recencyDecay(createdAt: Date, halfLifeDays: number = 14): number {
        const now = new Date();
        const days = Math.max(0.0, (now.getTime() - createdAt.getTime()) / (1000 * 60 * 60 * 24));
        // Exponential decay: score = 1 / (1 + days/half_life)
        return 1.0 / (1.0 + days / halfLifeDays);
    }

    /**
     * Calculate bonus score for philosophical lens alignment
     * 
     * @param activeLenses User's currently active philosophical lenses
     * @param itemLenses Lenses associated with this item
     * @returns Bonus score (0.0 to 0.2)
     */
    lensBonus(activeLenses: string[] | null, itemLenses: string[]): number {
        if (!activeLenses || activeLenses.length === 0) {
            return 0.0;
        }

        const lensSet = new Set(activeLenses);
        const itemLensSet = new Set(itemLenses);

        // Check for intersection
        for (const lens of itemLensSet) {
            if (lensSet.has(lens)) {
                return 0.2; // Strong bonus for lens alignment
            }
        }

        return 0.0;
    }

    /**
     * Calculate priority score from user and system priorities
     * 
     * @param userPriority User-assigned priority (1-10)
     * @param systemPriority System-calculated priority (1-10)
     * @returns Normalized priority score (0.0 to 1.0)
     */
    priorityScore(userPriority?: number, systemPriority?: number): number {
        // User priority takes precedence
        if (userPriority !== undefined && userPriority !== null) {
            return Math.min(1.0, Math.max(0.0, userPriority / 10.0));
        }

        if (systemPriority !== undefined && systemPriority !== null) {
            return Math.min(1.0, Math.max(0.0, systemPriority / 10.0));
        }

        return 0.5; // Default medium priority
    }

    /**
     * Calculate centrality score based on graph connections
     * 
     * @param connectionCount Number of connections this item has
     * @param maxConnections Maximum expected connections for normalization (default: 100)
     * @returns Centrality score (0.0 to 1.0)
     */
    centralityScore(connectionCount: number, maxConnections: number = 100): number {
        if (maxConnections <= 0) {
            return 0.0;
        }

        // Logarithmic scaling to prevent over-weighting highly connected nodes
        const normalized = Math.min(1.0, connectionCount / maxConnections);
        return Math.log(1 + normalized) / Math.log(2); // Log base 2 scaling
    }

    /**
     * Calculate text relevance score between query and text
     * 
     * @param query User's search query
     * @param text Text to score against
     * @returns Relevance score (0.0 to 1.0)
     */
    calculateTextRelevance(query: string, text: string): number {
        return this.keywordRelevanceScore(query, text);
    }

    /**
     * Calculate keyword-based relevance score (fallback when no embeddings)
     * 
     * @param query User's search query
     * @param text Text to score against
     * @returns Relevance score (0.0 to 1.0)
     */
    keywordRelevanceScore(query: string, text: string): number {
        if (!query || !text) {
            return 0.0;
        }

        const queryWords = new Set(query.toLowerCase().split(/\s+/).filter(word => word.length > 0));
        const textWords = new Set(text.toLowerCase().split(/\s+/).filter(word => word.length > 0));

        if (queryWords.size === 0) {
            return 0.0;
        }

        // Intersection over query size (how many query terms are present)
        let intersection = 0;
        for (const word of queryWords) {
            if (textWords.has(word)) {
                intersection++;
            }
        }

        return intersection / queryWords.size;
    }

    /**
     * Calculate semantic similarity using cosine similarity
     * 
     * @param queryVector Query embedding vector
     * @param textVector Text embedding vector
     * @returns Similarity score (0.0 to 1.0)
     */
    semanticSimilarity(queryVector: number[], textVector: number[]): number {
        if (!queryVector || !textVector || queryVector.length !== textVector.length) {
            return 0.0;
        }

        try {
            // Calculate cosine similarity
            let dotProduct = 0;
            let queryMagnitude = 0;
            let textMagnitude = 0;

            for (let i = 0; i < queryVector.length; i++) {
                dotProduct += queryVector[i] * textVector[i];
                queryMagnitude += queryVector[i] * queryVector[i];
                textMagnitude += textVector[i] * textVector[i];
            }

            queryMagnitude = Math.sqrt(queryMagnitude);
            textMagnitude = Math.sqrt(textMagnitude);

            if (queryMagnitude === 0 || textMagnitude === 0) {
                return 0.0;
            }

            const similarity = dotProduct / (queryMagnitude * textMagnitude);
            return Math.max(0.0, Math.min(1.0, similarity));

        } catch (error) {
            console.error('Error calculating semantic similarity:', error);
            return 0.0;
        }
    }

    /**
     * Calculate final weighted score for context ranking
     * 
     * @param relevance Semantic/keyword relevance score
     * @param recency Recency decay score
     * @param priority Priority score
     * @param centrality Graph centrality score
     * @param lensBonusVal Philosophical lens bonus
     * @param weights Custom weight configuration
     * @returns Final weighted score (0.0 to 1.0)
     */
    calculateFinalScore(
        relevance: number,
        recency: number,
        priority: number,
        centrality: number,
        lensBonusVal: number,
        weights?: ScoringWeights
    ): number {
        // Default weights - can be adjusted per reasoning mode
        const defaultWeights: ScoringWeights = {
            relevance: 0.4,   // Most important
            recency: 0.25,    // Recent is relevant
            priority: 0.2,    // User/system priorities
            centrality: 0.1,  // Graph importance
            lensBonus: 0.05   // Small lens alignment bonus
        };

        const w = weights || defaultWeights;

        const score = (
            w.relevance * relevance +
            w.recency * recency +
            w.priority * priority +
            w.centrality * centrality +
            w.lensBonus * lensBonusVal
        );

        return Math.min(1.0, Math.max(0.0, score));
    }

    /**
     * Get reasoning mode weights for specific mode
     * 
     * @param mode Reasoning mode
     * @returns Weight configuration for the mode
     */
    getReasoningModeWeights(mode: ReasoningMode): ScoringWeights {
        return this.REASONING_MODE_WEIGHTS[mode] || this.REASONING_MODE_WEIGHTS.analytical;
    }

    /**
     * Calculate hedging penalty for uncertainty expressions
     * 
     * @param text Text to analyze for hedging terms
     * @returns Penalty score (0.0 to 0.2)
     */
    calculateHedgingPenalty(text: string): number {
        if (!text) {
            return 0.0;
        }

        const textLower = text.toLowerCase();
        let hedgeCount = 0;

        for (const term of HEDGE_TERMS) {
            if (textLower.includes(term)) {
                hedgeCount++;
            }
        }

        // Cap penalty at 0.2 (20%)
        return Math.min(0.2, 0.03 * hedgeCount);
    }

    /**
     * Calculate text overlap ratio using word tokens
     * 
     * @param text1 First text
     * @param text2 Second text
     * @returns Overlap ratio (0.0 to 1.0)
     */
    textOverlap(text1: string, text2: string): number {
        try {
            const words1 = new Set(text1.toLowerCase().split(/\s+/).filter(word => word.length > 0));
            const words2 = new Set(text2.toLowerCase().split(/\s+/).filter(word => word.length > 0));

            if (words1.size === 0) {
                return 0.0;
            }

            let intersection = 0;
            for (const word of words1) {
                if (words2.has(word)) {
                    intersection++;
                }
            }

            return intersection / words1.size;

        } catch (error) {
            console.error('Error calculating text overlap:', error);
            return 0.0;
        }
    }

    /**
     * Calculate citation density in text
     * 
     * @param text Text to analyze
     * @returns Citation density score (0.0 to 1.0)
     */
    calculateCitationDensity(text: string): number {
        if (!text) {
            return 0.0;
        }

        const sentences = text.split(/[.!?]+/).filter(s => s.trim().length > 0);
        if (sentences.length === 0) {
            return 0.0;
        }

        // Look for citation markers like [1], (#), etc.
        let citationCount = 0;
        for (const sentence of sentences) {
            if (/\[[0-9]+\]/.test(sentence) || /\([^)]*\)/.test(sentence)) {
                citationCount++;
            }
        }

        return Math.min(1.0, citationCount / sentences.length);
    }

    /**
     * Calculate content complexity score
     * 
     * @param text Text to analyze
     * @returns Complexity score (0.0 to 1.0)
     */
    calculateComplexity(text: string): number {
        if (!text) {
            return 0.0;
        }

        const words = text.split(/\s+/).filter(word => word.length > 0);
        const sentences = text.split(/[.!?]+/).filter(s => s.trim().length > 0);

        if (words.length === 0 || sentences.length === 0) {
            return 0.0;
        }

        // Average sentence length
        const avgSentenceLength = words.length / sentences.length;

        // Average word length
        const avgWordLength = words.reduce((sum, word) => sum + word.length, 0) / words.length;

        // Complex words (more than 6 characters)
        const complexWords = words.filter(word => word.length > 6).length;
        const complexWordRatio = complexWords / words.length;

        // Combine metrics (normalized to 0-1 range)
        const lengthComplexity = Math.min(1.0, avgSentenceLength / 20); // Cap at 20 words per sentence
        const wordComplexity = Math.min(1.0, avgWordLength / 8); // Cap at 8 chars per word
        const vocabularyComplexity = complexWordRatio;

        return (lengthComplexity + wordComplexity + vocabularyComplexity) / 3;
    }

    /**
     * Batch score multiple items efficiently
     * 
     * @param items Items to score
     * @param query Query for relevance scoring
     * @param queryVector Optional query embedding
     * @param mode Reasoning mode for weighting
     * @param activeLenses Active philosophical lenses
     * @returns Scored items with individual score components
     */
    batchScore(
        items: Array<{
            text: string;
            createdAt: Date;
            userPriority?: number;
            systemPriority?: number;
            connectionCount?: number;
            lenses?: string[];
            embedding?: number[];
        }>,
        query: string,
        queryVector?: number[],
        mode: ReasoningMode = 'analytical',
        activeLenses?: string[]
    ): Array<{
        item: any;
        scores: {
            relevance: number;
            recency: number;
            priority: number;
            centrality: number;
            lensBonus: number;
            final: number;
        };
    }> {
        const weights = this.getReasoningModeWeights(mode);

        return items.map(item => {
            // Calculate individual scores
            const relevance = queryVector && item.embedding 
                ? this.semanticSimilarity(queryVector, item.embedding)
                : this.keywordRelevanceScore(query, item.text);

            const recency = this.recencyDecay(item.createdAt);
            const priority = this.priorityScore(item.userPriority, item.systemPriority);
            const centrality = this.centralityScore(item.connectionCount || 0);
            const lensBonusVal = this.lensBonus(activeLenses || null, item.lenses || []);

            // Calculate final weighted score
            const final = this.calculateFinalScore(
                relevance,
                recency,
                priority,
                centrality,
                lensBonusVal,
                weights
            );

            return {
                item,
                scores: {
                    relevance,
                    recency,
                    priority,
                    centrality,
                    lensBonus: lensBonusVal,
                    final
                }
            };
        });
    }
}