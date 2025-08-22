/**
 * Learned Concept Entity
 * Converted from Django rag/models.py LearnedConcept model
 */

import {
    Entity,
    PrimaryGeneratedColumn,
    Column,
    CreateDateColumn,
    UpdateDateColumn,
    Index,
    OneToMany,
    ManyToMany,
    JoinTable,
    Unique
} from 'typeorm';
import { ConceptDocumentLink } from './ConceptDocumentLink';
import { ConceptRelationship } from './ConceptRelationship';
import { DocumentSource } from './DocumentSource';

@Entity('learned_concepts')
@Unique(['userId', 'name'])
@Index(['userId', 'importanceScore'])
@Index(['lastReinforced'])
export class LearnedConcept {
    @PrimaryGeneratedColumn()
    id: number;

    @Column({ type: 'int' })
    @Index()
    userId: number;

    @Column({ type: 'varchar', length: 300 })
    name: string;

    @Column({ type: 'text' })
    description: string;

    @Column({ type: 'float', default: 0.5 })
    confidenceScore: number;

    @Column({ type: 'float', default: 0.5 })
    importanceScore: number;

    @Column({ type: 'text', nullable: true })
    personalNotes?: string;

    @Column({ type: 'json', default: () => "'[]'" })
    tags: string[];

    @CreateDateColumn()
    firstEncountered: Date;

    @UpdateDateColumn()
    lastReinforced: Date;

    @Column({ type: 'int', default: 1 })
    encounterCount: number;

    // Relationships
    @OneToMany(() => ConceptDocumentLink, link => link.concept)
    documentLinks: ConceptDocumentLink[];

    @OneToMany(() => ConceptRelationship, relation => relation.fromConcept)
    outgoingRelations: ConceptRelationship[];

    @OneToMany(() => ConceptRelationship, relation => relation.toConcept)
    incomingRelations: ConceptRelationship[];

    @ManyToMany(() => DocumentSource, source => source.conceptLinks)
    @JoinTable({
        name: 'concept_document_links',
        joinColumn: { name: 'conceptId', referencedColumnName: 'id' },
        inverseJoinColumn: { name: 'documentId', referencedColumnName: 'id' }
    })
    sourceDocuments: DocumentSource[];
}