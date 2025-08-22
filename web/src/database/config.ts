/**
 * TypeORM Database Configuration
 * Extracted from sis-core Django database setup
 */

import 'reflect-metadata';
import { DataSource } from 'typeorm';
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
} from './entities';

export const AppDataSource = new DataSource({
    type: 'sqlite',
    database: './sis-aurag.db',
    synchronize: true, // Auto-create/update tables (disable in production)
    logging: process.env.NODE_ENV === 'development',
    entities: [
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
    ],
    migrations: [],
    subscribers: [],
});

/**
 * Initialize database connection
 */
export async function initializeDatabase(): Promise<DataSource> {
    try {
        if (!AppDataSource.isInitialized) {
            await AppDataSource.initialize();
            console.log('Database connection established');
        }
        return AppDataSource;
    } catch (error) {
        console.error('Database connection failed:', error);
        throw error;
    }
}

/**
 * Close database connection
 */
export async function closeDatabase(): Promise<void> {
    if (AppDataSource.isInitialized) {
        await AppDataSource.destroy();
        console.log('Database connection closed');
    }
}