/**
 * Database Integration Test
 * Validates TypeORM models and AURAG service integration
 */

import 'reflect-metadata';
import { AURAG } from '../services/aurag';

export async function testAURAGIntegration(): Promise<void> {
    console.log('Testing AURAG Database Integration...\n');
    
    try {
        // Initialize AURAG service
        console.log('Initializing AURAG service...');
        const service = AURAG.createService(AURAG.configs.development);
        
        const initialized = await service.initialize();
        if (!initialized) {
            throw new Error('AURAG service initialization failed');
        }
        console.log('AURAG service initialized successfully\n');
        
        // Test document processing
        console.log('Testing document processing...');
        const testDocument = {
            title: 'AURAG Integration Test Document',
            content: `
                This is a test document for the AURAG system integration.
                
                Key concepts in this document:
                - AURAG (Advanced Unified RAG) represents intelligence in structure
                - TypeORM provides database abstraction for Node.js
                - Document chunking enables efficient retrieval
                - Concept extraction builds knowledge graphs
                
                This test validates that the Django models have been successfully
                converted to TypeORM entities and integrated with the AURAG pipeline.
            `,
            sourceType: 'note',
            author: 'AURAG System Test',
            metadata: {
                tags: ['test', 'integration', 'aurag'],
                notes: 'Automated integration test document'
            }
        };
        
        const result = await service.processDocument(
            1, // userId
            testDocument.title,
            testDocument.content,
            testDocument.sourceType,
            testDocument.author,
            testDocument.metadata
        );
        
        console.log('Document Processing Result:');
        console.log(`  Document ID: ${result.documentId}`);
        console.log(`  Chunks Created: ${result.chunksCreated}`);
        console.log(`  Concepts Extracted: ${result.conceptsExtracted}`);
        console.log(`  Processing Time: ${result.processingTimeMs}ms`);
        console.log(`  Success: ${result.success}`);
        
        if (!result.success) {
            console.error('  Error:', result.errorMessage);
        }
        console.log();
        
        // Test RAG query
        console.log('Testing RAG query...');
        const queryResult = await service.processRAGQuery(
            1, // userId
            'What is AURAG and how does it work?',
            'analytical', // philosophical lens
            8 // max context items
        );
        
        console.log('RAG Query Result:');
        console.log(`  Response: ${queryResult.responseText.substring(0, 100)}...`);
        console.log(`  Confidence: ${(queryResult.confidenceScore * 100).toFixed(1)}%`);
        console.log(`  Context Items: ${queryResult.contextItems.length}`);
        console.log(`  Processing Time: ${queryResult.processingTimeMs}ms`);
        console.log();
        
        // Test knowledge graph summary
        console.log('Testing knowledge graph summary...');
        const summary = await service.getKnowledgeGraphSummary(1);
        
        console.log('Knowledge Graph Summary:');
        console.log(`  Total Documents: ${summary.totalDocuments || 0}`);
        console.log(`  Total Concepts: ${summary.totalConcepts || 0}`);
        console.log(`  Total Relationships: ${summary.totalRelationships || 0}`);
        console.log(`  Knowledge Depth: ${((summary.knowledgeDepth || 0) * 100).toFixed(1)}%`);
        console.log(`  Graph Density: ${(summary.graphDensity || 0).toFixed(3)}`);
        console.log();
        
        console.log('AURAG Database Integration Test Completed Successfully');
        
    } catch (error) {
        console.error('AURAG Integration Test Failed:', error);
        throw error;
    }
}

// Run test if this file is executed directly
if (require.main === module) {
    testAURAGIntegration()
        .then(() => process.exit(0))
        .catch(error => {
            console.error(error);
            process.exit(1);
        });
}