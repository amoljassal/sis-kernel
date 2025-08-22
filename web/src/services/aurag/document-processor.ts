/**
 * SIS Document Processor
 * Extracted from sis-core Django implementation
 * 
 * Handles document chunking, embedding generation, and concept extraction
 */

import { Document, DocumentChunk, LearnedConcept } from './types';
import { 
    getDocumentChunkRepository,
    getLearnedConceptRepository,
    getConceptDocumentLinkRepository,
    getConceptRelationshipRepository
} from '../../database/repositories';
import { 
    DocumentChunk as DocumentChunkEntity,
    LearnedConcept as LearnedConceptEntity,
    ConceptDocumentLink,
    ConceptRelationship,
    RelationshipType
} from '../../database/entities';

export interface ChunkingOptions {
    maxChunkSize: number;
    chunkOverlap: number;
    preserveCodeBlocks: boolean;
    preserveHeaders: boolean;
}

export interface ConceptExtractionResult {
    concepts: Array<{
        name: string;
        description: string;
        confidence: number;
        importance: number;
        relatedTerms: string[];
    }>;
    relationships: Array<{
        fromConcept: string;
        toConcept: string;
        relationshipType: string;
        strength: number;
        evidence: string;
    }>;
}

export class DocumentProcessor {
    private initialized = false;
    private embeddingService: any; // Will be implemented with actual embedding service
    private knowledgeExtractor: any; // Will be implemented with concept extraction

    constructor() {
        // Initialize with default settings
    }

    async initialize(): Promise<void> {
        // Initialize embedding service and knowledge extractor
        // TODO: Implement actual initialization
        this.initialized = true;
    }

    /**
     * Chunk document content and store with embeddings
     * Extracted from sis-core unified_rag_service.py _chunk_document method
     */
    async chunkDocument(document: Document): Promise<number> {
        if (!this.initialized) {
            throw new Error('DocumentProcessor not initialized');
        }

        const options: ChunkingOptions = {
            maxChunkSize: 1000,
            chunkOverlap: 200,
            preserveCodeBlocks: true,
            preserveHeaders: true
        };

        try {
            // Chunk the document content
            const chunks = this.createChunks(document.content, {
                documentId: document.id,
                title: document.title
            }, options);

            // Extract text content for embedding generation
            const chunkTexts = chunks.map(chunk => chunk.content);

            if (chunkTexts.length === 0) {
                return 0;
            }

            // Generate embeddings for all chunks
            const embeddings = await this.generateEmbeddings(chunkTexts);

            // Store chunks with embeddings in database
            for (let i = 0; i < chunks.length; i++) {
                const chunk = chunks[i];
                const embedding = embeddings[i];

                await this.storeDocumentChunk({
                    documentId: document.id,
                    chunkIndex: i,
                    content: chunk.content,
                    contentHash: await this.createContentHash(chunk.content),
                    tokenCount: this.estimateTokenCount(chunk.content),
                    startChar: chunk.startChar,
                    endChar: chunk.endChar,
                    contentType: chunk.contentType,
                    complexity: this.calculateComplexity(chunk.content),
                    sectionTitle: chunk.sectionTitle,
                    embedding: embedding,
                    embeddingModel: 'ollama' // TODO: Make configurable
                });
            }

            console.log(`Created ${chunks.length} chunks for document: ${document.title}`);
            return chunks.length;

        } catch (error) {
            console.error('Error chunking document:', error);
            throw error;
        }
    }

    /**
     * Extract concepts from document and update knowledge base
     * Extracted from sis-core unified_rag_service.py _extract_document_concepts method
     */
    async extractConcepts(document: Document): Promise<number> {
        if (!this.initialized) {
            throw new Error('DocumentProcessor not initialized');
        }

        try {
            // Extract concepts using knowledge extractor
            const extractionResult = await this.performConceptExtraction({
                text: document.content,
                contentType: document.sourceType,
                sourceHash: document.contentHash
            });

            let conceptsCreated = 0;

            // Store extracted concepts
            for (const conceptData of extractionResult.concepts) {
                if (!conceptData.name) {
                    continue;
                }

                // Check if concept already exists
                const existingConcept = await this.findExistingConcept(
                    document.userId,
                    conceptData.name
                );

                if (existingConcept) {
                    // Update existing concept
                    await this.updateConcept(existingConcept.id, {
                        encounterCount: existingConcept.encounterCount + 1,
                        lastReinforced: new Date(),
                        confidenceScore: Math.min(1.0, existingConcept.confidenceScore + 0.1)
                    });
                } else {
                    // Create new concept
                    await this.createConcept({
                        userId: document.userId,
                        name: conceptData.name,
                        description: conceptData.description,
                        confidenceScore: conceptData.confidence,
                        importanceScore: conceptData.importance,
                        personalNotes: '',
                        tags: conceptData.relatedTerms,
                        encounterCount: 1,
                        firstEncountered: new Date(),
                        lastReinforced: new Date()
                    });
                    conceptsCreated++;
                }

                // Link concept to document
                await this.linkConceptToDocument({
                    conceptName: conceptData.name,
                    documentId: document.id,
                    relevanceScore: conceptData.importance
                });
            }

            // Store concept relationships
            for (const relationship of extractionResult.relationships) {
                try {
                    const fromConcept = await this.findExistingConcept(
                        document.userId,
                        relationship.fromConcept
                    );
                    const toConcept = await this.findExistingConcept(
                        document.userId,
                        relationship.toConcept
                    );

                    if (fromConcept && toConcept) {
                        await this.createConceptRelationship({
                            fromConceptId: fromConcept.id,
                            toConceptId: toConcept.id,
                            relationshipType: relationship.relationshipType,
                            strength: relationship.strength,
                            notes: relationship.evidence
                        });
                    }
                } catch (error) {
                    console.warn('Failed to create concept relationship:', error);
                    // Skip relationships where concepts don't exist
                    continue;
                }
            }

            console.log(`Extracted ${conceptsCreated} new concepts from document: ${document.title}`);
            return conceptsCreated;

        } catch (error) {
            console.error(`Concept extraction failed for document ${document.id}:`, error);
            return 0;
        }
    }

    // Private methods for chunking

    private createChunks(
        content: string,
        metadata: { documentId: number; title: string },
        options: ChunkingOptions
    ): Array<{
        content: string;
        startChar: number;
        endChar: number;
        contentType: string;
        sectionTitle?: string;
    }> {
        const chunks: Array<{
            content: string;
            startChar: number;
            endChar: number;
            contentType: string;
            sectionTitle?: string;
        }> = [];

        // Simple chunking by paragraphs and size limits
        const paragraphs = content.split(/\n\s*\n/);
        let currentChunk = '';
        let startChar = 0;

        for (const paragraph of paragraphs) {
            const trimmedParagraph = paragraph.trim();
            if (!trimmedParagraph) continue;

            // Check if adding this paragraph would exceed chunk size
            if (currentChunk.length + trimmedParagraph.length > options.maxChunkSize && currentChunk.length > 0) {
                // Create chunk from current content
                chunks.push({
                    content: currentChunk.trim(),
                    startChar: startChar,
                    endChar: startChar + currentChunk.length,
                    contentType: this.detectContentType(currentChunk),
                    sectionTitle: this.extractSectionTitle(currentChunk)
                });

                // Start new chunk with overlap
                const overlapSize = Math.min(options.chunkOverlap, currentChunk.length);
                const overlapText = currentChunk.slice(-overlapSize);
                startChar = startChar + currentChunk.length - overlapSize;
                currentChunk = overlapText + '\n' + trimmedParagraph;
            } else {
                // Add paragraph to current chunk
                if (currentChunk.length > 0) {
                    currentChunk += '\n\n';
                }
                currentChunk += trimmedParagraph;
            }
        }

        // Add final chunk if it has content
        if (currentChunk.trim().length > 0) {
            chunks.push({
                content: currentChunk.trim(),
                startChar: startChar,
                endChar: startChar + currentChunk.length,
                contentType: this.detectContentType(currentChunk),
                sectionTitle: this.extractSectionTitle(currentChunk)
            });
        }

        return chunks;
    }

    private detectContentType(text: string): string {
        // Simple content type detection
        if (/^#+\s/.test(text.trim())) {
            return 'header';
        }
        if (/```[\s\S]*```/.test(text)) {
            return 'code';
        }
        if (/^\d+\.|\*|\-/.test(text.trim())) {
            return 'list';
        }
        return 'paragraph';
    }

    private extractSectionTitle(text: string): string | undefined {
        // Extract section title from markdown headers
        const headerMatch = text.match(/^#+\s+(.+)/m);
        return headerMatch ? headerMatch[1].trim() : undefined;
    }

    private estimateTokenCount(text: string): number {
        // Rough token estimation: ~4 characters per token
        return Math.ceil(text.length / 4);
    }

    private calculateComplexity(text: string): number {
        // Simple complexity score based on sentence length and vocabulary
        const sentences = text.split(/[.!?]+/).filter(s => s.trim().length > 0);
        const words = text.split(/\s+/).filter(w => w.length > 0);

        if (sentences.length === 0 || words.length === 0) {
            return 0.0;
        }

        const avgSentenceLength = words.length / sentences.length;
        const avgWordLength = words.reduce((sum, word) => sum + word.length, 0) / words.length;

        // Normalize to 0-1 scale
        const lengthComplexity = Math.min(1.0, avgSentenceLength / 20);
        const vocabularyComplexity = Math.min(1.0, avgWordLength / 8);

        return (lengthComplexity + vocabularyComplexity) / 2;
    }

    // Placeholder methods for actual implementation

    private async generateEmbeddings(texts: string[]): Promise<number[][]> {
        // TODO: Implement actual embedding generation
        // This would call your embedding service (Ollama, OpenAI, etc.)
        return texts.map(() => new Array(384).fill(0).map(() => Math.random()));
    }

    private async performConceptExtraction(params: {
        text: string;
        contentType: string;
        sourceHash: string;
    }): Promise<ConceptExtractionResult> {
        // TODO: Implement actual concept extraction
        // This would use NLP techniques to extract concepts and relationships
        return {
            concepts: [],
            relationships: []
        };
    }

    private async createContentHash(content: string): Promise<string> {
        const encoder = new TextEncoder();
        const data = encoder.encode(content);
        const hashBuffer = await crypto.subtle.digest('SHA-256', data);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    }

    // Database interaction methods (to be implemented with your chosen database)

    private async storeDocumentChunk(chunkData: any): Promise<void> {
        try {
            const chunkRepo = getDocumentChunkRepository();
            const chunk = new DocumentChunkEntity();
            
            chunk.documentId = chunkData.documentId;
            chunk.chunkIndex = chunkData.chunkIndex;
            chunk.content = chunkData.content;
            chunk.contentHash = chunkData.contentHash;
            chunk.tokenCount = chunkData.tokenCount;
            chunk.startChar = chunkData.startChar;
            chunk.endChar = chunkData.endChar;
            chunk.contentType = chunkData.contentType;
            chunk.complexity = chunkData.complexity;
            chunk.sectionTitle = chunkData.sectionTitle;
            chunk.embedding = chunkData.embedding;
            chunk.embeddingModel = chunkData.embeddingModel;
            
            await chunkRepo.save(chunk);
            console.log('Stored chunk:', chunkData.contentHash.substring(0, 8));
        } catch (error) {
            console.error('Failed to store chunk:', error);
            throw error;
        }
    }

    private async findExistingConcept(userId: number, name: string): Promise<LearnedConceptEntity | null> {
        try {
            const conceptRepo = getLearnedConceptRepository();
            return await conceptRepo.findOne({
                where: { userId, name }
            });
        } catch (error) {
            console.error('Failed to find concept:', error);
            return null;
        }
    }

    private async createConcept(conceptData: any): Promise<LearnedConceptEntity> {
        try {
            const conceptRepo = getLearnedConceptRepository();
            const concept = conceptRepo.create(conceptData);
            const savedConcepts = await conceptRepo.save(concept);
            // TypeORM save returns array when passed entity, so get first element
            return Array.isArray(savedConcepts) ? savedConcepts[0] : savedConcepts;
        } catch (error) {
            console.error('Failed to create concept:', error);
            throw error;
        }
    }

    private async updateConcept(conceptId: number, updates: any): Promise<void> {
        try {
            const conceptRepo = getLearnedConceptRepository();
            await conceptRepo.update(conceptId, updates);
            console.log('Updated concept:', conceptId);
        } catch (error) {
            console.error('Failed to update concept:', error);
            throw error;
        }
    }

    private async linkConceptToDocument(linkData: any): Promise<void> {
        try {
            const linkRepo = getConceptDocumentLinkRepository();
            const conceptRepo = getLearnedConceptRepository();
            
            // Find the concept by name and userId
            const concept = await conceptRepo.findOne({
                where: { 
                    name: linkData.conceptName,
                    userId: linkData.userId || 1 // Default user for now
                }
            });
            
            if (concept) {
                const link = new ConceptDocumentLink();
                link.conceptId = concept.id;
                link.documentId = linkData.documentId;
                link.relevanceScore = linkData.relevanceScore;
                
                await linkRepo.save(link);
                console.log('Linked concept to document:', linkData.conceptName);
            }
        } catch (error) {
            console.error('Failed to link concept to document:', error);
            // Don't throw - this is not critical
        }
    }

    private async createConceptRelationship(relationshipData: any): Promise<void> {
        try {
            const relationshipRepo = getConceptRelationshipRepository();
            const relationship = new ConceptRelationship();
            
            relationship.fromConceptId = relationshipData.fromConceptId;
            relationship.toConceptId = relationshipData.toConceptId;
            relationship.relationshipType = relationshipData.relationshipType as RelationshipType;
            relationship.strength = relationshipData.strength;
            relationship.notes = relationshipData.notes;
            
            await relationshipRepo.save(relationship);
            console.log('Created concept relationship:', relationship.relationshipType);
        } catch (error) {
            console.error('Failed to create concept relationship:', error);
            // Don't throw - this is not critical
        }
    }
}