/**
 * SIS Backend Server
 * Node.js Express server for real AURAG and MLX functionality
 */

import express from 'express';
import cors from 'cors';
import { json } from 'body-parser';
import path from 'path';
import { initializeDatabase } from '../database/config';
import { createRealRAGService } from '../services/aurag/real-rag-service';
import { createRealMLXTrainingPipeline } from '../services/mlx/real-training-pipeline';

const app = express();
const PORT = process.env.PORT || 3001;

// Middleware
app.use(cors());
app.use(json({ limit: '50mb' }));

// Initialize services
let ragService: any = null;
let mlxPipeline: any = null;

// Initialize database and services
async function initializeServices() {
    try {
        console.log('Initializing backend services...');
        
        // Initialize database
        await initializeDatabase();
        console.log('Database connected');

        // Initialize AURAG service
        ragService = createRealRAGService({
            maxContextItems: 15,
            tokenBudget: 2000,
            defaultPhilosophicalLens: 'analytical',
            providers: {
                embedding: 'ollama',
                llm: 'ollama'
            }
        });
        
        await ragService.initialize();
        console.log('AURAG service initialized');

        // Initialize MLX pipeline
        mlxPipeline = createRealMLXTrainingPipeline('./backend_mlx_scripts');
        await mlxPipeline.initialize();
        console.log('MLX pipeline initialized');

    } catch (error) {
        console.error('Service initialization failed:', error);
        throw error;
    }
}

// AURAG API Routes
app.post('/api/aurag/document', async (req, res) => {
    try {
        const { userId, title, content } = req.body;
        
        if (!ragService) {
            return res.status(500).json({ error: 'AURAG service not initialized' });
        }

        const result = await ragService.processDocument(userId, title, content);
        res.json(result);
    } catch (error) {
        console.error('Document processing error:', error);
        res.status(500).json({ error: 'Document processing failed' });
    }
});

app.post('/api/aurag/query', async (req, res) => {
    try {
        const { userId, query, philosophicalLens } = req.body;
        
        if (!ragService) {
            return res.status(500).json({ error: 'AURAG service not initialized' });
        }

        const result = await ragService.processRAGQuery(userId, query, philosophicalLens);
        res.json(result);
    } catch (error) {
        console.error('RAG query error:', error);
        res.status(500).json({ error: 'RAG query failed' });
    }
});

app.get('/api/aurag/stats/:userId', async (req, res) => {
    try {
        const { userId } = req.params;
        
        if (!ragService) {
            return res.status(500).json({ error: 'AURAG service not initialized' });
        }

        const stats = await ragService.getKnowledgeGraphStats(parseInt(userId));
        res.json(stats);
    } catch (error) {
        console.error('Stats retrieval error:', error);
        res.status(500).json({ error: 'Stats retrieval failed' });
    }
});

// MLX API Routes
app.get('/api/mlx/status', (req, res) => {
    res.json({ 
        status: mlxPipeline ? 'initialized' : 'not_initialized',
        timestamp: new Date().toISOString()
    });
});

app.post('/api/mlx/train', async (req, res) => {
    try {
        const { trainingId, config } = req.body;
        
        if (!mlxPipeline) {
            return res.status(500).json({ error: 'MLX pipeline not initialized' });
        }

        const resultId = await mlxPipeline.startTraining(config.description, config.datasetPath);
        res.json({ trainingId: resultId });
    } catch (error) {
        console.error('Training start error:', error);
        res.status(500).json({ error: 'Training start failed' });
    }
});

app.get('/api/mlx/progress/:trainingId', async (req, res) => {
    try {
        const { trainingId } = req.params;
        
        if (!mlxPipeline) {
            return res.status(500).json({ error: 'MLX pipeline not initialized' });
        }

        const progress = await mlxPipeline.getTrainingProgress(trainingId);
        res.json(progress);
    } catch (error) {
        console.error('Progress retrieval error:', error);
        res.status(500).json({ error: 'Progress retrieval failed' });
    }
});

app.get('/api/mlx/result/:trainingId', async (req, res) => {
    try {
        const { trainingId } = req.params;
        
        if (!mlxPipeline) {
            return res.status(500).json({ error: 'MLX pipeline not initialized' });
        }

        const result = await mlxPipeline.getTrainingResult(trainingId);
        res.json(result);
    } catch (error) {
        console.error('Result retrieval error:', error);
        res.status(500).json({ error: 'Result retrieval failed' });
    }
});

app.post('/api/mlx/stop/:trainingId', async (req, res) => {
    try {
        const { trainingId } = req.params;
        
        if (!mlxPipeline) {
            return res.status(500).json({ error: 'MLX pipeline not initialized' });
        }

        const success = await mlxPipeline.stopTraining(trainingId);
        res.json({ success });
    } catch (error) {
        console.error('Training stop error:', error);
        res.status(500).json({ error: 'Training stop failed' });
    }
});

app.get('/api/mlx/sessions', async (req, res) => {
    try {
        if (!mlxPipeline) {
            return res.status(500).json({ error: 'MLX pipeline not initialized' });
        }

        const sessions = await mlxPipeline.listTrainingSessions();
        res.json({ sessions });
    } catch (error) {
        console.error('Sessions list error:', error);
        res.status(500).json({ error: 'Sessions list failed' });
    }
});

// Health check
app.get('/api/health', (req, res) => {
    res.json({
        status: 'ok',
        services: {
            aurag: ragService ? 'initialized' : 'not_initialized',
            mlx: mlxPipeline ? 'initialized' : 'not_initialized'
        },
        timestamp: new Date().toISOString()
    });
});

// Error handling middleware
app.use((error: any, req: express.Request, res: express.Response, next: express.NextFunction) => {
    console.error('Unhandled error:', error);
    res.status(500).json({
        error: 'Internal server error',
        message: error.message
    });
});

// Start server
async function startServer() {
    try {
        await initializeServices();
        
        app.listen(PORT, () => {
            console.log(`SIS Backend Server running on port ${PORT}`);
            console.log(`Health check: http://localhost:${PORT}/api/health`);
            console.log(`AURAG API: http://localhost:${PORT}/api/aurag/`);
            console.log(`MLX API: http://localhost:${PORT}/api/mlx/`);
        });
    } catch (error) {
        console.error('Failed to start server:', error);
        process.exit(1);
    }
}

// Handle graceful shutdown
process.on('SIGINT', () => {
    console.log('Shutting down server...');
    process.exit(0);
});

process.on('SIGTERM', () => {
    console.log('Shutting down server...');
    process.exit(0);
});

if (require.main === module) {
    startServer();
}

export default app;