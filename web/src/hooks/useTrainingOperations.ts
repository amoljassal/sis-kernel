/**
 * Training Operations Hook
 * Manages training state and API interactions
 */

import { useState, useEffect, useCallback } from 'react';
import { 
  trainingApi, 
  TrainingSession, 
  ModelStats, 
  ComputeResource,
  TrainingParameters,
  TrainingActivityEvent
} from '../services/api/trainingApi';

interface UseTrainingOperationsReturn {
  // State
  sessions: TrainingSession[];
  modelStats: ModelStats | null;
  computeResources: ComputeResource[];
  activityEvents: TrainingActivityEvent[];
  loading: boolean;
  error: string | null;
  
  // Actions
  refreshData: () => Promise<void>;
  startTraining: (parameters: TrainingParameters) => Promise<void>;
  pauseTraining: (sessionId: string) => Promise<void>;
  stopTraining: (sessionId: string) => Promise<void>;
  resumeTraining: (sessionId: string) => Promise<void>;
  exportMetrics: (sessionId: string, format?: 'json' | 'csv' | 'pdf') => Promise<void>;
}

export const useTrainingOperations = (): UseTrainingOperationsReturn => {
  const [sessions, setSessions] = useState<TrainingSession[]>([]);
  const [modelStats, setModelStats] = useState<ModelStats | null>(null);
  const [computeResources, setComputeResources] = useState<ComputeResource[]>([]);
  const [activityEvents, setActivityEvents] = useState<TrainingActivityEvent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load initial data
  const refreshData = useCallback(async () => {
    setLoading(true);
    setError(null);
    
    try {
      const [sessionsData, statsData, resourcesData, activityData] = await Promise.all([
        trainingApi.getTrainingSessions(),
        trainingApi.getModelStats(),
        trainingApi.getComputeResources(),
        trainingApi.getTrainingActivity()
      ]);

      setSessions(sessionsData);
      setModelStats(statsData);
      setComputeResources(resourcesData);
      setActivityEvents(activityData);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load training data';
      setError(errorMessage);
      console.error('Error loading training operations data:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Training actions
  const startTraining = useCallback(async (parameters: TrainingParameters) => {
    try {
      setError(null);
      const newSession = await trainingApi.startTraining(parameters);
      setSessions(prev => [newSession, ...prev]);
      
      // Refresh stats after starting training
      const updatedStats = await trainingApi.getModelStats();
      setModelStats(updatedStats);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to start training';
      setError(errorMessage);
      throw err;
    }
  }, []);

  const pauseTraining = useCallback(async (sessionId: string) => {
    try {
      setError(null);
      await trainingApi.pauseTraining(sessionId);
      
      // Update local state
      setSessions(prev => prev.map(session => 
        session.id === sessionId 
          ? { ...session, status: 'paused' as const }
          : session
      ));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to pause training';
      setError(errorMessage);
      throw err;
    }
  }, []);

  const stopTraining = useCallback(async (sessionId: string) => {
    try {
      setError(null);
      await trainingApi.stopTraining(sessionId);
      
      // Update local state
      setSessions(prev => prev.map(session => 
        session.id === sessionId 
          ? { ...session, status: 'failed' as const }
          : session
      ));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to stop training';
      setError(errorMessage);
      throw err;
    }
  }, []);

  const resumeTraining = useCallback(async (sessionId: string) => {
    try {
      setError(null);
      await trainingApi.resumeTraining(sessionId);
      
      // Update local state
      setSessions(prev => prev.map(session => 
        session.id === sessionId 
          ? { ...session, status: 'running' as const }
          : session
      ));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to resume training';
      setError(errorMessage);
      throw err;
    }
  }, []);

  const exportMetrics = useCallback(async (sessionId: string, format: 'json' | 'csv' | 'pdf' = 'json') => {
    try {
      setError(null);
      const blob = await trainingApi.exportMetrics(sessionId, format);
      
      // Create download link
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `training-metrics-${sessionId}.${format}`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to export metrics';
      setError(errorMessage);
      throw err;
    }
  }, []);

  // Set up real-time updates
  useEffect(() => {
    // Listen for session updates
    const handleSessionUpdate = (event: CustomEvent) => {
      const updatedSession = event.detail as TrainingSession;
      setSessions(prev => prev.map(session => 
        session.id === updatedSession.id ? updatedSession : session
      ));
    };

    // Listen for resource updates
    const handleResourceUpdate = (event: CustomEvent) => {
      const updatedResources = event.detail as ComputeResource[];
      setComputeResources(updatedResources);
    };

    // Add event listeners
    window.addEventListener('training_session_update', handleSessionUpdate as EventListener);
    window.addEventListener('compute_resource_update', handleResourceUpdate as EventListener);

    // Subscribe to activity events
    const unsubscribeActivity = trainingApi.subscribeToActivity((event: TrainingActivityEvent) => {
      setActivityEvents(prev => [event, ...prev.slice(0, 19)]); // Keep last 20 events
    });

    return () => {
      window.removeEventListener('training_session_update', handleSessionUpdate as EventListener);
      window.removeEventListener('compute_resource_update', handleResourceUpdate as EventListener);
      unsubscribeActivity();
    };
  }, []);

  // Load initial data on mount
  useEffect(() => {
    refreshData();
  }, [refreshData]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      trainingApi.cleanup();
    };
  }, []);

  return {
    sessions,
    modelStats,
    computeResources,
    activityEvents,
    loading,
    error,
    refreshData,
    startTraining,
    pauseTraining,
    stopTraining,
    resumeTraining,
    exportMetrics
  };
};