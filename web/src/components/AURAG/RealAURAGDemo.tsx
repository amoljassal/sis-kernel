/**
 * SIS AURAG Demo Component
 * Browser-compatible implementation with backend API integration
 */

import React, { useState, useEffect } from 'react';
import { AURAG, RAGResponse, DocumentProcessingResult } from '../../services/aurag';

const AURAGDemo: React.FC = () => {
    const [ragService, setRagService] = useState<any>(null);
    const [initialized, setInitialized] = useState(false);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    
    // Demo state
    const [query, setQuery] = useState('');
    const [response, setResponse] = useState<RAGResponse | null>(null);
    const [documentTitle, setDocumentTitle] = useState('');
    const [documentContent, setDocumentContent] = useState('');
    const [processingResult, setProcessingResult] = useState<DocumentProcessingResult | null>(null);
    const [knowledgeStats, setKnowledgeStats] = useState<any>(null);

    useEffect(() => {
        initializeRAG();
    }, []);

    const initializeRAG = async () => {
        try {
            setLoading(true);
            setError(null);
            console.log('Starting Real AURAG initialization...');
            
            // Check if AURAG is available
            if (!AURAG || !AURAG.createService) {
                throw new Error('AURAG service factory not available');
            }
            
            console.log('Creating Real AURAG service...');
            const service = AURAG.createService(AURAG.configs.development);
            
            if (!service) {
                throw new Error('Failed to create Real AURAG service');
            }
            
            console.log('Initializing database connection...');
            const initSuccess = await service.initialize();
            
            if (!initSuccess) {
                throw new Error('Failed to initialize AURAG database connection');
            }
            
            console.log('Real AURAG service initialized with database');
            setRagService(service);
            setInitialized(true);
            
            // Load initial knowledge graph stats
            await loadKnowledgeStats(service);
            
        } catch (error) {
            console.error('Error initializing Real AURAG:', error);
            setError(error instanceof Error ? error.message : 'Unknown initialization error');
        } finally {
            setLoading(false);
        }
    };

    const loadKnowledgeStats = async (service: any) => {
        try {
            const stats = await service.getKnowledgeGraphStats(1); // userId = 1 for demo
            setKnowledgeStats(stats);
        } catch (error) {
            console.error('Failed to load knowledge stats:', error);
        }
    };

    const handleDocumentSubmit = async () => {
        if (!ragService || !documentTitle || !documentContent) return;

        try {
            setLoading(true);
            setError(null);
            
            const result = await ragService.processDocument(
                1, // userId - in real app this would come from auth
                documentTitle,
                documentContent
            );

            setProcessingResult(result);
            
            if (result.success) {
                console.log('Document processed successfully:', result);
                // Refresh knowledge stats
                await loadKnowledgeStats(ragService);
            } else {
                setError(result.errorMessage || 'Document processing failed');
            }
            
        } catch (error) {
            console.error('Error processing document:', error);
            setError(error instanceof Error ? error.message : 'Unknown processing error');
        } finally {
            setLoading(false);
        }
    };

    const handleQuerySubmit = async () => {
        if (!ragService || !query) return;

        try {
            setLoading(true);
            setError(null);
            
            const result = await ragService.processRAGQuery(
                1, // userId
                query,
                'analytical' // philosophical lens
            );

            setResponse(result);
            console.log('RAG query result:', result);
            
        } catch (error) {
            console.error('Error processing RAG query:', error);
            setError(error instanceof Error ? error.message : 'Unknown query error');
        } finally {
            setLoading(false);
        }
    };

    if (!initialized) {
        return (
            <div className="max-w-4xl mx-auto p-6">
                <div className="glass rounded-lg p-6">
                    <h1 className="text-2xl font-bold text-white mb-4">
                        SIS Real AURAG System
                    </h1>
                    
                    {loading ? (
                        <div className="flex items-center justify-center py-8">
                            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-sis-blue-500"></div>
                            <span className="ml-3 text-sis-gray-300">Initializing Real AURAG system...</span>
                        </div>
                    ) : error ? (
                        <div className="text-center py-8">
                            <p className="text-red-400 mb-4">{error}</p>
                            <button
                                onClick={initializeRAG}
                                className="btn-primary"
                            >
                                Retry Initialization
                            </button>
                        </div>
                    ) : (
                        <div className="text-center py-8">
                            <p className="text-red-400 mb-4">Failed to initialize Real AURAG system</p>
                            <button
                                onClick={initializeRAG}
                                className="btn-primary"
                            >
                                Retry Initialization
                            </button>
                        </div>
                    )}
                </div>
            </div>
        );
    }

    return (
        <div className="max-w-6xl mx-auto p-6 space-y-6">
            {/* Header */}
            <div className="glass rounded-lg p-6">
                <h1 className="text-3xl font-bold text-white mb-2">
                    SIS Real AURAG System
                </h1>
                <p className="text-sis-gray-300">
                    Full TypeORM database implementation. Intelligence in Structure, Not Parameters.
                </p>
                
                <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div className="bg-green-500/20 p-3 rounded border border-green-500/30">
                        <div className="font-medium text-green-400">Status</div>
                        <div className="text-green-300">Database Connected</div>
                    </div>
                    <div className="bg-sis-blue-500/20 p-3 rounded border border-sis-blue-500/30">
                        <div className="font-medium text-sis-blue-400">Config</div>
                        <div className="text-sis-blue-300">Development</div>
                    </div>
                    <div className="bg-purple-500/20 p-3 rounded border border-purple-500/30">
                        <div className="font-medium text-purple-400">Lens</div>
                        <div className="text-purple-300">Analytical</div>
                    </div>
                    <div className="bg-orange-500/20 p-3 rounded border border-orange-500/30">
                        <div className="font-medium text-orange-400">Provider</div>
                        <div className="text-orange-300">Browser Demo</div>
                    </div>
                </div>
            </div>

            {/* Knowledge Graph Stats */}
            {knowledgeStats && (
                <div className="glass rounded-lg p-6">
                    <h2 className="text-xl font-semibold text-white mb-4">
                        Knowledge Graph Statistics
                    </h2>
                    
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                        <div className="bg-sis-gray-800/50 p-3 rounded">
                            <div className="font-medium text-sis-gray-300">Documents</div>
                            <div className="text-2xl font-bold text-white">{knowledgeStats.totalDocuments}</div>
                        </div>
                        <div className="bg-sis-gray-800/50 p-3 rounded">
                            <div className="font-medium text-sis-gray-300">Chunks</div>
                            <div className="text-2xl font-bold text-white">{knowledgeStats.totalChunks}</div>
                        </div>
                        <div className="bg-sis-gray-800/50 p-3 rounded">
                            <div className="font-medium text-sis-gray-300">Concepts</div>
                            <div className="text-2xl font-bold text-white">{knowledgeStats.totalConcepts}</div>
                        </div>
                        <div className="bg-sis-gray-800/50 p-3 rounded">
                            <div className="font-medium text-sis-gray-300">Knowledge Density</div>
                            <div className="text-2xl font-bold text-white">
                                {knowledgeStats.totalDocuments > 0 
                                    ? (knowledgeStats.totalConcepts / knowledgeStats.totalDocuments).toFixed(1)
                                    : '0'
                                }
                            </div>
                        </div>
                    </div>

                    {knowledgeStats.topConcepts.length > 0 && (
                        <div className="mt-4">
                            <h3 className="font-medium text-sis-gray-300 mb-2">Top Concepts:</h3>
                            <div className="flex flex-wrap gap-2">
                                {knowledgeStats.topConcepts.slice(0, 5).map((concept: any, index: number) => (
                                    <span 
                                        key={index}
                                        className="px-3 py-1 bg-sis-blue-600/30 text-sis-blue-300 rounded-full text-xs"
                                    >
                                        {concept.name} ({(concept.confidenceScore * 100).toFixed(0)}%)
                                    </span>
                                ))}
                            </div>
                        </div>
                    )}
                </div>
            )}

            {/* Document Processing */}
            <div className="glass rounded-lg p-6">
                <h2 className="text-xl font-semibold text-white mb-4">
                    Document Processing
                </h2>
                
                <div className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-sis-gray-300 mb-1">
                            Document Title
                        </label>
                        <input
                            type="text"
                            value={documentTitle}
                            onChange={(e) => setDocumentTitle(e.target.value)}
                            placeholder="Enter document title..."
                            className="w-full px-3 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-md text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                        />
                    </div>
                    
                    <div>
                        <label className="block text-sm font-medium text-sis-gray-300 mb-1">
                            Document Content
                        </label>
                        <textarea
                            value={documentContent}
                            onChange={(e) => setDocumentContent(e.target.value)}
                            placeholder="Enter document content..."
                            rows={6}
                            className="w-full px-3 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-md text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                        />
                    </div>
                    
                    <button
                        onClick={handleDocumentSubmit}
                        disabled={loading || !documentTitle || !documentContent}
                        className="btn-primary disabled:bg-sis-gray-600 disabled:cursor-not-allowed"
                    >
                        {loading ? 'Processing...' : 'Process Document'}
                    </button>
                </div>

                {processingResult && (
                    <div className="mt-4 p-4 bg-sis-gray-800/50 rounded-md">
                        <h3 className="font-medium text-white mb-2">Processing Result:</h3>
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                            <div>
                                <span className="text-sis-gray-400">Status:</span>
                                <span className={`ml-1 font-medium ${processingResult.success ? 'text-green-400' : 'text-red-400'}`}>
                                    {processingResult.success ? 'Success' : 'Failed'}
                                </span>
                            </div>
                            <div>
                                <span className="text-sis-gray-400">Chunks:</span>
                                <span className="ml-1 font-medium text-sis-blue-400">{processingResult.chunksCreated}</span>
                            </div>
                            <div>
                                <span className="text-sis-gray-400">Concepts:</span>
                                <span className="ml-1 font-medium text-purple-400">{processingResult.conceptsExtracted}</span>
                            </div>
                            <div>
                                <span className="text-sis-gray-400">Time:</span>
                                <span className="ml-1 font-medium text-orange-400">{processingResult.processingTimeMs}ms</span>
                            </div>
                        </div>
                        {processingResult.errorMessage && (
                            <div className="mt-2 text-red-400 text-sm">
                                Error: {processingResult.errorMessage}
                            </div>
                        )}
                    </div>
                )}
            </div>

            {/* RAG Query */}
            <div className="glass rounded-lg p-6">
                <h2 className="text-xl font-semibold text-white mb-4">
                    RAG Query
                </h2>
                
                <div className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-sis-gray-300 mb-1">
                            Query
                        </label>
                        <input
                            type="text"
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                            placeholder="Ask a question..."
                            className="w-full px-3 py-2 bg-sis-gray-800 border border-sis-gray-600 rounded-md text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
                        />
                    </div>
                    
                    <button
                        onClick={handleQuerySubmit}
                        disabled={loading || !query}
                        className="btn-primary disabled:bg-sis-gray-600 disabled:cursor-not-allowed"
                    >
                        {loading ? 'Processing...' : 'Submit Query'}
                    </button>
                </div>

                {response && (
                    <div className="mt-4 space-y-4">
                        <div className="p-4 bg-sis-blue-900/30 rounded-md border border-sis-blue-500/30">
                            <h3 className="font-medium text-sis-blue-300 mb-2">Response:</h3>
                            <p className="text-sis-blue-200 whitespace-pre-wrap">{response.responseText}</p>
                        </div>
                        
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                            <div className="bg-sis-gray-800/50 p-3 rounded">
                                <span className="text-sis-gray-400">Confidence:</span>
                                <span className="ml-1 font-medium text-green-400">
                                    {(response.confidenceScore * 100).toFixed(1)}%
                                </span>
                            </div>
                            <div className="bg-sis-gray-800/50 p-3 rounded">
                                <span className="text-sis-gray-400">Context Items:</span>
                                <span className="ml-1 font-medium text-sis-blue-400">{response.contextItems.length}</span>
                            </div>
                            <div className="bg-sis-gray-800/50 p-3 rounded">
                                <span className="text-sis-gray-400">Sources:</span>
                                <span className="ml-1 font-medium text-purple-400">{response.sourcesUsed.length}</span>
                            </div>
                            <div className="bg-sis-gray-800/50 p-3 rounded">
                                <span className="text-sis-gray-400">Time:</span>
                                <span className="ml-1 font-medium text-orange-400">{response.processingTimeMs}ms</span>
                            </div>
                        </div>
                        
                        {response.contextItems.length > 0 && (
                            <div className="p-4 bg-sis-gray-800/50 rounded-md">
                                <h4 className="font-medium text-white mb-2">Context Items:</h4>
                                <div className="space-y-2 max-h-64 overflow-y-auto">
                                    {response.contextItems.map((item, index) => (
                                        <div key={index} className="text-sm bg-sis-gray-900/50 p-3 rounded">
                                            <div className="flex items-center justify-between mb-1">
                                                <span className="font-medium text-sis-gray-300">
                                                    [{index + 1}] {item.kind}
                                                </span>
                                                <span className="text-sis-blue-400 text-xs">
                                                    score: {item.score.toFixed(3)}
                                                </span>
                                            </div>
                                            <p className="text-sis-gray-400 text-xs">
                                                {item.text.length > 150 ? item.text.substring(0, 150) + '...' : item.text}
                                            </p>
                                            <span className="text-sis-gray-500 text-xs">
                                                Source: {item.meta.source}
                                            </span>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>

            {/* Error Display */}
            {error && (
                <div className="bg-red-900/30 border border-red-500/30 p-4 rounded-md">
                    <p className="text-red-400">Error: {error}</p>
                </div>
            )}
        </div>
    );
};

export default AURAGDemo;