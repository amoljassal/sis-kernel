/**
 * SIS Confidence Scoring Engine
 * Extracted from sis-core apps/quality/confidence.py
 * 
 * Mathematical uncertainty quantification for RAG responses
 * Multi-factor confidence scoring with mathematical foundation
 */

import { ContextItem, ConfidenceResult, RetrievalDetails, AnswerDetails } from './types';

interface ProviderPriors {
    [provider: string]: number;
}

interface ModeWeights {
    beta_r: number; // Retrieval quality weight
    beta_l: number; // Lens alignment weight  
    beta_a: number; // Answer-evidence alignment weight
    beta_p: number; // Provider prior weight
}

// Hedging terms that indicate uncertainty
const HEDGE_TERMS = new Set([
    "might", "maybe", "perhaps", "possibly", "uncertain", "unsure",
    "not sure", "i think", "i believe", "it seems", "appears to be", 
    "could be", "may be", "potentially", "likely", "probably"
]);

export class ConfidenceScorer {
    private initialized = false;

    // Provider reliability priors (0.0-1.0)
    private readonly providerPriors: ProviderPriors = {
        "openai": 0.75,
        "claude": 0.73,
        "gemini": 0.68,
        "ollama": 0.60,
        "grok": 0.65
    };

    // Mode-specific weighting coefficients
    private readonly modeWeights: Record<string, ModeWeights> = {
        analytical: { beta_r: 0.40, beta_l: 0.10, beta_a: 0.40, beta_p: 0.10 },
        creative: { beta_r: 0.20, beta_l: 0.25, beta_a: 0.25, beta_p: 0.30 },
        philosophical: { beta_r: 0.25, beta_l: 0.35, beta_a: 0.25, beta_p: 0.15 },
        pragmatic: { beta_r: 0.35, beta_l: 0.15, beta_a: 0.35, beta_p: 0.15 },
        personal: { beta_r: 0.30, beta_l: 0.20, beta_a: 0.30, beta_p: 0.20 }
    };

    constructor(customProviderPriors?: ProviderPriors) {
        if (customProviderPriors) {
            this.providerPriors = { ...this.providerPriors, ...customProviderPriors };
        }
    }

    async initialize(): Promise<void> {
        this.initialized = true;
    }

    /**
     * Calculate comprehensive confidence score for AI response
     * 
     * @param context List of context items used for generation
     * @param answer Generated AI response text
     * @param mode Reasoning mode (analytical, creative, philosophical, pragmatic)
     * @param provider LLM provider name
     * @param activeLenses List of active philosophical lens names
     * @param queryVector Query embedding vector (if available)
     * @param tokenBudget Token budget for context window
     * @returns Confidence result with overall score and detailed factors
     */
    async calculateConfidence(
        context: ContextItem[],
        answer: string,
        mode: string = "analytical",
        provider: string = "openai",
        activeLenses?: string[],
        queryVector?: number[],
        tokenBudget: number = 1200
    ): Promise<ConfidenceResult> {
        try {
            const activeL = activeLenses || [];

            // Stage 1: Retrieval Quality Score (Sr)
            const SrData = this.scoreRetrieval(
                context,
                queryVector,
                tokenBudget,
                activeL
            );

            // Stage 2: Lens Alignment Score (Sl)
            const Sl = this.scoreLensAlignment(activeL, context);

            // Stage 3: Answer-Evidence Alignment Score (Sa)
            const SaData = this.scoreAnswerAlignment(answer, context);

            // Stage 4: Combined Confidence with Mode Weighting
            const overall = this.combineScores(
                SrData.Sr,
                Sl,
                SaData.Sa,
                mode,
                provider
            );

            return {
                overall: Math.max(0.0, Math.min(1.0, overall)),
                factors: {
                    Sr: SrData.Sr,
                    Sl: Sl,
                    Sa: SaData.Sa
                },
                details: {
                    retrieval: SrData.details,
                    answer: SaData.details,
                    mode: mode,
                    provider: provider,
                    providerPrior: this.providerPriors[provider] || 0.6
                }
            };

        } catch (error) {
            console.error('Confidence calculation failed:', error);
            return {
                overall: 0.5, // Neutral confidence on error
                factors: { Sr: 0.5, Sl: 0.5, Sa: 0.5 },
                details: {
                    retrieval: this.getEmptyRetrievalDetails(),
                    answer: this.getEmptyAnswerDetails(),
                    mode: mode,
                    provider: provider,
                    providerPrior: this.providerPriors[provider] || 0.6
                }
            };
        }
    }

    /**
     * Calculate retrieval quality score Sr with detailed factors
     * 
     * Components:
     * - Similarity: How well context matches query
     * - Coverage: Context utilization of token budget
     * - Diversity: Variety in retrieved content
     * - Recency: Freshness of context information
     * - Provenance: Source reliability (personal > documented > generic)
     */
    private scoreRetrieval(
        context: ContextItem[],
        queryVector?: number[],
        tokenBudget: number = 1200,
        activeLenses: string[] = []
    ): { Sr: number; details: RetrievalDetails } {
        if (!context || context.length === 0) {
            return {
                Sr: 0.0,
                details: {
                    similarity: 0.0,
                    coverage: 0.0,
                    diversity: 0.0,
                    recency: 0.0,
                    provenance: 0.0,
                    contextItems: 0,
                    totalTokens: 0
                }
            };
        }

        // Component scores
        const similarities: number[] = [];
        const recencyScores: number[] = [];
        const provenanceScores: number[] = [];
        const embeddings: number[][] = [];

        for (const item of context) {
            // Similarity: Use existing score as proxy if no embeddings
            if (queryVector && item.meta?.embedding) {
                const itemVec = item.meta.embedding;
                const similarity = this.cosineSimilarity(queryVector, itemVec);
                similarities.push(similarity);
                embeddings.push(itemVec);
            } else {
                // Fallback to existing relevance score
                similarities.push(Math.min(1.0, Math.max(0.0, item.score)));
            }

            // Recency: Time-based decay with 14-day half-life
            const createdAt = item.meta?.created_at || item.meta?.updated_at;
            if (createdAt) {
                let dateObj: Date;
                if (typeof createdAt === 'string') {
                    try {
                        dateObj = new Date(createdAt.replace('Z', '+00:00'));
                    } catch {
                        dateObj = new Date(); // Fallback
                    }
                } else {
                    dateObj = createdAt;
                }
                recencyScores.push(this.recencyScore(dateObj));
            } else {
                recencyScores.push(0.7); // Default for unknown dates
            }

            // Provenance: Source reliability mapping
            const kindScores: Record<string, number> = {
                "memory": 1.0,        // Personal memories most reliable
                "entity": 0.8,        // Personal entities
                "relationship": 0.8,  // Personal relationships
                "template": 0.7,      // Memory templates
                "document": 0.6,      // External documents
                "generic": 0.4        // Generic knowledge
            };
            provenanceScores.push(kindScores[item.kind] || 0.5);
        }

        // Calculate component scores
        const s_sim = similarities.length > 0 ? this.mean(similarities) : 0.0;
        const s_rec = recencyScores.length > 0 ? this.mean(recencyScores) : 0.5;
        const s_src = provenanceScores.length > 0 ? this.mean(provenanceScores) : 0.5;

        // Coverage: Token utilization
        const totalTokens = context.reduce((sum, item) => sum + item.text.split(' ').length, 0);
        const s_cov = Math.min(1.0, totalTokens / Math.max(1, tokenBudget));

        // Diversity: Lower pairwise similarity indicates higher diversity
        let s_div = 1.0;
        if (embeddings.length > 1) {
            const pairwiseSims: number[] = [];
            for (let i = 0; i < embeddings.length; i++) {
                for (let j = i + 1; j < embeddings.length; j++) {
                    const sim = this.cosineSimilarity(embeddings[i], embeddings[j]);
                    pairwiseSims.push(sim);
                }
            }
            s_div = pairwiseSims.length > 0 ? 1.0 - this.mean(pairwiseSims) : 1.0;
        }

        // Weighted combination
        const Sr = (0.40 * s_sim + 0.20 * s_cov + 0.15 * s_div + 
                   0.15 * s_rec + 0.10 * s_src);

        return {
            Sr: Math.max(0.0, Math.min(1.0, Sr)),
            details: {
                similarity: s_sim,
                coverage: s_cov,
                diversity: s_div,
                recency: s_rec,
                provenance: s_src,
                contextItems: context.length,
                totalTokens: totalTokens
            }
        };
    }

    /**
     * Calculate philosophical lens alignment score Sl
     * 
     * Measures consistency between requested philosophical perspectives
     * and the philosophical orientations present in retrieved context
     */
    private scoreLensAlignment(activeLenses: string[], context: ContextItem[]): number {
        if (!activeLenses || activeLenses.length === 0) {
            return 0.5; // Neutral when no specific lenses requested
        }

        if (!context || context.length === 0) {
            return 0.0; // No context means no alignment
        }

        // Collect all lenses from context items
        const contextLenses = new Set<string>();
        for (const item of context) {
            const itemLenses = item.meta?.lenses;
            if (Array.isArray(itemLenses)) {
                itemLenses.forEach(lens => contextLenses.add(lens));
            } else if (typeof itemLenses === 'string') {
                contextLenses.add(itemLenses);
            }
        }

        if (contextLenses.size === 0) {
            return 0.1; // Low alignment if context has no lens information
        }

        // Jaccard similarity between active and context lenses
        const activeSet = new Set(activeLenses);
        const intersection = new Set([...activeSet].filter(x => contextLenses.has(x)));
        const union = new Set([...activeSet, ...contextLenses]);

        return union.size > 0 ? intersection.size / union.size : 0.0;
    }

    /**
     * Calculate answer-evidence alignment score Sa
     * 
     * Components:
     * - Alignment: How well answer content matches evidence
     * - Citation: Density of references to context
     * - Hedging: Penalty for uncertainty expressions
     */
    private scoreAnswerAlignment(answer: string, context: ContextItem[]): { Sa: number; details: AnswerDetails } {
        if (!answer || answer.trim().length === 0) {
            return {
                Sa: 0.0,
                details: {
                    alignment: 0.0,
                    citation: 0.0,
                    hedgingPenalty: 0.0,
                    hedgeCount: 0,
                    sentenceCount: 0
                }
            };
        }

        if (!context || context.length === 0) {
            return {
                Sa: 0.3,
                details: {
                    alignment: 0.3,
                    citation: 0.0,
                    hedgingPenalty: 0.0,
                    hedgeCount: 0,
                    sentenceCount: answer.split(/[.!?]+/).filter(s => s.trim()).length
                }
            };
        }

        // Split answer into sentences for analysis
        const sentences = answer.split(/[.!?]+/).filter(s => s.trim().length > 0);
        if (sentences.length === 0) {
            return {
                Sa: 0.0,
                details: {
                    alignment: 0.0,
                    citation: 0.0,
                    hedgingPenalty: 0.0,
                    hedgeCount: 0,
                    sentenceCount: 0
                }
            };
        }

        // Sentence-level alignment with context
        const alignmentScores: number[] = [];
        for (const sentence of sentences) {
            let maxOverlap = 0.0;
            for (const item of context) {
                const overlap = this.textOverlap(sentence, item.text);
                maxOverlap = Math.max(maxOverlap, overlap);
            }
            alignmentScores.push(maxOverlap);
        }

        const s_alignment = alignmentScores.length > 0 ? this.mean(alignmentScores) : 0.0;

        // Citation density (look for reference markers like [1], (#), etc.)
        let citationMarkers = 0;
        for (const sentence of sentences) {
            if (/\[[0-9]+\]/.test(sentence) || /\([^)]*\)/.test(sentence)) {
                citationMarkers++;
            }
        }
        const s_citation = Math.min(1.0, citationMarkers / sentences.length);

        // Hedging penalty (uncertainty expressions reduce confidence)
        const answerLower = answer.toLowerCase();
        let hedgeCount = 0;
        for (const term of HEDGE_TERMS) {
            if (answerLower.includes(term)) {
                hedgeCount++;
            }
        }
        const hedgingPenalty = Math.min(0.2, 0.03 * hedgeCount);

        // Combined Sa score
        const Sa = Math.max(0.0, Math.min(1.0, 0.85 * s_alignment + 0.15 * s_citation - hedgingPenalty));

        return {
            Sa: Sa,
            details: {
                alignment: s_alignment,
                citation: s_citation,
                hedgingPenalty: hedgingPenalty,
                hedgeCount: hedgeCount,
                sentenceCount: sentences.length
            }
        };
    }

    /**
     * Combine individual scores with mode-specific weighting
     * 
     * Formula: beta_0 + beta_r*Sr + beta_l*Sl + beta_a*Sa + beta_p*P_provider
     */
    private combineScores(
        Sr: number,
        Sl: number,
        Sa: number,
        mode: string,
        provider: string
    ): number {
        const weights = this.modeWeights[mode] || this.modeWeights.analytical;
        const providerPrior = this.providerPriors[provider] || 0.6;

        const combined = (
            weights.beta_r * Sr +
            weights.beta_l * Sl +
            weights.beta_a * Sa +
            weights.beta_p * providerPrior
        );

        return Math.max(0.0, Math.min(1.0, combined));
    }

    // Utility methods

    private cosineSimilarity(vec1: number[], vec2: number[]): number {
        try {
            if (!vec1 || !vec2 || vec1.length !== vec2.length) {
                return 0.0;
            }

            let dotProduct = 0;
            let norm1 = 0;
            let norm2 = 0;

            for (let i = 0; i < vec1.length; i++) {
                dotProduct += vec1[i] * vec2[i];
                norm1 += vec1[i] * vec1[i];
                norm2 += vec2[i] * vec2[i];
            }

            norm1 = Math.sqrt(norm1);
            norm2 = Math.sqrt(norm2);

            if (norm1 === 0 || norm2 === 0) {
                return 0.0;
            }

            return dotProduct / (norm1 * norm2);
        } catch {
            return 0.0;
        }
    }

    private recencyScore(timestamp: Date, halfLifeDays: number = 14): number {
        try {
            if (!timestamp) {
                return 0.5;
            }
            const daysAgo = (Date.now() - timestamp.getTime()) / (1000 * 60 * 60 * 24);
            return 1.0 / (1.0 + daysAgo / halfLifeDays);
        } catch {
            return 0.5;
        }
    }

    private textOverlap(text1: string, text2: string): number {
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
        } catch {
            return 0.0;
        }
    }

    private mean(numbers: number[]): number {
        if (numbers.length === 0) return 0.0;
        return numbers.reduce((sum, num) => sum + num, 0) / numbers.length;
    }

    private getEmptyRetrievalDetails(): RetrievalDetails {
        return {
            similarity: 0.0,
            coverage: 0.0,
            diversity: 0.0,
            recency: 0.0,
            provenance: 0.0,
            contextItems: 0,
            totalTokens: 0
        };
    }

    private getEmptyAnswerDetails(): AnswerDetails {
        return {
            alignment: 0.0,
            citation: 0.0,
            hedgingPenalty: 0.0,
            hedgeCount: 0,
            sentenceCount: 0
        };
    }
}