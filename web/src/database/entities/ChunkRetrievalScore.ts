/**
 * Chunk Retrieval Score Entity
 * Converted from Django rag/models.py ChunkRetrievalScore model
 */

import {
    Entity,
    PrimaryGeneratedColumn,
    Column,
    CreateDateColumn,
    ManyToOne,
    JoinColumn
} from 'typeorm';
import { RAGQuery } from './RAGQuery';
import { DocumentChunk } from './DocumentChunk';

@Entity('chunk_retrieval_scores')
export class ChunkRetrievalScore {
    @PrimaryGeneratedColumn()
    id!: number;

    @Column({ type: 'int' })
    queryId!: number;

    @Column({ type: 'int' })
    chunkId!: number;

    @Column({ type: 'float' })
    relevanceScore!: number;

    @Column({ type: 'int' })
    rankPosition!: number;

    @Column({ type: 'boolean', default: false })
    usedInResponse!: boolean;

    @Column({ type: 'json', default: () => "'{}'" })
    metadata: Record<string, any>;

    @CreateDateColumn()
    createdAt: Date;

    // Relationships
    @ManyToOne(() => RAGQuery, query => query.chunkRetrievalScores, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'queryId' })
    query!: RAGQuery;

    @ManyToOne(() => DocumentChunk, chunk => chunk, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'chunkId' })
    chunk!: DocumentChunk;
}