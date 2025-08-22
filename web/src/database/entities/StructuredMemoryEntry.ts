/**
 * Structured Memory Entry Entity
 * Converted from Django rag/models.py StructuredMemoryEntry model
 */

import {
    Entity,
    PrimaryGeneratedColumn,
    Column,
    CreateDateColumn,
    UpdateDateColumn,
    Index,
    ManyToOne,
    JoinColumn
} from 'typeorm';
import { MemoryTemplate } from './MemoryTemplate';
import { DocumentSource } from './DocumentSource';

@Entity('structured_memory_entries')
@Index(['userId', 'templateId'])
@Index(['createdAt'])
export class StructuredMemoryEntry {
    @PrimaryGeneratedColumn()
    id: number;

    @Column({ type: 'int' })
    @Index()
    userId: number;

    @Column({ type: 'int' })
    templateId: number;

    @Column({ type: 'varchar', length: 500 })
    title: string;

    @Column({ type: 'json', default: () => "'{}'" })
    structuredData: Record<string, any>;

    @Column({ type: 'text' })
    fullText: string;

    @Column({ type: 'json', default: () => "'[]'" })
    extractedConcepts: string[];

    @Column({ type: 'int', nullable: true })
    sourceDocumentId?: number;

    @CreateDateColumn()
    createdAt: Date;

    @UpdateDateColumn()
    updatedAt: Date;

    // Relationships
    @ManyToOne(() => MemoryTemplate, template => template.entries, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'templateId' })
    template: MemoryTemplate;

    @ManyToOne(() => DocumentSource, source => source.structuredEntries, {
        onDelete: 'SET NULL'
    })
    @JoinColumn({ name: 'sourceDocumentId' })
    sourceDocument?: DocumentSource;
}