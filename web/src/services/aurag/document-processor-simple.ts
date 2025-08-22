/**
 * Simple Document Processor
 * Browser-compatible document processing for AURAG
 */

export interface ChunkingOptions {
    maxChunkSize: number;
    chunkOverlap: number;
    preserveCodeBlocks: boolean;
    preserveHeaders: boolean;
}

export class DocumentProcessor {
    private initialized = false;

    constructor() {}

    async initialize(): Promise<void> {
        this.initialized = true;
    }

    /**
     * Process document into chunks
     */
    async processDocument(content: string, title: string): Promise<Array<{content: string, startPosition?: number, endPosition?: number, metadata?: any}>> {
        const options: ChunkingOptions = {
            maxChunkSize: 1000,
            chunkOverlap: 200,
            preserveCodeBlocks: true,
            preserveHeaders: true
        };

        return this.createChunks(content, { title }, options);
    }

    /**
     * Extract concepts from document content
     */
    async extractConcepts(content: string): Promise<Array<{name: string, description?: string, category?: string, confidence?: number, relevance?: number, context?: string, metadata?: any}>> {
        // Simple concept extraction based on keywords and patterns
        const concepts = [];
        const words = content.toLowerCase().split(/\s+/);
        const sentences = content.split(/[.!?]+/);
        
        // Look for technical terms, proper nouns, and repeated important words
        const conceptCandidates = new Map();
        
        for (const word of words) {
            if (word.length > 3 && /^[a-zA-Z]+$/.test(word)) {
                conceptCandidates.set(word, (conceptCandidates.get(word) || 0) + 1);
            }
        }
        
        // Extract top concepts based on frequency and context
        const sortedConcepts = Array.from(conceptCandidates.entries())
            .filter(([word, count]) => count > 1)
            .sort((a, b) => b[1] - a[1])
            .slice(0, 10);
        
        for (const [word, frequency] of sortedConcepts) {
            const contextSentence = sentences.find(s => s.toLowerCase().includes(word));
            concepts.push({
                name: word.charAt(0).toUpperCase() + word.slice(1),
                description: contextSentence?.trim() || '',
                category: 'extracted',
                confidence: Math.min(0.9, frequency / words.length * 100),
                relevance: 0.7,
                context: contextSentence?.trim() || '',
                metadata: { frequency, extractedAt: new Date().toISOString() }
            });
        }
        
        return concepts;
    }

    /**
     * Create text chunks from content
     */
    private createChunks(content: string, context: any, options: ChunkingOptions): Array<{content: string, startPosition?: number, endPosition?: number, metadata?: any}> {
        const chunks = [];
        const maxSize = options.maxChunkSize;
        const overlap = options.chunkOverlap;
        
        // Simple sentence-based chunking
        const sentences = content.split(/[.!?]+/).filter(s => s.trim().length > 0);
        let currentChunk = '';
        let startPos = 0;
        
        for (let i = 0; i < sentences.length; i++) {
            const sentence = sentences[i].trim() + '.';
            
            if (currentChunk.length + sentence.length > maxSize && currentChunk.length > 0) {
                // Create chunk
                chunks.push({
                    content: currentChunk.trim(),
                    startPosition: startPos,
                    endPosition: startPos + currentChunk.length,
                    metadata: { chunkIndex: chunks.length, ...context }
                });
                
                // Start new chunk with overlap
                const words = currentChunk.split(' ');
                const overlapWords = words.slice(-Math.floor(overlap / 10)); // Rough overlap
                currentChunk = overlapWords.join(' ') + ' ' + sentence;
                startPos += currentChunk.length - overlap;
            } else {
                currentChunk += (currentChunk ? ' ' : '') + sentence;
            }
        }
        
        // Add final chunk
        if (currentChunk.trim()) {
            chunks.push({
                content: currentChunk.trim(),
                startPosition: startPos,
                endPosition: startPos + currentChunk.length,
                metadata: { chunkIndex: chunks.length, ...context }
            });
        }
        
        return chunks;
    }
}