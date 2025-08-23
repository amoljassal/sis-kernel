/**
 * Validation Reports
 * Historical validation data and detailed analysis reports
 */

import React, { useState } from 'react';
import {
  FileText,
  Download,
  Calendar,
  Filter,
  TrendingUp,
  AlertTriangle,
  CheckCircle,
  BarChart3,
  Clock,
  Users
} from 'lucide-react';

interface ValidationReport {
  id: string;
  modelName: string;
  version: string;
  date: string;
  status: 'passed' | 'failed' | 'warning';
  overallScore: number;
  testsPassed: number;
  testsTotal: number;
  duration: string;
  validator: string;
  categories: {
    accuracy: number;
    performance: number;
    robustness: number;
    bias: number;
    security: number;
  };
}

const SAMPLE_REPORTS: ValidationReport[] = [
  {
    id: 'report-001',
    modelName: 'Legal Document Analyzer',
    version: 'v2.1.3',
    date: '2024-01-20T14:30:00Z',
    status: 'passed',
    overallScore: 91.4,
    testsPassed: 8,
    testsTotal: 9,
    duration: '2h 14m',
    validator: 'AutoValidator',
    categories: {
      accuracy: 94.7,
      performance: 89.2,
      robustness: 87.3,
      bias: 76.3,
      security: 95.8
    }
  },
  {
    id: 'report-002',
    modelName: 'Medical Knowledge Assistant',
    version: 'v1.8.2',
    date: '2024-01-19T09:15:00Z',
    status: 'warning',
    overallScore: 84.2,
    testsPassed: 7,
    testsTotal: 9,
    duration: '1h 45m',
    validator: 'Manual Review',
    categories: {
      accuracy: 91.2,
      performance: 78.4,
      robustness: 82.1,
      bias: 69.8,
      security: 89.5
    }
  },
  {
    id: 'report-003',
    modelName: 'Code Generation Model',
    version: 'v3.0.1',
    date: '2024-01-18T16:45:00Z',
    status: 'failed',
    overallScore: 67.8,
    testsPassed: 4,
    testsTotal: 9,
    duration: '3h 22m',
    validator: 'AutoValidator',
    categories: {
      accuracy: 72.1,
      performance: 56.3,
      robustness: 68.9,
      bias: 71.2,
      security: 70.5
    }
  },
  {
    id: 'report-004',
    modelName: 'Sentiment Analysis Engine',
    version: 'v2.3.1',
    date: '2024-01-17T11:20:00Z',
    status: 'passed',
    overallScore: 93.6,
    testsPassed: 9,
    testsTotal: 9,
    duration: '1h 28m',
    validator: 'AutoValidator',
    categories: {
      accuracy: 96.2,
      performance: 91.8,
      robustness: 94.1,
      bias: 88.7,
      security: 97.2
    }
  },
  {
    id: 'report-005',
    modelName: 'Language Translation Bot',
    version: 'v1.5.0',
    date: '2024-01-16T08:30:00Z',
    status: 'passed',
    overallScore: 88.9,
    testsPassed: 8,
    testsTotal: 9,
    duration: '2h 52m',
    validator: 'Manual Review',
    categories: {
      accuracy: 92.4,
      performance: 85.7,
      robustness: 87.2,
      bias: 81.3,
      security: 91.9
    }
  }
];

export const ValidationReports: React.FC = () => {
  const [reports] = useState<ValidationReport[]>(SAMPLE_REPORTS);
  const [selectedStatus, setSelectedStatus] = useState<'all' | 'passed' | 'failed' | 'warning'>('all');
  const [selectedReport, setSelectedReport] = useState<ValidationReport | null>(null);
  const [sortBy, setSortBy] = useState<'date' | 'score' | 'model'>('date');

  const filteredReports = reports
    .filter(report => selectedStatus === 'all' || report.status === selectedStatus)
    .sort((a, b) => {
      switch (sortBy) {
        case 'score':
          return b.overallScore - a.overallScore;
        case 'model':
          return a.modelName.localeCompare(b.modelName);
        default:
          return new Date(b.date).getTime() - new Date(a.date).getTime();
      }
    });

  const getStatusIcon = (status: ValidationReport['status']) => {
    switch (status) {
      case 'passed': return <CheckCircle className="w-5 h-5 text-green-400" />;
      case 'warning': return <AlertTriangle className="w-5 h-5 text-yellow-400" />;
      case 'failed': return <AlertTriangle className="w-5 h-5 text-red-400" />;
    }
  };

  const getStatusColor = (status: ValidationReport['status']) => {
    switch (status) {
      case 'passed': return 'text-green-400';
      case 'warning': return 'text-yellow-400';
      case 'failed': return 'text-red-400';
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'});
  };

  const CategoryBar: React.FC<{ category: string; score: number }> = ({ category, score }) => (
    <div className="flex items-center justify-between mb-2">
      <span className="text-sm text-sis-gray-400 capitalize">{category}</span>
      <div className="flex items-center space-x-2">
        <div className="w-20 bg-sis-gray-700 rounded-full h-2">
          <div 
            className="h-2 rounded-full bg-gradient-to-r from-blue-500 to-purple-500"
            style={{ width: `${score}%` }}
          ></div>
        </div>
        <span className="text-sm text-white w-8">{score.toFixed(0)}</span>
      </div>
    </div>
  );

  if (selectedReport) {
    return (
      <div className="space-y-6">
        {/* Report Header */}
        <div className="flex items-center justify-between">
          <button
            onClick={() => setSelectedReport(null)}
            className="text-sis-blue-400 hover:text-sis-blue-300 transition-colors"
          >
            ← Back to Reports
          </button>
          
          <button className="btn-primary px-4 py-2 flex items-center space-x-2">
            <Download className="w-4 h-4" />
            <span>Export Report</span>
          </button>
        </div>

        {/* Detailed Report View */}
        <div className="card p-6">
          <div className="flex items-start justify-between mb-6">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">{selectedReport.modelName}</h2>
              <p className="text-sis-gray-400">Version {selectedReport.version}</p>
              <p className="text-sm text-sis-gray-500 mt-1">{formatDate(selectedReport.date)}</p>
            </div>
            
            <div className="text-right">
              <div className="flex items-center space-x-2 mb-2">
                {getStatusIcon(selectedReport.status)}
                <span className={`font-medium capitalize ${getStatusColor(selectedReport.status)}`}>
                  {selectedReport.status}
                </span>
              </div>
              <div className="text-3xl font-bold text-white">{selectedReport.overallScore.toFixed(1)}</div>
              <div className="text-sm text-sis-gray-400">Overall Score</div>
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {/* Test Results */}
            <div>
              <h3 className="text-lg font-semibold text-white mb-4">Test Results</h3>
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-sis-gray-400">Tests Passed</span>
                  <span className="text-white">{selectedReport.testsPassed} / {selectedReport.testsTotal}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sis-gray-400">Success Rate</span>
                  <span className="text-white">{((selectedReport.testsPassed / selectedReport.testsTotal) * 100).toFixed(1)}%</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sis-gray-400">Duration</span>
                  <span className="text-white">{selectedReport.duration}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sis-gray-400">Validator</span>
                  <span className="text-white">{selectedReport.validator}</span>
                </div>
              </div>
            </div>

            {/* Category Scores */}
            <div>
              <h3 className="text-lg font-semibold text-white mb-4">Category Breakdown</h3>
              <div className="space-y-2">
                {Object.entries(selectedReport.categories).map(([category, score]) => (
                  <CategoryBar key={category} category={category} score={score} />
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-white mb-2">Validation Reports</h2>
          <p className="text-sis-gray-400">Historical validation data and analysis</p>
        </div>
        
        <div className="flex items-center space-x-3">
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="bg-sis-gray-800 border border-sis-gray-600 rounded text-white text-sm px-3 py-2"
          >
            <option value="date">Sort by Date</option>
            <option value="score">Sort by Score</option>
            <option value="model">Sort by Model</option>
          </select>
          
          <button className="btn-primary px-4 py-2 flex items-center space-x-2">
            <Download className="w-4 h-4" />
            <span>Export All</span>
          </button>
        </div>
      </div>

      {/* Status Filter */}
      <div className="flex items-center space-x-2">
        {['all', 'passed', 'failed', 'warning'].map(status => (
          <button
            key={status}
            onClick={() => setSelectedStatus(status as any)}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              selectedStatus === status
                ? 'bg-sis-blue-600 text-white'
                : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
            }`}
          >
            {status.charAt(0).toUpperCase() + status.slice(1)}
          </button>
        ))}
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="card p-4 text-center">
          <div className="text-2xl font-bold text-white mb-1">{reports.length}</div>
          <div className="text-sm text-sis-gray-400">Total Reports</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="text-2xl font-bold text-green-400 mb-1">
            {reports.filter(r => r.status === 'passed').length}
          </div>
          <div className="text-sm text-sis-gray-400">Passed</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="text-2xl font-bold text-yellow-400 mb-1">
            {reports.filter(r => r.status === 'warning').length}
          </div>
          <div className="text-sm text-sis-gray-400">Warnings</div>
        </div>
        
        <div className="card p-4 text-center">
          <div className="text-2xl font-bold text-red-400 mb-1">
            {reports.filter(r => r.status === 'failed').length}
          </div>
          <div className="text-sm text-sis-gray-400">Failed</div>
        </div>
      </div>

      {/* Reports List */}
      <div className="space-y-4">
        {filteredReports.map(report => (
          <div
            key={report.id}
            onClick={() => setSelectedReport(report)}
            className="card p-6 cursor-pointer hover:border-sis-blue-500 transition-colors"
          >
            <div className="flex items-center justify-between">
              <div className="flex items-start space-x-4">
                <div className="flex-shrink-0 mt-1">
                  {getStatusIcon(report.status)}
                </div>
                
                <div>
                  <h3 className="text-white font-medium mb-1">{report.modelName}</h3>
                  <p className="text-sm text-sis-gray-400 mb-2">Version {report.version}</p>
                  <div className="flex items-center space-x-4 text-xs text-sis-gray-500">
                    <span className="flex items-center space-x-1">
                      <Calendar className="w-3 h-3" />
                      <span>{formatDate(report.date)}</span>
                    </span>
                    <span className="flex items-center space-x-1">
                      <Clock className="w-3 h-3" />
                      <span>{report.duration}</span>
                    </span>
                    <span className="flex items-center space-x-1">
                      <Users className="w-3 h-3" />
                      <span>{report.validator}</span>
                    </span>
                  </div>
                </div>
              </div>
              
              <div className="text-right">
                <div className="text-2xl font-bold text-white mb-1">
                  {report.overallScore.toFixed(1)}
                </div>
                <div className="text-sm text-sis-gray-400 mb-2">Score</div>
                <div className="text-xs text-sis-gray-500">
                  {report.testsPassed}/{report.testsTotal} tests passed
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {filteredReports.length === 0 && (
        <div className="text-center py-12">
          <FileText className="w-12 h-12 text-sis-gray-600 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-white mb-2">No reports found</h3>
          <p className="text-sis-gray-400">Try adjusting your filters or run a new validation</p>
        </div>
      )}
    </div>
  );
};

export default ValidationReports;