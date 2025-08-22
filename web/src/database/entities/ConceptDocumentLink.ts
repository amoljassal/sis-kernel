/**
 * Concept Document Link Entity
 * Converted from Django rag/models.py ConceptDocumentLink model
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
import { DocumentChunk } from './DocumentChunk';

@Entity('concept_document_links')
@Unique(['conceptId', 'documentId'])
export class ConceptDocumentLink {
    @PrimaryGeneratedColumn()
    id: number;

    @Column()
    conceptId: number;

    @Column()
    documentId: number;

    @Column({ type: 'float', default: 0.5 })
    relevanceScore: number;

    @CreateDateColumn()
    createdAt: Date;

    // Relationships
    @ManyToOne(() => LearnedConcept, concept => concept.documentLinks, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'conceptId' })
    concept: LearnedConcept;

    @ManyToOne(() => DocumentSource, document => document.conceptLinks, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'documentId' })
    document: DocumentSource;

    @ManyToMany(() => DocumentChunk)
    @JoinTable({
        name: 'concept_document_link_chunks',
        joinColumn: { name: 'linkId', referencedColumnName: 'id' },
        inverseJoinColumn: { name: 'chunkId', referencedColumnName: 'id' }
    })
    chunks: DocumentChunk[];
}