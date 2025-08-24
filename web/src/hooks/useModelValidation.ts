/**
 * Model Validation Hook
 * Manages model validation state and operations
 */

import { useState, useEffect, useCallback } from 'react';
import { 
  validationApi, 
  ValidationTest, 
  ModelInfo, 
  TestSuite, 
  TestConfiguration,
  TestResult,
  PerformanceMetrics
} from '../services/api/validationApi';

interface UseModelValidationReturn {
  // State
  models: ModelInfo[];
  selectedModel: ModelInfo | null;
  availableTests: ValidationTest[];
  currentSuite: TestSuite | null;
  testResults: Record<string, TestResult>;
  benchmarks: PerformanceMetrics[];
  loading: boolean;
  error: string | null;
  
  // Actions
  loadModels: () => Promise<void>;
  selectModel: (modelId: string) => Promise<void>;
  loadAvailableTests: () => Promise<void>;
  startValidation: (config: TestConfiguration) => Promise<void>;
  cancelTest: (testId: string) => Promise<void>;
  loadTestResults: (testId: string) => Promise<void>;
  exportReport: (format?: 'pdf' | 'json' | 'csv') => Promise<void>;
  runCustomTest: (testData: {
    name: string;
    inputs: any[];
    expectedOutputs: any[];
    testType: 'accuracy' | 'performance' | 'robustness';
  }) => Promise<void>;
}

export const useModelValidation = (): UseModelValidationReturn => {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [selectedModel, setSelectedModel] = useState<ModelInfo | null>(null);
  const [availableTests, setAvailableTests] = useState<ValidationTest[]>([]);
  const [currentSuite, setCurrentSuite] = useState<TestSuite | null>(null);
  const [testResults, setTestResults] = useState<Record<string, TestResult>>({});
  const [benchmarks, setBenchmarks] = useState<PerformanceMetrics[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load available models
  const loadModels = useCallback(async () => {
    try {
      setError(null);
      setLoading(true);
      const modelsData = await validationApi.getModels();
      setModels(modelsData);
      
      // Auto-select first model if none selected
      if (!selectedModel && modelsData.length > 0) {
        setSelectedModel(modelsData[0]);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load models';
      setError(errorMessage);
      console.error('Error loading models:', err);
    } finally {
      setLoading(false);
    }
  }, [selectedModel]);

  // Select a specific model
  const selectModel = useCallback(async (modelId: string) => {
    try {
      setError(null);
      setLoading(true);
      const modelInfo = await validationApi.getModelInfo(modelId);
      setSelectedModel(modelInfo);
      
      // Load benchmarks for this model's architecture
      const benchmarksData = await validationApi.getPerformanceBenchmarks(modelInfo.architecture);
      setBenchmarks(benchmarksData);
      
      // Reset current suite when changing models
      setCurrentSuite(null);
      setTestResults({});
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to select model';
      setError(errorMessage);
      console.error('Error selecting model:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Load available validation tests
  const loadAvailableTests = useCallback(async () => {
    try {
      setError(null);
      const testsData = await validationApi.getAvailableTests();
      setAvailableTests(testsData);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load tests';
      setError(errorMessage);
      console.error('Error loading tests:', err);
    }
  }, []);

  // Start validation process
  const startValidation = useCallback(async (config: TestConfiguration) => {
    if (!selectedModel) {
      setError('No model selected for validation');
      return;
    }

    try {
      setError(null);
      setLoading(true);
      const suite = await validationApi.startValidation(selectedModel.id, config);
      setCurrentSuite(suite);
      
      // Update available tests with running status
      setAvailableTests(prev => prev.map(test => 
        config.testIds.includes(test.id) 
          ? { ...test, status: 'running', progress: 0 }
          : test
      ));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to start validation';
      setError(errorMessage);
      console.error('Error starting validation:', err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [selectedModel]);

  // Cancel a running test
  const cancelTest = useCallback(async (testId: string) => {
    try {
      setError(null);
      await validationApi.cancelTest(testId);
      
      // Update test status locally
      setAvailableTests(prev => prev.map(test => 
        test.id === testId 
          ? { ...test, status: 'pending', progress: 0 }
          : test
      ));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to cancel test';
      setError(errorMessage);
      console.error('Error canceling test:', err);
      throw err;
    }
  }, []);

  // Load test results
  const loadTestResults = useCallback(async (testId: string) => {
    try {
      setError(null);
      const result = await validationApi.getTestResults(testId);
      setTestResults(prev => ({
        ...prev,
        [testId]: result
      }));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load test results';
      setError(errorMessage);
      console.error('Error loading test results:', err);
    }
  }, []);

  // Export validation report
  const exportReport = useCallback(async (format: 'pdf' | 'json' | 'csv' = 'pdf') => {
    if (!currentSuite) {
      setError('No test suite to export');
      return;
    }

    try {
      setError(null);
      const blob = await validationApi.exportReport(currentSuite.id, format);
      
      // Create download link
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `validation-report-${currentSuite.id}.${format}`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to export report';
      setError(errorMessage);
      console.error('Error exporting report:', err);
      throw err;
    }
  }, [currentSuite]);

  // Run custom test
  const runCustomTest = useCallback(async (testData: {
    name: string;
    inputs: any[];
    expectedOutputs: any[];
    testType: 'accuracy' | 'performance' | 'robustness';
  }) => {
    if (!selectedModel) {
      setError('No model selected for testing');
      return;
    }

    try {
      setError(null);
      setLoading(true);
      const result = await validationApi.runCustomTest(selectedModel.id, testData);
      
      // Add result to test results
      setTestResults(prev => ({
        ...prev,
        [`custom-${Date.now()}`]: result
      }));
      
      // Add custom test to available tests list
      const customTest: ValidationTest = {
        id: `custom-${Date.now()}`,
        name: testData.name,
        description: `Custom ${testData.testType} test`,
        status: result.status,
        duration: 0,
        category: testData.testType,
        score: result.score,
        details: result.details
      };
      
      setAvailableTests(prev => [...prev, customTest]);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to run custom test';
      setError(errorMessage);
      console.error('Error running custom test:', err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [selectedModel]);

  // Set up real-time progress updates
  useEffect(() => {
    const handleValidationProgress = (event: CustomEvent) => {
      const { testId, progress, test } = event.detail;
      
      // Update test progress
      setAvailableTests(prev => prev.map(t => 
        t.id === testId 
          ? { ...t, ...test, progress }
          : t
      ));
    };

    const handleValidationComplete = (event: CustomEvent) => {
      const { testId, result } = event.detail;
      
      // Update test with final results
      setAvailableTests(prev => prev.map(t => 
        t.id === testId 
          ? { ...t, status: result.status, score: result.score, details: result.details }
          : t
      ));
      
      // Store detailed results
      setTestResults(prev => ({
        ...prev,
        [testId]: result
      }));
    };

    // Add event listeners
    window.addEventListener('validation_progress', handleValidationProgress as EventListener);
    window.addEventListener('validation_complete', handleValidationComplete as EventListener);

    return () => {
      window.removeEventListener('validation_progress', handleValidationProgress as EventListener);
      window.removeEventListener('validation_complete', handleValidationComplete as EventListener);
    };
  }, []);

  // Load initial data on mount
  useEffect(() => {
    loadModels();
    loadAvailableTests();
  }, [loadModels, loadAvailableTests]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      validationApi.cleanup();
    };
  }, []);

  return {
    models,
    selectedModel,
    availableTests,
    currentSuite,
    testResults,
    benchmarks,
    loading,
    error,
    loadModels,
    selectModel,
    loadAvailableTests,
    startValidation,
    cancelTest,
    loadTestResults,
    exportReport,
    runCustomTest
  };
};