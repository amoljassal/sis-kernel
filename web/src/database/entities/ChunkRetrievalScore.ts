/**
 * Chunk Retrieval Score Entity
 * Converted from Django rag/models.py ChunkRetrievalScore model
 */

import {
    Entity,
    PrimaryGeneratedColumn,
    Column,
    ManyToOne,
    JoinColumn
} from 'typeorm';
import { RAGQuery } from './RAGQuery';
import { DocumentChunk } from './DocumentChunk';

@Entity('chunk_retrieval_scores')
export class ChunkRetrievalScore {
    @PrimaryGeneratedColumn()
    id!: number;

    @Column()
    queryId!: number;

    @Column()
    chunkId!: number;

    @Column({ type: 'float' })
    relevanceScore!: number;

    @Column()
    rankPosition!: number;

    @Column({ default: false })
    usedInResponse!: boolean;

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