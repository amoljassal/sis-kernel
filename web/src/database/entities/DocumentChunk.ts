/**
 * Document Chunk Entity
 * Converted from Django rag/models.py DocumentChunk model
 */

import {
    Entity,
    PrimaryGeneratedColumn,
    Column,
    CreateDateColumn,
    Index,
    ManyToOne,
    JoinColumn,
    Unique
} from 'typeorm';
import { DocumentSource } from './DocumentSource';

export enum ContentType {
    TEXT = 'text',
    CODE = 'code',
    TABLE = 'table',
    LIST = 'list',
    HEADING = 'heading'
}

export enum ComplexityLevel {
    LOW = 'low',
    MEDIUM = 'medium',
    HIGH = 'high'
}

@Entity('document_chunks')
@Unique(['document', 'chunkIndex'])
@Index(['documentId', 'chunkIndex'])
@Index(['contentHash'])
@Index(['contentType', 'complexity'])
export class DocumentChunk {
    @PrimaryGeneratedColumn()
    id: number;

    @Column()
    @Index()
    documentId: number;

    @Column()
    chunkIndex: number;

    @Column({ type: 'text' })
    content: string;

    @Column({ length: 64, unique: true })
    contentHash: string;

    @Column()
    tokenCount: number;

    @Column()
    startChar: number;

    @Column()
    endChar: number;

    @Column({
        type: 'enum',
        enum: ContentType,
        default: ContentType.TEXT
    })
    contentType: ContentType;

    @Column({
        type: 'enum',
        enum: ComplexityLevel,
        default: ComplexityLevel.MEDIUM
    })
    complexity: ComplexityLevel;

    @Column({ length: 300, nullable: true })
    sectionTitle?: string;

    @Column({ type: 'json', nullable: true })
    embedding?: number[];

    @Column({ length: 100, nullable: true })
    embeddingModel?: string;

    @CreateDateColumn()
    createdAt: Date;

    // Relationships
    @ManyToOne(() => DocumentSource, source => source.chunks, {
        onDelete: 'CASCADE'
    })
    @JoinColumn({ name: 'documentId' })
    document: DocumentSource;
}