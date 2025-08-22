/**
 * RAG Query Entity
 * Converted from Django rag/models.py RAGQuery model
 */

import {
    Entity,
    PrimaryGeneratedColumn,
    Column,
    CreateDateColumn,
    Index,
    OneToMany
} from 'typeorm';
import { ChunkRetrievalScore } from './ChunkRetrievalScore';
import { ConceptRetrievalScore } from './ConceptRetrievalScore';

@Entity('rag_queries')
@Index(['userId', 'createdAt'])
@Index(['philosophicalLens'])
export class RAGQuery {
    @PrimaryGeneratedColumn()
    id: number;

    @Column({ type: 'int' })
    @Index()
    userId: number;

    @Column({ type: 'text' })
    queryText: string;

    @Column({ type: 'varchar', length: 50, nullable: true })
    philosophicalLens?: string;

    @Column({ type: 'text' })
    responseText: string;

    @Column({ type: 'float', default: 0.5 })
    confidenceScore: number;

    @Column({ type: 'int' })
    processingTimeMs: number;

    @Column({ type: 'int', nullable: true })
    userRating?: number;

    @Column({ type: 'text', nullable: true })
    userFeedback?: string;

    @Column({ type: 'json', default: () => "'{}'" })
    metadata: Record<string, any>;

    @CreateDateColumn()
    createdAt: Date;

    // Relationships
    @OneToMany(() => ChunkRetrievalScore, score => score.query)
    chunkRetrievalScores: ChunkRetrievalScore[];

    @OneToMany(() => ConceptRetrievalScore, score => score.query)
    conceptRetrievalScores: ConceptRetrievalScore[];
}