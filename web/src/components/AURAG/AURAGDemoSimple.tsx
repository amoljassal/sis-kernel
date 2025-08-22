/**
 * Simple AURAG Demo Component
 * Browser-compatible demonstration version
 */

import React, { useState } from 'react';
import { Brain, FileText, Search, BarChart3 } from 'lucide-react';

interface DocumentProcessingResult {
  success: boolean;
  chunksCreated: number;
  conceptsExtracted: number;
  processingTimeMs: number;
}

interface RAGResponse {
  responseText: string;
  confidenceScore: number;
  processingTimeMs: number;
  contextItems: Array<{
    kind: string;
    text: string;
    score: number;
    source: string;
  }>;
  sourcesUsed: string[];
}

export const AURAGDemo: React.FC = () => {
  const [documentTitle, setDocumentTitle] = useState('');
  const [documentContent, setDocumentContent] = useState('');
  const [query, setQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [processingResult, setProcessingResult] = useState<DocumentProcessingResult | null>(null);
  const [response, setResponse] = useState<RAGResponse | null>(null);
  const [knowledgeStats, setKnowledgeStats] = useState({
    totalDocuments: 0,
    totalChunks: 0,
    totalConcepts: 0,
    topConcepts: []
  });

  const handleDocumentSubmit = async () => {
    if (!documentTitle || !documentContent) return;

    setLoading(true);
    
    // Simulate processing delay
    await new Promise(resolve => setTimeout(resolve, 1500));

    const result: DocumentProcessingResult = {
      success: true,
      chunksCreated: Math.ceil(documentContent.length / 200),
      conceptsExtracted: Math.ceil(documentContent.split(' ').length / 20),
      processingTimeMs: 1500
    };

    setProcessingResult(result);
    
    // Update stats
    setKnowledgeStats(prev => ({
      totalDocuments: prev.totalDocuments + 1,
      totalChunks: prev.totalChunks + result.chunksCreated,
      totalConcepts: prev.totalConcepts + result.conceptsExtracted,
      topConcepts: []
    }));

    setLoading(false);
    setDocumentTitle('');
    setDocumentContent('');
  };

  const handleQuerySubmit = async () => {
    if (!query) return;

    setLoading(true);
    
    // Simulate processing delay
    await new Promise(resolve => setTimeout(resolve, 1200));

    const mockResponse: RAGResponse = {
      responseText: `Based on your query "${query}", I found relevant information from your knowledge base. This demonstrates how AURAG (Advanced Unified RAG) would process your personal documents and provide contextually relevant answers using philosophical reasoning lenses. The system combines semantic search with concept relationships to deliver intelligent responses.`,
      confidenceScore: 0.85 + Math.random() * 0.1,
      processingTimeMs: 1200,
      contextItems: [
        {
          kind: 'document',
          text: 'Sample context from your processed documents would appear here...',
          score: 0.92,
          source: 'Document 1'
        },
        {
          kind: 'concept',
          text: 'Related concepts and their relationships would be shown here...',
          score: 0.78,
          source: 'Concept Graph'
        }
      ],
      sourcesUsed: ['Document 1', 'Concept Graph']
    };

    setResponse(mockResponse);
    setLoading(false);
    setQuery('');
  };

  return (
    <div className="max-w-6xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="glass rounded-lg p-6">
        <h1 className="text-3xl font-bold text-white mb-2 flex items-center space-x-3">
          <Brain className="w-8 h-8 text-sis-blue-400" />
          <span>SIS AURAG System</span>
        </h1>
        <p className="text-sis-gray-300">
          Advanced Unified RAG - Intelligence in Structure, Not Parameters
        </p>
        
        <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div className="bg-green-500/20 p-3 rounded border border-green-500/30">
            <div className="font-medium text-green-400">Status</div>
            <div className="text-green-300">Active</div>
          </div>
          <div className="bg-sis-blue-500/20 p-3 rounded border border-sis-blue-500/30">
            <div className="font-medium text-sis-blue-400">Mode</div>
            <div className="text-sis-blue-300">Analytical</div>
          </div>
          <div className="bg-purple-500/20 p-3 rounded border border-purple-500/30">
            <div className="font-medium text-purple-400">Lens</div>
            <div className="text-purple-300">Philosophical</div>
          </div>
          <div className="bg-orange-500/20 p-3 rounded border border-orange-500/30">
            <div className="font-medium text-orange-400">Provider</div>
            <div className="text-orange-300">Demo</div>
          </div>
        </div>
      </div>

      {/* Knowledge Graph Stats */}
      <div className="glass rounded-lg p-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center space-x-2">
          <BarChart3 className="w-5 h-5" />
          <span>Knowledge Graph Statistics</span>
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
            <div className="font-medium text-sis-gray-300">Density</div>
            <div className="text-2xl font-bold text-white">
              {knowledgeStats.totalDocuments > 0 
                ? (knowledgeStats.totalConcepts / knowledgeStats.totalDocuments).toFixed(1)
                : '0'
              }
            </div>
          </div>
        </div>
      </div>

      {/* Document Processing */}
      <div className="glass rounded-lg p-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center space-x-2">
          <FileText className="w-5 h-5" />
          <span>Document Processing</span>
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
                <span className="ml-1 font-medium text-green-400">Success</span>
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
          </div>
        )}
      </div>

      {/* RAG Query */}
      <div className="glass rounded-lg p-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center space-x-2">
          <Search className="w-5 h-5" />
          <span>RAG Query</span>
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
              placeholder="Ask a question about your knowledge base..."
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
              <p className="text-sis-blue-100 whitespace-pre-wrap">{response.responseText}</p>
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
          </div>
        )}
      </div>

      {/* Info Panel */}
      <div className="bg-sis-blue-900/30 border border-sis-blue-500/30 p-4 rounded-md">
        <h3 className="font-medium text-sis-blue-300 mb-2">Demo Mode</h3>
        <p className="text-sis-blue-200 text-sm">
          This is a demonstration of the AURAG system. In production, this would connect to real 
          TypeORM databases with actual document processing, concept extraction, and knowledge graph operations.
        </p>
      </div>
    </div>
  );
};

export default AURAGDemo;