/**
 * SIS AURAG Demo Component
 * Demonstration of the extracted AURAG system
 */

import React, { useState, useEffect } from 'react';
import { AURAG } from '../../services/aurag';
import type { RAGResponse, DocumentProcessingResult } from '../../services/aurag';

const AURAGDemo: React.FC = () => {
    const [ragService, setRagService] = useState<any>(null);
    const [initialized, setInitialized] = useState(false);
    const [loading, setLoading] = useState(false);
    
    // Demo state
    const [query, setQuery] = useState('');
    const [response, setResponse] = useState<RAGResponse | null>(null);
    const [documentTitle, setDocumentTitle] = useState('');
    const [documentContent, setDocumentContent] = useState('');
    const [processingResult, setProcessingResult] = useState<DocumentProcessingResult | null>(null);

    useEffect(() => {
        initializeRAG();
    }, []);

    const initializeRAG = async () => {
        try {
            setLoading(true);
            console.log('Starting AURAG initialization...');
            
            // Check if AURAG is available
            if (!AURAG || !AURAG.createService) {
                throw new Error('AURAG service factory not available');
            }
            
            console.log('Creating AURAG service...');
            const service = AURAG.createService(AURAG.configs.development);
            
            if (!service) {
                throw new Error('Failed to create AURAG service');
            }
            
            // For now, skip database initialization in browser
            // TODO: Implement browser-compatible storage
            console.log('AURAG service created (using mock mode for browser)');
            setRagService(service);
            setInitialized(true);
            
        } catch (error) {
            console.error('Error initializing AURAG:', error);
        } finally {
            setLoading(false);
        }
    };

    const handleDocumentSubmit = async () => {
        if (!ragService || !documentTitle || !documentContent) return;

        try {
            setLoading(true);
            
            const result = await ragService.processDocument(
                1, // userId - in real app this would come from auth
                documentTitle,
                documentContent
            );

            setProcessingResult(result);
            
            if (result.success) {
                console.log('Document processed successfully:', result);
            } else {
                console.error('Document processing failed:', result.errorMessage);
            }
            
        } catch (error) {
            console.error('Error processing document:', error);
        } finally {
            setLoading(false);
        }
    };

    const handleQuerySubmit = async () => {
        if (!ragService || !query) return;

        try {
            setLoading(true);
            
            const result = await ragService.processRAGQuery(
                1, // userId
                query,
                'analytical' // philosophical lens
            );

            setResponse(result);
            console.log('RAG query result:', result);
            
        } catch (error) {
            console.error('Error processing RAG query:', error);
        } finally {
            setLoading(false);
        }
    };

    if (!initialized) {
        return (
            <div className="max-w-4xl mx-auto p-6">
                <div className="bg-white rounded-lg shadow-lg p-6">
                    <h1 className="text-2xl font-bold text-gray-900 mb-4">
                        SIS AURAG System
                    </h1>
                    
                    {loading ? (
                        <div className="flex items-center justify-center py-8">
                            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                            <span className="ml-3 text-gray-600">Initializing AURAG system...</span>
                        </div>
                    ) : (
                        <div className="text-center py-8">
                            <p className="text-red-600 mb-4">Failed to initialize AURAG system</p>
                            <button
                                onClick={initializeRAG}
                                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
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
        <div className="max-w-4xl mx-auto p-6 space-y-6">
            {/* Header */}
            <div className="bg-white rounded-lg shadow-lg p-6">
                <h1 className="text-3xl font-bold text-gray-900 mb-2">
                    SIS AURAG System Demo
                </h1>
                <p className="text-gray-600">
                    Extracted from sis-core Django implementation. Intelligence in Structure, Not Parameters.
                </p>
                
                <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div className="bg-green-50 p-3 rounded">
                        <div className="font-medium text-green-800">Status</div>
                        <div className="text-green-600">Initialized</div>
                    </div>
                    <div className="bg-blue-50 p-3 rounded">
                        <div className="font-medium text-blue-800">Config</div>
                        <div className="text-blue-600">Development</div>
                    </div>
                    <div className="bg-purple-50 p-3 rounded">
                        <div className="font-medium text-purple-800">Lens</div>
                        <div className="text-purple-600">Analytical</div>
                    </div>
                    <div className="bg-orange-50 p-3 rounded">
                        <div className="font-medium text-orange-800">Provider</div>
                        <div className="text-orange-600">Ollama</div>
                    </div>
                </div>
            </div>

            {/* Document Processing */}
            <div className="bg-white rounded-lg shadow-lg p-6">
                <h2 className="text-xl font-semibold text-gray-900 mb-4">
                    Document Processing
                </h2>
                
                <div className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                            Document Title
                        </label>
                        <input
                            type="text"
                            value={documentTitle}
                            onChange={(e) => setDocumentTitle(e.target.value)}
                            placeholder="Enter document title..."
                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>
                    
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                            Document Content
                        </label>
                        <textarea
                            value={documentContent}
                            onChange={(e) => setDocumentContent(e.target.value)}
                            placeholder="Enter document content..."
                            rows={6}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>
                    
                    <button
                        onClick={handleDocumentSubmit}
                        disabled={loading || !documentTitle || !documentContent}
                        className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
                    >
                        {loading ? 'Processing...' : 'Process Document'}
                    </button>
                </div>

                {processingResult && (
                    <div className="mt-4 p-4 bg-gray-50 rounded-md">
                        <h3 className="font-medium text-gray-900 mb-2">Processing Result:</h3>
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                            <div>
                                <span className="text-gray-600">Status:</span>
                                <span className={`ml-1 font-medium ${processingResult.success ? 'text-green-600' : 'text-red-600'}`}>
                                    {processingResult.success ? 'Success' : 'Failed'}
                                </span>
                            </div>
                            <div>
                                <span className="text-gray-600">Chunks:</span>
                                <span className="ml-1 font-medium text-blue-600">{processingResult.chunksCreated}</span>
                            </div>
                            <div>
                                <span className="text-gray-600">Concepts:</span>
                                <span className="ml-1 font-medium text-purple-600">{processingResult.conceptsExtracted}</span>
                            </div>
                            <div>
                                <span className="text-gray-600">Time:</span>
                                <span className="ml-1 font-medium text-orange-600">{processingResult.processingTimeMs}ms</span>
                            </div>
                        </div>
                        {processingResult.errorMessage && (
                            <div className="mt-2 text-red-600 text-sm">
                                Error: {processingResult.errorMessage}
                            </div>
                        )}
                    </div>
                )}
            </div>

            {/* RAG Query */}
            <div className="bg-white rounded-lg shadow-lg p-6">
                <h2 className="text-xl font-semibold text-gray-900 mb-4">
                    RAG Query
                </h2>
                
                <div className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                            Query
                        </label>
                        <input
                            type="text"
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                            placeholder="Ask a question..."
                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>
                    
                    <button
                        onClick={handleQuerySubmit}
                        disabled={loading || !query}
                        className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
                    >
                        {loading ? 'Processing...' : 'Submit Query'}
                    </button>
                </div>

                {response && (
                    <div className="mt-4 space-y-4">
                        <div className="p-4 bg-blue-50 rounded-md">
                            <h3 className="font-medium text-blue-900 mb-2">Response:</h3>
                            <p className="text-blue-800">{response.responseText}</p>
                        </div>
                        
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                            <div className="bg-gray-50 p-3 rounded">
                                <span className="text-gray-600">Confidence:</span>
                                <span className="ml-1 font-medium text-green-600">
                                    {(response.confidenceScore * 100).toFixed(1)}%
                                </span>
                            </div>
                            <div className="bg-gray-50 p-3 rounded">
                                <span className="text-gray-600">Context Items:</span>
                                <span className="ml-1 font-medium text-blue-600">{response.contextItems.length}</span>
                            </div>
                            <div className="bg-gray-50 p-3 rounded">
                                <span className="text-gray-600">Sources:</span>
                                <span className="ml-1 font-medium text-purple-600">{response.sourcesUsed.length}</span>
                            </div>
                            <div className="bg-gray-50 p-3 rounded">
                                <span className="text-gray-600">Time:</span>
                                <span className="ml-1 font-medium text-orange-600">{response.processingTimeMs}ms</span>
                            </div>
                        </div>
                        
                        {response.contextItems.length > 0 && (
                            <div className="p-4 bg-gray-50 rounded-md">
                                <h4 className="font-medium text-gray-900 mb-2">Context Items:</h4>
                                <div className="space-y-2">
                                    {response.contextItems.map((item, index) => (
                                        <div key={index} className="text-sm">
                                            <span className="font-medium text-gray-700">[{index + 1}]</span>
                                            <span className="ml-2 text-gray-600">{item.kind}:</span>
                                            <span className="ml-1 text-gray-800">{item.text.substring(0, 100)}...</span>
                                            <span className="ml-2 text-blue-600">(score: {item.score.toFixed(3)})</span>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
};

export default AURAGDemo;