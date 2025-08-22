/**
 * Personal Knowledge Graph Entity
 * Converted from Django rag/models.py PersonalKnowledgeGraph model
 */

import {
    Entity,
    PrimaryGeneratedColumn,
    Column,
    UpdateDateColumn,
    Index
} from 'typeorm';

@Entity('personal_knowledge_graphs')
export class PersonalKnowledgeGraph {
    @PrimaryGeneratedColumn()
    id: number;

    @Column({ type: 'int', unique: true })
    @Index()
    userId: number;

    // Graph statistics
    @Column({ type: 'int', default: 0 })
    totalDocuments: number;

    @Column({ type: 'int', default: 0 })
    totalChunks: number;

    @Column({ type: 'int', default: 0 })
    totalConcepts: number;

    @Column({ type: 'int', default: 0 })
    totalRelationships: number;

    // Learning metrics
    @Column({ type: 'float', default: 0.0 })
    learningVelocity: number;

    @Column({ type: 'float', default: 0.0 })
    knowledgeDepth: number;

    @Column({ type: 'float', default: 0.0 })
    graphDensity: number;

    // Processing status
    @Column({ type: 'datetime', nullable: true })
    lastFullReindex?: Date;

    @Column({ type: 'varchar', length: 100, nullable: true })
    embeddingModelVersion?: string;

    @UpdateDateColumn()
    updatedAt: Date;
}