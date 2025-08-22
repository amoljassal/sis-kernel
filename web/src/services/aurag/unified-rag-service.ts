/**
 * SIS Unified RAG Service - Personal AI Intelligence Engine
 * Extracted and adapted from Django sis-core implementation
 * 
 * Main service coordinating all RAG components for personal AI
 * Philosophy: Intelligence in Structure, Not Parameters
 */

import { RAGResponse, DocumentProcessingResult, ContextItem } from './types';
import { ScoringAlgorithms } from './scoring-algorithms';
import { ConfidenceScorer } from './confidence-scorer';
import { DocumentProcessor } from './document-processor';
import { ContextBuilder } from './context-builder';
import { initializeDatabase } from '../../database/config';
import { 
    getDocumentSourceRepository, 
    getPersonalKnowledgeGraphRepository,
    getRAGQueryRepository,
    getLearnedConceptRepository,
    getConceptRelationshipRepository
} from '../../database/repositories';
import { DocumentSource as DocumentSourceEntity, SourceType } from '../../database/entities';

export interface RAGConfig {
    maxContextItems: number;
    tokenBudget: number;
    defaultPhilosophicalLens: string;
    providers: {
        embedding: string;
        llm: string;
    };
}

export class UnifiedRAGService {
    private initialized = false;
    private config: RAGConfig;
    private documentProcessor: DocumentProcessor;
    private contextBuilder: ContextBuilder;
    private scoringAlgorithms: ScoringAlgorithms;
    private confidenceScorer: ConfidenceScorer;

    constructor(config: Partial<RAGConfig> = {}) {
        this.config = {
            maxContextItems: 10,
            tokenBudget: 1200,
            defaultPhilosophicalLens: 'analytical',
            providers: {
                embedding: 'ollama',
                llm: 'ollama'
            },
            ...config
        };

        // Initialize components
        this.documentProcessor = new DocumentProcessor();
        this.contextBuilder = new ContextBuilder();
        this.scoringAlgorithms = new ScoringAlgorithms();
        this.confidenceScorer = new ConfidenceScorer();
    }

    async initialize(): Promise<boolean> {
        if (this.initialized) {
            return true;
        }

        try {
            // Initialize database connection first
            await initializeDatabase();
            
            // Initialize all components
            await Promise.all([
                this.documentProcessor.initialize(),
                this.contextBuilder.initialize(),
                this.scoringAlgorithms.initialize(),
                this.confidenceScorer.initialize()
            ]);

            this.initialized = true;
            console.log('SIS Unified RAG Service initialized successfully');
            return true;

        } catch (error) {
            console.error('Failed to initialize RAG service:', error);
            return false;
        }
    }

    /**
     * Process a document through the RAG pipeline
     * 
     * Steps:
     * 1. Create document record
     * 2. Chunk the content  
     * 3. Generate embeddings
     * 4. Extract concepts and relationships
     * 5. Update personal knowledge graph
     */
    async processDocument(
        userId: number,
        title: string,
        content: string,
        sourceType: string = 'other',
        author: string = '',
        metadata: Record<string, any> = {}
    ): Promise<DocumentProcessingResult> {
        if (!await this.initialize()) {
            throw new Error('RAG service not initialized');
        }

        const startTime = Date.now();

        try {
            // Create content hash for deduplication
            const contentHash = await this.createContentHash(content);
            
            // Check if document already exists
            const existingDoc = await this.checkExistingDocument(userId, contentHash);
            if (existingDoc) {
                console.log(`Document already exists: ${title}`);
                return {
                    documentId: existingDoc.id,
                    chunksCreated: existingDoc.chunkCount || 0,
                    conceptsExtracted: existingDoc.conceptCount || 0,
                    processingTimeMs: 0,
                    success: true
                };
            }

            // Process document
            const document = await this.createDocumentRecord({
                userId,
                title,
                sourceType,
                author,
                content,
                contentHash,
                metadata
            });

            // Process content through pipeline
            const chunksCreated = await this.documentProcessor.chunkDocument(document);
            const conceptsExtracted = await this.documentProcessor.extractConcepts(document);

            // Update document status
            await this.updateDocumentStatus(document.id, {
                processed: true,
                chunkCount: chunksCreated,
                conceptCount: conceptsExtracted
            });

            // Update knowledge graph statistics
            await this.updateKnowledgeGraphStats(userId);

            const processingTime = Date.now() - startTime;

            console.log(`Document processed: ${title} (${chunksCreated} chunks, ${conceptsExtracted} concepts)`);

            return {
                documentId: document.id,
                chunksCreated,
                conceptsExtracted,
                processingTimeMs: processingTime,
                success: true
            };

        } catch (error) {
            const processingTime = Date.now() - startTime;
            console.error('Document processing failed:', error);

            return {
                documentId: 0,
                chunksCreated: 0,
                conceptsExtracted: 0,
                processingTimeMs: processingTime,
                success: false,
                errorMessage: error instanceof Error ? error.message : String(error)
            };
        }
    }

    /**
     * Process a RAG query with intelligent context building
     * 
     * Steps:
     * 1. Build context using multi-stage retrieval
     * 2. Apply philosophical lens weighting
     * 3. Generate response using LLM with context  
     * 4. Store query for learning and optimization
     */
    async processRAGQuery(
        userId: number,
        query: string,
        philosophicalLens: string = this.config.defaultPhilosophicalLens,
        maxContextItems: number = this.config.maxContextItems
    ): Promise<RAGResponse> {
        if (!await this.initialize()) {
            throw new Error('RAG service not initialized');
        }

        const startTime = Date.now();

        try {
            // Build intelligent context
            const contextResult = await this.contextBuilder.buildContext({
                query,
                userId,
                philosophicalLens,
                maxItems: maxContextItems,
                tokenBudget: this.config.tokenBudget
            });

            // Generate system prompt with context
            const systemPrompt = await this.contextBuilder.buildSystemPrompt(
                query, 
                contextResult
            );

            // Process with LLM (placeholder - to be implemented with actual LLM integration)
            const llmResponse = await this.processWithLLM(systemPrompt, query);

            if (!llmResponse.success) {
                throw new Error('LLM processing failed');
            }

            const responseText = llmResponse.content;

            // Calculate confidence score based on context quality
            const confidenceScore = await this.confidenceScorer.calculateConfidence(
                contextResult.contextItems,
                responseText,
                philosophicalLens,
                this.config.providers.llm
            );

            // Extract sources used
            const sourcesUsed = contextResult.contextItems.map(item => 
                item.meta?.documentTitle || item.meta?.sourceId || 'Unknown'
            );

            const processingTime = Date.now() - startTime;

            // Store query for learning
            const queryRecord = await this.storeQueryRecord({
                userId,
                queryText: query,
                philosophicalLens,
                responseText,
                confidenceScore: confidenceScore.overall,
                processingTimeMs: processingTime
            });

            return {
                responseText,
                confidenceScore: confidenceScore.overall,
                contextItems: contextResult.contextItems,
                philosophicalLens,
                processingTimeMs: processingTime,
                sourcesUsed,
                queryId: queryRecord?.id
            };

        } catch (error) {
            const processingTime = Date.now() - startTime;
            console.error('RAG query processing failed:', error);

            return {
                responseText: `I encountered an error processing your query: ${error instanceof Error ? error.message : String(error)}`,
                confidenceScore: 0.0,
                contextItems: [],
                philosophicalLens,
                processingTimeMs: processingTime,
                sourcesUsed: []
            };
        }
    }

    /**
     * Get personal knowledge graph summary
     */
    async getKnowledgeGraphSummary(userId: number): Promise<Record<string, any>> {
        try {
            // This would query your database for knowledge graph statistics
            // Implementation depends on your database schema
            
            const summary = await this.fetchKnowledgeGraphStats(userId);
            
            return {
                totalDocuments: summary.totalDocuments || 0,
                totalConcepts: summary.totalConcepts || 0,
                totalRelationships: summary.totalRelationships || 0,
                knowledgeDepth: summary.knowledgeDepth || 0,
                learningVelocity: summary.learningVelocity || 0,
                graphDensity: summary.graphDensity || 0,
                topConcepts: summary.topConcepts || [],
                recentDocuments: summary.recentDocuments || []
            };

        } catch (error) {
            console.error('Failed to get knowledge graph summary:', error);
            return { error: error instanceof Error ? error.message : String(error) };
        }
    }

    // Private helper methods

    private async createContentHash(content: string): Promise<string> {
        const encoder = new TextEncoder();
        const data = encoder.encode(content);
        const hashBuffer = await crypto.subtle.digest('SHA-256', data);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    }

    private async checkExistingDocument(userId: number, contentHash: string): Promise<DocumentSourceEntity | null> {
        try {
            const docRepo = getDocumentSourceRepository();
            return await docRepo.findOne({
                where: { userId, contentHash }
            });
        } catch (error) {
            console.error('Failed to check existing document:', error);
            return null;
        }
    }

    private async createDocumentRecord(documentData: any): Promise<DocumentSourceEntity> {
        try {
            const docRepo = getDocumentSourceRepository();
            const document = new DocumentSourceEntity();
            
            document.userId = documentData.userId;
            document.title = documentData.title;
            document.sourceType = documentData.sourceType as SourceType;
            document.author = documentData.author;
            document.content = documentData.content;
            document.contentHash = documentData.contentHash;
            document.tags = documentData.metadata?.tags || [];
            document.notes = documentData.metadata?.notes || '';
            
            return await docRepo.save(document);
        } catch (error) {
            console.error('Failed to create document record:', error);
            throw error;
        }
    }

    private async updateDocumentStatus(documentId: number, status: any): Promise<void> {
        try {
            const docRepo = getDocumentSourceRepository();
            await docRepo.update(documentId, status);
        } catch (error) {
            console.error('Failed to update document status:', error);
        }
    }

    private async updateKnowledgeGraphStats(userId: number): Promise<void> {
        try {
            const graphRepo = getPersonalKnowledgeGraphRepository();
            const docRepo = getDocumentSourceRepository();
            const conceptRepo = getLearnedConceptRepository();
            const relationshipRepo = getConceptRelationshipRepository();
            
            // Get current counts
            const [totalDocuments, totalConcepts, totalRelationships] = await Promise.all([
                docRepo.count({ where: { userId } }),
                conceptRepo.count({ where: { userId } }),
                relationshipRepo
                    .createQueryBuilder('rel')
                    .leftJoin('rel.fromConcept', 'concept')
                    .where('concept.userId = :userId', { userId })
                    .getCount()
            ]);
            
            const totalChunks = await docRepo
                .createQueryBuilder('doc')
                .where('doc.userId = :userId', { userId })
                .getMany()
                .then(docs => docs.reduce((sum, doc) => sum + (doc.chunkCount || 0), 0));

            // Find or create knowledge graph record
            let knowledgeGraph = await graphRepo.findOne({ where: { userId } });
            
            if (!knowledgeGraph) {
                knowledgeGraph = graphRepo.create({ userId });
            }
            
            // Update statistics
            knowledgeGraph.totalDocuments = totalDocuments;
            knowledgeGraph.totalChunks = totalChunks;
            knowledgeGraph.totalConcepts = totalConcepts;
            knowledgeGraph.totalRelationships = totalRelationships;
            
            // Calculate derived metrics
            if (totalConcepts > 0) {
                knowledgeGraph.graphDensity = totalRelationships / totalConcepts;
                
                // Calculate average confidence (knowledge depth)
                const concepts = await conceptRepo.find({ where: { userId } });
                knowledgeGraph.knowledgeDepth = concepts.length > 0 
                    ? concepts.reduce((sum, c) => sum + c.confidenceScore, 0) / concepts.length 
                    : 0;
            }
            
            await graphRepo.save(knowledgeGraph);
        } catch (error) {
            console.error('Failed to update knowledge graph stats:', error);
        }
    }

    private async processWithLLM(systemPrompt: string, query: string): Promise<any> {
        // Placeholder for LLM integration
        // This will be implemented with your chosen LLM provider
        return {
            success: true,
            content: "This is a placeholder response. LLM integration to be implemented."
        };
    }

    private async storeQueryRecord(queryData: any): Promise<any> {
        try {
            const queryRepo = getRAGQueryRepository();
            const query = queryRepo.create(queryData);
            return await queryRepo.save(query);
        } catch (error) {
            console.error('Failed to store query record:', error);
            return { id: Date.now(), ...queryData };
        }
    }

    private async fetchKnowledgeGraphStats(userId: number): Promise<any> {
        try {
            const graphRepo = getPersonalKnowledgeGraphRepository();
            const docRepo = getDocumentSourceRepository();
            const conceptRepo = getLearnedConceptRepository();
            
            const knowledgeGraph = await graphRepo.findOne({ where: { userId } });
            
            // Get recent documents
            const recentDocuments = await docRepo.find({
                where: { userId },
                order: { createdAt: 'DESC' },
                take: 5
            });
            
            // Get top concepts by importance
            const topConcepts = await conceptRepo.find({
                where: { userId },
                order: { importanceScore: 'DESC' },
                take: 10
            });
            
            return {
                ...knowledgeGraph,
                recentDocuments: recentDocuments.map(doc => ({
                    id: doc.id,
                    title: doc.title,
                    sourceType: doc.sourceType,
                    createdAt: doc.createdAt
                })),
                topConcepts: topConcepts.map(concept => ({
                    id: concept.id,
                    name: concept.name,
                    confidenceScore: concept.confidenceScore,
                    importanceScore: concept.importanceScore
                }))
            };
        } catch (error) {
            console.error('Failed to fetch knowledge graph stats:', error);
            return {};
        }
    }
}

// Factory function for easy usage
export function createRAGService(config?: Partial<RAGConfig>): UnifiedRAGService {
    return new UnifiedRAGService(config);
}