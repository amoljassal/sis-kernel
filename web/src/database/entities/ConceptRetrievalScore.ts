/**
 * Concept Retrieval Score Entity
 * Converted from Django rag/models.py ConceptRetrievalScore model
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
import { LearnedConcept } from './LearnedConcept';

@Entity('concept_retrieval_scores')
export class ConceptRetrievalScore {
    @PrimaryGeneratedColumn()
    id: number;

    @Column({ type: 'int' })
    queryId: number;

    @Column({ type: 'int' })
    conceptId: number;

    @Column({ type: 'float' })
    relevanceScore: number;

    @Column({ type: 'int' })
    rankPosition: number;

    @Column({ type: 'boolean', default: false })
    usedInResponse: boolean;

    @Column({ type: 'json', default: () => "'{}'" })
    metadata: Record<string, any>;

    @CreateDateColumn()
    createdAt: Date;

    // Relationships
    @ManyToOne(() => RAGQuery, query => query.conceptRetrievalScores, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'queryId' })
    query: RAGQuery;

    @ManyToOne(() => LearnedConcept, concept => concept, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'conceptId' })
    concept: LearnedConcept;
}