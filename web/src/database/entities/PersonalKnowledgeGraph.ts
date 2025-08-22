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

    @Column({ unique: true })
    @Index()
    userId: number;

    // Graph statistics
    @Column({ default: 0 })
    totalDocuments: number;

    @Column({ default: 0 })
    totalChunks: number;

    @Column({ default: 0 })
    totalConcepts: number;

    @Column({ default: 0 })
    totalRelationships: number;

    // Learning metrics
    @Column({ type: 'float', default: 0.0 })
    learningVelocity: number;

    @Column({ type: 'float', default: 0.0 })
    knowledgeDepth: number;

    @Column({ type: 'float', default: 0.0 })
    graphDensity: number;

    // Processing status
    @Column({ nullable: true })
    lastFullReindex?: Date;

    @Column({ length: 100, nullable: true })
    embeddingModelVersion?: string;

    @UpdateDateColumn()
    updatedAt: Date;
}