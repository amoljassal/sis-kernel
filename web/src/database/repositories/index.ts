/**
 * Database Repository Exports
 * Centralized data access layer for AURAG system
 */

import { Repository } from 'typeorm';
import { AppDataSource } from '../config';
import {
    DocumentSource,
    DocumentChunk,
    LearnedConcept,
    ConceptDocumentLink,
    ConceptRelationship,
    MemoryTemplate,
    StructuredMemoryEntry,
    RAGQuery,
    ChunkRetrievalScore,
    ConceptRetrievalScore,
    PersonalKnowledgeGraph
} from '../entities';

/**
 * Repository factory function
 * Returns typed repositories for all entities
 */
export function getRepositories() {
    return {
        documentSource: AppDataSource.getRepository(DocumentSource),
        documentChunk: AppDataSource.getRepository(DocumentChunk),
        learnedConcept: AppDataSource.getRepository(LearnedConcept),
        conceptDocumentLink: AppDataSource.getRepository(ConceptDocumentLink),
        conceptRelationship: AppDataSource.getRepository(ConceptRelationship),
        memoryTemplate: AppDataSource.getRepository(MemoryTemplate),
        structuredMemoryEntry: AppDataSource.getRepository(StructuredMemoryEntry),
        ragQuery: AppDataSource.getRepository(RAGQuery),
        chunkRetrievalScore: AppDataSource.getRepository(ChunkRetrievalScore),
        conceptRetrievalScore: AppDataSource.getRepository(ConceptRetrievalScore),
        personalKnowledgeGraph: AppDataSource.getRepository(PersonalKnowledgeGraph),
    };
}

// Export individual repository getters for convenience
export const getDocumentSourceRepository = (): Repository<DocumentSource> => 
    AppDataSource.getRepository(DocumentSource);

export const getDocumentChunkRepository = (): Repository<DocumentChunk> => 
    AppDataSource.getRepository(DocumentChunk);

export const getLearnedConceptRepository = (): Repository<LearnedConcept> => 
    AppDataSource.getRepository(LearnedConcept);

export const getConceptDocumentLinkRepository = (): Repository<ConceptDocumentLink> => 
    AppDataSource.getRepository(ConceptDocumentLink);

export const getConceptRelationshipRepository = (): Repository<ConceptRelationship> => 
    AppDataSource.getRepository(ConceptRelationship);

export const getMemoryTemplateRepository = (): Repository<MemoryTemplate> => 
    AppDataSource.getRepository(MemoryTemplate);

export const getStructuredMemoryEntryRepository = (): Repository<StructuredMemoryEntry> => 
    AppDataSource.getRepository(StructuredMemoryEntry);

export const getRAGQueryRepository = (): Repository<RAGQuery> => 
    AppDataSource.getRepository(RAGQuery);

export const getChunkRetrievalScoreRepository = (): Repository<ChunkRetrievalScore> => 
    AppDataSource.getRepository(ChunkRetrievalScore);

export const getConceptRetrievalScoreRepository = (): Repository<ConceptRetrievalScore> => 
    AppDataSource.getRepository(ConceptRetrievalScore);

export const getPersonalKnowledgeGraphRepository = (): Repository<PersonalKnowledgeGraph> => 
    AppDataSource.getRepository(PersonalKnowledgeGraph);