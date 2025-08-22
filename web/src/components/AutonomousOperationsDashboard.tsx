import React, { useState, useEffect } from 'react';
import {
  ChartBarIcon,
  CpuChipIcon,
  ExclamationTriangleIcon,
  BoltIcon,
  AcademicCapIcon,
  ClockIcon,
  CurrencyDollarIcon,
  ShieldCheckIcon,
  ArrowTrendingUpIcon,
  ArrowTrendingDownIcon,
  CheckCircleIcon
} from '@heroicons/react/24/outline';
import { autonomousOperations, type AutonomousOperationsStatus } from '../services/autonomous-operations';

interface AutonomousOperationsDashboardProps {
  isVisible: boolean;
  onClose: () => void;
}

export const AutonomousOperationsDashboard: React.FC<AutonomousOperationsDashboardProps> = ({
  isVisible,
  onClose
}) => {
  const [status, setStatus] = useState<AutonomousOperationsStatus | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    if (isVisible && !isInitialized) {
      initializeAutonomousOps();
    }
  }, [isVisible, isInitialized]);

  useEffect(() => {
    if (isVisible) {
      const interval = setInterval(() => {
        const currentStatus = autonomousOperations.getOperationalStatus();
        setStatus(currentStatus);
      }, 5000); // Update every 5 seconds

      return () => clearInterval(interval);
    }
  }, [isVisible]);

  const initializeAutonomousOps = async () => {
    try {
      await autonomousOperations.initialize();
      setIsInitialized(true);
      const currentStatus = autonomousOperations.getOperationalStatus();
      setStatus(currentStatus);
    } catch (error) {
      console.error('Failed to initialize autonomous operations:', error);
    }
  };

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-gray-900 rounded-lg shadow-2xl max-w-7xl w-full max-h-screen overflow-auto">
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-gray-700">
          <div className="flex items-center space-x-3">
            <CpuChipIcon className="w-8 h-8 text-blue-400" />
            <div>
              <h2 className="text-2xl font-bold text-white">Autonomous Operations Center</h2>
              <p className="text-gray-400">AI-Driven Educational Platform Management</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {!isInitialized ? (
          <div className="p-8 text-center">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-400 mx-auto mb-4"></div>
            <p className="text-white">Initializing Autonomous Operations...</p>
            <p className="text-gray-400 text-sm mt-2">Loading ML models and AIOps systems</p>
          </div>
        ) : (
          <div className="p-6 space-y-6">
            {/* Key Metrics Overview */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <MetricCard
                title="System Health"
                value={`${status?.systemHealthScore || 0}%`}
                icon={ShieldCheckIcon}
                color="text-green-400"
                trend={status?.systemHealthScore && status.systemHealthScore > 85 ? 'up' : 'stable'}
              />
              <MetricCard
                title="Auto-Resolution Rate"
                value={`${((status?.autoResolutionRate || 0) * 100).toFixed(1)}%`}
                icon={CheckCircleIcon}
                color="text-blue-400"
                trend="up"
              />
              <MetricCard
                title="Student Experience"
                value={`${status?.studentExperienceScore || 0}%`}
                icon={AcademicCapIcon}
                color="text-purple-400"
                trend={status?.studentExperienceScore && status.studentExperienceScore > 90 ? 'up' : 'stable'}
              />
              <MetricCard
                title="Cost Optimization"
                value={`$${status?.costOptimization || 0}/hr`}
                icon={CurrencyDollarIcon}
                color="text-yellow-400"
                trend="up"
              />
            </div>

            {/* Traffic Predictions */}
            <div className="card p-6">
              <div className="flex items-center space-x-3 mb-4">
                <ChartBarIcon className="w-6 h-6 text-blue-400" />
                <h3 className="text-xl font-bold text-white">Traffic Predictions (Next 6 Hours)</h3>
                <span className="text-sm text-gray-400">
                  Accuracy: {((status?.predictionAccuracy || 0) * 100).toFixed(1)}%
                </span>
              </div>
              
              <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-4">
                {status?.trafficPredictions.map((prediction, index) => (
                  <div key={index} className="bg-gray-800 rounded-lg p-4 text-center">
                    <div className="text-sm text-gray-400 mb-1">
                      {new Date(prediction.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </div>
                    <div className="text-2xl font-bold text-white mb-2">
                      {prediction.predictedTraffic.toFixed(0)}
                    </div>
                    <div className="text-xs text-gray-400 mb-2">
                      {(prediction.confidence * 100).toFixed(0)}% confidence
                    </div>
                    <div className="flex justify-center space-x-1 text-xs">
                      {prediction.educationalContext.isSchoolHours && (
                        <span className="bg-blue-600 text-white px-1 py-0.5 rounded">School</span>
                      )}
                      {prediction.educationalContext.isExamPeriod && (
                        <span className="bg-red-600 text-white px-1 py-0.5 rounded">Exam</span>
                      )}
                      {prediction.educationalContext.isPeakAssignment && (
                        <span className="bg-yellow-600 text-white px-1 py-0.5 rounded">Assignment</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Active Incidents */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="card p-6">
                <div className="flex items-center space-x-3 mb-4">
                  <ExclamationTriangleIcon className="w-6 h-6 text-orange-400" />
                  <h3 className="text-xl font-bold text-white">Active Incidents</h3>
                  <span className="text-sm text-gray-400">
                    {status?.activeIncidents.length || 0} total
                  </span>
                </div>
                
                <div className="space-y-3">
                  {status?.activeIncidents.length === 0 ? (
                    <div className="text-center py-8 text-gray-400">
                      <CheckCircleIcon className="w-12 h-12 mx-auto mb-2 text-green-400" />
                      <p>No active incidents</p>
                      <p className="text-sm">System operating normally</p>
                    </div>
                  ) : (
                    status?.activeIncidents.slice(0, 5).map((incident) => (
                      <div key={incident.id} className="bg-gray-800 rounded-lg p-4">
                        <div className="flex justify-between items-start mb-2">
                          <div className="flex items-center space-x-2">
                            <div className={`w-3 h-3 rounded-full ${getSeverityColor(incident.severity)}`} />
                            <span className="font-medium text-white">{incident.component}</span>
                            {incident.autoResolved && (
                              <CheckCircleIcon className="w-4 h-4 text-green-400" />
                            )}
                          </div>
                          <span className="text-xs text-gray-400">
                            {new Date(incident.timestamp).toLocaleTimeString()}
                          </span>
                        </div>
                        <p className="text-sm text-gray-300 mb-2">{incident.description}</p>
                        <div className="flex justify-between items-center text-xs">
                          <span className="text-gray-400">
                            Educational Impact: {(incident.educationalImpact * 100).toFixed(0)}%
                          </span>
                          {incident.resolutionTime && (
                            <span className="text-green-400">
                              Resolved in {(incident.resolutionTime / 1000).toFixed(1)}s
                            </span>
                          )}
                        </div>
                        {incident.resolutionActions.length > 0 && (
                          <div className="mt-2">
                            <p className="text-xs text-gray-400 mb-1">Auto-healing actions:</p>
                            <ul className="text-xs text-gray-300 space-y-1">
                              {incident.resolutionActions.map((action, idx) => (
                                <li key={idx} className="flex items-center space-x-1">
                                  <span className="text-green-400">•</span>
                                  <span>{action}</span>
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}
                      </div>
                    ))
                  )}
                </div>
              </div>

              {/* Scaling Decisions */}
              <div className="card p-6">
                <div className="flex items-center space-x-3 mb-4">
                  <BoltIcon className="w-6 h-6 text-yellow-400" />
                  <h3 className="text-xl font-bold text-white">Auto-Scaling Decisions</h3>
                  <span className="text-sm text-gray-400">
                    {status?.activeScalingDecisions.length || 0} pending
                  </span>
                </div>
                
                <div className="space-y-3">
                  {status?.activeScalingDecisions.length === 0 ? (
                    <div className="text-center py-8 text-gray-400">
                      <BoltIcon className="w-12 h-12 mx-auto mb-2 text-yellow-400" />
                      <p>No scaling decisions pending</p>
                      <p className="text-sm">Resources optimally allocated</p>
                    </div>
                  ) : (
                    status?.activeScalingDecisions.map((decision, index) => (
                      <div key={index} className="bg-gray-800 rounded-lg p-4">
                        <div className="flex justify-between items-start mb-2">
                          <div className="flex items-center space-x-2">
                            <span className="font-medium text-white">{decision.component}</span>
                            {decision.direction === 'up' ? (
                              <ArrowTrendingUpIcon className="w-4 h-4 text-green-400" />
                            ) : decision.direction === 'down' ? (
                              <ArrowTrendingDownIcon className="w-4 h-4 text-red-400" />
                            ) : (
                              <span className="w-4 h-4 text-gray-400">—</span>
                            )}
                          </div>
                          <span className="text-xs text-gray-400">
                            {(decision.confidence * 100).toFixed(0)}% confidence
                          </span>
                        </div>
                        <div className="flex justify-between items-center text-sm mb-2">
                          <span className="text-gray-300">
                            {decision.currentInstances} → {decision.targetInstances} instances
                          </span>
                          <span className={`${decision.costImpact >= 0 ? 'text-red-400' : 'text-green-400'}`}>
                            {decision.costImpact >= 0 ? '+' : ''}${decision.costImpact.toFixed(2)}/hr
                          </span>
                        </div>
                        <p className="text-xs text-gray-400 mb-2">{decision.reasoning}</p>
                        <div className="flex justify-between items-center text-xs">
                          <span className="text-gray-400">
                            Execute: {new Date(decision.executionTime).toLocaleTimeString()}
                          </span>
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </div>

            {/* Educational Metrics */}
            <div className="card p-6">
              <div className="flex items-center space-x-3 mb-4">
                <AcademicCapIcon className="w-6 h-6 text-purple-400" />
                <h3 className="text-xl font-bold text-white">Educational Performance Metrics</h3>
              </div>
              
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
                <EducationalMetricCard
                  title="Student Success Rate"
                  value={`${((status?.educationalMetrics.studentSuccessRate || 0) * 100).toFixed(1)}%`}
                  target="99.5%"
                  color="text-green-400"
                />
                <EducationalMetricCard
                  title="Collaboration Effectiveness"
                  value={`${((status?.educationalMetrics.collaborationEffectiveness || 0) * 100).toFixed(1)}%`}
                  target="90%"
                  color="text-blue-400"
                />
                <EducationalMetricCard
                  title="AI Assistance Utilization"
                  value={`${((status?.educationalMetrics.aiAssistanceUtilization || 0) * 100).toFixed(1)}%`}
                  target="80%"
                  color="text-purple-400"
                />
                <EducationalMetricCard
                  title="Learning Velocity"
                  value={`${((status?.educationalMetrics.learningVelocity || 0) * 100).toFixed(1)}%`}
                  target="85%"
                  color="text-orange-400"
                />
                <EducationalMetricCard
                  title="Peak Hours Availability"
                  value={`${((status?.educationalMetrics.systemAvailabilityDuringPeakHours || 0) * 100).toFixed(2)}%`}
                  target="99.9%"
                  color="text-cyan-400"
                />
              </div>
            </div>

            {/* System Status Summary */}
            <div className="card p-6">
              <div className="flex items-center space-x-3 mb-4">
                <ClockIcon className="w-6 h-6 text-cyan-400" />
                <h3 className="text-xl font-bold text-white">Autonomous Operations Summary</h3>
              </div>
              
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="bg-gray-800 rounded-lg p-4">
                  <h4 className="font-semibold text-white mb-2">AIOps Performance</h4>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-400">Incidents Auto-Resolved:</span>
                      <span className="text-green-400">{((status?.autoResolutionRate || 0) * 100).toFixed(1)}%</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Avg Resolution Time:</span>
                      <span className="text-white">{(status?.averageResolutionTime || 0).toFixed(1)}s</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Prediction Accuracy:</span>
                      <span className="text-blue-400">{((status?.predictionAccuracy || 0) * 100).toFixed(1)}%</span>
                    </div>
                  </div>
                </div>
                
                <div className="bg-gray-800 rounded-lg p-4">
                  <h4 className="font-semibold text-white mb-2">Resource Efficiency</h4>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-400">Resource Efficiency:</span>
                      <span className="text-green-400">{status?.resourceEfficiency || 0}%</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Cost Optimization:</span>
                      <span className="text-yellow-400">${status?.costOptimization || 0}/hr saved</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Active Scaling:</span>
                      <span className="text-white">{status?.activeScalingDecisions.length || 0} decisions</span>
                    </div>
                  </div>
                </div>
                
                <div className="bg-gray-800 rounded-lg p-4">
                  <h4 className="font-semibold text-white mb-2">Educational Impact</h4>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-400">Student Experience:</span>
                      <span className="text-purple-400">{status?.studentExperienceScore || 0}%</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">System Health:</span>
                      <span className="text-green-400">{status?.systemHealthScore || 0}%</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Platform Status:</span>
                      <span className="text-green-400">Optimal</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

// Helper Components
interface MetricCardProps {
  title: string;
  value: string;
  icon: React.ComponentType<{ className?: string }>;
  color: string;
  trend: 'up' | 'down' | 'stable';
}

const MetricCard: React.FC<MetricCardProps> = ({ title, value, icon: Icon, color, trend }) => (
  <div className="card p-6">
    <div className="flex items-center justify-between mb-4">
      <Icon className={`w-8 h-8 ${color}`} />
      <div className="flex items-center space-x-1">
        {trend === 'up' && <ArrowTrendingUpIcon className="w-4 h-4 text-green-400" />}
        {trend === 'down' && <ArrowTrendingDownIcon className="w-4 h-4 text-red-400" />}
        {trend === 'stable' && <span className="w-4 h-4 text-gray-400">—</span>}
      </div>
    </div>
    <div className="text-2xl font-bold text-white mb-1">{value}</div>
    <div className="text-sm text-gray-400">{title}</div>
  </div>
);

interface EducationalMetricCardProps {
  title: string;
  value: string;
  target: string;
  color: string;
}

const EducationalMetricCard: React.FC<EducationalMetricCardProps> = ({ title, value, target, color }) => (
  <div className="bg-gray-800 rounded-lg p-4 text-center">
    <div className={`text-2xl font-bold ${color} mb-1`}>{value}</div>
    <div className="text-sm text-gray-400 mb-1">{title}</div>
    <div className="text-xs text-gray-500">Target: {target}</div>
  </div>
);

const getSeverityColor = (severity: string): string => {
  switch (severity) {
    case 'critical': return 'bg-red-500';
    case 'high': return 'bg-orange-500';
    case 'medium': return 'bg-yellow-500';
    case 'low': return 'bg-blue-500';
    default: return 'bg-gray-500';
  }
};

export default AutonomousOperationsDashboard;