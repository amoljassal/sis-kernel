/**
 * Concept Relationship Entity
 * Converted from Django rag/models.py ConceptRelationship model
 */

import {
    Entity,
    PrimaryGeneratedColumn,
    Column,
    CreateDateColumn,
    ManyToOne,
    ManyToMany,
    JoinColumn,
    JoinTable,
    Unique
} from 'typeorm';
import { LearnedConcept } from './LearnedConcept';
import { DocumentSource } from './DocumentSource';

export enum RelationshipType {
    IS_A = 'is_a',
    PART_OF = 'part_of',
    RELATES_TO = 'relates_to',
    OPPOSES = 'opposes',
    ENABLES = 'enables',
    CAUSED_BY = 'caused_by',
    SIMILAR_TO = 'similar_to'
}

@Entity('concept_relationships')
@Unique(['fromConceptId', 'toConceptId', 'relationshipType'])
export class ConceptRelationship {
    @PrimaryGeneratedColumn()
    id: number;

    @Column({ type: 'int' })
    fromConceptId: number;

    @Column({ type: 'int' })
    toConceptId: number;

    @Column({
        type: 'enum',
        enum: RelationshipType
    })
    relationshipType: RelationshipType;

    @Column({ type: 'float', default: 0.5 })
    strength: number;

    @Column({ type: 'text', nullable: true })
    notes?: string;

    @CreateDateColumn()
    createdAt: Date;

    // Relationships
    @ManyToOne(() => LearnedConcept, concept => concept.outgoingRelations, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'fromConceptId' })
    fromConcept: LearnedConcept;

    @ManyToOne(() => LearnedConcept, concept => concept.incomingRelations, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'toConceptId' })
    toConcept: LearnedConcept;

    @ManyToMany(() => DocumentSource)
    @JoinTable({
        name: 'concept_relationship_evidence',
        joinColumn: { name: 'relationshipId', referencedColumnName: 'id' },
        inverseJoinColumn: { name: 'documentId', referencedColumnName: 'id' }
    })
    evidenceDocuments: DocumentSource[];
}