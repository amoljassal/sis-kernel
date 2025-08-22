/**
 * SIS Context Builder
 * Extracted from sis-core Django implementation
 * 
 * Implements the 4-stage RAG context orchestration pipeline:
 * 1. Candidate Generation (parallel retrieval)
 * 2. Feature Enrichment (scoring)  
 * 3. Filtering & Ranking (prioritization)
 * 4. Context Assembly (token optimization)
 */

import { ContextItem, ContextResult, ReasoningMode } from './types';
import { ScoringAlgorithms } from './scoring-algorithms';
import { 
    getDocumentChunkRepository, 
    getLearnedConceptRepository,
    getConceptRelationshipRepository,
    getStructuredMemoryEntryRepository
} from '../../database/repositories';

export interface ContextBuildOptions {
    query: string;
    userId: number;
    philosophicalLens: string;
    maxItems: number;
    tokenBudget: number;
    includeMemories?: boolean;
    includeEntities?: boolean;
    includeRelationships?: boolean;
}

export interface CandidateItem {
    kind: string;
    text: string;
    createdAt: Date;
    updatedAt?: Date;
    lenses: string[];
    meta: Record<string, any>;
}

export class ContextBuilder {
    private initialized = false;
    private scoringAlgorithms: ScoringAlgorithms;
    private cacheManager: any; // Will be implemented with actual cache

    constructor() {
        this.scoringAlgorithms = new ScoringAlgorithms();
    }

    async initialize(): Promise<void> {
        await this.scoringAlgorithms.initialize();
        this.initialized = true;
    }

    /**
     * Main entry point for context building
     * Implements the 4-stage orchestration pipeline from sis-core
     */
    async buildContext(options: ContextBuildOptions): Promise<ContextResult> {
        if (!this.initialized) {
            throw new Error('ContextBuilder not initialized');
        }

        const startTime = Date.now();

        try {
            // Check cache first
            const cacheKey = this.generateCacheKey(options);
            const cachedResult = await this.getCachedContext(cacheKey);
            
            if (cachedResult) {
                console.log(`Context cache hit for user ${options.userId}`);
                return {
                    contextItems: cachedResult.items,
                    totalRelevance: cachedResult.totalRelevance,
                    processingTimeMs: Date.now() - startTime,
                    cacheHit: true
                };
            }

            // Stage 1: Candidate Generation (parallel retrieval)
            const candidates = await this.parallelCandidateGeneration(options);

            // Stage 2: Feature Enrichment (scoring)
            const enriched = await this.enrichCandidates(
                candidates,
                options.query,
                options.philosophicalLens
            );

            // Stage 3: Filtering & Ranking (prioritization)
            const ranked = this.filterAndRank(
                enriched,
                options.philosophicalLens as ReasoningMode,
                options.maxItems
            );

            // Stage 4: Context Assembly (token optimization)
            const assembled = this.assembleContext(ranked, options.tokenBudget, options.maxItems);

            // Calculate total relevance
            const totalRelevance = assembled.reduce((sum, item) => sum + item.score, 0);

            // Cache the result
            await this.setCachedContext(cacheKey, {
                items: assembled,
                totalRelevance,
                processingTimeMs: Date.now() - startTime,
                cachedAt: Date.now()
            });

            const processingTime = Date.now() - startTime;
            console.log(`Context built for user ${options.userId} in ${processingTime}ms, ${assembled.length} items`);

            return {
                contextItems: assembled,
                totalRelevance,
                processingTimeMs: processingTime,
                cacheHit: false
            };

        } catch (error) {
            console.error(`Context building failed for user ${options.userId}:`, error);
            return {
                contextItems: [],
                totalRelevance: 0,
                processingTimeMs: Date.now() - startTime,
                cacheHit: false
            };
        }
    }

    /**
     * Build system prompt with context
     * Generates a comprehensive system prompt incorporating the retrieved context
     */
    async buildSystemPrompt(query: string, contextResult: ContextResult): Promise<string> {
        const contextTexts = contextResult.contextItems.map((item, index) => 
            `[${index + 1}] ${item.text}`
        ).join('\n\n');

        const sourceSummary = contextResult.contextItems.map((item, index) => 
            `[${index + 1}] ${item.kind}: ${item.meta?.documentTitle || item.meta?.sourceId || 'Unknown'}`
        ).join('\n');

        return `You are an intelligent assistant with access to relevant context information. Use this context to provide accurate, helpful responses.

CONTEXT INFORMATION:
${contextTexts}

SOURCES:
${sourceSummary}

INSTRUCTIONS:
- Base your response primarily on the provided context
- Reference specific context items when relevant (e.g., "According to [1]...")
- If the context doesn't contain sufficient information, acknowledge this limitation
- Provide clear, well-structured responses
- Maintain accuracy and avoid speculation beyond the given context

USER QUERY: ${query}`;
    }

    // Stage 1: Parallel Candidate Generation

    private async parallelCandidateGeneration(options: ContextBuildOptions): Promise<CandidateItem[]> {
        const tasks: Promise<CandidateItem[]>[] = [];

        // Memory retrieval
        if (options.includeMemories !== false) {
            tasks.push(this.getMemoryCandidates(options.userId, options.query));
        }

        // Entity retrieval  
        if (options.includeEntities) {
            tasks.push(this.getEntityCandidates(options.userId, options.query));
        }

        // Relationship traversal
        if (options.includeRelationships) {
            tasks.push(this.getRelationshipCandidates(options.userId, options.query));
        }

        if (tasks.length === 0) {
            return [];
        }

        try {
            // Execute all candidate generation in parallel
            const results = await Promise.allSettled(tasks);
            const candidates: CandidateItem[] = [];

            for (const result of results) {
                if (result.status === 'fulfilled' && Array.isArray(result.value)) {
                    candidates.push(...result.value);
                } else if (result.status === 'rejected') {
                    console.warn('Candidate generation failed:', result.reason);
                }
            }

            return candidates;

        } catch (error) {
            console.error('Parallel candidate generation failed:', error);
            return [];
        }
    }

    private async getMemoryCandidates(userId: number, query: string): Promise<CandidateItem[]> {
        try {
            // TODO: Implement actual memory retrieval from database
            // This should use bounded scanning and caching as in the original
            
            // Placeholder implementation
            const memories = await this.queryMemories(userId, query);
            
            return memories.map(memory => ({
                kind: 'memory',
                text: memory.content || memory.text || '',
                createdAt: memory.createdAt || new Date(),
                updatedAt: memory.updatedAt,
                lenses: memory.lenses || [],
                meta: {
                    id: memory.id,
                    template: memory.template,
                    confidenceScore: memory.confidenceScore || 0.0
                }
            }));

        } catch (error) {
            console.error('Memory candidate retrieval failed:', error);
            return [];
        }
    }

    private async getEntityCandidates(userId: number, query: string): Promise<CandidateItem[]> {
        try {
            // TODO: Implement entity retrieval from knowledge graph
            const entities = await this.queryEntities(userId, query);
            
            return entities.map(entity => ({
                kind: 'entity',
                text: entity.description || entity.name || '',
                createdAt: entity.createdAt || new Date(),
                lenses: entity.lenses || [],
                meta: {
                    id: entity.id,
                    name: entity.name,
                    type: entity.type
                }
            }));

        } catch (error) {
            console.error('Entity candidate retrieval failed:', error);
            return [];
        }
    }

    private async getRelationshipCandidates(userId: number, query: string): Promise<CandidateItem[]> {
        try {
            // TODO: Implement relationship traversal
            const relationships = await this.queryRelationships(userId, query);
            
            return relationships.map(rel => ({
                kind: 'relationship',
                text: `${rel.fromEntity} ${rel.relationshipType} ${rel.toEntity}`,
                createdAt: rel.createdAt || new Date(),
                lenses: [],
                meta: {
                    id: rel.id,
                    type: rel.relationshipType,
                    strength: rel.strength
                }
            }));

        } catch (error) {
            console.error('Relationship candidate retrieval failed:', error);
            return [];
        }
    }

    // Stage 2: Feature Enrichment

    private async enrichCandidates(
        candidates: CandidateItem[],
        query: string,
        philosophicalLens: string
    ): Promise<ContextItem[]> {
        if (candidates.length === 0) {
            return [];
        }

        const startTime = Date.now();

        try {
            // Get query embedding if available
            const queryEmbedding = await this.getQueryEmbedding(query);

            // Process candidates concurrently
            const enrichmentTasks = candidates.map(candidate => 
                this.enrichSingleCandidate(candidate, query, queryEmbedding, [philosophicalLens])
            );

            const enrichedResults = await Promise.allSettled(enrichmentTasks);

            // Filter out failed enrichments
            const enriched: ContextItem[] = [];
            let exceptionCount = 0;

            for (const result of enrichedResults) {
                if (result.status === 'fulfilled' && result.value) {
                    enriched.push(result.value);
                } else {
                    exceptionCount++;
                    console.warn('Candidate enrichment failed:', result.status === 'rejected' ? result.reason : 'Unknown error');
                }
            }

            const enrichmentTime = Date.now() - startTime;
            console.log(`Enrichment completed in ${enrichmentTime}ms: ${enriched.length} success, ${exceptionCount} failed`);

            return enriched;

        } catch (error) {
            console.error('Concurrent enrichment failed:', error);
            // Fallback to sequential processing
            return this.enrichCandidatesSequential(candidates, query, [philosophicalLens]);
        }
    }

    private async enrichSingleCandidate(
        candidate: CandidateItem,
        query: string,
        queryEmbedding?: number[],
        activeLenses: string[] = []
    ): Promise<ContextItem | null> {
        try {
            // Calculate individual scores
            const relevance = await this.calculateRelevance(candidate, query, queryEmbedding);
            const recency = this.scoringAlgorithms.recencyDecay(candidate.createdAt);
            const priority = this.scoringAlgorithms.priorityScore(
                candidate.meta?.userPriority,
                candidate.meta?.systemPriority
            );
            const centrality = this.scoringAlgorithms.centralityScore(
                candidate.meta?.connectionCount || 0
            );
            const lensBonus = this.scoringAlgorithms.lensBonus(activeLenses, candidate.lenses);

            // Create enriched context item
            const item: ContextItem = {
                kind: candidate.kind,
                text: candidate.text,
                score: 0.0, // Will be calculated in ranking stage
                meta: candidate.meta,
                relevanceScore: relevance,
                recencyScore: recency,
                priorityScore: priority,
                centralityScore: centrality,
                lensBonusScore: lensBonus
            };

            return item;

        } catch (error) {
            console.warn(`Failed to enrich candidate ${candidate.kind}:`, error);
            return null;
        }
    }

    private async enrichCandidatesSequential(
        candidates: CandidateItem[],
        query: string,
        activeLenses: string[]
    ): Promise<ContextItem[]> {
        const enriched: ContextItem[] = [];
        const queryEmbedding = await this.getQueryEmbedding(query);

        for (const candidate of candidates) {
            const item = await this.enrichSingleCandidate(candidate, query, queryEmbedding, activeLenses);
            if (item) {
                enriched.push(item);
            }
        }

        return enriched;
    }

    // Stage 3: Filtering & Ranking

    private filterAndRank(
        enriched: ContextItem[],
        reasoningMode: ReasoningMode,
        topK: number
    ): ContextItem[] {
        // Get weights for reasoning mode
        const weights = this.scoringAlgorithms.getReasoningModeWeights(reasoningMode);

        // Calculate final scores
        for (const item of enriched) {
            item.score = this.scoringAlgorithms.calculateFinalScore(
                item.relevanceScore || 0,
                item.recencyScore || 0,
                item.priorityScore || 0,
                item.centralityScore || 0,
                item.lensBonusScore || 0,
                weights
            );
        }

        // Sort by final score and return top_k
        return enriched
            .sort((a, b) => b.score - a.score)
            .slice(0, topK);
    }

    // Stage 4: Context Assembly

    private assembleContext(
        ranked: ContextItem[],
        tokenBudget: number,
        maxItems: number
    ): ContextItem[] {
        const assembled: ContextItem[] = [];
        let usedTokens = 0;

        for (const item of ranked.slice(0, maxItems)) {
            // Estimate tokens (rough approximation: 4 chars per token)
            const estimatedTokens = Math.ceil(item.text.length / 4);

            if (usedTokens + estimatedTokens <= tokenBudget) {
                assembled.push(item);
                usedTokens += estimatedTokens;
            } else {
                // Try to fit truncated version
                const remainingTokens = tokenBudget - usedTokens;
                if (remainingTokens > 20) { // Minimum useful content
                    const truncatedChars = remainingTokens * 4 - 20; // Leave room for truncation marker
                    if (truncatedChars > 0) {
                        const truncatedItem = { ...item };
                        truncatedItem.text = item.text.substring(0, truncatedChars) + "...";
                        assembled.push(truncatedItem);
                    }
                }
                break;
            }
        }

        return assembled;
    }

    // Helper methods

    private async calculateRelevance(
        candidate: CandidateItem,
        query: string,
        queryEmbedding?: number[]
    ): Promise<number> {
        if (queryEmbedding && candidate.meta?.embedding) {
            // Use semantic similarity if embeddings are available
            return this.scoringAlgorithms.semanticSimilarity(queryEmbedding, candidate.meta.embedding);
        }

        // Fallback to keyword relevance
        return this.scoringAlgorithms.keywordRelevanceScore(query, candidate.text);
    }

    private async getQueryEmbedding(query: string): Promise<number[] | undefined> {
        try {
            // TODO: Implement actual embedding generation
            // This would call your embedding service
            return undefined;
        } catch (error) {
            console.debug('Query embedding generation failed:', error);
            return undefined;
        }
    }

    private generateCacheKey(options: ContextBuildOptions): string {
        const keyData = [
            options.userId,
            options.query,
            options.philosophicalLens,
            options.maxItems,
            options.tokenBudget,
            options.includeMemories ? 'mem' : '',
            options.includeEntities ? 'ent' : '',
            options.includeRelationships ? 'rel' : ''
        ].join(':');

        // Simple hash function
        let hash = 0;
        for (let i = 0; i < keyData.length; i++) {
            const char = keyData.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32-bit integer
        }

        return `context:${Math.abs(hash)}`;
    }

    // Placeholder methods for database integration

    private async queryMemories(userId: number, query: string): Promise<any[]> {
        try {
            const memoryRepo = getStructuredMemoryEntryRepository();
            const memories = await memoryRepo
                .createQueryBuilder('memory')
                .where('memory.userId = :userId', { userId })
                .andWhere('memory.fullText LIKE :query', { query: `%${query}%` })
                .orderBy('memory.createdAt', 'DESC')
                .limit(20)
                .getMany();
            
            return memories;
        } catch (error) {
            console.error('Memory query failed:', error);
            return [];
        }
    }

    private async queryEntities(userId: number, query: string): Promise<any[]> {
        try {
            const conceptRepo = getLearnedConceptRepository();
            const concepts = await conceptRepo
                .createQueryBuilder('concept')
                .where('concept.userId = :userId', { userId })
                .andWhere('(concept.name LIKE :query OR concept.description LIKE :query)', 
                    { query: `%${query}%` })
                .orderBy('concept.importanceScore', 'DESC')
                .limit(15)
                .getMany();
            
            return concepts;
        } catch (error) {
            console.error('Entity query failed:', error);
            return [];
        }
    }

    private async queryRelationships(userId: number, query: string): Promise<any[]> {
        try {
            const relationshipRepo = getConceptRelationshipRepository();
            const relationships = await relationshipRepo
                .createQueryBuilder('rel')
                .leftJoinAndSelect('rel.fromConcept', 'fromConcept')
                .leftJoinAndSelect('rel.toConcept', 'toConcept')
                .where('fromConcept.userId = :userId', { userId })
                .andWhere('(fromConcept.name LIKE :query OR toConcept.name LIKE :query OR rel.notes LIKE :query)', 
                    { query: `%${query}%` })
                .orderBy('rel.strength', 'DESC')
                .limit(10)
                .getMany();
            
            return relationships;
        } catch (error) {
            console.error('Relationship query failed:', error);
            return [];
        }
    }

    private async getCachedContext(key: string): Promise<any> {
        // TODO: Implement actual cache retrieval
        return null;
    }

    private async setCachedContext(key: string, data: any): Promise<void> {
        // TODO: Implement actual cache storage
    }
}