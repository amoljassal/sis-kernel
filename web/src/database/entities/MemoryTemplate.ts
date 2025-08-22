/**
 * Memory Template Entity
 * Converted from Django rag/models.py MemoryTemplate model
 */

import {
    Entity,
    PrimaryGeneratedColumn,
    Column,
    CreateDateColumn,
    OneToMany,
    Unique
} from 'typeorm';
import { StructuredMemoryEntry } from './StructuredMemoryEntry';

export enum TemplateType {
    JOURNAL = 'journal',
    BOOK_REVIEW = 'book_review',
    COURSE_LEARNING = 'course_learning',
    QUOTE = 'quote',
    MEETING = 'meeting',
    PROJECT = 'project',
    CUSTOM = 'custom'
}

@Entity('memory_templates')
@Unique(['userId', 'name'])
export class MemoryTemplate {
    @PrimaryGeneratedColumn()
    id: number;

    @Column({ type: 'int' })
    userId: number;

    @Column({ type: 'varchar', length: 200 })
    name: string;

    @Column({
        type: 'enum',
        enum: TemplateType
    })
    templateType: TemplateType;

    @Column({ type: 'json', default: () => "'{}'" })
    fieldsSchema: Record<string, any>;

    @Column({ type: 'json', default: () => "'{}'" })
    uiLayout: Record<string, any>;

    @Column({ type: 'json', default: () => "'{}'" })
    extractionRules: Record<string, any>;

    @Column({ type: 'text', nullable: true })
    defaultContent?: string;

    @Column({ type: 'boolean', default: true })
    isActive: boolean;

    @CreateDateColumn()
    createdAt: Date;

    // Relationships
    @OneToMany(() => StructuredMemoryEntry, entry => entry.template)
    entries: StructuredMemoryEntry[];
}