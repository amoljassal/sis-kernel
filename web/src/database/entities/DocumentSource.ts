/**
 * Document Source Entity
 * Converted from Django rag/models.py DocumentSource model
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
    BeforeInsert,
    BeforeUpdate
} from 'typeorm';
import { DocumentChunk } from './DocumentChunk';
import { StructuredMemoryEntry } from './StructuredMemoryEntry';
import { ConceptDocumentLink } from './ConceptDocumentLink';

export enum SourceType {
    BOOK = 'book',
    ARTICLE = 'article',
    JOURNAL = 'journal',
    NOTE = 'note',
    COURSE = 'course',
    CONVERSATION = 'conversation',
    WEB = 'web',
    PDF = 'pdf',
    OTHER = 'other'
}

@Entity('document_sources')
@Index(['userId', 'sourceType'])
@Index(['contentHash'])
@Index(['processed'])
export class DocumentSource {
    @PrimaryGeneratedColumn()
    id: number;

    @Column()
    @Index()
    userId: number;

    @Column({ length: 500 })
    title: string;

    @Column({
        type: 'enum',
        enum: SourceType,
        default: SourceType.OTHER
    })
    sourceType: SourceType;

    @Column({ length: 300, nullable: true })
    author?: string;

    @Column({ type: 'text', nullable: true })
    originalPath?: string;

    @Column({ nullable: true })
    url?: string;

    @Column({ type: 'text' })
    content: string;

    @Column({ length: 64, unique: true })
    contentHash: string;

    @Column({ type: 'json', default: () => "'[]'" })
    tags: string[];

    @Column({ type: 'int', nullable: true })
    rating?: number;

    @Column({ type: 'text', nullable: true })
    notes?: string;

    @Column({ default: false })
    processed: boolean;

    @Column({ default: 0 })
    chunkCount: number;

    @Column({ default: 0 })
    conceptCount: number;

    @Column({ default: 0 })
    wordCount: number;

    @CreateDateColumn()
    createdAt: Date;

    @UpdateDateColumn()
    updatedAt: Date;

    // Relationships
    @OneToMany(() => DocumentChunk, chunk => chunk.document)
    chunks: DocumentChunk[];

    @OneToMany(() => StructuredMemoryEntry, entry => entry.sourceDocument)
    structuredEntries: StructuredMemoryEntry[];

    @OneToMany(() => ConceptDocumentLink, link => link.document)
    conceptLinks: ConceptDocumentLink[];

    @BeforeInsert()
    @BeforeUpdate()
    calculateWordCount() {
        if (this.content) {
            this.wordCount = this.content.split(/\s+/).filter(word => word.length > 0).length;
        }
    }
}